//! Optional debug serial communication over USB.

extern crate feather_m0 as bsp;

use bsp::hal;
use cortex_m::peripheral::NVIC;
use hal::clock::GenericClockController;
use hal::pac::{interrupt, PM, USB};
use hal::usb::UsbBus;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

pub struct USBSerial {
    usb_bus: UsbDevice<'static, UsbBus>,
    usb_serial: SerialPort<'static, UsbBus>,
}

static mut USB_SERIAL: Option<USBSerial> = None;
static mut BUS_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;

impl USBSerial {
    /// Initializes the `USBSerial` singleton.
    ///
    /// # Arguments
    ///  * `pm_perph`: The power management peripheral
    ///  * `usb_perph`: The USB peripheral
    ///  * core: The `CorePeripheral` instance for NVIC modifications
    ///  * clocks: The clocks instance for USB peripheral clocking
    ///  * dm: The d- GPIO pad
    ///  * dp: The d+ GPIO pad
    ///  * port: the GPIO port
    pub fn init(
        pm_perph: &mut PM,
        usb_perph: USB,
        nvic: &mut hal::pac::NVIC,
        clocks: &mut GenericClockController,
        dm: impl Into<bsp::UsbDm>,
        dp: impl Into<bsp::UsbDp>,
    ) {
        let bus_allocator = unsafe {
            BUS_ALLOCATOR = Some(bsp::usb_allocator(usb_perph, clocks, pm_perph, dm, dp));
            BUS_ALLOCATOR.as_ref().unwrap()
        };

        unsafe {
            // Initialize our USBSerial singlton
            USB_SERIAL = Some(Self {
                usb_serial: SerialPort::new(bus_allocator), /* This must initialize first! */
                usb_bus: UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x16c0, 0x27dd))
                    .manufacturer("Fake company")
                    .product("Serial port")
                    .serial_number("TEST")
                    .device_class(USB_CLASS_CDC)
                    .build(),
            });

            // enable interrupts
            nvic.set_priority(interrupt::USB, 1);
            NVIC::unmask(interrupt::USB);
        }
    }

    /// Writes a message over USB serial
    ///
    /// # Arguments
    /// * message: The message to write to the USB port
    ///
    /// # Returns
    /// number of bytes successfully written
    pub fn write_to_usb(message: &str) -> usize {
        let message_bytes = message.as_bytes();
        unsafe {
            USB_SERIAL
                .as_mut()
                .unwrap()
                .usb_serial
                .write(message_bytes)
                .unwrap_or(0)
        }
    }

    /// Polls the USB peripheral, reading out whatever bytes are available
    ///
    /// # Arguments
    /// * `read_buffer`: The buffer we should read the bytes into
    ///
    /// # Returns
    /// Number of bytes read
    fn poll_usb(read_buffer: &mut [u8]) -> usize {
        let mut total_bytes_read = 0;
        unsafe {
            if let Some(usbserial) = USB_SERIAL.as_mut() {
                usbserial.usb_bus.poll(&mut [&mut usbserial.usb_serial]);

                if let Ok(bytes_read) = usbserial.usb_serial.read(read_buffer) {
                    total_bytes_read = bytes_read;
                    // We can panic if we write in interrupt & main context! No need to echo chars, so don't write here
                    // usbserial
                    //     .usb_serial
                    //     .write(&read_buffer[0..bytes_read])
                    //     .unwrap();
                    // return bytes_read;
                }
            }
        }
        total_bytes_read
    }
}

#[interrupt]
fn USB() {
    let mut read_buf: [u8; 64] = [0u8; 64];
    USBSerial::poll_usb(&mut read_buf);
}

/// Writes the given message out over USB serial.
#[macro_export]
macro_rules! serial_write {
    ($($tt:tt)*) => {{
        #[cfg(feature = "usbserial")]
        {
            use heapless::String;
            use ufmt::uwrite;
            use usbserial::USBSerial;

            let mut s: String<64> = String::new();
            uwrite!(
                ufmt_utils::WriteAdapter(&mut s), $($tt)*
            )
            .unwrap();
            USBSerial::write_to_usb(s.as_str());
        }
    }};
}

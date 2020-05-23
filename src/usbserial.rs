extern crate feather_m0 as hal;
extern crate usb_device;
extern crate usbd_serial;

use cortex_m::peripheral::NVIC;
use hal::clock::GenericClockController;
use hal::gpio::{Floating, Input, Port};
use hal::pac::{interrupt, CorePeripherals, PM, USB};
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
    ///  * pm_perph: The power management peripheral
    ///  * usb_perph: The USB peripheral
    ///  * core: The `CorePeripheral` instance for NVIC modifications
    ///  * clocks: The clocks instance for USB peripheral clocking
    ///  * dm: The d- GPIO pad
    ///  * dp: The d+ GPIO pad
    ///  * port: the GPIO port
    pub fn init(
        pm_perph: &mut PM,
        usb_perph: USB,
        core: &mut CorePeripherals,
        clocks: &mut GenericClockController,
        dm: hal::gpio::Pa24<Input<Floating>>,
        dp: hal::gpio::Pa25<Input<Floating>>,
        port: &mut Port,
    ) {
        unsafe {
            if USB_SERIAL.is_none() {
                BUS_ALLOCATOR = Some(hal::usb_allocator(
                    usb_perph, clocks, pm_perph, dm, dp, port,
                ));
                USB_SERIAL = Some(USBSerial {
                    usb_bus: UsbDeviceBuilder::new(
                        BUS_ALLOCATOR.as_ref().unwrap(),
                        UsbVidPid(0x16c0, 0x27dd),
                    )
                    .manufacturer("Fake company")
                    .product("Serial port")
                    .serial_number("TEST")
                    .device_class(USB_CLASS_CDC)
                    .build(),
                    usb_serial: SerialPort::new(BUS_ALLOCATOR.as_ref().unwrap()),
                });

                core.NVIC.set_priority(interrupt::USB, 1);
                NVIC::unmask(interrupt::USB);
            }
        }
    }

    /// Writes a message over USB serial
    ///
    /// # Arguments
    /// * message: The message to write to the USB port
    pub fn write_to_usb(message: &str) {
        unsafe {
            USB_SERIAL
                .as_mut()
                .unwrap()
                .usb_serial
                .write(message.as_bytes())
                .unwrap();
        }
    }

    /// Polls the USB peripheral, reading out whatever bytes are available
    ///
    /// # Arguments
    /// * read_buffer: The buffer we should read the bytes into
    ///
    /// # Returns
    /// Number of bytes read
    fn poll_usb(read_buffer: &mut [u8]) -> usize {
        unsafe {
            let usbserial: &mut USBSerial = USB_SERIAL.as_mut().unwrap();
            usbserial.usb_bus.poll(&mut [&mut usbserial.usb_serial]);

            if let Ok(bytes_read) = usbserial.usb_serial.read(read_buffer) {
                usbserial
                    .usb_serial
                    .write(&read_buffer[0..bytes_read])
                    .unwrap();
                return bytes_read;
            };
        };
        0
    }
}

#[interrupt]
fn USB() {
    let mut read_buf: [u8; 64] = [0u8; 64];
    USBSerial::poll_usb(&mut read_buf);
}

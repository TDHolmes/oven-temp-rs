#![no_std]
#![no_main]

const ADC_FULLSCALE: u32 = 4095; // 12 bit ADC
/// Using VDDA / 2 with digital gain 1/2, our reference is ~3.3v
const ADC_REF_VOLTAGE: f32 = 3.3;

const DELAY_OFF_MS: u32 = 1_000; // 60_000; 60 seconds
const DELAY_COOLDOWN_MS: u32 = 1_000; // 15_000; 15 seconds
const DELAY_RUNNING_MS: u32 = 1_000; // 1 second

#[cfg(feature = "usbserial")]
extern crate heapless;
extern crate panic_semihosting;
#[cfg(feature = "usbserial")]
extern crate ufmt;
#[cfg(feature = "usbserial")]
extern crate ufmt_utils;

mod ht16k33;
mod oventemp;
#[cfg(feature = "usbserial")]
mod usbserial;

use oventemp::{OvenTemp, OvenTempState};
#[cfg(feature = "usbserial")]
use usbserial::USBSerial;

use core::sync::atomic;
use cortex_m::peripheral::NVIC;
use feather_m0 as hal;
use hal::adc::Adc;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::entry;
use hal::pac::{adc, interrupt, CorePeripherals, Peripherals, TC4};
use hal::prelude::*;
use hal::sleeping_delay::SleepingDelay;
use hal::timer;

/// Writes the given message out over USB serial.
macro_rules! serial_write {
    ($($tt:tt)*) => {{
        #[cfg(feature = "usbserial")]
        {
            use heapless::consts::*;
            use heapless::String;
            use ufmt::uwrite;
            let mut s: String<U63> = String::new();
            uwrite!(
                ufmt_utils::WriteAdapter(&mut s), $($tt)*
            )
            .unwrap();
            USBSerial::write_to_usb(s.as_str());
        }
    }};
}

static mut INTERRUPT_FIRED: Option<atomic::AtomicBool> = None;

#[entry]
fn main() -> ! {
    #[allow(unused_mut)] // Only used when usbserial is enabled
    let mut core = CorePeripherals::take().unwrap();
    let mut peripherals = Peripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut pins = hal::Pins::new(peripherals.PORT);
    let interrupt_fired = unsafe {
        INTERRUPT_FIRED = Some(atomic::AtomicBool::default());
        INTERRUPT_FIRED.as_ref().unwrap()
    };

    #[cfg(feature = "usbserial")]
    USBSerial::init(
        &mut peripherals.PM,
        peripherals.USB,
        &mut core.NVIC,
        &mut clocks,
        pins.usb_dm,
        pins.usb_dp,
        &mut pins.port,
    );

    let mut i2c = hal::i2c_master(
        &mut clocks,
        400.khz(),
        peripherals.SERCOM3,
        &mut peripherals.PM,
        pins.sda,
        pins.scl,
        &mut pins.port,
    );

    let mut red_led = pins.d13.into_open_drain_output(&mut pins.port);
    red_led.set_high().unwrap();

    let mut delay = Delay::new(core.SYST, &mut clocks);
    let mut display = match ht16k33::HT16K33::init(0x70, &mut i2c) {
        Ok(disp) => disp,
        Err(_) => loop {
            error(&mut red_led, &mut delay);
        },
    };

    display.clear();
    display.write_display(&mut i2c).unwrap();
    display.write_digit_ascii(0, ' ', false);
    display.write_digit_ascii(1, 'H', false);
    display.write_digit_ascii(2, 'I', false);
    display.write_digit_ascii(3, ' ', false);
    display.write_display(&mut i2c).unwrap();
    delay.delay_ms(500u32);
    display.clear();
    display.write_display(&mut i2c).unwrap();

    // Get a clock & make a sleeping delay object
    let gclk0 = clocks.gclk0();
    let tc45 = &clocks.tc4_tc5(&gclk0).unwrap();
    let timer = timer::TimerCounter::tc4_(tc45, peripherals.TC4, &mut peripherals.PM);
    let mut sleeping_delay = SleepingDelay::new(timer, interrupt_fired);

    unsafe {
        // enable interrupts
        core.NVIC.set_priority(interrupt::TC4, 2);
        NVIC::unmask(interrupt::TC4);
    }

    let mut adc = Adc::adc(peripherals.ADC, &mut peripherals.PM, &mut clocks);
    adc.gain(adc::inputctrl::GAIN_A::DIV2);
    adc.reference(adc::refctrl::REFSEL_A::INTVCC1);
    let mut therm_out = pins.a5.into_function_b(&mut pins.port);

    red_led.set_low().unwrap();

    let mut oven_state = OvenTemp::new();

    loop {
        let therm_reading: u16 = adc.read(&mut therm_out).unwrap();
        let therm_voltage: f32 =
            (therm_reading as f32 / ADC_FULLSCALE as f32) * ADC_REF_VOLTAGE as f32;
        let temp_c: f32 = (therm_voltage - 1.25) / 0.005;
        let temp: f32 = temp_c * (9. / 5.) + 32.;

        serial_write!("reading: {}.{}\r\n", temp as u32, (temp * 10.) as u32 % 10);

        if run(
            oven_state.state,
            temp,
            &mut i2c,
            &mut display,
            &mut sleeping_delay,
        )
        .is_err()
        {
            loop {
                error(&mut red_led, &mut delay);
            }
        }

        if let Some(new_state) = oven_state.check_transition(temp) {
            match new_state {
                OvenTempState::Off | OvenTempState::CoolingDown => {
                    display.clear();
                    if display.write_display(&mut i2c).is_err() {
                        loop {
                            error(&mut red_led, &mut delay);
                        }
                    }
                }
                _ => (),
            }
        }
    }
}

/// Display the given temperature on the display
fn display_temp<I2C, CommE>(
    temp_f: f32,
    i2c: &mut I2C,
    display: &mut ht16k33::HT16K33,
) -> Result<(), CommE>
where
    I2C: embedded_hal::blocking::i2c::Write<Error = CommE>,
{
    display.clear();
    if temp_f >= 1000. || temp_f < 10. {
        display.write_digit_ascii(0, 'E', false);
        display.write_digit_ascii(1, 'R', false);
        display.write_digit_ascii(2, 'R', false);
        display.write_digit_ascii(3, '!', false);
    } else if temp_f < 100. {
        let tens_place: u8 = (temp_f / 10.) as u8;
        let ones_place: u8 = (temp_f % 10.) as u8;
        let tenths_place: u8 = ((temp_f * 10.) % 10.) as u8;
        let hundredths_place: u8 = ((temp_f * 100.) % 10.) as u8;
        display.write_digit_value(0, tens_place, false);
        display.write_digit_value(1, ones_place, true);
        display.write_digit_value(2, tenths_place, false);
        display.write_digit_value(3, hundredths_place, false);
    } else if temp_f < 1000. {
        let hundreds_place: u8 = (temp_f / 100.) as u8;
        let tens_place: u8 = ((temp_f / 10.) % 10.) as u8;
        let ones_place: u8 = (temp_f % 10.) as u8;
        let tenths_place: u8 = ((temp_f * 10.) % 10.) as u8;
        display.write_digit_value(0, hundreds_place, false);
        display.write_digit_value(1, tens_place, false);
        display.write_digit_value(2, ones_place, true);
        display.write_digit_value(3, tenths_place, false);
    }
    display.write_display(i2c)
}

/// Blinks the red LED indicating an error
///
/// # Parameters
/// * red_led: The LED pin to blink
/// * delay: The `Delay` instance to wait
fn error<PIN>(red_led: &mut PIN, delay: &mut hal::delay::Delay)
where
    PIN: embedded_hal::digital::v2::OutputPin<Error = ()>,
{
    red_led.set_low().unwrap();
    delay.delay_ms(1000u32);
    red_led.set_high().unwrap();
    delay.delay_ms(1000u32);
}

/// Run the main state display/sleep logic
pub fn run<I2C, CommError>(
    state: OvenTempState,
    temp: f32,
    i2c: &mut I2C,
    display: &mut ht16k33::HT16K33,
    delay: &mut SleepingDelay<timer::TimerCounter4>,
) -> Result<(), CommError>
where
    I2C: embedded_hal::blocking::i2c::Write<Error = CommError>,
{
    match state {
        OvenTempState::Off => {
            serial_write!("Delaying for {} ms\r\n", DELAY_OFF_MS);
            delay.delay_ms(DELAY_OFF_MS);
            Ok(())
        }
        OvenTempState::CoolingDown => {
            // TODO: sleep for a long time, but not as long as Off
            serial_write!("Delaying for {} ms\r\n", DELAY_COOLDOWN_MS);
            delay.delay_ms(DELAY_COOLDOWN_MS);
            Ok(())
        }
        _ => {
            // All other states we're on and displaying the temp
            let ret = display_temp(temp, i2c, display);
            serial_write!("Delaying for {} ms\r\n", DELAY_RUNNING_MS);
            delay.delay_ms(DELAY_RUNNING_MS);
            ret
        }
    }
}

#[interrupt]
fn TC4() {
    unsafe {
        // Let the sleepingtimer know that the interrupt fired, and clear it
        INTERRUPT_FIRED
            .as_ref()
            .unwrap()
            .store(true, atomic::Ordering::Relaxed);
        TC4::ptr()
            .as_ref()
            .unwrap()
            .count16()
            .intflag
            .modify(|_, w| w.ovf().set_bit());
    }
}

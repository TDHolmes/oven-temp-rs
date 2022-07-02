//! Battery powered oven temperature monitor for ovens without digital read-outs.

#![no_std]
#![no_main]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

/// 12 bit ADC
const ADC_FULLSCALE: u32 = 4095;
/// Using VDDA / 2 with digital gain 1/2, our reference is ~3.3v
const ADC_REF_VOLTAGE: f32 = 3.3;
/// The threshold for showing a low battery indication.
/// We can't descern much below this voltage due to drop out
const LOW_BATTERY_VOLTAGE: f32 = 3.7;

const DELAY_OFF_MS: u32 = 1_000;
const DELAY_COOLDOWN_MS: u32 = 1_000;
const DELAY_RUNNING_MS: u32 = 1_000;
const SECS_BETWEEN_BLINK: u32 = 5;

use panic_semihosting as _; // Panic handler

#[cfg(feature = "usbserial")]
use oven_temp_rs::serial_write;
#[cfg(feature = "usbserial")]
use oven_temp_rs::usbserial;
use oven_temp_rs::{
    battery, ht16k33,
    oventemp::{OvenTemp, OvenTempState},
};

use core::sync::atomic;
use cortex_m::peripheral::NVIC;
use feather_m0 as hal;
use hal::adc::Adc;
use hal::clock::{enable_internal_32kosc, ClockGenId, ClockSource, GenericClockController};
use hal::entry;
use hal::pac::{adc, interrupt, CorePeripherals, Peripherals, TC4};
use hal::prelude::*;

#[cfg(not(feature = "usbserial"))]
macro_rules! serial_write {
    ($($tt:tt)*) => {{}};
}

/// boolean indicating if our timer interrupt has fired
#[allow(unused)]
static INTERRUPT_FIRED: atomic::AtomicBool = atomic::AtomicBool::new(false);

/// Main function, controlling all of our logic
#[entry]
fn main() -> ! {
    #[allow(unused_mut)] // Only used when usbserial is enabled
    let mut core = CorePeripherals::take().unwrap();
    let mut peripherals = Peripherals::take().unwrap();
    let mut pins = hal::Pins::new(peripherals.PORT);

    // just 8 MHz for lower power consumption
    #[cfg(not(feature = "usbserial"))]
    let mut clocks = GenericClockController::with_internal_8mhz(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    // 48 MHz needed for USB
    #[cfg(feature = "usbserial")]
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    #[cfg(feature = "usbserial")]
    {
        use oven_temp_rs::usbserial::USBSerial;
        USBSerial::init(
            &mut peripherals.PM,
            peripherals.USB,
            &mut core.NVIC,
            &mut clocks,
            pins.usb_dm,
            pins.usb_dp,
            &mut pins.port,
        );
    }

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

    #[cfg(feature = "sleeping-delay")]
    let mut runner_delay = {
        use hal::sleeping_delay::SleepingDelay;
        use hal::timer;

        // Get a clock & make a sleeping delay object. use internal 32k clock that runs
        // in standby
        enable_internal_32kosc(&mut peripherals.SYSCTRL);
        let timer_clock = clocks
            .configure_gclk_divider_and_source(ClockGenId::GCLK1, 1, ClockSource::OSC32K, false)
            .unwrap();
        clocks.configure_standby(ClockGenId::GCLK1, true);
        let tc45 = &clocks.tc4_tc5(&timer_clock).unwrap();
        let timer = timer::TimerCounter::tc4_(tc45, peripherals.TC4, &mut peripherals.PM);
        // We can also use it in standby mode, if all of the clocks are configured to
        //   opperate in standby, for even more power savings
        core.SCB.set_sleepdeep();

        unsafe {
            // enable interrupts
            core.NVIC.set_priority(interrupt::TC4, 2);
            NVIC::unmask(interrupt::TC4);
        }

        SleepingDelay::new(timer, &INTERRUPT_FIRED)
    };

    #[cfg(not(feature = "sleeping-delay"))]
    let mut runner_delay = {
        use hal::delay::Delay;

        Delay::new(core.SYST, &mut clocks)
    };

    // wait here until the display is plugged in and communicating
    let mut display = loop {
        match ht16k33::HT16K33::init(0x70, &mut i2c) {
            Ok(disp) => break disp,
            _ => runner_delay.delay_ms(1_000_u32),
        };
    };

    display.clear();
    display.set_brightness(1, &mut i2c).unwrap();
    display.write_str(" HI ");
    display.write_display(&mut i2c).unwrap();
    runner_delay.delay_ms(500_u32);
    display.clear();
    display.write_display(&mut i2c).unwrap();

    let mut adc = Adc::adc(peripherals.ADC, &mut peripherals.PM, &mut clocks);
    adc.gain(adc::inputctrl::GAIN_A::DIV2);
    adc.reference(adc::refctrl::REFSEL_A::INTVCC1);
    adc.samples(adc::avgctrl::SAMPLENUM_A::_32);
    let mut therm_out = pins.a4.into_function_b(&mut pins.port);

    // check the battery voltage (external HW divides the reading by two)
    let mut batt_in_div_2 = pins.d9.into_function_b(&mut pins.port);

    red_led.set_low().unwrap();

    let mut oven_state = OvenTemp::new();
    let mut iteration = 0_u32;

    loop {
        // Check to make sure our battery is in good shape
        let mut battery_reading: f32 = adc.read(&mut batt_in_div_2).unwrap();
        battery_reading =
            2.0 * ((battery_reading as f32 / ADC_FULLSCALE as f32) * ADC_REF_VOLTAGE as f32);

        if battery_reading <= LOW_BATTERY_VOLTAGE {
            // inform user of low battery. Thermocouple readings are not accurate
            display.write_str("LOW");
            display.write_display(&mut i2c).unwrap();
            runner_delay.delay_ms(500_u32);
            display.write_str("BATT");
            display.write_display(&mut i2c).unwrap();
            runner_delay.delay_ms(500_u32);
            display.clear();
            display.write_display(&mut i2c).unwrap();

            // Delay for a long while with display in standby to save some power
            display.configure_standby(&mut i2c, true).unwrap();
            runner_delay.delay_ms(5_000_u32);
            display.configure_standby(&mut i2c, false).unwrap();
            runner_delay.delay_ms(100_u32);
            continue; // Do not run the typical thermocouple routine
        }

        // Check the thermocouple
        let therm_reading: u16 = adc.read(&mut therm_out).unwrap();
        let therm_voltage: f32 =
            (therm_reading as f32 / ADC_FULLSCALE as f32) * ADC_REF_VOLTAGE as f32;
        let temp_c: f32 = (therm_voltage - 1.25) / 0.005;
        let temp: f32 = temp_c * (9. / 5.) + 32.;
        iteration += 1;

        serial_write!("reading: {}.{}\r\n", temp as u32, (temp * 10.) as u32 % 10);

        if run(
            oven_state.state,
            temp,
            &mut i2c,
            &mut display,
            &mut runner_delay,
        )
        .is_err()
        {
            error(&mut red_led, &mut runner_delay);
        }

        if let Some(new_state) = oven_state.check_transition(temp) {
            match new_state {
                OvenTempState::Off | OvenTempState::CoolingDown => {
                    // clear and turn off the display
                    display.clear();
                    if display.write_display(&mut i2c).is_err() {
                        error(&mut red_led, &mut runner_delay);
                    }
                    if display.configure_standby(&mut i2c, true).is_err() {
                        error(&mut red_led, &mut runner_delay);
                    }
                }
                _ => {
                    // take the display out of standby mode
                    if display.configure_standby(&mut i2c, false).is_err() {
                        error(&mut red_led, &mut runner_delay);
                    }
                }
            }
        }

        // blink a dot to show we're alive, and show battery percentage
        let battery_percentage = battery::voltage_to_percentage(battery_reading);
        let mut blink_index: u8 = 0;
        if battery_percentage >= 75 {
            blink_index = 3;
        } else if battery_percentage >= 50 {
            blink_index = 2;
        } else if battery_percentage >= 25 {
            blink_index = 1;
        }

        // blink the dot
        if (oven_state.state == OvenTempState::Off
            || oven_state.state == OvenTempState::CoolingDown)
            && (iteration % SECS_BETWEEN_BLINK == SECS_BETWEEN_BLINK - 1)
        {
            // Turn display on
            if display.configure_standby(&mut i2c, false).is_err() {
                error(&mut red_led, &mut runner_delay);
            }

            // Blink dot
            display.clear();
            display.write_digit_ascii(blink_index, ' ', true);
            if display.write_display(&mut i2c).is_err() {
                error(&mut red_led, &mut runner_delay);
            }
            runner_delay.delay_ms(50_u32);
            display.clear();
            if display.write_display(&mut i2c).is_err() {
                error(&mut red_led, &mut runner_delay);
            }

            // turn display back off
            if display.configure_standby(&mut i2c, false).is_err() {
                error(&mut red_led, &mut runner_delay);
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
    if (10. ..100.).contains(&temp_f) {
        let tens_place: u8 = (temp_f / 10.) as u8;
        let ones_place: u8 = (temp_f % 10.) as u8;
        let tenths_place: u8 = ((temp_f * 10.) % 10.) as u8;
        let hundredths_place: u8 = ((temp_f * 100.) % 10.) as u8;
        display.write_digit_value(0, tens_place, false);
        display.write_digit_value(1, ones_place, true);
        display.write_digit_value(2, tenths_place, false);
        display.write_digit_value(3, hundredths_place, false);
    } else if (100. ..600.).contains(&temp_f) {
        let hundreds_place: u8 = (temp_f / 100.) as u8;
        let tens_place: u8 = ((temp_f / 10.) % 10.) as u8;
        let ones_place: u8 = (temp_f % 10.) as u8;
        let tenths_place: u8 = ((temp_f * 10.) % 10.) as u8;
        display.write_digit_value(0, hundreds_place, false);
        display.write_digit_value(1, tens_place, false);
        display.write_digit_value(2, ones_place, true);
        display.write_digit_value(3, tenths_place, false);
    } else {
        // Temp < 10 or >= 600. >= 600 is generally disconnected thermocouple
        display.write_str("ERR!");
    }

    display.write_display(i2c)
}

/// Blinks an SOS pattern indicating an error
///
/// # Parameters
/// * `red_led`: The LED pin to blink
/// * `delay`: The `Delay` instance to wait
fn error<PIN, T>(red_led: &mut PIN, delay: &mut T)
where
    PIN: embedded_hal::digital::v2::OutputPin<Error = ()>,
    T: embedded_hal::blocking::delay::DelayMs<u32>,
{
    const SHORT_BLIP_MS: u32 = 250;
    const LONG_BLIP_MS: u32 = 500;

    // S
    red_led.set_high().unwrap();
    delay.delay_ms(SHORT_BLIP_MS);
    red_led.set_low().unwrap();
    delay.delay_ms(SHORT_BLIP_MS);
    red_led.set_high().unwrap();
    delay.delay_ms(SHORT_BLIP_MS);
    red_led.set_low().unwrap();
    delay.delay_ms(SHORT_BLIP_MS);
    red_led.set_high().unwrap();
    delay.delay_ms(SHORT_BLIP_MS);
    red_led.set_low().unwrap();
    delay.delay_ms(SHORT_BLIP_MS);

    // O
    red_led.set_high().unwrap();
    delay.delay_ms(LONG_BLIP_MS);
    red_led.set_low().unwrap();
    delay.delay_ms(LONG_BLIP_MS);
    red_led.set_high().unwrap();
    delay.delay_ms(LONG_BLIP_MS);
    red_led.set_low().unwrap();
    delay.delay_ms(LONG_BLIP_MS);
    red_led.set_high().unwrap();
    delay.delay_ms(LONG_BLIP_MS);
    red_led.set_low().unwrap();
    delay.delay_ms(LONG_BLIP_MS);

    // S
    red_led.set_high().unwrap();
    delay.delay_ms(SHORT_BLIP_MS);
    red_led.set_low().unwrap();
    delay.delay_ms(SHORT_BLIP_MS);
    red_led.set_high().unwrap();
    delay.delay_ms(SHORT_BLIP_MS);
    red_led.set_low().unwrap();
    delay.delay_ms(SHORT_BLIP_MS);
    red_led.set_high().unwrap();
    delay.delay_ms(SHORT_BLIP_MS);
    red_led.set_low().unwrap();

    delay.delay_ms(2 * LONG_BLIP_MS);
}

/// Run the main state display/sleep logic
pub fn run<I2C, CommError, T>(
    state: OvenTempState,
    temp: f32,
    i2c: &mut I2C,
    display: &mut ht16k33::HT16K33,
    delay: &mut T,
) -> Result<(), CommError>
where
    I2C: embedded_hal::blocking::i2c::Write<Error = CommError>,
    T: embedded_hal::blocking::delay::DelayMs<u32>,
{
    match state {
        OvenTempState::Off => {
            serial_write!("Off\r\n");
            delay.delay_ms(DELAY_OFF_MS);
            Ok(())
        }
        OvenTempState::CoolingDown => {
            serial_write!("CoolingDown\r\n");
            delay.delay_ms(DELAY_COOLDOWN_MS);
            Ok(())
        }
        _ => {
            serial_write!("WarmingUp or AtTemp\r\n");
            let ret = display_temp(temp, i2c, display);
            delay.delay_ms(DELAY_RUNNING_MS);
            ret
        }
    }
}

/// The sleeping timer interrupt that wakes us up
#[interrupt]
fn TC4() {
    // Let the sleepingtimer know that the interrupt fired, and clear it
    INTERRUPT_FIRED.store(true, atomic::Ordering::Relaxed);
    unsafe {
        TC4::ptr()
            .as_ref()
            .unwrap()
            .count16()
            .intflag
            .modify(|_, w| w.ovf().set_bit());
    }
}

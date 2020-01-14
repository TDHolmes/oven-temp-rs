#![no_std]
#![no_main]

const ADC_FULLSCALE: u32 = 4095; // 12 bit ADC
/// Threshold at which we start displaying the temperature
const TEMP_ON_THRESHOLD: f32 = 100.;
/// Threshold at which we turn the display back off as the oven cools off
const TEMP_OFF_THRESHOLD: f32 = 200.;
/// Some hysteresis to avoid thrash
const TEMP_HYSTERESIS: f32 = 10.;
/// Using VDDA / 2 with digital gain 1/2, our reference is ~3.3v
const ADC_REF_VOLTAGE: f32 = 3.3;

const DELAY_OFF_MS: u32 = 1_000; // 60_000; 60 seconds
const DELAY_COOLDOWN_MS: u32 = 1_000; // 15_000; 15 seconds
const DELAY_RUNNING_MS: u32 = 1_000; // 1 second

extern crate cortex_m;
extern crate panic_halt; // panic handler
mod ht16k33;

use feather_m0 as hal;
use hal::adc::Adc;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::entry;
use hal::pac::adc::{inputctrl, refctrl};
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;

enum OvenTempStates {
    Off,
    HeatingUp,
    AtTemp,
    CoolingDown,
}

struct OvenTemp {
    state: OvenTempStates,
    temp_previous: f32,
}

impl OvenTemp {
    fn new() -> OvenTemp {
        OvenTemp {
            state: OvenTempStates::AtTemp,
            temp_previous: 0.,
        }
    }

    fn check_transition<I2C, CommE>(
        &mut self,
        temp: f32,
        i2c: &mut I2C,
        display: &mut ht16k33::HT16K33,
    ) -> Result<(), CommE>
    where
        I2C: embedded_hal::blocking::i2c::Write<Error = CommE>,
    {
        // Potentially move to a new state
        let new_state_opt = match &mut self.state {
            OvenTempStates::Off => {
                if temp >= TEMP_ON_THRESHOLD + TEMP_HYSTERESIS {
                    Some(OvenTempStates::HeatingUp)
                } else {
                    None
                }
            }
            OvenTempStates::HeatingUp => {
                if temp >= TEMP_OFF_THRESHOLD + TEMP_HYSTERESIS {
                    Some(OvenTempStates::AtTemp)
                } else if temp >= TEMP_ON_THRESHOLD - TEMP_HYSTERESIS {
                    Some(OvenTempStates::Off)
                } else {
                    None
                }
            }
            OvenTempStates::AtTemp => {
                if temp <= TEMP_OFF_THRESHOLD - TEMP_HYSTERESIS {
                    Some(OvenTempStates::CoolingDown)
                } else {
                    None
                }
            }
            OvenTempStates::CoolingDown => {
                if temp <= TEMP_ON_THRESHOLD - TEMP_HYSTERESIS {
                    Some(OvenTempStates::Off)
                } else {
                    None
                }
            }
        };

        // If we're transitioning to a state with no display, turn it off!
        let mut result: Result<(), CommE> = Ok(());
        if let Some(new_state) = new_state_opt {
            match new_state {
                OvenTempStates::Off | OvenTempStates::CoolingDown => {
                    display.clear();
                    result = display.write_display(i2c);
                }
                _ => (),
            };
            self.state = new_state;
        }
        result
    }

    fn run<I2C, CommE>(
        &mut self,
        temp: f32,
        i2c: &mut I2C,
        display: &mut ht16k33::HT16K33,
        delay: &mut Delay,
    ) -> Result<(), CommE>
    where
        I2C: embedded_hal::blocking::i2c::Write<Error = CommE>,
    {
        self.temp_previous = temp;
        match &mut self.state {
            OvenTempStates::Off => {
                delay.delay_ms(DELAY_OFF_MS);
                Ok(())
            }
            OvenTempStates::CoolingDown => {
                // TODO: sleep for a long time, but not as long as Off
                delay.delay_ms(DELAY_COOLDOWN_MS);
                Ok(())
            }
            _ => {
                // All other states we're on and displaying the temp
                let ret = display_temp(temp, i2c, display);
                delay.delay_ms(DELAY_RUNNING_MS);
                ret
            }
        }
    }
}

#[entry]
fn main() -> ! {
    // cortex_m::Peripherals::take().unwrap()
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut pins = hal::Pins::new(peripherals.PORT);

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
    delay.delay_ms(500u32);
    display.write_digit_ascii(0, ' ', false);
    display.write_digit_ascii(1, 'H', false);
    display.write_digit_ascii(2, 'I', false);
    display.write_digit_ascii(3, ' ', false);
    display.write_display(&mut i2c).unwrap();
    delay.delay_ms(500u32);
    display.clear();
    display.write_display(&mut i2c).unwrap();

    let mut adc = Adc::adc(peripherals.ADC, &mut peripherals.PM, &mut clocks);
    adc.gain(inputctrl::GAIN_A::DIV2);
    adc.reference(refctrl::REFSEL_A::INTVCC1);
    let mut therm_out = pins.a5.into_function_b(&mut pins.port);

    red_led.set_low().unwrap();

    let mut state = OvenTemp::new();

    loop {
        let therm_reading: u16 = adc.read(&mut therm_out).unwrap();

        let therm_voltage: f32 =
            (therm_reading as f32 / ADC_FULLSCALE as f32) * ADC_REF_VOLTAGE as f32;
        let temp_c: f32 = (therm_voltage - 1.25) / 0.005;
        let temp: f32 = temp_c * (9. / 5.) + 32.;

        state.run(temp, &mut i2c, &mut display, &mut delay).unwrap();
        if let Err(_) = state.check_transition(temp, &mut i2c, &mut display) {
            loop {
                error(&mut red_led, &mut delay);
            }
        }
    }
}

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

fn error<PIN>(red_led: &mut PIN, delay: &mut hal::delay::Delay)
where
    PIN: embedded_hal::digital::v2::OutputPin<Error = ()>,
{
    red_led.set_low().unwrap();
    delay.delay_ms(1000u32);
    red_led.set_high().unwrap();
    delay.delay_ms(1000u32);
}

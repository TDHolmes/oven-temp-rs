#![no_std]
#![no_main]

const ADC_FULLSCALE: u32 = 4095; // 12 bit ADC

extern crate cortex_m;
extern crate panic_halt; // panic handler
mod ht16k33;

use feather_m0 as hal;
use hal::adc::{refsel, Adc};
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::entry;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;

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
            red_led.set_low().unwrap();
            delay.delay_ms(1000u32);
            red_led.set_high().unwrap();
            delay.delay_ms(1000u32);
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

    let mut adc = Adc::adc(
        peripherals.ADC,
        &mut peripherals.PM,
        &mut clocks,
        refsel::AREFA,
    );
    let mut a5 = pins.a5.into_function_b(&mut pins.port);

    red_led.set_low().unwrap();

    let mut raw: bool = false;
    loop {
        let data: u16 = adc.read(&mut a5).unwrap();
        // let data: u16 = data / 2;

        // if adc.adc.inputctrl.gain().is_1x() {
        //     loop {
        //         red_led.set_low().unwrap();
        //         delay.delay_ms(1000u32);
        //         red_led.set_high().unwrap();
        //         delay.delay_ms(1000u32);
        //     }
        // }

        // let data: u16 = 2016;
        display.clear();
        if raw {
            display.write_digit_value(0, ((data / 1000) % 10) as u8, false);
            display.write_digit_value(1, ((data / 100) % 10) as u8, false);
            display.write_digit_value(2, ((data / 10) % 10) as u8, false);
            display.write_digit_value(3, ((data) % 10) as u8, false);
        } else {
            let temp_c: f32 = (((data as f32 / ADC_FULLSCALE as f32) * 3.3) - 1.25) / 0.005;
            let temp: f32 = temp_c * (9. / 5.) + 32.;
            if temp >= 1000. || temp < 10. {
                display.write_digit_ascii(0, 'E', false);
                display.write_digit_ascii(1, 'R', false);
                display.write_digit_ascii(2, 'R', false);
                display.write_digit_ascii(3, '!', false);
            } else if temp < 100. {
                let tens_place: u8 = (temp / 10.) as u8;
                let ones_place: u8 = (temp % 10.) as u8;
                let tenths_place: u8 = ((temp * 10.) % 10.) as u8;
                let hundredths_place: u8 = ((temp * 100.) % 10.) as u8;
                display.write_digit_value(0, tens_place, false);
                display.write_digit_value(1, ones_place, true);
                display.write_digit_value(2, tenths_place, false);
                display.write_digit_value(3, hundredths_place, false);
            } else if temp < 1000. {
                let hundreds_place: u8 = (temp / 100.) as u8;
                let tens_place: u8 = ((temp / 10.) % 10.) as u8;
                let ones_place: u8 = (temp % 10.) as u8;
                let tenths_place: u8 = ((temp * 10.) % 10.) as u8;
                display.write_digit_value(0, hundreds_place, false);
                display.write_digit_value(1, tens_place, false);
                display.write_digit_value(2, ones_place, true);
                display.write_digit_value(3, tenths_place, false);
            }
        }
        display.write_display(&mut i2c).unwrap();
        delay.delay_ms(1000 as u32);

        if raw {
            raw = false;
        } else {
            raw = true;
        }
    }
}

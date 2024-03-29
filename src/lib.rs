//! The abstracted components for the oven-temp binary.

#![no_std]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

pub mod battery;
pub mod ht16k33;
pub mod oventemp;
#[cfg(feature = "usbserial")]
pub mod usbserial;

#[cfg(test)]
mod test {
    #[test]
    fn testy_test() {}
}

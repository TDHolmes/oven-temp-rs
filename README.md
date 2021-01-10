# Oven Temperature Monitor

This project utilizes a [Feather M0] board, a [quad alphanumeric display], and a
[thermocouple amplifier] to monitor the temperature of dumb ovens that do
not have digital temperature readout. It is battery opperated, and written
in [rust].

Read more about it [on my website!](https://www.holmesengineering.com/oven-temp-rs/)

# Building and Flashing

As this project uses a [Feather M0] board, it has the [HF2 bootloader] pre-installed from
Adafruit. With the [cargo-hf2] crate installed, building and flashing is as simple as:

1. Place board into bootloader mode (double-press the reset button)
2. Run `cargo hf2 <options>`

To install [cargo-hf2], run `cargo install cargo-hf2`. Additional setup may be needed depending
on your OS. Refer to the crates.io page for more information.

# License

This code is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

# Schematic / Pictures

![Schematic - very simple][schematic]

![Picture of the board][oven-temp-board]

[Feather M0]: https://www.adafruit.com/product/2772
[quad alphanumeric display]: https://www.adafruit.com/product/3127
[thermocouple amplifier]: https://www.adafruit.com/product/1778
[rust]: https://rust-lang.org
[oven-temp-board]: https://github.com/TDHolmes/oven-temp-rs/raw/master/docs/images/oven-temp-off.jpeg
[schematic]: https://github.com/TDHolmes/oven-temp-rs/raw/master/docs/images/schematic.png
[HF2 bootloader]: https://github.com/jacobrosenthal/hf2-rs/tree/master/hf2
[cargo-hf2]: https://crates.io/crates/cargo-hf2

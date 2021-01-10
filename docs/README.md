# Rust Oven Temperature Monitor

This project came about when my wife noticed that the oven in our appartment didn't seem to
be at the correct temperature, as everything would bake too quickly. Seeing this as a great
excuse to do a project, I set off on making an oven temperature monitor. To give myself some
constraints, however, I wanted it to be battery powered and last a long time.

My first attempt was written in C using an STM32. you can read more about that [here](https://github.com/TDHolmes/OvenTemp).
This is the new and improved version, written in Rust.

![alt text][oven-temp-board]

## Hardware Summary

This project utilizes a [Feather M0] board, a [quad alphanumeric display], and a
[thermocouple amplifier] to monitor the temperature of dumb ovens that do
not have digital temperature readout. It is battery opperated, and written
in [rust].

## Initial Implementation

My first cut at this was simple: port all of the basic functionality over from the C-based
version of the project to rust. This was relatively easy given how far along the embedded
rust ecosystem is, especially for these simple projects. Since I'm using a feather M0 board, I used the
wonderful [atsamd-rs] HAL crates. This is a great and welcoming codebase tha also had all of the basic
features I needed to get this project functional. I also took the oportunity to write my own driver for
my alphanumeric display, which I based off of the Adafruit C++ library, with some tweaks for power
I'll get into later on.

## Power Optimizations

I had a functional project up and running quickly, but it definitely wasn't going to work as the
final project. The timer APIs as existed at the time had no capability for interrupt-based
operation, so sleeping the core for power savings wasn't easy. Because of this limitation, I
[opened a PR](https://github.com/atsamd-rs/atsamd/pull/205) for this functionality,
saving 6 mA of current consumption, and I didn't have to worry about interrupt memory saftey since rust didn't
allow me to (easily) do anything unsafe.

![alt text][power - measurement]
![alt text][power - overall]
![alt text][power - wfi]

This change allowed simple sleeping of the core without powering down much else. A lot more could be
gained by configuring the power management peripheral to sleep more of the cores busses. I implemented
these changes in [this PR](https://github.com/atsamd-rs/atsamd/pull/291), which reduced our current consumption
(apart from the display) from ~870 uA to ~250 uA.

![alt text][power - standby]

This was a large win, however we were still consuming about 5 mA of power in idle. The big delta here came
from the display, which it turns out has an idle mode that isn't exposed in the C++ library I was referencing
when making my rust driver. With the addition of this change, the displays idle current consumption drops
dramatically, bringing us to what I think is the minimum current consumption without making hardware changes,
like a more efficient LDO.

[Feather M0]: https://www.adafruit.com/product/2772
[quad alphanumeric display]: https://www.adafruit.com/product/3127
[thermocouple amplifier]: https://www.adafruit.com/product/1778
[rust]: https://rust-lang.org
[atsamd-rs]: https://github.com/atsamd-rs/atsamd

[oven-temp-board]: https://github.com/TDHolmes/oven-temp-rs/raw/main/docs/images/oven-temp-off.jpeg "The oven temp monitor in place"
[power - wfi]: https://github.com/TDHolmes/oven-temp-rs/raw/main/docs/images/power-consumption-idle.png "Idle power consumption numbers with and without WFI"
[power - overall]: https://github.com/TDHolmes/oven-temp-rs/raw/main/docs/images/power-consumption.png "Measurement of the overall power consumption while displaying a temperature"
[power - measurement]: https://github.com/TDHolmes/oven-temp-rs/raw/main/docs/images/power-measurement.jpeg "Measuring power consumption with the EEVBlog's uCurrent"
[power - standby]: https://github.com/TDHolmes/oven-temp-rs/raw/main/docs/images/sleeping-timer-rtc.png "Power measurements after implementing standby sleeping. The spike is an LED blinking"
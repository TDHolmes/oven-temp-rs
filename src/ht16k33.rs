// use embedded_hal::blocking::i2c::Write;

const DISPLAY_BUFFER_SIZE: usize = 8;

const ALPHA_FONT_TABLE: [u16; 128] = [
    0b0000_0000_0000_0001,
    0b0000_0000_0000_0010,
    0b0000_0000_0000_0100,
    0b0000_0000_0000_1000,
    0b0000_0000_0001_0000,
    0b0000_0000_0010_0000,
    0b0000_0000_0100_0000,
    0b0000_0000_1000_0000,
    0b0000_0001_0000_0000,
    0b0000_0010_0000_0000,
    0b0000_0100_0000_0000,
    0b0000_1000_0000_0000,
    0b0001_0000_0000_0000,
    0b0010_0000_0000_0000,
    0b0100_0000_0000_0000,
    0b1000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0000_0000_0000_0000,
    0b0001_0010_1100_1001,
    0b0001_0101_1100_0000,
    0b0001_0010_1111_1001,
    0b0000_0000_1110_0011,
    0b0000_0101_0011_0000,
    0b0001_0010_1100_1000,
    0b0011_1010_0000_0000,
    0b0001_0111_0000_0000,
    0b0000_0000_0000_0000, //
    0b0000_0000_0000_0110, // !
    0b0000_0010_0010_0000, // "
    0b0001_0010_1100_1110, // #
    0b0001_0010_1110_1101, // $
    0b0000_1100_0010_0100, // %
    0b0010_0011_0101_1101, // &
    0b0000_0100_0000_0000, // '
    0b0010_0100_0000_0000, // (
    0b0000_1001_0000_0000, // )
    0b0011_1111_1100_0000, // *
    0b0001_0010_1100_0000, // +
    0b0000_1000_0000_0000, // ,
    0b0000_0000_1100_0000, // -
    0b0000_0000_0000_0000, // .
    0b0000_1100_0000_0000, // /
    0b0000_1100_0011_1111, // 0
    0b0000_0000_0000_0110, // 1
    0b0000_0000_1101_1011, // 2
    0b0000_0000_1000_1111, // 3
    0b0000_0000_1110_0110, // 4
    0b0010_0000_0110_1001, // 5
    0b0000_0000_1111_1101, // 6
    0b0000_0000_0000_0111, // 7
    0b0000_0000_1111_1111, // 8
    0b0000_0000_1110_1111, // 9
    0b0001_0010_0000_0000, // :
    0b0000_1010_0000_0000, // ;
    0b0010_0100_0000_0000, // <
    0b0000_0000_1100_1000, // =
    0b0000_1001_0000_0000, // >
    0b0001_0000_1000_0011, // ?
    0b0000_0010_1011_1011, // @
    0b0000_0000_1111_0111, // A
    0b0001_0010_1000_1111, // B
    0b0000_0000_0011_1001, // C
    0b0001_0010_0000_1111, // D
    0b0000_0000_1111_1001, // E
    0b0000_0000_0111_0001, // F
    0b0000_0000_1011_1101, // G
    0b0000_0000_1111_0110, // H
    0b0001_0010_0000_0000, // I
    0b0000_0000_0001_1110, // J
    0b0010_0100_0111_0000, // K
    0b0000_0000_0011_1000, // L
    0b0000_0101_0011_0110, // M
    0b0010_0001_0011_0110, // N
    0b0000_0000_0011_1111, // O
    0b0000_0000_1111_0011, // P
    0b0010_0000_0011_1111, // Q
    0b0010_0000_1111_0011, // R
    0b0000_0000_1110_1101, // S
    0b0001_0010_0000_0001, // T
    0b0000_0000_0011_1110, // U
    0b0000_1100_0011_0000, // V
    0b0010_1000_0011_0110, // W
    0b0010_1101_0000_0000, // X
    0b0001_0101_0000_0000, // Y
    0b0000_1100_0000_1001, // Z
    0b0000_0000_0011_1001, // [
    0b0010_0001_0000_0000, //
    0b0000_0000_0000_1111, // ]
    0b0000_1100_0000_0011, // ^
    0b0000_0000_0000_1000, // _
    0b0000_0001_0000_0000, // `
    0b0001_0000_0101_1000, // a
    0b0010_0000_0111_1000, // b
    0b0000_0000_1101_1000, // c
    0b0000_1000_1000_1110, // d
    0b0000_1000_0101_1000, // e
    0b0000_0000_0111_0001, // f
    0b0000_0100_1000_1110, // g
    0b0001_0000_0111_0000, // h
    0b0001_0000_0000_0000, // i
    0b0000_0000_0000_1110, // j
    0b0011_0110_0000_0000, // k
    0b0000_0000_0011_0000, // l
    0b0001_0000_1101_0100, // m
    0b0001_0000_0101_0000, // n
    0b0000_0000_1101_1100, // o
    0b0000_0001_0111_0000, // p
    0b0000_0100_1000_0110, // q
    0b0000_0000_0101_0000, // r
    0b0010_0000_1000_1000, // s
    0b0000_0000_0111_1000, // t
    0b0000_0000_0001_1100, // u
    0b0010_0000_0000_0100, // v
    0b0010_1000_0001_0100, // w
    0b0010_1000_1100_0000, // x
    0b0010_0000_0000_1100, // y
    0b0000_1000_0100_1000, // z
    0b0000_1001_0100_1001, // {
    0b0001_0010_0000_0000, // |
    0b0010_0100_1000_1001, // }
    0b0000_0101_0010_0000, // ~
    0b0011_1111_1111_1111,
];

// const DISP_I2C_ADDR: u8 = (0x70 << 1);

// const LED_ON: u8 = 1;
// const LED_OFF: u8 = 0;

// const LED_RED: u8 = 1;
// const LED_YELLOW: u8 = 2;
// const LED_GREEN: u8 = 3;

const HT16K33_BLINK_CMD: u8 = 0x80;
const HT16K33_BLINK_DISPLAYON: u8 = 0x01;
const HT16K33_BLINK_OFF: u8 = 0;
// const HT16K33_BLINK_2HZ: u8 = 1;
// const HT16K33_BLINK_1HZ: u8 = 2;
// const HT16K33_BLINK_HALFHZ: u8 = 3;

const HT16K33_CMD_BRIGHTNESS: u8 = 0xE0;
// const SEVENSEG_DIGITS: u8 = 5;

// const DEC: u8 = 10;
// const HEX: u8 = 16;
// const OCT: u8 = 8;
// const BIN: u8 = 2;
// const BYTE: u8 = 0;

const ALPHA_POINT_MASK: u16 = 1 << 14;

pub struct HT16K33 {
    i2c_addr: u8,
    display_buffer: [u16; DISPLAY_BUFFER_SIZE],
}

impl HT16K33 {
    pub fn init<I2C, CommE>(addr: u8, i2c: &mut I2C) -> Result<Self, CommE>
    where
        I2C: embedded_hal::blocking::i2c::Write<Error = CommE>,
    {
        let mut ht = Self {
            i2c_addr: addr,
            display_buffer: [0; DISPLAY_BUFFER_SIZE],
        };

        // turn on oscillator
        let data: [u8; 1] = [0x21];
        i2c.write(ht.i2c_addr, &data)?;
        ht.blink_rate(HT16K33_BLINK_OFF, i2c)?;
        ht.set_brightness(15, i2c)?; // max brightness

        Ok(ht)
    }

    pub fn set_brightness<I2C, CommE>(&mut self, b: u8, i2c: &mut I2C) -> Result<(), CommE>
    where
        I2C: embedded_hal::blocking::i2c::Write<Error = CommE>,
    {
        let mut b = b;
        if b > 15 {
            b = 15;
        }
        let data: [u8; 1] = [HT16K33_CMD_BRIGHTNESS | b];
        i2c.write(self.i2c_addr, &data)?;
        Ok(())
    }

    pub fn blink_rate<I2C, CommE>(&mut self, b: u8, i2c: &mut I2C) -> Result<(), CommE>
    where
        I2C: embedded_hal::blocking::i2c::Write<Error = CommE>,
    {
        let mut bm = b;
        if bm > 3 {
            bm = 0; // turn off if not sure
        }

        let data: [u8; 1] = [HT16K33_BLINK_CMD | HT16K33_BLINK_DISPLAYON | (bm << 1)];
        i2c.write(self.i2c_addr, &data)?;
        Ok(())
    }

    pub fn clear(&mut self) {
        for i in 0..8 {
            self.display_buffer[i] = 0;
        }
    }

    pub fn write_digit_value(&mut self, n: u8, number: u8, point: bool) {
        self.display_buffer[n as usize] = ALPHA_FONT_TABLE[(number + 0x30) as usize];
        if point {
            self.display_buffer[n as usize] |= ALPHA_POINT_MASK;
        }
    }

    pub fn write_digit_ascii(&mut self, n: u8, character: char, point: bool) {
        self.display_buffer[n as usize] = ALPHA_FONT_TABLE[character as usize];
        if point {
            self.display_buffer[n as usize] |= ALPHA_POINT_MASK;
        }
    }

    pub fn write_display<I2C, CommE>(&mut self, i2c: &mut I2C) -> Result<(), CommE>
    where
        I2C: embedded_hal::blocking::i2c::Write<Error = CommE>,
    {
        // Build up the payload. Start with 0
        let mut data: [u8; 17] = [0; 17];
        data[0] = 0;
        for i in 0..8 {
            data[2 * i + 1] = (self.display_buffer[i as usize] & 0xFF) as u8;
            data[2 * i + 2] = (self.display_buffer[i as usize] >> 8) as u8;
        }
        i2c.write(self.i2c_addr, &data)?;
        Ok(())
    }
}

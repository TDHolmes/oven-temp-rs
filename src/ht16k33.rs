// use embedded_hal::blocking::i2c::Write;

const DISPLAY_BUFFER_SIZE: usize = 8;

const ALPHA_FONT_TABLE: [u16; 128] = [
    0b0000000000000001,
    0b0000000000000010,
    0b0000000000000100,
    0b0000000000001000,
    0b0000000000010000,
    0b0000000000100000,
    0b0000000001000000,
    0b0000000010000000,
    0b0000000100000000,
    0b0000001000000000,
    0b0000010000000000,
    0b0000100000000000,
    0b0001000000000000,
    0b0010000000000000,
    0b0100000000000000,
    0b1000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0001001011001001,
    0b0001010111000000,
    0b0001001011111001,
    0b0000000011100011,
    0b0000010100110000,
    0b0001001011001000,
    0b0011101000000000,
    0b0001011100000000,
    0b0000000000000000, //
    0b0000000000000110, // !
    0b0000001000100000, // "
    0b0001001011001110, // #
    0b0001001011101101, // $
    0b0000110000100100, // %
    0b0010001101011101, // &
    0b0000010000000000, // '
    0b0010010000000000, // (
    0b0000100100000000, // )
    0b0011111111000000, // *
    0b0001001011000000, // +
    0b0000100000000000, // ,
    0b0000000011000000, // -
    0b0000000000000000, // .
    0b0000110000000000, // /
    0b0000110000111111, // 0
    0b0000000000000110, // 1
    0b0000000011011011, // 2
    0b0000000010001111, // 3
    0b0000000011100110, // 4
    0b0010000001101001, // 5
    0b0000000011111101, // 6
    0b0000000000000111, // 7
    0b0000000011111111, // 8
    0b0000000011101111, // 9
    0b0001001000000000, // :
    0b0000101000000000, // ;
    0b0010010000000000, // <
    0b0000000011001000, // =
    0b0000100100000000, // >
    0b0001000010000011, // ?
    0b0000001010111011, // @
    0b0000000011110111, // A
    0b0001001010001111, // B
    0b0000000000111001, // C
    0b0001001000001111, // D
    0b0000000011111001, // E
    0b0000000001110001, // F
    0b0000000010111101, // G
    0b0000000011110110, // H
    0b0001001000000000, // I
    0b0000000000011110, // J
    0b0010010001110000, // K
    0b0000000000111000, // L
    0b0000010100110110, // M
    0b0010000100110110, // N
    0b0000000000111111, // O
    0b0000000011110011, // P
    0b0010000000111111, // Q
    0b0010000011110011, // R
    0b0000000011101101, // S
    0b0001001000000001, // T
    0b0000000000111110, // U
    0b0000110000110000, // V
    0b0010100000110110, // W
    0b0010110100000000, // X
    0b0001010100000000, // Y
    0b0000110000001001, // Z
    0b0000000000111001, // [
    0b0010000100000000, //
    0b0000000000001111, // ]
    0b0000110000000011, // ^
    0b0000000000001000, // _
    0b0000000100000000, // `
    0b0001000001011000, // a
    0b0010000001111000, // b
    0b0000000011011000, // c
    0b0000100010001110, // d
    0b0000100001011000, // e
    0b0000000001110001, // f
    0b0000010010001110, // g
    0b0001000001110000, // h
    0b0001000000000000, // i
    0b0000000000001110, // j
    0b0011011000000000, // k
    0b0000000000110000, // l
    0b0001000011010100, // m
    0b0001000001010000, // n
    0b0000000011011100, // o
    0b0000000101110000, // p
    0b0000010010000110, // q
    0b0000000001010000, // r
    0b0010000010001000, // s
    0b0000000001111000, // t
    0b0000000000011100, // u
    0b0010000000000100, // v
    0b0010100000010100, // w
    0b0010100011000000, // x
    0b0010000000001100, // y
    0b0000100001001000, // z
    0b0000100101001001, // {
    0b0001001000000000, // |
    0b0010010010001001, // }
    0b0000010100100000, // ~
    0b0011111111111111,
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
    pub fn init<I2C, CommE>(addr: u8, i2c: &mut I2C) -> Result<HT16K33, CommE>
    where
        I2C: embedded_hal::blocking::i2c::Write<Error = CommE>,
    {
        let mut ht = HT16K33 {
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
        let mut b = b;
        if b > 3 {
            b = 0; // turn off if not sure
        }

        let data: [u8; 1] = [HT16K33_BLINK_CMD | HT16K33_BLINK_DISPLAYON | (b << 1)];
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

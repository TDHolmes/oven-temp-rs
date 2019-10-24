#![feature(const_generics)]

extern crate embedded_hal as hal;
use embedded_hal::blocking::i2c as blocking_i2c;

/// I2C address of the SSD1306
const ADDR: u8 = 0x3C;

const MAX_WIDTH: usize = 128;
const MAX_HEIGHT: usize = 64;

pub enum SSD1306Errors {
    DimensionsInvalid,
    GenericError, // TODO: improve lol
    I2CWriteFailure,
    I2CReadFailure,
}

pub enum Colors {
    /// Draw 'off' pixels
    BLACK =   0,
    /// Draw 'on' pixels
    WHITE =   1,
    /// Invert pixels
    INVERSE = 2,
}


pub struct DisplayDimensions {
    pub width: usize,
    pub height: usize,
}


const MEMORYMODE: u8 =          0x20;
const COLUMNADDR: u8 =          0x21;
const PAGEADDR: u8 =            0x22;
const SETCONTRAST: u8 =         0x81;
const CHARGEPUMP: u8 =          0x8D;
const SEGREMAP: u8 =            0xA0;
const DISPLAYALLON_RESUME: u8 = 0xA4;
const DISPLAYALLON: u8 =        0xA5;
const NORMALDISPLAY: u8 =       0xA6;
const INVERTDISPLAY: u8 =       0xA7;
const SETMULTIPLEX: u8 =        0xA8;
const DISPLAYOFF: u8 =          0xAE;
const DISPLAYON: u8 =           0xAF;
const COMSCANINC: u8 =          0xC0;
const COMSCANDEC: u8 =          0xC8;
const SETDISPLAYOFFSET: u8 =    0xD3;
const SETDISPLAYCLOCKDIV: u8 =  0xD5;
const SETPRECHARGE: u8 =        0xD9;
const SETCOMPINS: u8 =          0xDA;
const SETVCOMDETECT: u8 =       0xDB;

const SETLOWCOLUMN: u8 =        0x00;
const SETHIGHCOLUMN: u8 =       0x10;
const SETSTARTLINE: u8 =        0x40;

/// External display voltage source
const EXTERNALVCC: u8 =         0x01;

/// Gen. display voltage from 3.3V
const SWITCHCAPVCC: u8 =        0x02;

/// Init rt scroll
const RIGHT_HORIZONTAL_SCROLL: u8 =              0x26;
/// Init left scroll
const LEFT_HORIZONTAL_SCROLL: u8 =               0x27;
/// Init diag scroll
const VERTICAL_AND_RIGHT_HORIZONTAL_SCROLL: u8 = 0x29;
/// Init diag scroll
const VERTICAL_AND_LEFT_HORIZONTAL_SCROLL: u8 =  0x2A;
/// Stop scroll
const DEACTIVATE_SCROLL: u8 =                    0x2E;
/// Start scroll
const ACTIVATE_SCROLL: u8 =                      0x2F;
/// Set scroll range
const SET_VERTICAL_SCROLL_AREA: u8 =             0xA3;

/// TODO: parameterize?
const VCC_STATE: u8 = SWITCHCAPVCC;


// TODO: Const generics man
pub struct SSD1306 {
    // Display buffer with one byte at the beginning for a command (for I2C)
    display_buffer: [u8; (MAX_HEIGHT + 7) * MAX_WIDTH / 8 + 1],
    pub width: usize,
    pub height: usize,
    command_buffer: [u8; 10],  // TODO: figure out max command size
}


impl SSD1306 {

    /* Public API */

    pub fn new(dimensions: &DisplayDimensions) -> Result<SSD1306, SSD1306Errors> {
        if dimensions.width > MAX_WIDTH || dimensions.height > MAX_HEIGHT {
            Err(SSD1306Errors::DimensionsInvalid)
        } else if SSD1306::dimensions_valid(dimensions) == false {
            Err(SSD1306Errors::DimensionsInvalid)
        } else {
            let mut s = SSD1306 {
                width: dimensions.width,
                height: dimensions.height,
                display_buffer: [0xff; (MAX_HEIGHT + 7) * MAX_WIDTH / 8 + 1],
                command_buffer: [0; 10],
            };

            // TODO: magic command / number that triggers the writing of the display
            s.display_buffer[0] = 0x40;
            Ok(s)
        }
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, color: Colors) -> Result<(), SSD1306Errors> {
        if (x < self.width) && (y < self.height) {
            let index: usize = x + (y/8) * self.width + 1;
            match color {
                Colors::WHITE =>   self.display_buffer[index] |=   1 << (y&7),
                Colors::BLACK =>   self.display_buffer[index] &= !(1 << (y&7)),
                Colors::INVERSE => self.display_buffer[index] ^=   1 << (y&7),
            }
            Ok(())
        } else {
            Err(SSD1306Errors::DimensionsInvalid)
        }
    }

    /* Actual device interactions */

    pub fn dim<I2C: blocking_i2c::Write + blocking_i2c::Read>(&mut self, i2c: &mut I2C, dim: bool) -> Result<(), SSD1306Errors> {
        let mut contrast: u8 = 0;

        if !dim {
            if VCC_STATE == EXTERNALVCC {
                contrast = 0x9F;
            } else {
                contrast = 0xCF;
            }
        }
        // the range of contrast to too small to be really useful
        // it is useful to dim the display
        self.command_buffer[1] = SETCONTRAST; self.command(i2c, 1)?;
        self.command_buffer[1] = contrast;    self.command(i2c, 1)?;
        Ok(())
    }

    pub fn init_display<I2C: blocking_i2c::Write + blocking_i2c::Read>(&mut self, i2c: &mut I2C) ->
            Result<(), SSD1306Errors>
        {

        // Init sequence, taken from https://github.com/adafruit/Adafruit_SSD1306 begin function

        // Clear display and configure basics
        self.command_buffer[1] = DISPLAYOFF;
        self.command_buffer[2] = SETDISPLAYCLOCKDIV;
        self.command_buffer[3] = 0x80;               // the suggested ratio 0x80
        self.command_buffer[4] = SETMULTIPLEX;
        self.command(i2c, 4)?;

        self.command_buffer[1] = self.height as u8 - 1;
        self.command(i2c, 1)?;

        self.command_buffer[1] = SETDISPLAYOFFSET;
        self.command_buffer[2] = 0x0;              // no offset
        self.command_buffer[3] = SETSTARTLINE;     // line #0
        self.command_buffer[4] = CHARGEPUMP;
        self.command(i2c, 4)?;

        // TODO: Figure out what 0x10 and 0x14 mean
        if VCC_STATE == EXTERNALVCC {
            self.command_buffer[1] = 0x10;
        } else {
            self.command_buffer[1] = 0x14;
        }
        self.command(i2c, 1)?;

        self.command_buffer[1] = MEMORYMODE;
        self.command_buffer[2] = 0x00;              // 0x0 act like ks0108
        self.command_buffer[3] = (SEGREMAP) | 0x1;
        self.command_buffer[4] = COMSCANDEC;
        self.command(i2c, 4)?;

        if (self.width == 128) && (self.height == 32) {
            self.command_buffer[1] = SETCOMPINS;
            self.command_buffer[2] = 0x02;
            self.command_buffer[3] = SETCONTRAST;
            self.command_buffer[4] = 0x8F;
            self.command(i2c, 4)?;
        } else if (self.width == 128) && (self.height == 64) {
            self.command_buffer[1] = SETCOMPINS;
            self.command_buffer[2] = 0x12;
            self.command_buffer[3] = SETCONTRAST;
            self.command(i2c, 3)?;
            if VCC_STATE == EXTERNALVCC {
                self.command_buffer[1] = 0x9F;
            } else {
                self.command_buffer[1] = 0xCF;
            }
            self.command(i2c, 1)?;
        } else if (self.width == 96) && (self.height == 16) {
            self.command_buffer[1] = SETCOMPINS;
            self.command_buffer[2] = 0x2;
            self.command_buffer[3] = SETCONTRAST;
            self.command(i2c, 3)?;
            if VCC_STATE == EXTERNALVCC {
                self.command_buffer[1] = 0x10;
            } else {
                self.command_buffer[1] = 0xAF;
            }
        } else {
            // Other screen varieties -- TBD
            return Err(SSD1306Errors::GenericError);
        }

        self.command_buffer[1] = SETPRECHARGE;
        self.command(i2c, 1)?;
        if VCC_STATE == EXTERNALVCC {
            self.command_buffer[1] = 0x22;
        } else {
            self.command_buffer[1] = 0xF1;
        }
        self.command(i2c, 1)?;

        // final configuration / turn on
        self.command_buffer[1] = SETVCOMDETECT;
        self.command_buffer[2] = 0x40;
        self.command_buffer[3] = DISPLAYALLON_RESUME;
        self.command_buffer[4] = NORMALDISPLAY;
        self.command_buffer[5] = DEACTIVATE_SCROLL;
        self.command_buffer[6] = DISPLAYON;
        self.command(i2c, 6)?;
        Ok(())
    }

    pub fn display<I2C: blocking_i2c::Write + blocking_i2c::Read>(&mut self, i2c: &mut I2C) -> Result<(), SSD1306Errors> {
        self.command_buffer[1] = PAGEADDR;
        self.command_buffer[2] = 0;              // Page start address
        self.command_buffer[3] = 0xFF;           // Page end (not really, but works here)
        self.command_buffer[4] = COLUMNADDR;
        self.command_buffer[5] = 0;              // Column start address
        self.command(i2c, 5)?;

        // Column end address
        self.command_buffer[1] = self.width as u8 - 1;
        self.command(i2c, 1)?;

        // use seperate, direct i2c write as buffers are different
        match i2c.write(ADDR, &self.display_buffer)  {
            Err(_) => return Err(SSD1306Errors::I2CWriteFailure),
            Ok(_) => (),
        }

        Ok(())
    }

    /* Private Functions */

    fn dimensions_valid(dim: &DisplayDimensions) -> bool {
        if (dim.width == 128 && dim.height == 32) ||
                (dim.width == 128 && dim.height == 64) ||
                (dim.width == 96 && dim.height == 16) {
            return true;
        } else {
            return false;
        }
    }

    fn command<I2C: blocking_i2c::Write + blocking_i2c::Read>(&self, i2c: &mut I2C, len: usize) -> Result<(), SSD1306Errors> {
        if 1 + len > self.command_buffer.len() {
            return Err(SSD1306Errors::GenericError);
        }
        match i2c.write(ADDR, &self.command_buffer[..len + 1])  {
            Err(_) => return Err(SSD1306Errors::I2CWriteFailure),
            Ok(_) =>  return Ok(()),
        }
    }

}
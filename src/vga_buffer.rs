#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
///
/// Color in VGA
///
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl ScreenChar {
    fn new(ascii_character: u8, color_code: ColorCode) -> Self {
        Self {
            ascii_character,
            color_code,
        }
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

use volatile::Volatile;

#[repr(transparent)]
struct Buffer {
    // We don't want rust to optimize this
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let end = BUFFER_HEIGHT - 1;
                // Needs 2 bytes, first byte is the character,
                //  second byte is the color foreground and background
                self.buffer.chars[end][self.column_position]
                    .write(ScreenChar::new(byte, self.color));
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn clear_row(&mut self, row: usize) {
        for i in 0..BUFFER_WIDTH {
            self.buffer.chars[row][i].write(ScreenChar::new(
                0,
                ColorCode::new(Color::Black, Color::Black),
            ));
        }
    }

    fn new_line(&mut self) {
        for i in 1..BUFFER_HEIGHT {
            for j in 0..BUFFER_WIDTH {
                self.buffer.chars[i - 1][j].write(self.buffer.chars[i][j].read());
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }
}

pub fn print_something() {
    let mut writer = Writer {
        column_position: 0,
        color: ColorCode::new(Color::Yellow, Color::Black),
        // the VGA buffer is in the memory address 0xb8000
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };
    writer.write_string("Hello Wörld!");
    use core::fmt::Write;
    write!(writer, "que tal {}\n", 1).unwrap();
    write!(
        writer,
        "que tal aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaasdssdsdasdasda"
    )
    .unwrap();
}

use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

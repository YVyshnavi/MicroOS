//! ==============================================================================
//! RustOS: 80x25 VGA Color Text Buffer Driver with Hardware Cursor Control
//! Memory-mapped I/O at physical address 0xb8000
//! ==============================================================================

use crate::ports::outb;
use core::fmt;
use core::ptr::write_volatile;

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;
pub const VGA_BUFFER_ADDR: usize = 0xb8000;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(pub u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode(((background as u8) << 4) | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

pub struct VgaWriter {
    pub column_position: usize,
    pub row_position: usize,
    pub color_code: ColorCode,
    pub buffer: *mut ScreenChar,
}

// Global writer instance
pub static mut WRITER: VgaWriter = VgaWriter {
    column_position: 0,
    row_position: 0,
    color_code: ColorCode::new(Color::White, Color::Black),
    buffer: VGA_BUFFER_ADDR as *mut ScreenChar,
};

impl VgaWriter {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            0x08 => self.backspace(), // Backspace
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;
                let offset = row * BUFFER_WIDTH + col;

                unsafe {
                    write_volatile(
                        self.buffer.add(offset),
                        ScreenChar {
                            ascii_character: byte,
                            color_code: self.color_code,
                        },
                    );
                }
                self.column_position += 1;
                self.update_cursor();
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' | 0x08 => self.write_byte(byte),
                _ => self.write_byte(0xfe), // Print square symbol for unprintable
            }
        }
    }

    pub fn new_line(&mut self) {
        self.column_position = 0;
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            // Scroll screen up by 1 row
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let src_idx = row * BUFFER_WIDTH + col;
                    let dst_idx = (row - 1) * BUFFER_WIDTH + col;
                    unsafe {
                        let ch = core::ptr::read_volatile(self.buffer.add(src_idx));
                        write_volatile(self.buffer.add(dst_idx), ch);
                    }
                }
            }
            // Clear last row
            self.clear_row(BUFFER_HEIGHT - 1);
        }
        self.update_cursor();
    }

    pub fn backspace(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            let offset = self.row_position * BUFFER_WIDTH + self.column_position;
            unsafe {
                write_volatile(
                    self.buffer.add(offset),
                    ScreenChar {
                        ascii_character: b' ',
                        color_code: self.color_code,
                    },
                );
            }
            self.update_cursor();
        }
    }

    pub fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            let offset = row * BUFFER_WIDTH + col;
            unsafe {
                write_volatile(self.buffer.add(offset), blank);
            }
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.row_position = 0;
        self.column_position = 0;
        self.update_cursor();
    }

    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    pub fn update_cursor(&self) {
        let pos = (self.row_position * BUFFER_WIDTH + self.column_position) as u16;
        unsafe {
            outb(0x3D4, 0x0F);
            outb(0x3D5, (pos & 0xFF) as u8);
            outb(0x3D4, 0x0E);
            outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
        }
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        let _ = WRITER.write_fmt(args);
    }
}

pub fn set_color(fg: Color, bg: Color) {
    unsafe {
        WRITER.set_color(fg, bg);
    }
}

pub fn clear_screen() {
    unsafe {
        WRITER.clear_screen();
    }
}

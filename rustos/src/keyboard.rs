//! ==============================================================================
//! RustOS: PS/2 Keyboard Controller Driver (IRQ1 / Vector 33)
//! Decodes Scancode Set 1 (US QWERTY Layout) with Ring Buffer
//! ==============================================================================

use crate::ports::inb;

pub const KEYBOARD_DATA_PORT: u16 = 0x60;
pub const KEYBOARD_STATUS_PORT: u16 = 0x64;

const BUFFER_CAPACITY: usize = 128;
static mut KEY_BUFFER: [char; BUFFER_CAPACITY] = ['\0'; BUFFER_CAPACITY];
static mut HEAD: usize = 0;
static mut TAIL: usize = 0;
static mut SHIFT_ACTIVE: bool = false;

pub fn push_char(c: char) {
    unsafe {
        let next_tail = (TAIL + 1) % BUFFER_CAPACITY;
        if next_tail != HEAD {
            KEY_BUFFER[TAIL] = c;
            TAIL = next_tail;
        }
    }
}

pub fn pop_char() -> Option<char> {
    unsafe {
        if HEAD == TAIL {
            None
        } else {
            let c = KEY_BUFFER[HEAD];
            HEAD = (HEAD + 1) % BUFFER_CAPACITY;
            Some(c)
        }
    }
}

pub fn handle_keyboard_interrupt() {
    unsafe {
        let status = inb(KEYBOARD_STATUS_PORT);
        if (status & 0x01) == 0 {
            return;
        }
        let scancode = inb(KEYBOARD_DATA_PORT);

        // Shift key tracking
        if scancode == 0x2A || scancode == 0x36 {
            SHIFT_ACTIVE = true;
            return;
        }
        if scancode == 0xAA || scancode == 0xB6 {
            SHIFT_ACTIVE = false;
            return;
        }

        // Key releases (break codes: scancode >= 0x80)
        if scancode >= 0x80 {
            return;
        }

        let character = match scancode {
            0x02 => if SHIFT_ACTIVE { '!' } else { '1' },
            0x03 => if SHIFT_ACTIVE { '@' } else { '2' },
            0x04 => if SHIFT_ACTIVE { '#' } else { '3' },
            0x05 => if SHIFT_ACTIVE { '$' } else { '4' },
            0x06 => if SHIFT_ACTIVE { '%' } else { '5' },
            0x07 => if SHIFT_ACTIVE { '^' } else { '6' },
            0x08 => if SHIFT_ACTIVE { '&' } else { '7' },
            0x09 => if SHIFT_ACTIVE { '*' } else { '8' },
            0x0A => if SHIFT_ACTIVE { '(' } else { '9' },
            0x0B => if SHIFT_ACTIVE { ')' } else { '0' },
            0x0C => if SHIFT_ACTIVE { '_' } else { '-' },
            0x0D => if SHIFT_ACTIVE { '+' } else { '=' },
            0x0E => '\x08', // Backspace
            0x0F => '\t',   // Tab
            0x10 => if SHIFT_ACTIVE { 'Q' } else { 'q' },
            0x11 => if SHIFT_ACTIVE { 'W' } else { 'w' },
            0x12 => if SHIFT_ACTIVE { 'E' } else { 'e' },
            0x13 => if SHIFT_ACTIVE { 'R' } else { 'r' },
            0x14 => if SHIFT_ACTIVE { 'T' } else { 't' },
            0x15 => if SHIFT_ACTIVE { 'Y' } else { 'y' },
            0x16 => if SHIFT_ACTIVE { 'U' } else { 'u' },
            0x17 => if SHIFT_ACTIVE { 'I' } else { 'i' },
            0x18 => if SHIFT_ACTIVE { 'O' } else { 'o' },
            0x19 => if SHIFT_ACTIVE { 'P' } else { 'p' },
            0x1C => '\n',   // Enter
            0x1E => if SHIFT_ACTIVE { 'A' } else { 'a' },
            0x1F => if SHIFT_ACTIVE { 'S' } else { 's' },
            0x20 => if SHIFT_ACTIVE { 'D' } else { 'd' },
            0x21 => if SHIFT_ACTIVE { 'F' } else { 'f' },
            0x22 => if SHIFT_ACTIVE { 'G' } else { 'g' },
            0x23 => if SHIFT_ACTIVE { 'H' } else { 'h' },
            0x24 => if SHIFT_ACTIVE { 'J' } else { 'j' },
            0x25 => if SHIFT_ACTIVE { 'K' } else { 'k' },
            0x26 => if SHIFT_ACTIVE { 'L' } else { 'l' },
            0x2C => if SHIFT_ACTIVE { 'Z' } else { 'z' },
            0x2D => if SHIFT_ACTIVE { 'X' } else { 'x' },
            0x2E => if SHIFT_ACTIVE { 'C' } else { 'c' },
            0x2F => if SHIFT_ACTIVE { 'V' } else { 'v' },
            0x30 => if SHIFT_ACTIVE { 'B' } else { 'b' },
            0x31 => if SHIFT_ACTIVE { 'N' } else { 'n' },
            0x32 => if SHIFT_ACTIVE { 'M' } else { 'm' },
            0x33 => if SHIFT_ACTIVE { '<' } else { ',' },
            0x34 => if SHIFT_ACTIVE { '>' } else { '.' },
            0x35 => if SHIFT_ACTIVE { '?' } else { '/' },
            0x39 => ' ',    // Space
            _ => '\0',
        };

        if character != '\0' {
            push_char(character);
        }
    }
}

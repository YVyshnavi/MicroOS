//! ==============================================================================
//! RustOS: 8254 Programmable Interval Timer (PIT) Driver
//! Fires periodic IRQ0 timer interrupts (Vector 32)
//! ==============================================================================

use crate::ports::outb;

pub const PIT_CHANNEL0_DATA: u16 = 0x40;
pub const PIT_COMMAND_PORT: u16 = 0x43;
pub const PIT_BASE_FREQUENCY: u32 = 1193182;
pub const TARGET_FREQUENCY: u32 = 100; // 100 Hz = 10 ms per tick

pub static mut TICKS: u64 = 0;

pub fn init() {
    let divisor = (PIT_BASE_FREQUENCY / TARGET_FREQUENCY) as u16;

    unsafe {
        // Command byte: 0x36 = 00 11 011 0
        // Channel 0 | Access mode lo/hi byte | Mode 3 (Square wave generator) | 16-bit binary
        outb(PIT_COMMAND_PORT, 0x36);
        outb(PIT_CHANNEL0_DATA, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL0_DATA, ((divisor >> 8) & 0xFF) as u8);
    }
}

pub fn handle_timer_interrupt() {
    unsafe {
        TICKS = TICKS.wrapping_add(1);
    }
}

pub fn get_ticks() -> u64 {
    unsafe { TICKS }
}

pub fn get_uptime_seconds() -> u64 {
    get_ticks() / (TARGET_FREQUENCY as u64)
}

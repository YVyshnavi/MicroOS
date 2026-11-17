//! ==============================================================================
//! RustOS: 8259 Programmable Interrupt Controller (PIC) Driver & Remapping
//! Maps IRQs 0..15 to interrupt vectors 32..47 (0x20..0x2F)
//! ==============================================================================

use crate::ports::{inb, io_wait, outb};

pub const PIC1_COMMAND: u16 = 0x20;
pub const PIC1_DATA: u16 = 0x21;
pub const PIC2_COMMAND: u16 = 0xA0;
pub const PIC2_DATA: u16 = 0xA1;

pub const PIC_EOI: u8 = 0x20;

pub const ICW1_INIT: u8 = 0x10;
pub const ICW1_ICW4: u8 = 0x01;
pub const ICW4_8086: u8 = 0x01;

pub const PIC1_OFFSET: u8 = 32; // IRQ 0..7 mapped to 32..39
pub const PIC2_OFFSET: u8 = 40; // IRQ 8..15 mapped to 40..47

pub fn init() {
    unsafe {
        // Save interrupt masks
        let mask1 = inb(PIC1_DATA);
        let mask2 = inb(PIC2_DATA);

        // Start initialization sequence in cascade mode
        outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();

        // ICW2: Vector offsets
        outb(PIC1_DATA, PIC1_OFFSET);
        io_wait();
        outb(PIC2_DATA, PIC2_OFFSET);
        io_wait();

        // ICW3: Cascade configuration
        outb(PIC1_DATA, 4); // Tell Master PIC that there is a slave PIC at IRQ2 (0000 0100)
        io_wait();
        outb(PIC2_DATA, 2); // Tell Slave PIC its cascade identity (0000 0010)
        io_wait();

        // ICW4: 8086 mode
        outb(PIC1_DATA, ICW4_8086);
        io_wait();
        outb(PIC2_DATA, ICW4_8086);
        io_wait();

        // Restore masks (or enable IRQ0 timer & IRQ1 keyboard)
        // Enable IRQ0 (Timer: bit 0) and IRQ1 (Keyboard: bit 1)
        outb(PIC1_DATA, 0xFC); // 1111 1100 -> only IRQ0 and IRQ1 enabled
        outb(PIC2_DATA, 0xFF); // Disable all slave IRQs for now
    }
}

pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_COMMAND, PIC_EOI);
        }
        outb(PIC1_COMMAND, PIC_EOI);
    }
}

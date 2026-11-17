//! ==============================================================================
//! RustOS: 32-bit x86 Interrupt Descriptor Table (IDT) & ISR Handlers
//! Manages CPU Exceptions (0..31) and Hardware IRQs (32..47)
//! ==============================================================================

use crate::keyboard::handle_keyboard_interrupt;
use crate::pic::send_eoi;
use crate::timer::handle_timer_interrupt;
use crate::vga_buffer::{set_color, Color};
use crate::{print, println};

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_attr: u8,
    offset_high: u16,
}

impl IdtEntry {
    pub const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            zero: 0,
            type_attr: 0,
            offset_high: 0,
        }
    }

    pub fn set_handler(&mut self, handler: u32, selector: u16, flags: u8) {
        self.offset_low = (handler & 0xFFFF) as u16;
        self.selector = selector;
        self.zero = 0;
        self.type_attr = flags;
        self.offset_high = ((handler >> 16) & 0xFFFF) as u16;
    }
}

#[repr(C, packed)]
pub struct IdtPtr {
    limit: u16,
    base: u32,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::empty(); 256];
static mut IDT_PTR: IdtPtr = IdtPtr { limit: 0, base: 0 };

extern "C" {
    fn isr0();
    fn isr8();
    fn isr13();
    fn isr14();
    fn irq0_handler();
    fn irq1_handler();
}

core::arch::global_asm!(
    r#"
    .global isr0
    isr0:
        push eax
        push ecx
        push edx
        call exception_divide_zero
        pop edx
        pop ecx
        pop eax
        iretd

    .global isr8
    isr8:
        push eax
        push ecx
        push edx
        call exception_double_fault
        pop edx
        pop ecx
        pop eax
        add esp, 4
        iretd

    .global isr13
    isr13:
        push eax
        push ecx
        push edx
        call exception_gpf
        pop edx
        pop ecx
        pop eax
        add esp, 4
        iretd

    .global isr14
    isr14:
        push eax
        push ecx
        push edx
        call exception_page_fault
        pop edx
        pop ecx
        pop eax
        add esp, 4
        iretd

    .global irq0_handler
    irq0_handler:
        push eax
        push ecx
        push edx
        call timer_irq_dispatcher
        pop edx
        pop ecx
        pop eax
        iretd

    .global irq1_handler
    irq1_handler:
        push eax
        push ecx
        push edx
        call keyboard_irq_dispatcher
        pop edx
        pop ecx
        pop eax
        iretd
    "#
);

pub fn init() {
    unsafe {
        IDT_PTR.limit = (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16;
        IDT_PTR.base = IDT.as_ptr() as u32;

        // Populate Exception Handlers
        IDT[0].set_handler(isr0 as usize as u32, 0x08, 0x8E);  // Divide Error
        IDT[8].set_handler(isr8 as usize as u32, 0x08, 0x8E);  // Double Fault
        IDT[13].set_handler(isr13 as usize as u32, 0x08, 0x8E); // General Protection Fault
        IDT[14].set_handler(isr14 as usize as u32, 0x08, 0x8E); // Page Fault

        // Populate Hardware IRQs
        IDT[32].set_handler(irq0_handler as usize as u32, 0x08, 0x8E); // IRQ0: Timer
        IDT[33].set_handler(irq1_handler as usize as u32, 0x08, 0x8E); // IRQ1: Keyboard

        core::arch::asm!(
            "lidt [{0}]",
            in(reg) &IDT_PTR,
            options(nostack)
        );
    }
}

#[inline(always)]
pub fn enable() {
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}

#[inline(always)]
pub fn disable() {
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack));
    }
}

#[no_mangle]
extern "C" fn timer_irq_dispatcher() {
    handle_timer_interrupt();
    send_eoi(0);
}

#[no_mangle]
extern "C" fn keyboard_irq_dispatcher() {
    handle_keyboard_interrupt();
    send_eoi(1);
}

#[no_mangle]
extern "C" fn exception_divide_zero() {
    set_color(Color::LightRed, Color::Black);
    println!("\n[CPU EXCEPTION #0]: Divide-by-Zero occurred!");
    set_color(Color::White, Color::Black);
}

#[no_mangle]
extern "C" fn exception_double_fault() {
    set_color(Color::LightRed, Color::Black);
    println!("\n[CPU EXCEPTION #8]: Double Fault Kernel Panic!");
    set_color(Color::White, Color::Black);
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[no_mangle]
extern "C" fn exception_gpf() {
    set_color(Color::LightRed, Color::Black);
    println!("\n[CPU EXCEPTION #13]: General Protection Fault (#GP)!");
    set_color(Color::White, Color::Black);
}

#[no_mangle]
extern "C" fn exception_page_fault() {
    let fault_addr: u32;
    unsafe {
        core::arch::asm!("mov {0}, cr2", out(reg) fault_addr);
    }
    set_color(Color::LightRed, Color::Black);
    println!("\n[CPU EXCEPTION #14]: Page Fault at address 0x{:08X}!", fault_addr);
    set_color(Color::White, Color::Black);
}

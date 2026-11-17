#![no_std]
#![no_main]

//! ==============================================================================
//! 👑 RUSTOS: 32-bit Bare-Metal Operating System Kernel in Freestanding Rust
//! Built from scratch (#![no_std], #![no_main]) with zero external C dependencies.
//!
//! Subsystems:
//!   - 32-bit Multiboot 1 compliant boot header & stack frame
//!   - Memory-mapped VGA 80x25 Color Text Driver with Hardware Cursor
//!   - Custom 32-bit x86 Global Descriptor Table (GDT)
//!   - Dual 8259 Programmable Interrupt Controller (PIC) Remapping
//!   - 256-entry Interrupt Descriptor Table (IDT) with CPU Exception & IRQ ISRs
//!   - 8254 Programmable Interval Timer (PIT) @ 100 Hz
//!   - PS/2 Keyboard Controller Driver (Set 1 Scancode Decoder)
//!   - Interactive Kernel Command Shell & Mini Retro Arcade
//!   - In-Kernel Bare-Metal Workload-Aware CPU Scheduling Benchmark
//!
//! Author: Yeddula Vyshnavi | USN: 25BTRCL294 | Section: E
//! Course: 25PCCS201: Operating Systems Research Activity
//! Institution: JAIN (Deemed-to-be University), FET Campus
//! ==============================================================================

pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod pic;
pub mod ports;
pub mod sched_demo;
pub mod shell;
pub mod timer;
pub mod vga_buffer;

use core::panic::PanicInfo;
use shell::SHELL;
use vga_buffer::{clear_screen, set_color, Color};

// ==============================================================================
// Multiboot 1 Header & Bootstrap Assembly
// ==============================================================================
core::arch::global_asm!(
    r#"
    .set MULTIBOOT_MAGIC,    0x1BADB002
    .set MULTIBOOT_FLAGS,    0x00000003
    .set MULTIBOOT_CHECKSUM, -(MULTIBOOT_MAGIC + MULTIBOOT_FLAGS)

    .section .multiboot_header, "a"
    .align 4
    .long MULTIBOOT_MAGIC
    .long MULTIBOOT_FLAGS
    .long MULTIBOOT_CHECKSUM

    .section .text
    .global _start
    _start:
        cli
        mov esp, offset _stack_top
        call kernel_main
    1:  hlt
        jmp 1b

    .section .bss
    .align 16
    _stack_bottom:
        .skip 16384 # 16KB Initial Kernel Stack
    _stack_top:
    "#
);

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // 1. Initialize Hardware Screen
    clear_screen();

    // 2. Display System Banner
    set_color(Color::LightCyan, Color::Black);
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║       🦀 RUSTOS BARE-METAL KERNEL (32-bit x86 Protected Mode)            ║");
    println!("║   Freestanding Rust Kernel with Custom GDT, IDT, PIC, PIT & VGA Drivers  ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");

    set_color(Color::Yellow, Color::Black);
    println!(" Candidate   : Yeddula Vyshnavi | USN: 25BTRCL294 | Section: E");
    println!(" Institution : JAIN (Deemed-to-be University), FET Campus");
    println!(" Research    : Workload-Aware Scheduling Strategy Selection");
    set_color(Color::White, Color::Black);
    println!("----------------------------------------------------------------------------");

    // 3. Hardware Subsystems Initialization Sequence
    set_color(Color::LightGreen, Color::Black);
    print!("[OK] ");
    set_color(Color::White, Color::Black);
    println!("VGA 80x25 text driver initialized (MMIO 0xb8000).");

    print!("[..] Initializing Global Descriptor Table (GDT)... ");
    gdt::init();
    set_color(Color::LightGreen, Color::Black);
    println!("[OK]");
    set_color(Color::White, Color::Black);

    print!("[..] Remapping 8259 Dual PIC (IRQ 0..15 -> INT 32..47)... ");
    pic::init();
    set_color(Color::LightGreen, Color::Black);
    println!("[OK]");
    set_color(Color::White, Color::Black);

    print!("[..] Initializing Interrupt Descriptor Table (IDT 256 gates)... ");
    interrupts::init();
    set_color(Color::LightGreen, Color::Black);
    println!("[OK]");
    set_color(Color::White, Color::Black);

    print!("[..] Configuring 8254 PIT Timer (Channel 0 @ 100 Hz)... ");
    timer::init();
    set_color(Color::LightGreen, Color::Black);
    println!("[OK]");
    set_color(Color::White, Color::Black);

    print!("[..] Enabling CPU Hardware Interrupts (sti)... ");
    interrupts::enable();
    set_color(Color::LightGreen, Color::Black);
    println!("[OK]");
    set_color(Color::White, Color::Black);

    println!("----------------------------------------------------------------------------");
    set_color(Color::LightCyan, Color::Black);
    println!("Type 'help' for commands, 'sched' for research benchmark, 'snake' for game.");
    set_color(Color::White, Color::Black);
    println!();

    // 4. Start Kernel Shell
    unsafe {
        SHELL.print_prompt();
    }

    // 5. Main Kernel Event Loop (Sleep on HLT until next hardware IRQ)
    loop {
        unsafe {
            SHELL.update();
            core::arch::asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    set_color(Color::LightRed, Color::Black);
    println!("\n========================================================");
    println!(" ⚠️  RUSTOS BARE-METAL KERNEL PANIC                      ");
    println!("========================================================");
    println!("{}", info);
    set_color(Color::White, Color::Black);
    loop {
        unsafe {
            core::arch::asm!("cli", "hlt");
        }
    }
}

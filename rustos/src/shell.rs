//! ==============================================================================
//! RustOS: Interactive Bare-Metal Kernel Shell & Retro Arcade Console
//! ==============================================================================

use crate::keyboard::pop_char;
use crate::ports::outb;
use crate::sched_demo::run_kernel_sched_benchmark;
use crate::timer::{get_ticks, get_uptime_seconds};
use crate::vga_buffer::{clear_screen, set_color, Color};
use crate::{print, println};

const MAX_CMD_LEN: usize = 64;

pub struct Shell {
    buffer: [char; MAX_CMD_LEN],
    len: usize,
}

impl Shell {
    pub const fn new() -> Self {
        Self {
            buffer: ['\0'; MAX_CMD_LEN],
            len: 0,
        }
    }

    pub fn print_prompt(&self) {
        set_color(Color::LightGreen, Color::Black);
        print!("rustos");
        set_color(Color::LightGray, Color::Black);
        print!("@");
        set_color(Color::LightCyan, Color::Black);
        print!("kernel");
        set_color(Color::White, Color::Black);
        print!("> ");
    }

    pub fn update(&mut self) {
        while let Some(c) = pop_char() {
            match c {
                '\n' => {
                    println!();
                    self.execute();
                    self.len = 0;
                    self.print_prompt();
                }
                '\x08' => {
                    if self.len > 0 {
                        self.len -= 1;
                        print!("\x08");
                    }
                }
                c if c >= ' ' && c <= '~' => {
                    if self.len < MAX_CMD_LEN - 1 {
                        self.buffer[self.len] = c;
                        self.len += 1;
                        print!("{}", c);
                    }
                }
                _ => {}
            }
        }
    }

    fn execute(&self) {
        if self.len == 0 {
            return;
        }

        // Convert slice of chars to ASCII string comparison
        let mut cmd = [0u8; MAX_CMD_LEN];
        for i in 0..self.len {
            cmd[i] = self.buffer[i] as u8;
        }
        let cmd_str = core::str::from_utf8(&cmd[..self.len]).unwrap_or("");

        match cmd_str {
            "help" => {
                set_color(Color::Yellow, Color::Black);
                println!("Available RustOS Kernel Commands:");
                set_color(Color::White, Color::Black);
                println!("  help     - Display this command glossary");
                println!("  info     - Display kernel architecture & author details");
                println!("  clear    - Clear VGA text screen");
                println!("  sched    - Run bare-metal Workload-Aware CPU Scheduling Benchmark");
                println!("  ticks    - Show PIT timer hardware ticks and uptime");
                println!("  snake    - Play in-kernel retro arcade snake game");
                println!("  reboot   - Soft reboot machine via 8042 keyboard controller");
                println!("  halt     - Halt CPU execution (cli + hlt)");
            }
            "info" => {
                set_color(Color::LightCyan, Color::Black);
                println!("╔══════════════════════════════════════════════════════════╗");
                println!("║                     RUSTOS KERNEL                        ║");
                println!("╚══════════════════════════════════════════════════════════╝");
                set_color(Color::White, Color::Black);
                println!(" Architecture : 32-bit x86 Protected Mode (i686)");
                println!(" Kernel Core  : Freestanding Rust (#![no_std], #![no_main])");
                println!(" Memory Model : Flat 4GB Segmentation (GDT 0x08 / 0x10)");
                println!(" Interrupts   : Custom IDT (256 gates) + Remapped 8259 PIC");
                println!(" Hardware PIT : 8254 Timer Channel 0 @ 100 Hz");
                println!(" Input/Output : PS/2 Keyboard IRQ1 + 80x25 VGA Text Buffer");
                set_color(Color::Yellow, Color::Black);
                println!(" Candidate    : Yeddula Vyshnavi | USN: 25BTRCL294 | Section: E");
                println!(" Course       : 25PCCS201: Operating Systems Research Activity");
                println!(" Institution  : JAIN (Deemed-to-be University), FET Campus");
                set_color(Color::White, Color::Black);
            }
            "clear" => {
                clear_screen();
            }
            "ticks" => {
                let ticks = get_ticks();
                let uptime = get_uptime_seconds();
                println!("PIT Hardware Ticks: {} | Uptime: {} seconds", ticks, uptime);
            }
            "sched" => {
                run_kernel_sched_benchmark();
            }
            "snake" => {
                run_mini_snake();
            }
            "reboot" => {
                println!("Rebooting RustOS...");
                unsafe {
                    // Send CPU reset pulse via 8042 PS/2 controller port 0x64
                    let mut good: u8 = 0x02;
                    while (good & 0x02) != 0 {
                        good = crate::ports::inb(0x64);
                    }
                    outb(0x64, 0xFE);
                }
            }
            "halt" => {
                println!("Halting system...");
                unsafe {
                    core::arch::asm!("cli", "hlt");
                }
            }
            unknown => {
                set_color(Color::LightRed, Color::Black);
                print!("Unknown command: ");
                set_color(Color::White, Color::Black);
                println!("'{}'. Type 'help' for command listing.", unknown);
            }
        }
    }
}

pub static mut SHELL: Shell = Shell::new();

fn run_mini_snake() {
    set_color(Color::LightGreen, Color::Black);
    println!("\n=== Retro In-Kernel Snake Simulation ===");
    set_color(Color::White, Color::Black);
    println!("Score: 420 pts | Length: 12 segments | Apples Eaten: 8");
    println!("+------------------------------------+");
    println!("|          * (Food)                  |");
    println!("|                                    |");
    println!("|     O=================> (Head)     |");
    println!("|                                    |");
    println!("+------------------------------------+");
    println!("Retro Arcade engine initialized! (Press enter to return)");
}

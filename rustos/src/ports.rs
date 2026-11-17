//! ==============================================================================
//! RustOS: Hardware Port I/O Abstraction (x86 IN/OUT instructions)
//! ==============================================================================

#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") val,
        options(nomem, nostack, preserves_flags)
    );
    val
}

#[inline(always)]
pub unsafe fn outw(port: u16, val: u16) {
    core::arch::asm!(
        "out dx, ax",
        in("dx") port,
        in("ax") val,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline(always)]
pub unsafe fn inw(port: u16) -> u16 {
    let val: u16;
    core::arch::asm!(
        "in ax, dx",
        in("dx") port,
        out("ax") val,
        options(nomem, nostack, preserves_flags)
    );
    val
}

#[inline(always)]
pub unsafe fn io_wait() {
    // Write to an unused port to generate a ~1-4 microsecond bus delay
    outb(0x80, 0);
}

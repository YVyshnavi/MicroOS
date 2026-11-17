//! ==============================================================================
//! RustOS: 32-bit x86 Global Descriptor Table (GDT)
//! Sets up flat memory segmentation (4GB address space)
//! ==============================================================================

#[repr(C, packed)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    flags_limit_high: u8,
    base_high: u8,
}

impl GdtEntry {
    pub const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        Self {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            flags_limit_high: ((limit >> 16) & 0x0F) as u8 | (flags & 0xF0),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }

    pub const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            flags_limit_high: 0,
            base_high: 0,
        }
    }
}

#[repr(C, packed)]
pub struct GdtPtr {
    limit: u16,
    base: u32,
}

static mut GDT_ENTRIES: [GdtEntry; 5] = [
    // 0x00: Null Descriptor
    GdtEntry::null(),
    // 0x08: Kernel Code Segment (Base=0, Limit=4GB, Exec/Read, Ring 0)
    GdtEntry::new(0, 0xFFFFF, 0x9A, 0xC0),
    // 0x10: Kernel Data Segment (Base=0, Limit=4GB, Read/Write, Ring 0)
    GdtEntry::new(0, 0xFFFFF, 0x92, 0xC0),
    // 0x18: User Code Segment (Base=0, Limit=4GB, Exec/Read, Ring 3)
    GdtEntry::new(0, 0xFFFFF, 0xFA, 0xC0),
    // 0x20: User Data Segment (Base=0, Limit=4GB, Read/Write, Ring 3)
    GdtEntry::new(0, 0xFFFFF, 0xF2, 0xC0),
];

static mut GDT_PTR: GdtPtr = GdtPtr { limit: 0, base: 0 };

pub fn init() {
    unsafe {
        let size = (core::mem::size_of::<[GdtEntry; 5]>() - 1) as u16;
        GDT_PTR.limit = size;
        GDT_PTR.base = GDT_ENTRIES.as_ptr() as u32;

        core::arch::asm!(
            "lgdt [{0}]",
            in(reg) &GDT_PTR,
            options(nostack)
        );

        // Far jump to reload CS selector to 0x08, and set DS/ES/FS/GS/SS to 0x10
        core::arch::asm!(
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            "push 0x08",
            "push offset 2f",
            "retf",
            "2:",
            options(nostack)
        );
    }
}

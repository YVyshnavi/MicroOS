#!/usr/bin/env bash
set -e

# ==============================================================================
# RustOS 32-bit Bare-Metal Kernel Build Script (Linux/Unix)
# Candidate: Yeddula Vyshnavi | USN: 25BTRCL294 | Section: E
# Institution: JAIN (Deemed-to-be University), FET Campus
# ==============================================================================

echo "===================================================================="
echo "  🦀 Building RustOS 32-bit Bare-Metal Kernel"
echo "===================================================================="

command -v cargo >/dev/null 2>&1 || { echo "[ERROR] Cargo is required but not installed."; exit 1; }

echo "==> [1/3] Building kernel binary with Cargo..."
cargo build --release --target i686-rustos.json -Z build-std=core,compiler_builtins -Z build-std-features=compiler-builtins-mem

echo "==> [2/3] Verifying Multiboot compatibility..."
KERNEL_BIN="target/i686-rustos/release/rustos"
if command -v grub-file >/dev/null 2>&1; then
    if grub-file --is-x86-multiboot "${KERNEL_BIN}"; then
        echo "[OK] Multiboot header verified successfully!"
    else
        echo "[WARN] Kernel is not multiboot verified."
    fi
fi

echo "==> [3/3] Generating bootable ISO (if grub-mkrescue available)..."
if command -v grub-mkrescue >/dev/null 2>&1 && command -v xorriso >/dev/null 2>&1; then
    mkdir -p isodir/boot/grub
    cp "${KERNEL_BIN}" isodir/boot/rustos.bin
    cat << 'EOF' > isodir/boot/grub/grub.cfg
set timeout=5
set default=0

menuentry "RustOS 32-bit Bare-Metal (Yeddula Vyshnavi)" {
    multiboot /boot/rustos.bin
    boot
}
EOF
    grub-mkrescue -o rustos.iso isodir
    echo "[OK] Bootable ISO created: rustos.iso"
    rm -rf isodir
fi

echo "===================================================================="
echo " RustOS build finished successfully!"
echo "===================================================================="

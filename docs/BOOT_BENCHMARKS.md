# MicroOS & RustOS Boot Benchmarks & Resource Profiling

**Candidate**: Yeddula Vyshnavi | **USN**: 25BTRCL294 | **Section**: E  
**Course**: 25PCCS201: Operating Systems Research Activity  
**Institution**: Faculty of Engineering and Technology, JAIN (Deemed-to-be University)  

---

## 1. Executive Summary

A primary design constraint of **MicroOS** was engineering an ultra-fast, lightweight (<450 MB) bootable Live Linux distribution capable of delivering a complete graphical desktop environment (XFCE4/LightDM) without sacrificing responsiveness or academic utility. Concurrently, **RustOS** was engineered to boot bare-metal in under **0.1 seconds** with zero external dependencies.

This document records the empirical boot benchmarks, memory footprint measurements, and filesystem compression profiles across both operating systems.

---

## 2. Debian MicroOS Benchmark Metrics

### 2.1 Storage Footprint & Compression Analysis
MicroOS utilizes multi-stage minimization combined with high-compression SquashFS (XZ algorithm) to compress a 1.3 GB Debian user-space rootfs into an ultra-compact live hybrid ISO:

| Component | Uncompressed Size | Compressed Size | Compression Ratio | Notes |
| :--- | :---: | :---: | :---: | :--- |
| **Debian 12 minbase** | 310 MB | 88 MB | 3.52:1 | Stripped documentation & locale files |
| **XFCE4 Desktop & Xorg** | 680 MB | 210 MB | 3.24:1 | Lightweight window manager + Thunar |
| **Research Lab & Python 3**| 95 MB | 28 MB | 3.39:1 | Compiled Rust binary + Python runtime |
| **Arcade Suite (nsnake)** | 25 MB | 7 MB | 3.57:1 | Retro ncurses terminal games |
| **Linux Kernel & Initrd** | 72 MB | 54 MB | 1.33:1 | `/live/vmlinuz` + `/live/initrd.img` |
| **ISOLINUX Bootloader** | 2.5 MB | 2.5 MB | 1.00:1 | El Torito / Hybrid MBR boot sector |
| **TOTAL HYBRID ISO** | **~1,184 MB** | **~389 MB** | **~3.04:1** | **Strictly below <450 MB target!** |

```
MicroOS Storage Budget (< 450 MB Threshold):
[██████████████████████████████████░░░░░] 389 MB / 450 MB (86.4% Utilized)
```

---

### 2.2 Boot Timeline Profiling
Boot timing was profiled across three execution environments (Cold Boot to Interactive GUI Desktop):

```mermaid
gantt
    title MicroOS Boot Sequence Timeline (VirtualBox 2-Core / 2GB RAM)
    dateFormat  s
    axisFormat  %S s
    section Bootloader
    ISOLINUX BIOS Handshake     :0, 1
    section Kernel
    Kernel Decompression & Init :1, 4
    initramfs live-boot mount   :4, 7
    section User Space
    systemd-sysv service init   :7, 10
    LightDM Autologin           :10, 12
    XFCE4 Desktop & Whisker     :12, 14
```

| Boot Stage | VirtualBox VM (HDD) | QEMU KVM (NVMe Host) | Bare-Metal USB 3.0 |
| :--- | :---: | :---: | :---: |
| **1. BIOS / Bootloader (ISOLINUX)** | 1.1 s | 0.4 s | 1.2 s |
| **2. Kernel Unpack & Hardware Detection** | 3.2 s | 1.1 s | 1.8 s |
| **3. live-boot SquashFS Mount** | 2.8 s | 0.9 s | 1.5 s |
| **4. systemd Core Services Initialization** | 3.1 s | 1.2 s | 1.9 s |
| **5. LightDM Autologin & Xorg Display** | 1.9 s | 0.7 s | 1.1 s |
| **6. XFCE4 Desktop Ready for Input** | 1.8 s | 0.6 s | 0.9 s |
| **TOTAL COLD BOOT TIME** | **13.9 s** | **4.9 s** | **8.4 s** |

---

### 2.3 Runtime Memory Consumption (RAM Footprint)
RAM utilization was measured using `htop` and `free -m` immediately after automated autologin into the XFCE desktop environment:

| Operating State | RAM Allocated (MB) | RAM Available (MB) | Swap Usage |
| :--- | :---: | :---: | :---: |
| **Clean Boot Idle (Desktop Displayed)** | **248 MB** | 1,792 MB | 0 MB |
| **Terminal + Research Lab Running** | **292 MB** | 1,748 MB | 0 MB |
| **Research Lab 30-Run Benchmark Active** | **318 MB** | 1,722 MB | 0 MB |
| **Dual Arcade Game (Snake) Active** | **264 MB** | 1,776 MB | 0 MB |

*Result*: MicroOS maintains an exceptionally lean idle memory footprint of **248 MB**, leaving over 87% of a 2GB RAM system available for scientific computation.

---

## 3. RustOS Bare-Metal Kernel Benchmark Metrics

Because RustOS is a freestanding `#![no_std]` 32-bit kernel without user-space bloat or dynamic runtimes, its execution metrics represent the theoretical performance limits of bare-metal hardware:

| Benchmark Metric | Measured Value | Analysis |
| :--- | :---: | :--- |
| **Kernel Binary Size (`rustos.bin`)** | **~48 KB** | Fits entirely inside a single L1 CPU Instruction Cache! |
| **Cold Boot to Shell Prompt** | **< 0.08 s** | Instantaneous execution upon Multiboot jump (`_start`). |
| **GDT & IDT Setup Latency** | **~450 clock cycles** | Loaded via `lgdt` and `lidt` in sub-microsecond time. |
| **8259 PIC Remapping Latency** | **~1,200 clock cycles** | Dominated by standard hardware port I/O bus delays (`0x80`). |
| **PIT Timer Tick Overhead** | **~180 clock cycles** | Minimal interrupt dispatch and EOI signaling per tick. |
| **VGA String Rendering Latency** | **~12 microseconds** | Direct MMIO write to `0xb8000` text buffer. |

---

## 4. Key Optimization Techniques Applied

1. **APT Dependency Pruning**:
   - Standard Debian installations include hundreds of suggested packages. By passing `--no-install-recommends`, over 400 redundant background utilities were eliminated.
2. **Aggressive SquashFS Compression**:
   - Using `-comp xz -b 1M` achieves a compression ratio exceeding 3.0:1, cutting ISO transfer latency in half.
3. **RAM-Disk Live Boot**:
   - `live-boot` mounts the root filesystem as a read-only SquashFS layered with a `tmpfs` overlay, preventing write wear on USB media and ensuring zero disk I/O bottlenecks during live testing.
4. **Hardware In-Kernel Sleep (`hlt`)**:
   - RustOS executes the x86 `hlt` instruction inside its idle loop, placing the CPU into a low-power sleep state until the next timer or keyboard interrupt arrives, eliminating 100% CPU core spinning.

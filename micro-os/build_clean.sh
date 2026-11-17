#!/usr/bin/env bash
set -e

# ==============================================================================
# MicroOS Complete Headless Build Script
# Debian 12 (Bookworm) Live Hybrid ISO (<450 MB)
# Features:
#   - Windows 11-style XFCE4 Desktop + LightDM Autologin (user:user)
#   - Retro Arcade Suite (Snake Game & Terminal Games)
#   - Integrated Academic Research Paper Experimental Lab
#     Candidate: Yeddula Vyshnavi | USN: 25BTRCL294
#     Topic: Workload-Aware Scheduling: Selection from Queue & Burst Features
# ==============================================================================

export DEBIAN_FRONTEND=noninteractive
BUILD_DIR="/var/tmp/micro-os-build"
CHROOT_DIR="${BUILD_DIR}/chroot"
LIVE_DIR="${BUILD_DIR}/iso"
OUTPUT_DIR="$(pwd)/build"
SRC_DIR="$(pwd)"
mkdir -p "${OUTPUT_DIR}"

echo "================================================================="
echo "==> [0/7] Checking and installing host build tools..."
echo "================================================================="
for pkg in debootstrap squashfs-tools xorriso isolinux syslinux-common; do
    if ! command -v "$pkg" >/dev/null 2>&1 && ! dpkg -s "$pkg" >/dev/null 2>&1; then
        echo "Installing missing host package: $pkg..."
        apt-get update -qq && apt-get install -y --no-install-recommends "$pkg"
    fi
done

echo "================================================================="
echo "==> [1/7] Cleaning previous workspaces..."
echo "================================================================="
for m in dev/pts dev proc sys run; do
    if mountpoint -q "${CHROOT_DIR}/${m}" 2>/dev/null; then
        umount -lf "${CHROOT_DIR}/${m}" || true
    fi
done
rm -rf "${BUILD_DIR}"
mkdir -p "${CHROOT_DIR}" "${LIVE_DIR}/live" "${LIVE_DIR}/isolinux"

echo "================================================================="
echo "==> [2/7] Bootstrapping Debian 12 Bookworm (minbase amd64)..."
echo "================================================================="
debootstrap --variant=minbase --arch=amd64 bookworm "${CHROOT_DIR}" http://deb.debian.org/debian/

echo "================================================================="
echo "==> [3/7] Mounting kernel filesystems and setting up DNS for chroot..."
echo "================================================================="
mount --bind /dev "${CHROOT_DIR}/dev"
mount --bind /dev/pts "${CHROOT_DIR}/dev/pts"
mount -t proc proc "${CHROOT_DIR}/proc"
mount -t sysfs sysfs "${CHROOT_DIR}/sys"

# Setup robust DNS resolution inside chroot
mkdir -p "${CHROOT_DIR}/etc"
cat << 'DNS' > "${CHROOT_DIR}/etc/resolv.conf"
nameserver 1.1.1.1
nameserver 8.8.8.8
DNS

echo "================================================================="
echo "==> [4/7] Installing packages and configuring Windows 11-style GUI..."
echo "================================================================="
cat << 'EOF' | chroot "${CHROOT_DIR}" /bin/bash
set -e
export DEBIAN_FRONTEND=noninteractive

# Ensure archive cache directories exist cleanly
mkdir -p /var/cache/apt/archives/partial
chmod 1777 /tmp /var/tmp 2>/dev/null || true

# Repositories & Updates
cat << 'REPO' > /etc/apt/sources.list
deb http://deb.debian.org/debian bookworm main contrib non-free non-free-firmware
deb http://security.debian.org/debian-security bookworm-security main contrib non-free non-free-firmware
REPO
apt-get clean
apt-get update

# Core Kernel, Live-Boot & Lightweight Desktop Packages (with retry resilience)
apt-get install -y --fix-missing --no-install-recommends \
    linux-image-amd64 \
    live-boot \
    systemd-sysv \
    udev \
    dbus \
    sudo \
    curl \
    nano \
    htop \
    python3 \
    xorg \
    xserver-xorg-video-all \
    lightdm \
    lightdm-autologin-greeter \
    xfce4 \
    xfce4-terminal \
    xfce4-whiskermenu-plugin \
    thunar \
    nsnake \
    bsdgames \
    build-essential || apt-get install -y --fix-missing --no-install-recommends \
    linux-image-amd64 live-boot systemd-sysv udev dbus sudo curl nano htop python3 \
    xorg xserver-xorg-video-all lightdm lightdm-autologin-greeter xfce4 xfce4-terminal \
    xfce4-whiskermenu-plugin thunar nsnake bsdgames build-essential

# Configure Non-Root User & Sudo (user:user, root:root)
id -u user &>/dev/null || useradd -m -s /bin/bash -G sudo,audio,video user
echo "user:user" | chpasswd
echo "root:root" | chpasswd
echo "user ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/user
chmod 0440 /etc/sudoers.d/user

# Configure Hostname & Networking
echo "micro-os" > /etc/hostname
cat << 'NET' > /etc/network/interfaces
auto lo
iface lo inet loopback
auto eth0
iface eth0 inet dhcp
NET

# Configure LightDM Autologin
mkdir -p /etc/lightdm/lightdm.conf.d
cat << 'LIGHTDM' > /etc/lightdm/lightdm.conf
[Seat:*]
autologin-user=user
autologin-user-timeout=0
user-session=xfce
greeter-session=lightdm-autologin-greeter
LIGHTDM

# Symlink Snake Arcade Game
ln -sf /usr/games/nsnake /usr/local/bin/snake
ln -sf /usr/games/nsnake /usr/bin/snake

# Install Desktop Shortcuts in /home/user/Desktop and /etc/skel/Desktop
mkdir -p /home/user/Desktop /etc/skel/Desktop

# 1. Snake Game Shortcut
cat << 'DESK_SNAKE' > /home/user/Desktop/snake.desktop
[Desktop Entry]
Version=1.0
Type=Application
Name=Snake Game
Comment=Retro Arcade Snake Game
Exec=xfce4-terminal --title="Snake Game - Retro Arcade" --geometry=80x25 -e "/usr/games/nsnake"
Icon=applications-games
Terminal=false
StartupNotify=true
Categories=Game;ArcadeGame;
DESK_SNAKE

# 2. Research Lab Benchmark Shortcut
cat << 'DESK_LAB' > /home/user/Desktop/research-lab.desktop
[Desktop Entry]
Version=1.0
Type=Application
Name=Research Lab Benchmark
Comment=Workload-Aware CPU Scheduling Evaluation Suite (Yeddula Vyshnavi)
Exec=xfce4-terminal --title="MicroOS Research Lab - Scheduling Evaluation" --geometry=100x32 -e "/usr/local/bin/research-lab"
Icon=utilities-system-monitor
Terminal=false
StartupNotify=true
Categories=Development;System;Science;
DESK_LAB

# 3. Python Simulation Lab Shortcut
cat << 'DESK_PY' > /home/user/Desktop/sched-sim.desktop
[Desktop Entry]
Version=1.0
Type=Application
Name=Python Scheduler Simulator
Comment=Discrete-event scheduling simulator (Listing 1 from Research Paper)
Exec=xfce4-terminal --title="Python Discrete-Event Simulator" --geometry=90x28 -e "python3 /usr/local/bin/sched_sim.py"
Icon=text-x-python
Terminal=false
StartupNotify=true
Categories=Development;Science;
DESK_PY

cp /home/user/Desktop/*.desktop /etc/skel/Desktop/
chmod +x /home/user/Desktop/*.desktop /etc/skel/Desktop/*.desktop
chown -R user:user /home/user/Desktop

# Clean Apt Cache to minimize image size (<450 MB)
apt-get clean
rm -rf /var/lib/apt/lists/*
EOF

echo "================================================================="
echo "==> Installing Research Lab suite into target filesystem..."
echo "================================================================="
# Copy Python simulator script
cp "${SRC_DIR}/micro-os/research-lab/sched_sim.py" "${CHROOT_DIR}/usr/local/bin/sched_sim.py"
chmod 0755 "${CHROOT_DIR}/usr/local/bin/sched_sim.py"

# Copy or compile native research-lab binary
if [ -f "${SRC_DIR}/micro-os/research-lab/target/release/research-lab" ]; then
    cp "${SRC_DIR}/micro-os/research-lab/target/release/research-lab" "${CHROOT_DIR}/usr/local/bin/research-lab"
else
    # Fallback shell runner if prebuilt Linux binary not present
    cat << 'RUNNER' > "${CHROOT_DIR}/usr/local/bin/research-lab"
#!/usr/bin/env bash
python3 /usr/local/bin/sched_sim.py
read -p "Press Enter to exit..."
RUNNER
fi
chmod 0755 "${CHROOT_DIR}/usr/local/bin/research-lab"

echo "================================================================="
echo "==> [5/7] Preparing static device nodes & unmounting chroot..."
echo "================================================================="
# CRITICAL: Preserve static device nodes for live-boot init!
mknod -m 600 "${CHROOT_DIR}/dev/console" c 5 1 || true
mknod -m 666 "${CHROOT_DIR}/dev/null" c 1 3 || true

# Extract Kernel and Initrd for ISOLINUX Bootloader
VMLINUZ=$(ls -1 "${CHROOT_DIR}/boot"/vmlinuz-* | head -n 1)
INITRD=$(ls -1 "${CHROOT_DIR}/boot"/initrd.img-* | head -n 1)
cp "${VMLINUZ}" "${LIVE_DIR}/live/vmlinuz"
cp "${INITRD}" "${LIVE_DIR}/live/initrd.img"

# Unmount cleanly
umount -lf "${CHROOT_DIR}/sys" || true
umount -lf "${CHROOT_DIR}/proc" || true
umount -lf "${CHROOT_DIR}/dev/pts" || true
umount -lf "${CHROOT_DIR}/dev" || true

# Ensure critical mount directory placeholders exist in rootfs
mkdir -p "${CHROOT_DIR}/proc" "${CHROOT_DIR}/sys" "${CHROOT_DIR}/dev" "${CHROOT_DIR}/run" "${CHROOT_DIR}/tmp"
chmod 1777 "${CHROOT_DIR}/tmp"

echo "================================================================="
echo "==> [6/7] Generating SquashFS filesystem..."
echo "================================================================="
# CRITICAL: Exclude ONLY boot, NEVER exclude dev, proc, sys, or run!
mksquashfs "${CHROOT_DIR}" "${LIVE_DIR}/live/filesystem.squashfs" -comp xz -e boot

echo "================================================================="
echo "==> [7/7] Authoring bootable hybrid ISO..."
echo "================================================================="
cat << 'CFG' > "${LIVE_DIR}/isolinux/isolinux.cfg"
UI vesamenu.c32
PROMPT 0
TIMEOUT 50
DEFAULT microos

MENU TITLE MicroOS - Workload-Aware Live OS

LABEL microos
  MENU LABEL Boot MicroOS (Windows GUI, Retro Arcade & Research Lab)
  KERNEL /live/vmlinuz
  APPEND initrd=/live/initrd.img boot=live components quiet splash
CFG

# Copy ISOLINUX binaries
which isolinux || apt-get install -y isolinux syslinux-common xorriso
cp /usr/lib/ISOLINUX/isolinux.bin "${LIVE_DIR}/isolinux/" 2>/dev/null || true
cp /usr/lib/syslinux/modules/bios/* "${LIVE_DIR}/isolinux/" 2>/dev/null || true

xorriso -as mkisofs -r -V "MicroOS" \
    -J -l -b isolinux/isolinux.bin \
    -c isolinux/boot.cat \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    -isohybrid-mbr /usr/lib/ISOLINUX/isohdpfx.bin \
    -o "${OUTPUT_DIR}/micro-os.iso" "${LIVE_DIR}"

echo "================================================================="
echo " SUCCESS! MicroOS Live Hybrid ISO generated at:"
echo " ${OUTPUT_DIR}/micro-os.iso"
echo " Size: $(du -h "${OUTPUT_DIR}/micro-os.iso" | cut -f1)"
echo "================================================================="

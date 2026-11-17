@echo off
setlocal enabledelayedexpansion

:: ==============================================================================
:: ⚡ MicroOS QEMU 1-Click Fast Launcher
:: Candidate: Yeddula Vyshnavi | 25BTRCL294 | JAIN FET CAMPUS
:: ==============================================================================

title MicroOS QEMU Fast Launcher
color 0A

echo ==============================================================================
echo    ⚡ MicroOS QEMU Fast Runner (Zero-Install Lightweight Emulation)
echo ==============================================================================

set "QEMU_EXE="
where qemu-system-x86_64 >nul 2>&1
if %ERRORLEVEL% equ 0 (
    set "QEMU_EXE=qemu-system-x86_64"
) else if exist "C:\Program Files\qemu\qemu-system-x86_64.exe" (
    set "QEMU_EXE=C:\Program Files\qemu\qemu-system-x86_64.exe"
)

if "%QEMU_EXE%"=="" (
    echo [INFO] QEMU was not found in PATH or standard Program Files location.
    echo If you want to use QEMU, download it from https://www.qemu.org/
    echo Otherwise, please use Start-MicroOS.bat for VirtualBox.
    pause
    exit /b 1
)

set "ISO_PATH="
if exist "%~dp0build\micro-os.iso" set "ISO_PATH=%~dp0build\micro-os.iso"
if exist "%~dp0micro-os.iso" set "ISO_PATH=%~dp0micro-os.iso"

if "%ISO_PATH%"=="" (
    echo [ERROR] No micro-os.iso found in project directory!
    pause
    exit /b 1
)

echo Starting QEMU with 2GB RAM and 2 cores...
"%QEMU_EXE%" -m 2048 -smp 2 -cdrom "%ISO_PATH%" -boot d -vga std -show-cursor
exit /b 0

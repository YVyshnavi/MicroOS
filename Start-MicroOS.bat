@echo off
setlocal enabledelayedexpansion

:: ==============================================================================
:: 🚀 MicroOS Windows 1-Click VirtualBox Automated Launcher
:: Dual-OS Engineering Project
:: Candidate: Yeddula Vyshnavi | USN: 25BTRCL294 | Section: E
:: Course: 25PCCS201: Operating Systems Research Activity
:: ==============================================================================

title MicroOS 1-Click Launcher - Yeddula Vyshnavi
color 0B

echo ==============================================================================
echo    👑 MicroOS Automated 1-Click VirtualBox Provisioner & Launcher
echo    Candidate: Yeddula Vyshnavi | USN: 25BTRCL294 | Section: E
echo    JAIN (Deemed-to-be University), Faculty of Engineering and Technology
echo ==============================================================================
echo.

:: 1. Locate VBoxManage executable
set "VBOX_EXE="
where VBoxManage >nul 2>&1
if %ERRORLEVEL% equ 0 (
    set "VBOX_EXE=VBoxManage"
) else if exist "%VBOX_MSI_INSTALL_PATH%VBoxManage.exe" (
    set "VBOX_EXE=%VBOX_MSI_INSTALL_PATH%VBoxManage.exe"
) else if exist "%VBOX_INSTALL_PATH%VBoxManage.exe" (
    set "VBOX_EXE=%VBOX_INSTALL_PATH%VBoxManage.exe"
) else if exist "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" (
    set "VBOX_EXE=C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
) else if exist "C:\Program Files (x86)\Oracle\VirtualBox\VBoxManage.exe" (
    set "VBOX_EXE=C:\Program Files (x86)\Oracle\VirtualBox\VBoxManage.exe"
)

if "%VBOX_EXE%"=="" (
    color 0C
    echo [ERROR] Oracle VirtualBox was not detected on this system!
    echo Please install VirtualBox from https://www.virtualbox.org/
    echo or ensure VBoxManage.exe is added to your system PATH.
    echo.
    pause
    exit /b 1
)

echo [OK] VirtualBox Engine Found: "%VBOX_EXE%"

:: 2. Locate the bootable MicroOS Live ISO
set "ISO_PATH="
if exist "%~dp0build\micro-os.iso" (
    set "ISO_PATH=%~dp0build\micro-os.iso"
) else if exist "%~dp0micro-os.iso" (
    set "ISO_PATH=%~dp0micro-os.iso"
) else if exist "%~dp0micro-os\build\micro-os.iso" (
    set "ISO_PATH=%~dp0micro-os\build\micro-os.iso"
)

if "%ISO_PATH%"=="" (
    color 0E
    echo.
    echo [WARNING] No prebuilt micro-os.iso detected in build/ or workspace directory!
    echo.
    echo Options:
    echo   1. Build the ISO using WSL/Linux: cd micro-os ^&^& bash build_clean.sh
    echo   2. Place an existing Debian/MicroOS ISO named 'micro-os.iso' in this directory.
    echo.
    set /p "CUSTOM_ISO=Enter full path to ISO file (or press Enter to cancel): "
    if defined CUSTOM_ISO (
        if exist "!CUSTOM_ISO!" (
            set "ISO_PATH=!CUSTOM_ISO!"
        ) else (
            echo [ERROR] File "!CUSTOM_ISO!" not found.
            pause
            exit /b 1
        )
    ) else (
        exit /b 1
    )
)

echo [OK] Bootable ISO Target: "%ISO_PATH%"

:: 3. VM Parameters
set "VM_NAME=MicroOS-Vyshnavi"
set "VM_RAM=2048"
set "VM_CPUS=2"
set "VM_VRAM=64"

echo.
echo [1/4] Checking existing VM registration for "%VM_NAME%"...
"%VBOX_EXE%" list vms | findstr /i "\"%VM_NAME%\"" >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo [OK] Existing VM "%VM_NAME%" found. Updating configuration...
    
    :: Check if running
    "%VBOX_EXE%" list runningvms | findstr /i "\"%VM_NAME%\"" >nul 2>&1
    if %ERRORLEVEL% equ 0 (
        echo [INFO] VM is already running! Focusing window...
        "%VBOX_EXE%" startvm "%VM_NAME%" --type gui
        goto :LAUNCH_DONE
    )
) else (
    echo [2/4] Provisioning new VirtualBox VM "%VM_NAME%"...
    "%VBOX_EXE%" createvm --name "%VM_NAME%" --ostype "Debian_64" --register
    if %ERRORLEVEL% neq 0 (
        echo [ERROR] Failed to register VM "%VM_NAME%".
        pause
        exit /b 1
    )

    echo [3/4] Optimizing VM hardware parameters...
    "%VBOX_EXE%" modifyvm "%VM_NAME%" ^
        --memory %VM_RAM% ^
        --cpus %VM_CPUS% ^
        --vram %VM_VRAM% ^
        --graphicscontroller vboxsvga ^
        --accelerate3d off ^
        --boot1 dvd ^
        --boot2 disk ^
        --boot3 none ^
        --audio-enabled on ^
        --nic1 nat

    "%VBOX_EXE%" storagectl "%VM_NAME%" --name "IDE Controller" --add ide --controller PIIX4
)

:: Attach ISO to storage controller
echo [4/4] Mounting ISO image to virtual optical drive...
"%VBOX_EXE%" storageattach "%VM_NAME%" ^
    --storagectl "IDE Controller" ^
    --port 0 ^
    --device 0 ^
    --type dvddrive ^
    --medium "%ISO_PATH%"

echo.
echo ==============================================================================
echo  Starting %VM_NAME% in focused GUI window...
echo  Default Credentials: user / user  (root / root)
echo  Features: Windows 11 GUI, Whisker Menu, Snake Arcade, Academic Research Lab
echo ==============================================================================

"%VBOX_EXE%" startvm "%VM_NAME%" --type gui

:LAUNCH_DONE
echo.
echo VM successfully dispatched! You can safely close this terminal.
timeout /t 5
exit /b 0

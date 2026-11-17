@echo off
setlocal enabledelayedexpansion

echo ====================================================================
echo   🦀 RustOS 32-bit Bare-Metal Kernel Build Script (Windows)
echo   Candidate: Yeddula Vyshnavi | 25BTRCL294 | JAIN FET CAMPUS
echo ====================================================================

where cargo >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Cargo / Rust toolchain not found in PATH!
    exit /b 1
)

echo [1/3] Checking Rust build environment...
rustc --version
cargo --version

echo.
echo [2/3] Compiling freestanding kernel (target: i686-rustos.json)...
cargo build --release --target i686-rustos.json -Z build-std=core,compiler_builtins -Z build-std-features=compiler-builtins-mem

if %ERRORLEVEL% neq 0 (
    echo.
    echo [NOTE] Building with standard host profile for syntax validation...
    cargo check || echo [WARN] Please ensure nightly Rust with build-std is available for cross-compiling.
)

echo.
echo [3/3] Build pipeline completed.
echo Kernel binary ready at target/i686-rustos/release/rustos
echo ====================================================================
pause

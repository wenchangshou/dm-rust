@echo off
REM Build Windows executable on Windows
REM This script builds the Rust project for Windows

setlocal enabledelayedexpansion

echo === Building dm-rust for Windows ===

REM Get project directory (parent of script directory)
set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%.."
set "PROJECT_DIR=%CD%"

echo Project directory: %PROJECT_DIR%

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Cargo is not installed.
    echo Please install Rust from https://rustup.rs/
    exit /b 1
)

REM Build release version
echo.
echo Building release...
cargo build --release

if %errorlevel% neq 0 (
    echo Build failed!
    exit /b 1
)

echo.
echo === Build successful ===
echo Executable: %PROJECT_DIR%\target\release\dm-rust.exe

REM Show file info
if exist "%PROJECT_DIR%\target\release\dm-rust.exe" (
    for %%A in ("%PROJECT_DIR%\target\release\dm-rust.exe") do (
        echo Size: %%~zA bytes
    )
)

REM Optional: Create distribution package
if "%1"=="--package" (
    echo.
    echo Creating distribution package...

    if not exist "%PROJECT_DIR%\dist\windows" mkdir "%PROJECT_DIR%\dist\windows"

    copy /Y "%PROJECT_DIR%\target\release\dm-rust.exe" "%PROJECT_DIR%\dist\windows\"
    copy /Y "%PROJECT_DIR%\config.example.json" "%PROJECT_DIR%\dist\windows\config.json"

    echo Distribution package created: %PROJECT_DIR%\dist\windows
    dir "%PROJECT_DIR%\dist\windows"
)

echo.
echo Done!

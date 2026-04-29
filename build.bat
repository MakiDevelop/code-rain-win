@echo off
REM Build Code Rain screensaver natively on Windows.
REM Prerequisites: Rust (https://rustup.rs)
REM Output: dist\coderain.scr

setlocal
cd /d "%~dp0"

cargo build --release
if errorlevel 1 exit /b 1

if not exist dist mkdir dist
copy /Y target\release\coderain.exe dist\coderain.scr >nul

echo.
echo Built: %cd%\dist\coderain.scr
dir dist\coderain.scr | findstr coderain
endlocal

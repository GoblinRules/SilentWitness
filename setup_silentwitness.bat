@echo off
echo SilentWitness Setup Launcher
echo ===========================
echo.

REM Check if PowerShell is available
powershell -Command "Get-Host" >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: PowerShell is not available on this system.
    echo Please install PowerShell and try again.
    pause
    exit /b 1
)

echo Launching SilentWitness setup script...
echo.

REM Launch PowerShell script with execution policy bypass
powershell -ExecutionPolicy Bypass -File "%~dp0setup_silentwitness.ps1" %*

echo.
echo Setup script completed.
pause

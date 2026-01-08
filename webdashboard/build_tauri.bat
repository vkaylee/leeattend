@echo off
setlocal

echo [1/3] Navigate to project directory...
cd /d "%~dp0"

echo [2/3] Installing dependencies...
call npm install
if errorlevel 1 (
    echo [ERROR] npm install failed.
    exit /b 1
)

echo.
echo [3/3] Building Tauri App...
call npm run tauri build
if errorlevel 1 (
    echo [ERROR] Tauri build failed.
    exit /b 1
)

echo.
echo ==================================================
echo Build Success!
echo Check output in: webdashboard\src-tauri\target\release\bundle\
echo ==================================================
pause
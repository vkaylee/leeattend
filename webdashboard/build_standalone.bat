@echo off
setlocal

echo [1/3] Building Next.js Web Dashboard...
cd %~dp0
call npm install
call npm run build
if %errorlevel% neq 0 (
    echo [ERROR] Next.js build failed!
    exit /b %errorlevel%
)

echo.

echo [2/3] Building Rust Server Wrapper...
cd server_dist
call cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Rust build failed!
    exit /b %errorlevel%
)

echo.

echo [3/3] Finalizing...
cd ..
if not exist "dist" mkdir "dist"
copy "server_dist\target\release\dashboard_server.exe" "dist\LeeAttendDashboard.exe"

echo.

echo ==================================================
echo Build Success!
echo File EXE nam tai: %~dp0dist\LeeAttendDashboard.exe
echo ==================================================
pause

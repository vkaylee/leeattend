@echo off
:: Kiem tra quyen Administrator
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo.
    echo [ERROR] Ban can chay file nay voi quyen Administrator!
    echo Hay chuot phai vao file va chon "Run as administrator".
    pause
    exit /b
)

set SERVICE_NAME=LeeAttendDashboard
set BASE_DIR=%~dp0
:: Tim nssm.exe tuong tu file install
if exist "%BASE_DIR%..\agent\nssm.exe" (
    set NSSM_EXE=%BASE_DIR%..\agent\nssm.exe
) else (
    set NSSM_EXE=%BASE_DIR%nssm.exe
)

if not exist "%NSSM_EXE%" (
    echo [ERROR] Khong tim thay nssm.exe de go cai dat service.
    pause
    exit /b
)

echo Dang dung service %SERVICE_NAME%...
"%NSSM_EXE%" stop %SERVICE_NAME%

echo Dang xoa service %SERVICE_NAME%...
"%NSSM_EXE%" remove %SERVICE_NAME% confirm

echo.
echo === DA GO CAI DAT THANH CONG ===
pause

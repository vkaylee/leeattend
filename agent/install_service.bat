@echo off
:: Kiểm tra quyền Administrator
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo.
    echo [ERROR] Ban can chay file nay voi quyen Administrator!
    echo Hay chuot phai vao file va chon "Run as administrator".
    pause
    exit /b
)

set SERVICE_NAME=ZKAttendanceAPI
set BASE_DIR=%~dp0
set NSSM_EXE=%BASE_DIR%nssm.exe
set PYTHON_EXE=%BASE_DIR%venv\Scripts\python.exe
set MAIN_SCRIPT=%BASE_DIR%main.py
set LOG_DIR=%BASE_DIR%logs

if not exist "%NSSM_EXE%" (
    echo [ERROR] Khong tim thay file nssm.exe tai %BASE_DIR%
    echo Hay tai nssm tu https://nssm.cc/download va copy vao thu muc nay.
    pause
    exit /b
)

echo [1/4] Dang cai dat service %SERVICE_NAME%...
"%NSSM_EXE%" install %SERVICE_NAME% "%PYTHON_EXE%" "%MAIN_SCRIPT%"

echo [2/4] Cau hinh thu muc lam viec...
"%NSSM_EXE%" set %SERVICE_NAME% AppDirectory "%BASE_DIR%"

echo [3/4] Cau hinh log file...
"%NSSM_EXE%" set %SERVICE_NAME% AppStdout "%LOG_DIR%\stdout.log"
"%NSSM_EXE%" set %SERVICE_NAME% AppStderr "%LOG_DIR%\stderr.log"
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateFiles 1
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateOnline 1
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateSeconds 86400
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateBytes 10485760

echo [4/4] Dang khoi dong service...
"%NSSM_EXE%" start %SERVICE_NAME%

echo.
echo === CAI DAT THANH CONG ===
echo Service %SERVICE_NAME% dang chay ngam.
echo Ban co the kiem tra log tai: %LOG_DIR%
pause

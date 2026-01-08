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
:: Su dung nssm tu thu muc agent neu co, hoac tim trong thu muc hien tai
if exist "%BASE_DIR%..\agent\nssm.exe" (
    set NSSM_EXE=%BASE_DIR%..\agent\nssm.exe
) else (
    set NSSM_EXE=%BASE_DIR%nssm.exe
)

set APP_EXE=%BASE_DIR%dist\LeeAttendDashboard.exe
set LOG_DIR=%BASE_DIR%logs
set APP_PORT=3001

if not exist "%APP_EXE%" (
    echo [ERROR] Khong tim thay file %APP_EXE%
    echo Hay chay "build_standalone.bat" truoc de tao file exe.
    pause
    exit /b
)

if not exist "%NSSM_EXE%" (
    echo [ERROR] Khong tim thay nssm.exe!
    echo Vui long copy nssm.exe vao thu muc nay hoac thu muc agent.
    pause
    exit /b
)

if not exist "%LOG_DIR%" mkdir "%LOG_DIR%"

echo [1/4] Dang cai dat service %SERVICE_NAME%...
:: QUAN TRONG: Khong dung dau ngoac kep cho duong dan APP_EXE de tranh loi "Error 3" cua NSSM
"%NSSM_EXE%" install %SERVICE_NAME% %APP_EXE%

echo [2/4] Cau hinh tham so va thu muc...
:: Set AppParameters (Port va No Open Browser)
"%NSSM_EXE%" set %SERVICE_NAME% AppParameters "--port %APP_PORT% --no-open"

:: Set AppDirectory la thu muc chua file exe
"%NSSM_EXE%" set %SERVICE_NAME% AppDirectory "%BASE_DIR%dist"

echo [3/4] Cau hinh log file...
"%NSSM_EXE%" set %SERVICE_NAME% AppStdout "%LOG_DIR%\dashboard.log"
"%NSSM_EXE%" set %SERVICE_NAME% AppStderr "%LOG_DIR%\dashboard.err"
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateFiles 1
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateOnline 1
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateSeconds 86400
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateBytes 10485760

echo [4/4] Dang khoi dong service...
"%NSSM_EXE%" start %SERVICE_NAME%

echo.
echo === CAI DAT THANH CONG ===
echo Dashboard Service dang chay tai: http://localhost:%APP_PORT%
echo Kiem tra log tai: %LOG_DIR%
pause
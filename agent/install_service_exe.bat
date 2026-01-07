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

set SERVICE_NAME=ZKAttendanceAgent
set BASE_DIR=%~dp0
set NSSM_EXE=%BASE_DIR%nssm.exe
set APP_EXE=%BASE_DIR%dist\agent.exe
set WORK_DIR=%BASE_DIR%dist
set LOG_DIR=%BASE_DIR%logs
set ENV_SOURCE=%BASE_DIR%.env
set ENV_DEST=%WORK_DIR%\.env

if not exist "%APP_EXE%" (
    echo [ERROR] Khong tim thay %APP_EXE%
    echo Hay chay build_exe.bat truoc!
    pause
    exit /b
)

if not exist "%NSSM_EXE%" (
    echo [ERROR] Khong tim thay file nssm.exe tai %BASE_DIR%
    echo Hay tai nssm va copy vao thu muc nay.
    pause
    exit /b
)

:: Copy file .env vao dist de exe doc duoc
if exist "%ENV_SOURCE%" (
    echo [INFO] Copying .env to dist folder...
    copy /Y "%ENV_SOURCE%" "%ENV_DEST%" >nul
) else (
    echo [WARN] Khong tim thay file .env goc. Hay tao .env trong thu muc dist sau.
)

:: Tao thu muc log
if not exist "%LOG_DIR%" mkdir "%LOG_DIR%"

echo [1/3] Dang cai dat service %SERVICE_NAME% (EXE Mode)...
"%NSSM_EXE%" install %SERVICE_NAME% "%APP_EXE%"

echo [2/3] Cau hinh thu muc lam viec...
:: Set Working Dir vao dist de DB va Log nam gon trong do (hoac logs ra ngoai)
"%NSSM_EXE%" set %SERVICE_NAME% AppDirectory "%WORK_DIR%"

echo [3/3] Cau hinh log file...
"%NSSM_EXE%" set %SERVICE_NAME% AppStdout "%LOG_DIR%\agent_stdout.log"
"%NSSM_EXE%" set %SERVICE_NAME% AppStderr "%LOG_DIR%\agent_stderr.log"
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateFiles 1
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateOnline 1
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateSeconds 86400
"%NSSM_EXE%" set %SERVICE_NAME% AppRotateBytes 10485760

echo [4/4] Dang khoi dong service...
"%NSSM_EXE%" start %SERVICE_NAME%

echo.
echo === CAI DAT THANH CONG (EXE MODE) ===
echo Service %SERVICE_NAME% dang chay tu file EXE.
pause

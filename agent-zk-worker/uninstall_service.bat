@echo off
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo.
    echo [ERROR] Ban can chay file nay voi quyen Administrator!
    pause
    exit /b
)

set SERVICE_NAME=ZKAttendanceAgent
set BASE_DIR=%~dp0
set NSSM_EXE=%BASE_DIR%nssm.exe

echo Dang dung service %SERVICE_NAME%...
"%NSSM_EXE%" stop %SERVICE_NAME%

echo Dang go bo service %SERVICE_NAME%...
"%NSSM_EXE%" remove %SERVICE_NAME% confirm

echo.
echo === DA GO BO SERVICE ===
pause

@echo off
set BASE_DIR=%~dp0
cd /d "%BASE_DIR%"

if not exist "venv" (
    echo [ERROR] Chua co moi truong venv. Hay cai dat truoc.
    pause
    exit /b
)

echo [1/3] Cai dat PyInstaller...
.\venv\Scripts\pip install pyinstaller

echo [2/3] Dang dong goi agent.exe...
:: --onefile: Gom gon 1 file
:: --console: De hien thi log cho NSSM bat
:: --hidden-import: Them cac module thuong bi sot
.\venv\Scripts\pyinstaller --noconfirm --onefile --console --name "agent" ^
    --clean ^
    --add-data "alembic.ini;." ^
    --add-data "migrations;migrations" ^
    --hidden-import "sqlalchemy.sql.default_comparator" ^
    --hidden-import "sqlalchemy.ext.baked" ^
    --hidden-import "pg8000" ^
    --hidden-import "psycopg2" ^
    main.py

echo [3/3] Hoan tat!
echo File exe nam tai: dist\agent.exe
echo Hay copy file .env vao thu muc dist truoc khi chay thu.
pause

@echo off
set BASE_DIR=%~dp0
cd /d "%BASE_DIR%"

:: [1/4] Kiem tra venv
if not exist "venv" (
    echo [INFO] Dang khoi tao moi truong venv cho Windows...
    python -m venv venv
    .\venv\Scripts\pip install --upgrade pip
    .\venv\Scripts\pip install -r requirements.txt
)

echo [2/4] Dam bao PyInstaller da duoc cai dat...
.\venv\Scripts\pip install pyinstaller

echo [3/4] Dang dong goi [agent-standalone.exe] (Python Standalone Mode)...
.\venv\Scripts\pyinstaller --noconfirm --onefile --console --name "agent-standalone" \
    --clean \
    --add-data "alembic.ini;."
    --add-data "migrations;migrations"
    --hidden-import "sqlalchemy.sql.default_comparator" \
    --hidden-import "sqlalchemy.ext.baked" \
    --hidden-import "pg8000" \
    --hidden-import "psycopg2" \
    main.py

echo [4/4] Dang dong goi [agent-worker.exe] (Rust Sidecar Mode)...
.\venv\Scripts\pyinstaller --noconfirm --onefile --console --name "agent-worker" \
    --clean \
    --hidden-import "pyzk2" \
    zk_worker.py

echo ---------------------------------------------------
echo HOAN TAT!
echo File ket qua nam tai thu muc: dist\
echo.

echo 👉 De chay Rust Agent: Copy 'dist\agent-worker.exe' va file '.env' vao cung thu muc voi binary Rust.
pause

#!/bin/bash

# Dừng script nếu gặp lỗi
set -e

BASE_DIR=$(dirname "$0")
cd "$BASE_DIR"

# [1/4] Kiểm tra và khởi tạo môi trường ảo
if [ ! -d "venv_linux" ]; then
    echo "---------------------------------------------------"
    echo "🚀 [INFO] Khởi tạo môi trường venv_linux mới..."
    python3 -m venv venv_linux
    source venv_linux/bin/activate
    echo "[INFO] Đang cài đặt thư viện từ requirements.txt..."
    pip install --upgrade pip
    pip install -r requirements.txt
    echo "✅ Đã thiết lập môi trường ảo thành công."
else
    echo "---------------------------------------------------"
    echo "🚀 [INFO] Sử dụng môi trường venv_linux hiện có."
    source venv_linux/bin/activate
fi

echo "---------------------------------------------------"
echo "[2/4] Kiểm tra PyInstaller..."
if ! command -v pyinstaller &> /dev/null; then
    echo "[INFO] Đang cài đặt PyInstaller..."
    pip install pyinstaller
fi

echo "---------------------------------------------------"
echo "[3/4] Đóng gói [agent-standalone] (Python Standalone Mode)..."
pyinstaller --noconfirm --onefile --console --name "agent-standalone" \
    --clean \
    --add-data "alembic.ini:." \
    --add-data "migrations:migrations" \
    --hidden-import "sqlalchemy.sql.default_comparator" \
    --hidden-import "sqlalchemy.ext.baked" \
    --hidden-import "pg8000" \
    --hidden-import "psycopg2" \
    main.py

echo "---------------------------------------------------"
echo "[4/4] Đóng gói [agent-zk-worker] (Rust Sidecar Mode)..."
pyinstaller --noconfirm --onefile --console --name "agent-zk-worker" \
    --clean \
    --hidden-import "pyzk2" \
    zk_worker.py

echo "---------------------------------------------------"
echo "✅ HOÀN TẤT!"
echo "   - Agent Standalone: dist/agent-standalone"
echo "   - ZK Sidecar:       dist/agent-zk-worker"
echo ""
echo "👉 Để chạy Rust Agent: Copy file 'dist/agent-zk-worker' và '.env' vào cùng thư mục với binary Rust."
import pytest
from fastapi.testclient import TestClient
from main import app, ZKService
import inspect

client = TestClient(app)

def test_endpoint_deletion_safety():
    """Đảm bảo endpoint xóa dữ liệu không tồn tại"""
    response = client.post("/attendance/clear")
    assert response.status_code == 404, "Endpoint /attendance/clear vẫn tồn tại!"

def test_service_method_safety():
    """Đảm bảo class ZKService không có hàm xóa dữ liệu"""
    methods = [member[0] for member in inspect.getmembers(ZKService, predicate=inspect.isfunction)]
    assert "clear_attendance" not in methods, "Phương thức clear_attendance vẫn tồn tại trong ZKService!"
    assert "delete_user" not in methods, "Phương thức delete_user vẫn tồn tại trong ZKService!"

def test_source_code_safety():
    """Kiểm tra quét mã nguồn để đảm bảo không gọi các hàm xóa của thư viện pyzk"""
    with open("main.py", "r") as f:
        content = f.read()
        # Kiểm tra xem có gọi các hàm nguy hiểm của thư viện ZK không
        assert ".clear_attendance()" not in content, "Phát hiện lệnh gọi clear_attendance() trong mã nguồn!"
        assert ".delete_user(" not in content, "Phát hiện lệnh gọi delete_user() trong mã nguồn!"
        assert ".clear_data(" not in content, "Phát hiện lệnh gọi clear_data() trong mã nguồn!"

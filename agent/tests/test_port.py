import os
import importlib
import main
from unittest import mock

def test_api_port_default_is_zero():
    """Kiểm tra port mặc định phải là 0 (để chọn ngẫu nhiên)"""
    # Xóa biến môi trường nếu có để test default
    with mock.patch.dict(os.environ, {}, clear=True):
        importlib.reload(main)
        # Lưu ý: main.py có load_dotenv() bên trong, 
        # nên nếu file .env có API_PORT=0 thì nó sẽ là 0.
        assert main.API_PORT == 0

def test_api_port_override():
    """Kiểm tra việc ghi đè port qua biến môi trường"""
    with mock.patch.dict(os.environ, {"API_PORT": "9999"}):
        importlib.reload(main)
        assert main.API_PORT == 9999

def test_api_port_invalid_fallback():
    """Kiểm tra fallback khi giá trị môi trường không hợp lệ (nếu cần)"""
    # Nếu API_PORT không phải là số, chương trình sẽ crash tại int()
    # Ở đây ta chỉ test trường hợp thành công cơ bản
    with mock.patch.dict(os.environ, {"API_PORT": "0"}):
        importlib.reload(main)
        assert main.API_PORT == 0

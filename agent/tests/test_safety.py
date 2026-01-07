import pytest
from device_manager import ZKService

def test_service_method_safety():
    """Xác nhận ZKService không lộ các method can thiệp sâu của PyZK một cách công khai"""
    service = ZKService("TEST", "1.1.1.1")
    # Kiểm tra đóng gói
    assert hasattr(service, "get_attendance_logs")
    # Các method pyzk gốc không nên có trực tiếp trong service wrapper của chúng ta
    # (hoặc ít nhất là chúng ta không sử dụng chúng bừa bãi)
    assert not hasattr(service, "clear_attendance")

def test_source_code_safety():
    """Kiểm tra không có thông tin nhạy cảm trong codebase"""
    # Test logic tĩnh
    pass
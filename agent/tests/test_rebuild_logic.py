import pytest
from unittest.mock import MagicMock, patch
import main

@pytest.fixture(autouse=True)
def cleanup_services():
    """Reset global services dict before each test"""
    main.services.clear()
    yield
    main.services.clear()

class MockZKService:
    def __init__(self, device_id, ip, port):
        self.device_id = device_id
        self.ip = ip
        self.port = port

def test_rebuild_initial_load():
    """Kiểm tra lần nạp đầu tiên"""
    devices_data = [
        {"id": "D1", "ip": "192.168.1.10", "port": 4370}
    ]
    
    with patch("main.ZKService", side_effect=MockZKService):
        result = main.rebuild_services(devices_data)
        assert result is True
        assert "D1" in main.services
        assert main.services["D1"].ip == "192.168.1.10"

def test_rebuild_no_change():
    """Kiểm tra khi cấu hình không thay đổi"""
    devices_data = [
        {"id": "D1", "ip": "192.168.1.10", "port": 4370}
    ]
    
    with patch("main.ZKService", side_effect=MockZKService):
        # Lần 1: Nạp
        main.rebuild_services(devices_data)
        
        # Lần 2: Nạp lại cùng dữ liệu
        result = main.rebuild_services(devices_data)
        assert result is False # Không có thay đổi
        assert len(main.services) == 1

def test_rebuild_with_ip_change():
    """Kiểm tra khi đổi IP của thiết bị"""
    old_data = [{"id": "D1", "ip": "192.168.1.10", "port": 4370}]
    new_data = [{"id": "D1", "ip": "192.168.1.99", "port": 4370}]
    
    with patch("main.ZKService", side_effect=MockZKService):
        main.rebuild_services(old_data)
        result = main.rebuild_services(new_data)
        
        assert result is True
        assert main.services["D1"].ip == "192.168.1.99"

def test_rebuild_add_remove_device():
    """Kiểm tra khi thêm và bớt thiết bị"""
    data_1 = [{"id": "D1", "ip": "1.1.1.1"}]
    data_2 = [
        {"id": "D1", "ip": "1.1.1.1"},
        {"id": "D2", "ip": "2.2.2.2"}
    ]
    
    with patch("main.ZKService", side_effect=MockZKService):
        main.rebuild_services(data_1)
        assert len(main.services) == 1
        
        main.rebuild_services(data_2)
        assert len(main.services) == 2
        
        main.rebuild_services(data_1)
        assert len(main.services) == 1

def test_rebuild_with_empty_data():

    """Kiểm tra việc xóa sạch thiết bị khi nhận list rỗng"""

    old_data = [{"id": "D1", "ip": "1.1.1.1"}]

    

    with patch("main.ZKService", side_effect=MockZKService):

        main.rebuild_services(old_data)

        assert len(main.services) == 1

        

        # Nhận list rỗng -> Phải trả về True để báo hiệu thay đổi và xóa sạch services

        result = main.rebuild_services([])

        assert result is True 

        assert len(main.services) == 0



def test_rebuild_safety_none_data():

    """Kiểm tra cơ chế bảo vệ khi nhận dữ liệu None (lỗi)"""

    old_data = [{"id": "D1", "ip": "1.1.1.1"}]

    

    with patch("main.ZKService", side_effect=MockZKService):

        main.rebuild_services(old_data)

        

        # Nhận None -> Phải trả về False và giữ nguyên cấu hình cũ

        result = main.rebuild_services(None)

        assert result is False

        assert len(main.services) == 1

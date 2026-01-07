import pytest
import json
import os
from unittest.mock import MagicMock, patch, mock_open
from central_client import CentralClient

@pytest.fixture
def client():
    return CentralClient()

def test_save_local_config(client):
    """Kiểm tra việc lưu cấu hình vào file devices.json"""
    test_devices = [
        {"device_id": "DEV_01", "ip": "192.168.1.10", "port": 4370, "name": "Máy 1"},
        {"device_id": "DEV_02", "ip": "192.168.1.11"}
    ]
    
    # Mock open function
    m = mock_open()
    with patch("builtins.open", m):
        client._save_local_config(test_devices)
    
    # Kiểm tra xem file có được mở để ghi (mode 'w')
    m.assert_called_once_with("devices.json", "w", encoding="utf-8")
    
    # Kiểm tra nội dung ghi vào file
    handle = m()
    written_data = "".join(call.args[0] for call in handle.write.call_args_list)
    parsed_data = json.loads(written_data)
    
    assert len(parsed_data) == 2
    assert parsed_data[0]["id"] == "DEV_01"
    assert parsed_data[1]["port"] == 4370 # Default port

def test_save_local_config_empty(client):
    """Kiểm tra việc lưu danh sách rỗng vào file devices.json"""
    m = mock_open()
    with patch("builtins.open", m):
        client._save_local_config([])
    
    handle = m()
    written_data = "".join(call.args[0] for call in handle.write.call_args_list)
    assert written_data.strip() == "[]"

@pytest.mark.asyncio
async def test_receive_config_push():
    """Kiểm tra logic xử lý message config_push"""
    client = CentralClient()
    mock_callback = MagicMock()
    client.on_config_received = mock_callback
    
    # Giả lập message từ server
    test_msg = json.dumps({
        "type": "config_push",
        "data": {
            "devices": [
                {"device_id": "NEW_DEV", "ip": "10.0.0.1", "port": 4370}
            ]
        }
    })
    
    # Mock websocket iterator
    mock_ws = MagicMock()
    mock_ws.__aiter__.return_value = [test_msg]
    
    # Patch _save_local_config để không ghi file thật
    with patch.object(client, "_save_local_config") as mock_save:
        # Chạy receiver một vòng (nó sẽ thoát sau khi hết message trong iterator)
        await client._receiver(mock_ws)
        
        # Kiểm tra callback được gọi
        mock_callback.assert_called_once()
        devices_passed = mock_callback.call_args[0][0]
        assert devices_passed[0]["device_id"] == "NEW_DEV"
        
        # Kiểm tra file cache được lưu
        mock_save.assert_called_once()

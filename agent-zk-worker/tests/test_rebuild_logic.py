import pytest
from unittest.mock import MagicMock, patch
from app_container import AttendanceAgent
from device_manager import ZKService

@pytest.mark.asyncio
async def test_rebuild_initial_load(test_agent):
    """Kiểm tra việc nạp cấu hình lần đầu"""
    agent = test_agent
    devices = [{"id": "D1", "ip": "1.1.1.1"}]
    
    with patch.object(agent.device_manager, 'add_device') as mock_add:
        await agent.apply_central_config(devices)
        mock_add.assert_called_once_with("D1", "1.1.1.1", 4370)

@pytest.mark.asyncio
async def test_rebuild_no_change(test_agent):
    """Kiểm tra nếu cấu hình không đổi thì không nạp lại"""
    agent = test_agent
    # Giả lập đã có thiết bị
    mock_service = MagicMock(spec=ZKService)
    mock_service.ip = "1.1.1.1"
    mock_service.port = 4370
    agent.device_manager.services["D1"] = mock_service
    
    devices = [{"id": "D1", "ip": "1.1.1.1"}]
    with patch.object(agent.device_manager, 'add_device') as mock_add:
        await agent.apply_central_config(devices)
        mock_add.assert_not_called()

@pytest.mark.asyncio
async def test_rebuild_with_ip_change(test_agent):
    """Kiểm tra việc cập nhật khi IP thay đổi"""
    agent = test_agent
    mock_service = MagicMock(spec=ZKService)
    mock_service.ip = "1.1.1.1"
    mock_service.port = 4370
    agent.device_manager.services["D1"] = mock_service
    
    devices = [{"id": "D1", "ip": "2.2.2.2"}]
    with patch.object(agent.device_manager, 'add_device') as mock_add:
        await agent.apply_central_config(devices)
        mock_add.assert_called_once_with("D1", "2.2.2.2", 4370)

@pytest.mark.asyncio
async def test_rebuild_add_remove_device(test_agent):
    """Kiểm tra thêm và xóa thiết bị đồng thời"""
    agent = test_agent
    agent.device_manager.services["D1"] = MagicMock(spec=ZKService)
    
    devices = [{"id": "D2", "ip": "2.2.2.2"}]
    with patch.object(agent.device_manager, 'add_device') as mock_add, \
         patch.object(agent.device_manager, 'remove_device') as mock_remove:
        await agent.apply_central_config(devices)
        mock_remove.assert_called_once_with("D1")
        mock_add.assert_called_once_with("D2", "2.2.2.2", 4370)
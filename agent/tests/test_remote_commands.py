import pytest
from unittest.mock import MagicMock, patch
from main import handle_remote_command, services, ZKService

@pytest.fixture
def mock_service():
    device_id = "TEST_DEV"
    service = ZKService(device_id, "127.0.0.1", 4370)
    services[device_id] = service
    return service

@pytest.mark.asyncio
async def test_handle_command_ping():
    command = {"method": "ping"}
    result = await handle_remote_command(command)
    assert result["success"] is True
    assert result["data"] == "pong"

@pytest.mark.asyncio
async def test_handle_command_sync_time(mock_service):
    with patch.object(ZKService, 'sync_time', return_value="2026-01-07 12:00:00"):
        command = {"method": "sync_time", "device_id": "TEST_DEV"}
        result = await handle_remote_command(command)
        assert result["success"] is True
        assert result["data"]["new_time"] == "2026-01-07 12:00:00"

@pytest.mark.asyncio
async def test_handle_command_get_users(mock_service):
    mock_users = [{"uid": 1, "user_id": "1", "name": "Test User"}]
    with patch.object(ZKService, 'get_users', return_value=mock_users):
        command = {"method": "get_users", "device_id": "TEST_DEV"}
        result = await handle_remote_command(command)
        assert result["success"] is True
        assert result["data"]["users"] == mock_users

@pytest.mark.asyncio
async def test_handle_command_device_not_found():
    command = {"method": "sync_time", "device_id": "NON_EXISTENT"}
    result = await handle_remote_command(command)
    assert result["success"] is False
    assert "not found" in result["error"]

@pytest.mark.asyncio
async def test_handle_command_unknown_method():
    command = {"method": "unknown_method"}
    result = await handle_remote_command(command)
    assert result["success"] is False
    assert "not implemented" in result["error"]

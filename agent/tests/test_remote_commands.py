import pytest
from unittest.mock import MagicMock, patch
import main
from device_manager import ZKService

@pytest.fixture
def mock_service(test_ctx, test_agent):
    device_id = "TEST_DEV"
    # Dùng factory để tạo mock và đăng ký vào agent.device_manager.services
    mock = test_ctx.create_mock_service(device_id)
    return mock

@pytest.mark.asyncio
async def test_handle_command_ping(test_agent):
    agent = test_agent
    command = {"method": "ping"}
    result = await agent.handle_remote_command(command)
    assert result["success"] is True
    assert result["data"] == "pong"

@pytest.mark.asyncio
async def test_handle_command_sync_time(test_agent, mock_service):
    agent = test_agent
    mock_service.sync_time.return_value = "2026-01-07 12:00:00"
    command = {"method": "sync_time", "device_id": "TEST_DEV"}
    result = await agent.handle_remote_command(command)
    assert result["success"] is True
    assert result["data"]["new_time"] == "2026-01-07 12:00:00"

@pytest.mark.asyncio
async def test_handle_command_get_users(test_agent, mock_service):
    agent = test_agent
    mock_users = [{"uid": 1, "user_id": "1", "name": "Test User"}]
    mock_service.get_users.return_value = mock_users
    command = {"method": "get_users", "device_id": "TEST_DEV"}
    result = await agent.handle_remote_command(command)
    assert result["success"] is True
    assert result["data"]["users"] == mock_users

@pytest.mark.asyncio
async def test_handle_command_device_not_found(test_agent):
    agent = test_agent
    command = {"method": "sync_time", "device_id": "NON_EXISTENT"}
    result = await agent.handle_remote_command(command)
    assert result["success"] is False
    assert "not found" in result["error"]

@pytest.mark.asyncio
async def test_handle_command_unknown_method(test_agent):
    agent = test_agent
    # Cần truyền device_id hợp lệ để vượt qua bước check device
    agent.device_manager.services["DEV1"] = MagicMock()
    command = {"method": "unknown_method", "device_id": "DEV1"}
    result = await agent.handle_remote_command(command)
    assert result["success"] is False
    assert "not implemented" in result["error"]

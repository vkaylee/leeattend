import pytest
from datetime import datetime

@pytest.mark.asyncio
async def test_agent_status_format(test_agent):
    """Kiểm tra định dạng dữ liệu status của agent"""
    data = test_agent.get_agent_status()
    assert data["status"] == "running"
    assert "uptime_seconds" in data
    assert "total_processed" in data
    assert "devices" in data
    assert isinstance(data["devices"], list)

@pytest.mark.asyncio
async def test_status_reflects_stats(test_agent):
    """Xác nhận status phản ánh đúng dữ liệu nội bộ"""
    test_agent.total_processed = 99
    data = test_agent.get_agent_status()
    assert data["total_processed"] == 99

@pytest.mark.asyncio
async def test_status_reflects_devices(test_agent):
    """Xác nhận status chứa thông tin các thiết bị đang quản lý"""
    test_agent.device_manager.add_device("D1", "1.1.1.1")
    data = test_agent.get_agent_status()
    assert len(data["devices"]) == 1
    assert data["devices"][0]["device_id"] == "D1"
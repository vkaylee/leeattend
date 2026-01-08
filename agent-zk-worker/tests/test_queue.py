import pytest
import asyncio
from unittest.mock import MagicMock, patch
import main
from device_manager import ZKService

@pytest.mark.asyncio
async def test_system_event_on_startup(test_ctx, test_agent):
    """Xác nhận monitor gửi thông báo hệ thống khi khởi động"""
    agent = test_agent
    mock_service = test_ctx.create_mock_service("SYS_TEST")
    
    from tests.test_utils import LoopBreak
    with test_ctx.mock_infinite_loop(max_loops=0):
        try:
            # Gọi phương thức monitor của DeviceManager
            await agent.device_manager._monitor_device(mock_service)
        except LoopBreak:
            pass

    # Kiểm tra queue
    item = await agent.log_queue.get()
    assert item is not None
    assert item.get("type") == "system"
    assert "Bắt đầu giám sát" in item.get("message")

@pytest.mark.asyncio
async def test_system_event_on_failure(test_ctx, test_agent):
    """Xác nhận monitor gửi cảnh báo WARN khi mất kết nối"""
    agent = test_agent
    mock_service = test_ctx.create_mock_service("FAIL_TEST")
    mock_service.get_attendance_logs.side_effect = Exception("Hardware Error")
    
    from tests.test_utils import LoopBreak
    with test_ctx.mock_infinite_loop(max_loops=0):
        try:
            await agent.device_manager._monitor_device(mock_service)
        except LoopBreak:
            pass

    # Kiểm tra queue
    # Message 1: Bắt đầu giám sát
    await agent.log_queue.get()
    # Message 2: Cảnh báo lỗi
    item = await agent.log_queue.get()
    assert item is not None
    assert item.get("type") == "system"
    assert item.get("level") == "WARN"

@pytest.mark.asyncio
async def test_main_runner_handles_system_msg(test_ctx, test_agent):
    """Kiểm tra main_runner in đúng định dạng cho tin nhắn hệ thống"""
    agent = test_agent
    test_msg = {
        "type": "system",
        "level": "INFO",
        "device_id": "DEV_01",
        "message": "Test System Message"
    }

    # Mock print và central_client
    with patch("builtins.print"), \
         patch.object(agent, "central_client") as mock_central:
        
        # Khởi chạy main_runner như một background task
        runner_task = asyncio.create_task(agent._main_runner())
        
        # Đẩy tin nhắn vào queue
        await agent.log_queue.put(test_msg)
        
        # Đợi cho đến khi mock được gọi (tối đa 2s)
        for _ in range(20):
            if mock_central.push_system_event.called:
                break
            if runner_task.done() and runner_task.exception():
                break
            await asyncio.sleep(0.1)
        
        # Kiểm tra xem central_client.push_system_event có được gọi đúng không
        mock_central.push_system_event.assert_called_with("INFO", "Test System Message", "DEV_01")
        
        # Hủy runner task và dọn dẹp
        runner_task.cancel()
        try:
            await runner_task
        except asyncio.CancelledError:
            pass
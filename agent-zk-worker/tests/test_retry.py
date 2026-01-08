import pytest
import asyncio
from unittest.mock import MagicMock, patch
from device_manager import ZKService

@pytest.mark.asyncio
async def test_exponential_backoff_logic(test_ctx, test_agent):
    """Kiểm tra thuật toán tăng thời gian chờ khi gặp lỗi"""
    agent = test_agent
    mock_service = MagicMock(spec=ZKService)
    mock_service.device_id = "TEST_RETRY"
    mock_service.ip = "127.0.0.1"
    
    # Giả lập luôn bị lỗi khi gọi lấy log
    mock_service.get_attendance_logs.side_effect = Exception("Connection lost")
    
    delays = []
    
    # Mock asyncio.sleep để ghi lại thời gian chờ
    async def mock_sleep(delay):
        delays.append(delay)
        if len(delays) >= 3:
            raise asyncio.CancelledError()
    
    with patch("asyncio.sleep", side_effect=mock_sleep):
        try:
            await agent.device_manager._monitor_device(mock_service)
        except asyncio.CancelledError:
            pass

    # Kết quả: Lần 1 lỗi -> sleep [10, 10], Lần 2 lỗi -> sleep [10, 20], Lần 3 lỗi -> sleep [10, 40]
    assert 10.0 <= delays[0] <= 10.1
    assert 10.0 <= delays[1] <= 20.0
    assert 10.0 <= delays[2] <= 40.0

@pytest.mark.asyncio
async def test_retry_reset_on_success(test_ctx, test_agent):
    """Kiểm tra việc reset thời gian chờ khi kết nối lại thành công"""
    agent = test_agent
    mock_service = MagicMock(spec=ZKService)
    mock_service.device_id = "TEST_RESET"
    mock_service.ip = "127.0.0.1"
    
    # Lần 1 lỗi, Lần 2 thành công
    mock_service.get_attendance_logs.side_effect = [
        Exception("Fail"), 
        [] # Success
    ]
    
    delays = []
    async def mock_sleep(delay):
        delays.append(delay)
        if len(delays) >= 2:
            raise asyncio.CancelledError()

    with patch("asyncio.sleep", side_effect=mock_sleep):
        try:
            await agent.device_manager._monitor_device(mock_service)
        except asyncio.CancelledError:
            pass

    # Lần 1 lỗi -> sleep 10. Lần 2 thành công -> reset delay -> sleep 10
    assert 10.0 <= delays[0] <= 10.1
    assert 10.0 <= delays[1] <= 10.1
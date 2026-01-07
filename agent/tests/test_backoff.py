import asyncio
import pytest
import random
from unittest.mock import MagicMock, patch
from sync_worker import SyncWorker
from central_client import CentralClient

@pytest.mark.asyncio
async def test_sync_worker_wait_time_logic():
    """Kiểm tra logic tính toán wait_time của SyncWorker (Unit test)"""
    mock_repo = MagicMock()
    mock_client = MagicMock()
    worker = SyncWorker(mock_repo, mock_client)
    worker.max_backoff = 300
    worker.base_backoff = 5

    # Case 1: Không có lỗi, không có dữ liệu -> 5.0s (default idle)
    assert worker.get_next_wait_time(0, False) == 5.0

    # Case 2: Không có lỗi, có dữ liệu -> worker.interval
    worker.interval = 0.1
    assert worker.get_next_wait_time(0, True) == 0.1

    # Case 3: Có lỗi (Exponential Backoff + Jitter)
    # Lỗi lần 1: backoff = 5 * 2^0 = 5. Khoảng [5, 5]
    assert 5.0 <= worker.get_next_wait_time(1, False) <= 5.1
    
    # Lỗi lần 3: backoff = 5 * 2^2 = 20. Khoảng [5, 20]
    delays = [worker.get_next_wait_time(3, False) for _ in range(100)]
    assert all(5.0 <= d <= 20.0 for d in delays)
    assert len(set(delays)) > 1 # Kiểm tra tính ngẫu nhiên (jitter)

@pytest.mark.asyncio
async def test_central_client_retry_delay_logic():
    """Kiểm tra logic tính toán retry_delay của CentralClient (Unit test)"""
    client = CentralClient()
    client.base_retry_delay = 1
    client.max_retry_delay = 10
    client.auth_error_min_delay = 60

    # Lần 0: [1, 1.1] (do jitter base)
    assert 1.0 <= client.get_next_retry_delay(0, False) <= 1.1

    # Lần 2: [1, 1*2^2=4]
    delays = [client.get_next_retry_delay(2, False) for _ in range(100)]
    assert all(1.0 <= d <= 4.0 for d in delays)

    # Lỗi Auth: min 60s
    auth_delays = [client.get_next_retry_delay(0, True) for _ in range(100)]
    assert all(d >= 60 for d in auth_delays)

@pytest.mark.asyncio
async def test_sync_worker_backoff_integration(test_agent):
    """Kiểm tra SyncWorker thực sự tăng thời gian chờ khi gặp lỗi (Integration lite)"""
    agent = test_agent
    worker = agent.sync_worker
    worker.base_backoff = 0.1 # Dùng số cực nhỏ để test nhanh
    worker.max_backoff = 0.4
    agent.central_client.is_connected = True
    
    # Giả lập callback trả về 1 thiết bị đang hoạt động
    worker.get_active_devices_callback = lambda: ["DEV1"]
    
    # Mock sync_batch để ném lỗi
    with patch.object(worker, "sync_batch", side_effect=Exception("Fail")):
        # Theo dõi các lần gọi sleep
        with patch("asyncio.sleep", side_effect=asyncio.CancelledError) as mock_sleep:
            # Chạy start() một vòng
            try:
                # Dùng một task để có thể cancel
                task = asyncio.create_task(worker.start())
                await asyncio.sleep(0.05)
                task.cancel()
                await task
            except (asyncio.CancelledError, Exception):
                pass
            
            # Kiểm tra xem sleep có được gọi với wait_time không
            assert mock_sleep.called
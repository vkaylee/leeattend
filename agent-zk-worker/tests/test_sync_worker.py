import asyncio
import pytest
from unittest.mock import MagicMock, patch
from datetime import datetime
from sync_worker import SyncWorker
from models import ZKTRawAttendance

@pytest.mark.asyncio
async def test_sync_worker_batch():
    """Kiểm tra SyncWorker lấy dữ liệu từ DB và cập nhật sau khi nhận ACK"""
    # Mock dependencies
    mock_repo = MagicMock()
    mock_client = MagicMock()
    
    # Giả lập dữ liệu chưa sync
    mock_log = ZKTRawAttendance(
        id=1, device_id="D1", user_id="101",
        timestamp=datetime.now(), status=1, punch=0, is_synced=0
    )
    mock_repo.get_unsynced_logs.return_value = [mock_log]

    # Giả lập Future cho ACK
    mock_future = asyncio.Future()
    mock_future.msg_id = "test-msg-id"
    mock_future.set_result(True)
    mock_client.push_attendance_sync.return_value = mock_future
    mock_client.is_connected = True

    worker = SyncWorker(mock_repo, mock_client)
    worker.batch_size = 5

    await worker.sync_batch(active_ids=["D1"])

    # Kiểm tra push_attendance_sync được gọi đúng cách
    mock_client.push_attendance_sync.assert_called_once()
    # Kiểm tra repo được gọi để đánh dấu sync
    mock_repo.mark_as_synced.assert_called_once_with([1])

@pytest.mark.asyncio
async def test_sync_worker_timeout():
    """Kiểm tra SyncWorker xử lý khi chờ ACK bị timeout"""
    mock_repo = MagicMock()
    mock_client = MagicMock()
    
    mock_log = ZKTRawAttendance(id=1, device_id="D1", user_id="101", timestamp=datetime.now())
    mock_repo.get_unsynced_logs.return_value = [mock_log]

    # Giả lập timeout
    mock_future = asyncio.Future()
    mock_future.msg_id = "test-timeout-id"
    # Đừng set result để gây timeout
    mock_client.push_attendance_sync.return_value = mock_future
    mock_client.is_connected = True

    worker = SyncWorker(mock_repo, mock_client)
    
    # Mock asyncio.wait_for để ném TimeoutError
    with patch("asyncio.wait_for", side_effect=asyncio.TimeoutError):
        result = await worker.sync_batch(active_ids=["D1"])
        assert result is False
        # Đảm bảo vẫn gọi dọn dẹp
        mock_client.discard_pending_ack.assert_called_with("test-timeout-id")
        # Không được mark as synced
        mock_repo.mark_as_synced.assert_not_called()

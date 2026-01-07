import pytest
import asyncio
from unittest.mock import MagicMock, patch
from datetime import datetime
from sync_worker import SyncWorker
from models import ZKTRawAttendance

@pytest.mark.asyncio
async def test_sync_worker_batch():
    """Kiểm tra SyncWorker lấy dữ liệu từ DB và cập nhật sau khi nhận ACK"""
    # Mock central_client
    with patch("sync_worker.central_client") as mock_client, \
         patch("sync_worker.SessionLocal") as mock_session_local:
        
        # Thiết lập mock DB session
        mock_db = MagicMock()
        mock_session_local.return_value = mock_db
        
        # Giả lập dữ liệu chưa sync
        mock_log = ZKTRawAttendance(
            id=1, device_id="D1", user_id="101", 
            timestamp=datetime.now(), status=1, punch=0, is_synced=0
        )
        mock_db.query.return_value.filter.return_value.order_by.return_value.limit.return_value.all.return_value = [mock_log]
        
        # Giả lập Future cho ACK
        mock_future = asyncio.Future()
        mock_future.set_result(True)
        mock_client.push_attendance_sync.return_value = mock_future
        mock_client.is_connected = True
        
        worker = SyncWorker() # Đã sửa: Không truyền batch_size
        worker.batch_size = 5 # Set thủ công cho test
        
        await worker.sync_batch()
        
        # Kiểm tra push_attendance_sync được gọi đúng cách
        mock_client.push_attendance_sync.assert_called_once()
        
        # Kiểm tra DB đã được commit (đánh dấu sync)
        mock_db.commit.assert_called_once()

@pytest.mark.asyncio
async def test_sync_worker_timeout():
    """Kiểm tra SyncWorker xử lý khi chờ ACK bị timeout"""
    with patch("sync_worker.central_client") as mock_client, \
         patch("sync_worker.SessionLocal") as mock_session_local:
        
        mock_db = MagicMock()
        mock_session_local.return_value = mock_db
        
        mock_log = ZKTRawAttendance(
            id=1, device_id="D1", user_id="101", 
            timestamp=datetime.now(), status=1, punch=0, is_synced=0
        )
        mock_db.query.return_value.filter.return_value.order_by.return_value.limit.return_value.all.return_value = [mock_log]
        
        # Giả lập timeout bằng Future chưa bao giờ kết thúc
        mock_future = asyncio.Future()
        mock_client.push_attendance_sync.return_value = mock_future
        mock_client.is_connected = True
        
        worker = SyncWorker()
        worker.batch_size = 5
        
        # Patch asyncio.wait_for để ném TimeoutError nhanh hơn cho test
        with patch("asyncio.wait_for", side_effect=asyncio.TimeoutError()):
            await worker.sync_batch()
        
        # Kiểm tra commit KHÔNG được gọi (do timeout)
        mock_db.commit.assert_not_called()
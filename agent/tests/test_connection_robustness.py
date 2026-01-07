import asyncio
import pytest
from unittest.mock import MagicMock, patch
from central_client import CentralClient
from tests.test_utils import LoopBreak

@pytest.mark.asyncio
async def test_singleton_connection_task(test_ctx):
    """Xác nhận chỉ có duy nhất 1 loop kết nối được chạy tại một thời điểm"""
    client = CentralClient()
    client.url = "ws://localhost:9999"
    
    # Mock websockets.connect để nó đứng đợi mãi mãi
    async def slow_connect(*args, **kwargs):
        await asyncio.sleep(10)
        
    with patch("websockets.connect", side_effect=slow_connect):
        # Chạy lần 1
        task1 = asyncio.create_task(client.connect())
        await asyncio.sleep(0.1)
        assert client._connection_task is not None
        
        # Chạy lần 2 khi lần 1 đang chạy
        # Log debug "Connection task already running" sẽ được trigger
        task2 = asyncio.create_task(client.connect())
        await asyncio.sleep(0.1)
        
        # Task 2 sẽ kết thúc ngay lập tức mà không gán lại _connection_task
        assert client._connection_task == task1
        
        task1.cancel()
        task2.cancel()

@pytest.mark.asyncio
async def test_subtask_error_triggers_retry(test_ctx):
    """Xác nhận lỗi từ task con (receiver) ném được ra loop ngoài để retry"""
    client = CentralClient()
    client.url = "ws://localhost:9999"
    client.attempts = 0
    
    # Mock receiver để ném lỗi ConnectionResetError ngay khi vừa chạy
    async def mock_receiver(*args):
        raise ConnectionResetError("Reset by peer")

    # Mock connect thành công nhưng ném lỗi ở sub-task
    mock_ws = MagicMock()
    mock_ws.send.return_value = asyncio.Future()
    mock_ws.send.return_value.set_result(None)
    mock_ws.close.return_value = asyncio.Future()
    mock_ws.close.return_value.set_result(None)

    with patch("websockets.connect") as mock_connect:
        mock_connect.return_value.__aenter__.return_value = mock_ws
        with patch.object(client, "_receiver", side_effect=mock_receiver):
            with patch("asyncio.sleep") as mock_sleep:
                # Ném LoopBreak ở vòng lặp thứ 2 để thoát connect()
                mock_sleep.side_effect = [None, LoopBreak("Exit after one retry")]
                
                try:
                    await client.connect()
                except LoopBreak:
                    pass
                
                # Xác nhận attempts tăng lên 1 (do lỗi ở receiver được ném ra ngoài)
                assert client.attempts == 1
                # Xác nhận có gọi sleep để backoff
                assert mock_sleep.called

@pytest.mark.asyncio
async def test_mandatory_delay_enforcement(test_ctx):
    """Xác nhận cơ chế bảo vệ Mandatory Delay (min 1s) hoạt động khi crash quá nhanh"""
    client = CentralClient()
    client.url = "ws://localhost:9999"
    
    # Mock websockets.connect ném lỗi ngay lập tức (không tốn thời gian)
    with patch("websockets.connect", side_effect=Exception("Instant fail")):
        with patch("asyncio.sleep") as mock_sleep:
            # Vòng 1: Crash ngay lập tức -> sleep(5.0) (mặc định cho Unexpected error)
            # Vòng 2: Thoát
            mock_sleep.side_effect = [None, LoopBreak()]
            
            try:
                await client.connect()
            except LoopBreak:
                pass
            
            # Kiểm tra các lần gọi sleep
            # Lần 1: Sleep 5.0 (backoff)
            # Lần 2: Sleep ~1.0 (mandatory delay trong finally)
            sleep_calls = [args[0] for args, _ in mock_sleep.call_args_list]
            assert any(4.9 <= s <= 5.1 for s in sleep_calls) # Backoff
            assert any(0.9 <= s <= 1.1 for s in sleep_calls) # Mandatory delay

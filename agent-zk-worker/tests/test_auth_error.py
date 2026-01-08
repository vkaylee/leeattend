import pytest
import asyncio
import json
from unittest.mock import MagicMock, patch
import main
from central_client import CentralClient

@pytest.mark.asyncio
async def test_central_client_error_callback(test_agent):
    """Kiểm tra CentralClient gọi callback khi nhận lỗi từ server"""
    client = test_agent.central_client
    mock_callback = MagicMock()
    # Chuyển callback thành async để mock được
    async def async_mock_callback(code, msg):
        mock_callback(code, msg)
    
    client.on_error_received = async_mock_callback
    
    # Giả lập message lỗi từ websocket
    error_msg = json.dumps({
        "type": "error",
        "code": "INVALID_TOKEN",
        "message": "Token mismatch"
    })
    
    # Mock websocket để receiver nhận được tin nhắn
    mock_ws = MagicMock()
    mock_ws.__aiter__.return_value = [error_msg]
    
    # Chạy receiver một vòng
    try:
        await asyncio.wait_for(client._receiver(mock_ws), timeout=0.1)
    except (asyncio.TimeoutError, StopAsyncIteration):
        pass
    
    # Cho task ẩn (asyncio.create_task) có thời gian chạy
    await asyncio.sleep(0.01)
        
    # Xác nhận callback được gọi đúng code và message
    mock_callback.assert_called_with("INVALID_TOKEN", "Token mismatch")
    
    # Xác nhận last_error_code được cập nhật
    assert client.last_error_code == "INVALID_TOKEN"

@pytest.mark.asyncio
async def test_central_client_retry_logic_auth_error(test_ctx, test_agent):
    """Kiểm tra logic retry của CentralClient khi gặp lỗi Auth sử dụng Factory"""
    client = test_agent.central_client
    client.url = "ws://localhost:9999"
    
    # Giả lập connection fail và set error code qua side effect
    def mock_connect(*args, **kwargs):
        client.last_error_code = "AGENT_NOT_TRUSTED"
        raise Exception("Connection Closed")

    with patch("websockets.connect", side_effect=mock_connect):
        from tests.test_utils import LoopBreak
        # Patch sleep riêng để dễ assert
        with patch("asyncio.sleep") as mock_sleep:
            # Ném LoopBreak ở lần gọi sleep thứ 2 (sau khi đã gọi mandatory delay 1s)
            mock_sleep.side_effect = [None, LoopBreak()]
            try:
                await client.connect()
            except LoopBreak:
                pass
            
            # Kiểm tra các lần gọi sleep
            # Lần 1: Mandatory delay (~1s) trong finally
            # Lần 2: Auth delay (>= 60s) trong _handle_connection_error
            sleep_calls = [args[0] for args, _ in mock_sleep.call_args_list]
            assert any(s >= 60 for s in sleep_calls)

@pytest.mark.asyncio
async def test_handle_central_error_queue(test_agent):
    """Kiểm tra handle_central_error đẩy đúng dữ liệu vào log_queue"""
    agent = test_agent
    # Dọn dẹp queue trước khi test
    while not agent.log_queue.empty():
        agent.log_queue.get_nowait()
        
    await agent.handle_central_error("AGENT_NOT_TRUSTED", "Please wait for approval")
    
    assert agent.log_queue.qsize() == 1
    item = await agent.log_queue.get()
    
    assert item["type"] == "system"
    assert item["level"] == "ERROR"
    assert "AGENT_NOT_TRUSTED" in item["message"]
    assert "Please wait for approval" in item["message"]

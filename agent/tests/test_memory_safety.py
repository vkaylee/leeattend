import asyncio
import pytest
from unittest.mock import MagicMock, patch
from central_client import CentralClient

@pytest.mark.asyncio
async def test_pending_acks_cleanup_on_timeout():
    """Kiểm tra việc dọn dẹp pending_acks khi gặp timeout (Memory Safety)"""
    client = CentralClient()
    client.agent_id = "TEST_AGENT"
    
    # Mock send_queue.put_nowait để không thực sự gửi
    client.send_queue = MagicMock()
    
    # 1. Giả lập việc push log
    future = client.push_attendance_sync("DEV1", {"test": "data"})
    msg_id = future.msg_id
    
    assert msg_id in client.pending_acks
    assert len(client.pending_acks) == 1
    
    # 2. Giả lập timeout trong code sử dụng (như SyncWorker)
    try:
        await asyncio.wait_for(future, timeout=0.01)
    except asyncio.TimeoutError:
        # Code sử dụng phải gọi discard_pending_ack
        client.discard_pending_ack(msg_id)
        
    # 3. Kiểm tra pending_acks phải trống
    assert msg_id not in client.pending_acks
    assert len(client.pending_acks) == 0

@pytest.mark.asyncio
async def test_pending_acks_cleanup_on_success():
    """Kiểm tra việc dọn dẹp pending_acks khi nhận được ACK thành công"""
    client = CentralClient()
    client.agent_id = "TEST_AGENT"
    client.send_queue = MagicMock()
    
    future = client.push_attendance_sync("DEV1", {"test": "data"})
    msg_id = future.msg_id
    assert len(client.pending_acks) == 1
    
    # Giả lập nhận ACK từ server
    ack_msg = json_message = '{"type": "ack", "msg_id": "' + msg_id + '", "status": "success"}'
    
    # Trong code thực tế, _receiver sẽ gọi pop
    # Ở đây ta giả lập logic của _receiver
    import json
    data = json.loads(ack_msg)
    if data["type"] == "ack":
        mid = data["msg_id"]
        if mid in client.pending_acks:
            f = client.pending_acks.pop(mid)
            f.set_result(data)
            
    assert future.done()
    assert len(client.pending_acks) == 0

import asyncio
import json
import pytest
import websockets
import os
from central_client import CentralClient
from unittest.mock import patch

class MockErrorServer:
    async def handler(self, websocket):
        async for message in websocket:
            data = json.loads(message)
            if data.get("type") == "handshake":
                error_msg = {
                    "type": "error",
                    "code": "DUPLICATE_ID",
                    "message": "Agent ID TEST_AGENT is already active on the server."
                }
                await websocket.send(json.dumps(error_msg))

@pytest.mark.asyncio
async def test_agent_exits_on_duplicate_id():
    """Kiểm tra Agent phải exit khi nhận lỗi DUPLICATE_ID từ server"""
    server = MockErrorServer()
    
    async with websockets.serve(server.handler, "127.0.0.1", 0) as ws_server:
        port = ws_server.sockets[0].getsockname()[1]
        server_url = f"ws://127.0.0.1:{port}"
        
        client = CentralClient()
        client.url = server_url
        client.agent_id = "TEST_AGENT"
        client.token = "TEST_TOKEN"
        
        # Vì os._exit(1) dừng tiến trình ngay lập tức, ta cần mock nó để không làm sập test runner
        with patch("os._exit") as mock_exit:
            # Chạy client connect
            # Ta dùng timeout vì client.connect() là vòng lặp vô tận
            try:
                await asyncio.wait_for(client.connect(), timeout=2.0)
            except asyncio.TimeoutError:
                pass
            
            # Kiểm tra xem os._exit(1) đã được gọi chưa
            mock_exit.assert_called_once_with(1)

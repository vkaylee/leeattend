import asyncio
import json
import pytest
import websockets
from datetime import datetime
from central_client import CentralClient

class MockCentralServer:
    def __init__(self):
        self.received_messages = []
        self.connected_event = asyncio.Event()

    async def handler(self, websocket):
        self.connected_event.set()
        try:
            async for message in websocket:
                data = json.loads(message)
                self.received_messages.append(data)
                
                if data.get("type") == "handshake":
                    # Đợi một chút để client khởi động xong _receiver
                    await asyncio.sleep(0.5)
                    command = {
                        "type": "command",
                        "id": "test_cmd_123",
                        "method": "ping",
                        "timestamp": datetime.now().isoformat()
                    }
                    await websocket.send(json.dumps(command))
        except websockets.exceptions.ConnectionClosed:
            pass

@pytest.mark.asyncio
async def test_central_client_connection_and_tunnel(test_ctx):
    """Test quy trình kết nối, handshake và phản hồi lệnh về server"""
    server = MockCentralServer()
    
    # Đảm bảo reset trạng thái trước khi bắt đầu
    from central_client import central_client
    central_client.is_connected = False
    while not central_client.send_queue.empty():
        central_client.send_queue.get_nowait()
    
    async with websockets.serve(server.handler, "127.0.0.1", 0) as ws_server:
        port = ws_server.sockets[0].getsockname()[1]
        server_url = f"ws://127.0.0.1:{port}"
        
        client = CentralClient()
        client.url = server_url
        client.agent_id = "TEST_AGENT"
        client.token = "TEST_TOKEN"
        
        # Thêm callback giả lập xử lý lệnh
        async def mock_handle_command(cmd):
            return {"success": True, "data": "pong"}
        client.on_command_received = mock_handle_command
        
        client_task = asyncio.create_task(client.connect())
        
        await asyncio.wait_for(server.connected_event.wait(), timeout=5.0)
        
        # Đợi phản hồi xuất hiện trong danh sách tin nhắn nhận được của Server (tối đa 5s)
        start_time = asyncio.get_event_loop().time()
        response = None
        while asyncio.get_event_loop().time() - start_time < 5.0:
            response = next((m for m in server.received_messages if m.get("type") == "response"), None)
            if response:
                break
            await asyncio.sleep(0.1)
            
        assert response is not None, f"Server did not receive response. Received: {server.received_messages}"
        assert response["type"] == "response"
        assert response["id"] == "test_cmd_123"
        assert response["status"] == "success"
        assert response["data"] == "pong"
        
        client_task.cancel()
        try:
            await client_task
        except asyncio.CancelledError:
            pass

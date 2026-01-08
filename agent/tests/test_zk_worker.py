import asyncio
import json
import pytest
import pytest_asyncio
from zk_worker import ZKWorker, ZKService
from unittest.mock import MagicMock, AsyncMock, patch

@pytest_asyncio.fixture
async def worker_server():
    worker = ZKWorker()
    server = await asyncio.start_server(worker.handle_rpc, '127.0.0.1', 0)
    host, port = server.sockets[0].getsockname()
    task = asyncio.create_task(server.serve_forever())
    yield host, port
    
    # Cleanup
    server.close()
    await server.wait_closed()
    task.cancel()
    try:
        await task
    except asyncio.CancelledError:
        pass

@pytest.mark.asyncio
async def test_rpc_method_not_found(worker_server):
    host, port = worker_server
    async with asyncio.timeout(5):
        reader, writer = await asyncio.open_connection(host, port)
        try:
            request = {
                "jsonrpc": "2.0",
                "method": "invalid.method",
                "params": {},
                "id": 1
            }
            writer.write(json.dumps(request).encode() + b"\n")
            await writer.drain()
            
            data = await reader.read(4096)
            response = json.loads(data.decode())
            
            assert response["id"] == 1
            assert response["error"]["code"] == -32601
        finally:
            writer.close()
            await writer.wait_closed()

@pytest.mark.asyncio
async def test_zk_connect_success(worker_server):
    host, port = worker_server
    async with asyncio.timeout(5):
        with patch('zk_worker.ZKService.connect', new_callable=AsyncMock) as mock_connect:
            mock_connect.return_value = True
            
            reader, writer = await asyncio.open_connection(host, port)
            try:
                request = {
                    "jsonrpc": "2.0",
                    "method": "zk.connect",
                    "params": {"device_id": "TEST_DEV", "ip": "1.1.1.1"},
                    "id": 2
                }
                writer.write(json.dumps(request).encode() + b"\n")
                await writer.drain()
                
                data = await reader.read(4096)
                response = json.loads(data.decode())
                
                assert response["result"] is True
            finally:
                writer.close()
                await writer.wait_closed()

@pytest.mark.asyncio
async def test_zk_get_attendance_data(worker_server):
    host, port = worker_server
    async with asyncio.timeout(5):
        mock_logs = [
            {"user_id": "101", "timestamp": "2026-01-08 10:00:00", "status": 1, "punch": 0},
            {"user_id": "102", "timestamp": "2026-01-08 10:05:00", "status": 1, "punch": 0}
        ]
        with patch('zk_worker.ZKService.get_attendance_logs', new_callable=AsyncMock) as mock_get:
            mock_get.return_value = mock_logs
            # Mock connect cũng cần thiết vì method handler gọi connect trước
            with patch('zk_worker.ZKService.connect', new_callable=AsyncMock):
                reader, writer = await asyncio.open_connection(host, port)
                try:
                    # 1. Connect device
                    writer.write(json.dumps({
                        "jsonrpc": "2.0", "method": "zk.connect", 
                        "params": {"device_id": "D1", "ip": "1.1.1.1"}, "id": 10
                    }).encode() + b"\n")
                    await writer.drain()
                    await reader.read(4096)

                    # 2. Get attendance
                    writer.write(json.dumps({
                        "jsonrpc": "2.0", "method": "zk.get_attendance", 
                        "params": {"device_id": "D1"}, "id": 11
                    }).encode() + b"\n")
                    await writer.drain()
                    
                    data = await reader.read(4096)
                    response = json.loads(data.decode())
                    
                    assert len(response["result"]) == 2
                    assert response["result"][0]["user_id"] == "101"
                finally:
                    writer.close()
                    await writer.wait_closed()

@pytest.mark.asyncio
async def test_zk_hardware_error_propagation(worker_server):
    host, port = worker_server
    async with asyncio.timeout(5):
        # Giả lập lỗi mạng khi kết nối máy ZK
        with patch('zk_worker.ZKService.connect', side_effect=Exception("Network Timeout")):
            reader, writer = await asyncio.open_connection(host, port)
            try:
                writer.write(json.dumps({
                    "jsonrpc": "2.0", "method": "zk.connect", 
                    "params": {"device_id": "ERR_DEV", "ip": "1.1.1.1"}, "id": 20
                }).encode() + b"\n")
                await writer.drain()
                
                data = await reader.read(4096)
                response = json.loads(data.decode())
                
                assert "error" in response
                assert "Network Timeout" in response["error"]["message"]
            finally:
                writer.close()
                await writer.wait_closed()

@pytest.mark.asyncio
async def test_zk_concurrent_requests(worker_server):
    host, port = worker_server
    async with asyncio.timeout(10):
        # Tạo 20 tasks gửi lệnh đồng thời
        async def send_req(i):
            reader, writer = await asyncio.open_connection(host, port)
            try:
                writer.write(json.dumps({
                    "jsonrpc": "2.0", "method": "zk.invalid", "id": i
                }).encode() + b"\n")
                await writer.drain()
                data = await reader.read(4096)
                return json.loads(data.decode())
            finally:
                writer.close()
                await writer.wait_closed()

        results = await asyncio.gather(*[send_req(i) for i in range(20)])
        assert len(results) == 20
        for i, res in enumerate(results):
            assert res["id"] in range(20)
            assert "error" in res

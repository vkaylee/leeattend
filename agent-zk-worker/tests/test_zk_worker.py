import pytest
import asyncio
import json
from unittest.mock import AsyncMock, MagicMock, patch
from zk_worker import ZKService, ZKWorker
from main import start_worker # Import the new start_worker function from main.py

# Mock the ZK library for ZKService tests
@pytest.fixture
def mock_zk_client():
    with patch('zk_worker.ZK') as mock_zk_class:
        mock_zk_instance = MagicMock()
        mock_conn_obj = MagicMock() # Represents the blocking connection object
        mock_zk_instance.connect.return_value = mock_conn_obj
        mock_zk_instance.disconnect.return_value = None
        mock_zk_class.return_value = mock_zk_instance
        yield mock_zk_instance # Yield the mock_zk_instance instead of mock_zk_class

@pytest.fixture
def zk_service_instance(mock_zk_client):
    return ZKService("TEST_DEV", "127.0.0.1")

@pytest.fixture
def zk_worker_instance():
    return ZKWorker()

@pytest.mark.asyncio
async def test_zk_service_connect(zk_service_instance, mock_zk_client):
    conn = await zk_service_instance.connect()
    mock_zk_client.connect.assert_called_once() # Assert directly on mock_zk_client.connect
    assert conn is not None

@pytest.mark.asyncio
async def test_zk_service_disconnect(zk_service_instance, mock_zk_client):
    zk_service_instance.conn = MagicMock() # Simulate a connected state with a non-awaitable mock
    await zk_service_instance.disconnect()
    mock_zk_client.disconnect.assert_called_once()
    assert zk_service_instance.conn is None

@pytest.mark.asyncio
async def test_zk_service_disconnect_no_connection(zk_service_instance, mock_zk_client):
    zk_service_instance.conn = None # Simulate no connection
    await zk_service_instance.disconnect()
    mock_zk_client.disconnect.assert_not_called()

@pytest.mark.asyncio
async def test_zk_service_get_attendance_logs(zk_service_instance, mock_zk_client):
    mock_conn = mock_zk_client.connect.return_value # The mocked connection object
    mock_log_entry = MagicMock()
    mock_log_entry.user_id = '1'
    mock_log_entry.timestamp.strftime.return_value = "2026-01-01 08:00:00"
    mock_log_entry.status = '0'
    mock_log_entry.punch = '0'
    mock_conn.get_attendance.return_value = [mock_log_entry] # This blocking call returns a list directly
    
    # Simulate connection being established
    zk_service_instance.conn = mock_conn
    logs = await zk_service_instance.get_attendance_logs()
    mock_conn.get_attendance.assert_called_once()
    assert logs[0]["user_id"] == '1'
    assert logs[0]["timestamp"] == "2026-01-01 08:00:00"

@pytest.mark.asyncio
async def test_zk_service_sync_time(zk_service_instance, mock_zk_client):
    mock_conn = mock_zk_client.connect.return_value
    zk_service_instance.conn = mock_conn # Simulate connection being established
    with patch('zk_worker.datetime') as mock_datetime:
        mock_datetime.now.return_value = MagicMock(strftime=lambda x: "2026-01-01 10:00:00")
        sync_time_str = await zk_service_instance.sync_time()
        mock_conn.set_time.assert_called_once_with(mock_datetime.now.return_value)
        assert sync_time_str == "2026-01-01 10:00:00"

@pytest.mark.asyncio
async def test_zk_service_get_users(zk_service_instance, mock_zk_client):
    mock_conn = mock_zk_client.connect.return_value
    mock_user_entry = MagicMock()
    mock_user_entry.uid = '1'
    mock_user_entry.user_id = 'U1'
    mock_user_entry.name = 'User One'
    mock_conn.get_users.return_value = [mock_user_entry] # This blocking call returns a list directly
    
    # Simulate connection being established
    zk_service_instance.conn = mock_conn
    users = await zk_service_instance.get_users()
    mock_conn.get_users.assert_called_once()
    assert users[0]["name"] == 'User One'

@pytest.mark.asyncio
async def test_handle_rpc_zk_connect(zk_worker_instance):
    reader = AsyncMock()
    writer = AsyncMock()
    
    request = {"jsonrpc": "2.0", "method": "zk.connect", "params": {"device_id": "D1", "ip": "1.1.1.1"}, "id": 1}
    reader.read.return_value = json.dumps(request).encode() + b"\n"
    reader.read.side_effect = [json.dumps(request).encode() + b"\n", b""] # First read then EOF

    with patch('zk_worker.ZKService', autospec=True) as MockZKService:
        mock_service_instance = MockZKService.return_value
        mock_service_instance.connect = AsyncMock(return_value=True)

        await zk_worker_instance.handle_rpc(reader, writer)
        
        MockZKService.assert_called_with("D1", "1.1.1.1", 4370)
        mock_service_instance.connect.assert_called_once()
        writer.write.assert_called_once()
        response = json.loads(writer.write.call_args[0][0].decode().strip())
        assert response["result"] == True

@pytest.mark.asyncio
async def test_handle_rpc_get_attendance(zk_worker_instance):
    reader = AsyncMock()
    writer = AsyncMock()
    
    mock_logs = [{"user_id": "1", "timestamp": "2026-01-01 08:00:00", "status": "0", "punch": "0"}]
    
    with patch('zk_worker.ZKService', autospec=True) as MockZKService:
        mock_service_instance = MockZKService.return_value
        mock_service_instance.get_attendance_logs = AsyncMock(return_value=mock_logs)
        zk_worker_instance.services["D1"] = mock_service_instance # Pre-connect

        request = {"jsonrpc": "2.0", "method": "zk.get_attendance", "params": {"device_id": "D1"}, "id": 1}
        reader.read.return_value = json.dumps(request).encode() + b"\n"
        reader.read.side_effect = [json.dumps(request).encode() + b"\n", b""]

        await zk_worker_instance.handle_rpc(reader, writer)
        
        mock_service_instance.get_attendance_logs.assert_called_once()
        response = json.loads(writer.write.call_args[0][0].decode().strip())
        assert response["result"] == mock_logs

@pytest.mark.asyncio
async def test_handle_rpc_zk_disconnect(zk_worker_instance):
    reader = AsyncMock()
    writer = AsyncMock()
    
    with patch('zk_worker.ZKService', autospec=True) as MockZKService:
        mock_service_instance = MockZKService.return_value
        mock_service_instance.disconnect = AsyncMock()
        zk_worker_instance.services["D1"] = mock_service_instance # Pre-connect

        request = {"jsonrpc": "2.0", "method": "zk.disconnect", "params": {"device_id": "D1"}, "id": 1}
        reader.read.return_value = json.dumps(request).encode() + b"\n"
        reader.read.side_effect = [json.dumps(request).encode() + b"\n", b""]

        await zk_worker_instance.handle_rpc(reader, writer)
        
        mock_service_instance.disconnect.assert_called_once()
        assert "D1" not in zk_worker_instance.services
        response = json.loads(writer.write.call_args[0][0].decode().strip())
        assert response["result"] == True

@pytest.mark.asyncio
async def test_handle_rpc_method_not_found(zk_worker_instance):
    reader = AsyncMock()
    writer = AsyncMock()
    
    request = {"jsonrpc": "2.0", "method": "zk.invalid_method", "params": {}, "id": 1}
    reader.read.return_value = json.dumps(request).encode() + b"\n"
    reader.read.side_effect = [json.dumps(request).encode() + b"\n", b""]

    await zk_worker_instance.handle_rpc(reader, writer)
    
    response = json.loads(writer.write.call_args[0][0].decode().strip())
    assert response["error"]["code"] == -32601

@pytest.mark.asyncio
async def test_handle_rpc_hardware_error_propagation(zk_worker_instance):
    reader = AsyncMock()
    writer = AsyncMock()
    
    with patch('zk_worker.ZKService', autospec=True) as MockZKService:
        mock_service_instance = MockZKService.return_value
        mock_service_instance.connect.side_effect = Exception("Network Timeout")
        
        request = {"jsonrpc": "2.0", "method": "zk.connect", "params": {"device_id": "ERR_DEV", "ip": "1.1.1.1"}, "id": 1}
        reader.read.return_value = json.dumps(request).encode() + b"\n"
        reader.read.side_effect = [json.dumps(request).encode() + b"\n", b""]

        await zk_worker_instance.handle_rpc(reader, writer)
        
        response = json.loads(writer.write.call_args[0][0].decode().strip())
        assert response["error"]["message"] == "Network Timeout"

@pytest.mark.asyncio
async def test_zk_worker_run():
    with patch('zk_worker.asyncio.start_server', new_callable=AsyncMock) as mock_start_server:
        mock_server = AsyncMock()
        mock_start_server.return_value = mock_server
        mock_server.serve_forever = AsyncMock()

        worker = ZKWorker()
        
        # We need to make worker.run() exit, for example, by raising an exception
        mock_server.serve_forever.side_effect = asyncio.CancelledError

        with pytest.raises(asyncio.CancelledError):
            await worker.run("127.0.0.1", 5000)

        mock_start_server.assert_called_once_with(worker.handle_rpc, "127.0.0.1", 5000)
        mock_server.serve_forever.assert_called_once()

@pytest.mark.asyncio
@pytest.mark.asyncio
async def test_start_worker_execution(): # Renamed from test_main_py_execution
    with patch("main.ZKWorker", autospec=True) as MockZKWorker,          patch("asyncio.Event", autospec=True) as MockEvent:
        mock_worker_instance = MockZKWorker.return_value
        mock_worker_instance.run = AsyncMock()
        
        # Mock event to resolve immediately
        mock_event_instance = MockEvent.return_value
        mock_event_instance.wait = AsyncMock()

        # Call the new start_worker function
        await start_worker()

        MockZKWorker.assert_called_once()
        mock_worker_instance.run.assert_called_once()

async def test_handle_rpc_zk_get_users(zk_worker_instance):
    reader = AsyncMock()
    writer = AsyncMock()
    
    mock_users = [{"uid": 1, "user_id": "U1", "name": "User One"}]
    
    with patch('zk_worker.ZKService', autospec=True) as MockZKService:
        mock_service_instance = MockZKService.return_value
        mock_service_instance.get_users = AsyncMock(return_value=mock_users)
        zk_worker_instance.services["D1"] = mock_service_instance # Pre-connect

        request = {"jsonrpc": "2.0", "method": "zk.get_users", "params": {"device_id": "D1"}, "id": 1}
        reader.read.return_value = json.dumps(request).encode() + b"\n"
        reader.read.side_effect = [json.dumps(request).encode() + b"\n", b""]

        await zk_worker_instance.handle_rpc(reader, writer)
        
        mock_service_instance.get_users.assert_called_once()
        response = json.loads(writer.write.call_args[0][0].decode().strip())
        assert response["result"] == mock_users

@pytest.mark.asyncio
async def test_handle_rpc_zk_sync_time(zk_worker_instance):
    reader = AsyncMock()
    writer = AsyncMock()
    
    mock_time = "2026-01-01 10:00:00"
    
    with patch('zk_worker.ZKService', autospec=True) as MockZKService:
        mock_service_instance = MockZKService.return_value
        mock_service_instance.sync_time = AsyncMock(return_value=mock_time)
        zk_worker_instance.services["D1"] = mock_service_instance # Pre-connect

        request = {"jsonrpc": "2.0", "method": "zk.sync_time", "params": {"device_id": "D1"}, "id": 1}
        reader.read.return_value = json.dumps(request).encode() + b"\n"
        reader.read.side_effect = [json.dumps(request).encode() + b"\n", b""]

        await zk_worker_instance.handle_rpc(reader, writer)
        
        mock_service_instance.sync_time.assert_called_once()
        response = json.loads(writer.write.call_args[0][0].decode().strip())
        assert response["result"] == mock_time

@pytest.mark.asyncio
async def test_handle_rpc_json_decode_error(zk_worker_instance):
    reader = AsyncMock()
    writer = AsyncMock()
    
    reader.read.return_value = b"invalid json\n"
    reader.read.side_effect = [b"invalid json\n", b""]

    await zk_worker_instance.handle_rpc(reader, writer)
    # Expect no response written, as error occurs before response construction
    writer.write.assert_not_called()
    writer.close.assert_called_once()

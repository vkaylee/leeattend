import pytest
import asyncio
import json
from unittest.mock import AsyncMock, MagicMock, patch
from zk_worker import ZKService, ZKWorkerClient

@pytest.fixture
def mock_zk_client():
    with patch('zk_worker.ZK') as mock_zk_class:
        mock_zk_instance = MagicMock()
        mock_conn_obj = MagicMock()
        mock_zk_instance.connect.return_value = mock_conn_obj
        mock_zk_instance.disconnect.return_value = None
        mock_zk_class.return_value = mock_zk_instance
        yield mock_zk_instance

@pytest.fixture
def zk_service_instance(mock_zk_client):
    return ZKService('127.0.0.1')

@pytest.fixture
def zk_worker_instance():
    return ZKWorkerClient()

@pytest.mark.asyncio
async def test_zk_service_connect(zk_service_instance, mock_zk_client):
    conn = await zk_service_instance.connect()
    mock_zk_client.connect.assert_called_once()
    assert conn is not None

@pytest.mark.asyncio
async def test_zk_service_disconnect(zk_service_instance, mock_zk_client):
    zk_service_instance.conn = MagicMock()
    await zk_service_instance.disconnect()
    mock_zk_client.disconnect.assert_called_once()
    assert zk_service_instance.conn is None

@pytest.mark.asyncio
async def test_zk_service_get_attendance_logs(zk_service_instance, mock_zk_client):
    mock_conn = mock_zk_client.connect.return_value
    mock_log_entry = MagicMock()
    mock_log_entry.user_id = '1'
    mock_log_entry.timestamp.strftime.return_value = '2026-01-01 08:00:00'
    mock_conn.get_attendance.return_value = [mock_log_entry]
    zk_service_instance.conn = mock_conn
    logs = await zk_service_instance.get_attendance_logs()
    assert logs[0]['user_id'] == '1'

def create_mock_writer():
    writer = MagicMock()
    writer.write = MagicMock()
    writer.drain = AsyncMock()
    return writer

@pytest.mark.asyncio
async def test_handle_requests_device_connect(zk_worker_instance):
    reader = AsyncMock()
    zk_worker_instance.writer = create_mock_writer()
    request = {'jsonrpc': '2.0', 'method': 'device.connect', 'params': {'ip': '1.1.1.1'}, 'id': 1}
    reader.readline.side_effect = [json.dumps(request).encode() + b'\n', b'']
    with patch('zk_worker.ZKService', autospec=True) as MockZKService:
        mock_service_instance = MockZKService.return_value
        mock_service_instance.connect = AsyncMock()
        await zk_worker_instance.handle_requests(reader)
        MockZKService.assert_called_with('1.1.1.1', 4370)
        mock_service_instance.connect.assert_called_once()
        zk_worker_instance.writer.write.assert_called_once()
        response = json.loads(zk_worker_instance.writer.write.call_args[0][0].decode().strip())
        assert response['result'] == True

@pytest.mark.asyncio
async def test_handle_requests_method_not_found(zk_worker_instance):
    reader = AsyncMock()
    zk_worker_instance.writer = create_mock_writer()
    request = {'jsonrpc': '2.0', 'method': 'invalid_method', 'params': {}, 'id': 1}
    reader.readline.side_effect = [json.dumps(request).encode() + b'\n', b'']
    await zk_worker_instance.handle_requests(reader)
    response = json.loads(zk_worker_instance.writer.write.call_args[0][0].decode().strip())
    assert response['error']['code'] == -32601

@pytest.mark.asyncio
async def test_handle_requests_empty_line(zk_worker_instance):
    reader = AsyncMock()
    zk_worker_instance.writer = create_mock_writer()
    request = {'jsonrpc': '2.0', 'method': 'common.ping', 'params': {}, 'id': 1}
    reader.readline.side_effect = [b'\n', json.dumps(request).encode() + b'\n', b'']
    await zk_worker_instance.handle_requests(reader)
    zk_worker_instance.writer.write.assert_called_once()
    response = json.loads(zk_worker_instance.writer.write.call_args[0][0].decode().strip())
    assert response['result'] == 'pong'

@pytest.mark.asyncio
async def test_zk_worker_run(zk_worker_instance):
    with patch('asyncio.open_connection', new_callable=AsyncMock) as mock_open:
        mock_reader = AsyncMock()
        mock_writer = create_mock_writer()
        mock_open.return_value = (mock_reader, mock_writer)
        mock_reader.readline.side_effect = [b'']
        task = asyncio.create_task(zk_worker_instance.run())
        await asyncio.sleep(0.1)
        task.cancel()
        mock_open.assert_called()

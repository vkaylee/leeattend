import pytest
import os
import json
from fastapi.testclient import TestClient
from unittest.mock import MagicMock, patch
from datetime import datetime
from main import app, services, ZKService, processed_logs, load_log_cache, save_log_cache, LOG_CACHE_FILE

client = TestClient(app)

class MockLog:
    def __init__(self, user_id, timestamp, status, punch):
        self.user_id = user_id
        self.timestamp = timestamp
        self.status = status
        self.punch = punch

class MockUser:
    def __init__(self, uid, user_id, name, privilege, password, group_id, card):
        self.uid = uid
        self.user_id = user_id
        self.name = name
        self.privilege = privilege
        self.password = password
        self.group_id = group_id
        self.card = card

@pytest.fixture(autouse=True)
def setup_teardown():
    # Setup: Ensure a test service exists
    if "TEST_01" not in services:
        services["TEST_01"] = ZKService("TEST_01", "127.0.0.1", 4370)
    yield
    # Teardown: Clean up cache file if created during tests
    if os.path.exists(LOG_CACHE_FILE):
        os.remove(LOG_CACHE_FILE)
    processed_logs.clear()

def test_ping():
    response = client.get("/ping")
    assert response.status_code == 200
    assert response.json() == {"message": "pong"}

@patch("pyzk2.ZK.connect")
def test_list_devices(mock_connect):
    mock_conn = MagicMock()
    mock_connect.return_value = mock_conn
    mock_conn.get_device_name.return_value = "Test Device"
    mock_conn.get_firmware_version.return_value = "v1"
    mock_conn.get_users.return_value = [1, 2]

    response = client.get("/devices")
    assert response.status_code == 200
    data = response.json()
    assert any(d["id"] == "TEST_01" for d in data)

@patch("pyzk2.ZK.connect")
def test_fetch_logs_multi(mock_connect):
    mock_conn = MagicMock()
    mock_connect.return_value = mock_conn
    mock_logs = [MockLog("user1", datetime(2026, 1, 6, 8, 0, 0), 1, 0)]
    mock_conn.get_attendance.return_value = mock_logs

    response = client.get("/devices/TEST_01/logs")
    assert response.status_code == 200
    data = response.json()
    assert data["count"] == 1
    assert data["logs"][0]["user_id"] == "user1"

@patch("pyzk2.ZK.connect")
def test_list_users_multi(mock_connect):
    mock_conn = MagicMock()
    mock_connect.return_value = mock_conn
    mock_users = [MockUser(1, "1", "Admin", 0, "123", 1, "0")]
    mock_conn.get_users.return_value = mock_users

    response = client.get("/devices/TEST_01/users")
    assert response.status_code == 200
    data = response.json()
    assert data["count"] == 1
    assert data["users"][0]["name"] == "Admin"

@patch("pyzk2.ZK.connect")
def test_sync_time_multi(mock_connect):
    mock_conn = MagicMock()
    mock_connect.return_value = mock_conn
    
    response = client.post("/devices/TEST_01/sync-time")
    assert response.status_code == 200
    assert "new_time" in response.json()

def test_device_not_found():
    response = client.get("/devices/NON_EXISTENT/logs")
    assert response.status_code == 404

def test_cache_persistence():
    # Manually add a log signature
    sig = "TEST_01_user1_2026-01-06 10:00:00"
    processed_logs.add(sig)
    
    # Save to file
    save_log_cache()
    assert os.path.exists(LOG_CACHE_FILE)
    
    # Clear RAM
    processed_logs.clear()
    
    # Load from file
    load_log_cache()
    assert sig in processed_logs

def test_parse_attendance_log():
    """Test unit logic for parsing a single log entry"""
    service_inst = ZKService("TEST_01", "127.0.0.1", 4370)
    mock_raw_log = MockLog("user99", datetime(2026, 1, 6, 12, 0, 0), 1, 0)
    
    parsed = service_inst._parse_attendance_log(mock_raw_log)
    
    assert parsed["user_id"] == "user99"
    assert parsed["device_id"] == "TEST_01"
    assert parsed["timestamp"] == "2026-01-06 12:00:00"

def test_format_attendance_display():
    """Test console output formatting"""
    from main import format_attendance_display
    log_data = {
        "user_id": "U1",
        "timestamp": "2026-01-01 10:00:00",
        "status": 1
    }
    output = format_attendance_display("DEV1", log_data)
    
    assert "DEV1" in output
    assert "U1" in output
    assert "2026-01-01 10:00:00" in output

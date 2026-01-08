import pytest
import asyncio
from central_client import CentralClient

@pytest.mark.asyncio
async def test_push_log_to_queue():
    client = CentralClient()
    client.agent_id = "TEST_AGENT"
    
    log_data = {"user_id": "123", "timestamp": "2026-01-07 10:00:00"}
    client.push_log("DEV1", log_data)
    
    assert client.send_queue.qsize() == 1
    msg = await client.send_queue.get()
    assert msg["type"] == "attendance"
    assert msg["agent_id"] == "TEST_AGENT"
    assert msg["data"] == log_data

@pytest.mark.asyncio
async def test_push_system_event_to_queue():
    client = CentralClient()
    client.agent_id = "TEST_AGENT"
    
    client.push_system_event("WARN", "Disk full", "DEV1")
    
    assert client.send_queue.qsize() == 1
    msg = await client.send_queue.get()
    assert msg["type"] == "system"
    assert msg["level"] == "WARN"
    assert msg["message"] == "Disk full"

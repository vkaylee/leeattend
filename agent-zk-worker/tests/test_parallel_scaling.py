import pytest
import asyncio
import time
import json
from unittest.mock import MagicMock, AsyncMock, patch
from zk_worker import ZKWorkerClient, ZKService

# Mock ZKService that sleeps
class SlowZKService:
    def __init__(self, ip, port):
        pass
    async def connect(self):
        pass
    async def get_time(self):
        await asyncio.sleep(0.5)
        return "2026-01-01 12:00:00"
    async def get_attendance_logs(self):
        return []
    async def sync_time(self):
        return "ok"

@pytest.mark.asyncio
async def test_worker_parallel_scaling():
    # Test cases: (max_parallel, expected_time_factor)
    # sending 2 requests taking 0.5s each.
    # if parallel=1, time approx 1.0s
    # if parallel=2, time approx 0.5s

    print("\n--- Testing Parallel=1 (Default) ---")
    duration_1 = await run_worker_test(max_parallel=1, requests_count=2)
    print(f"Parallel=1 took {duration_1:.4f}s")
    assert duration_1 >= 1.0, f"Parallel=1 should take at least 1.0s (sequential), took {duration_1}"

    print("\n--- Testing Parallel=2 (Scaled Up) ---")
    duration_2 = await run_worker_test(max_parallel=2, requests_count=2)
    print(f"Parallel=2 took {duration_2:.4f}s")
    assert duration_2 < 0.8, f"Parallel=2 should take approx 0.5s (concurrent), took {duration_2}"
    
    print("\n--- Testing Parallel=1 (Scaled Down) ---")
    duration_3 = await run_worker_test(max_parallel=1, requests_count=2)
    print(f"Parallel=1 took {duration_3:.4f}s")
    assert duration_3 >= 1.0, f"Parallel=1 should take at least 1.0s, took {duration_3}"

async def run_worker_test(max_parallel, requests_count):
    client = ZKWorkerClient(max_parallel=max_parallel)
    # Mock writer using a real class-like mock that supports context manager
    mock_writer = AsyncMock()
    mock_writer.drain = AsyncMock()
    mock_writer.write = MagicMock()
    client.writer = mock_writer
    
    # Create a stream reader populated with requests
    reader = AsyncMock()
    
    requests = []
    for i in range(requests_count):
        req = {
            "jsonrpc": "2.0",
            "method": "device.get_time",
            "id": i
        }
        requests.append(json.dumps(req).encode() + b"\n")
    requests.append(b"") # End of stream
    
    reader.readuntil.side_effect = requests
    
    # Mock ZKService to be slow
    with patch("zk_worker.ZKService", side_effect=SlowZKService):
        # Pre-initialize service so we don"t fail on "Device not connected"
        client.service = SlowZKService("1.1.1.1", 4370)
        
        start_time = time.time()
        await client.handle_requests(reader)
        end_time = time.time()
        
    return end_time - start_time

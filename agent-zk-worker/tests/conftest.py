import pytest
import asyncio
from zk_worker import ZKWorker, ZKService

@pytest.fixture
async def test_agent():
    """Tạo một instance Worker cho mỗi test"""
    worker = ZKWorker()
    yield worker
    # Cleanup nếu cần
    # await worker.stop()

@pytest.fixture
async def test_ctx(test_agent):
    """Fixture tương thích ngược cho các test cũ sử dụng test_ctx"""
    from tests.test_utils import AsyncTestFactory
    factory = AsyncTestFactory()
    yield factory

@pytest.fixture(scope="session")
def event_loop():
    try:
        loop = asyncio.get_running_loop()
    except RuntimeError:
        loop = asyncio.new_event_loop()
    yield loop
    loop.close()
import pytest
import asyncio
from app_container import AttendanceAgent
import main

@pytest.fixture
async def test_agent():
    """Tạo một instance Agent hoàn toàn cô lập cho mỗi test"""
    # Khởi tạo agent với config giả lập
    agent = AttendanceAgent(
        agent_id="TEST_AGENT",
        token="TEST_TOKEN",
        server_url="ws://localhost:9999"
    )
    
    # Ghi đè singleton trong main để API TestClient có thể dùng
    main.agent_instance = agent
    
    yield agent
    
    # Cleanup triệt để
    await agent.stop()
    main.agent_instance = None

@pytest.fixture
async def test_ctx(test_agent):
    """Fixture tương thích ngược cho các test cũ sử dụng test_ctx"""
    from tests.test_utils import AsyncTestFactory
    factory = AsyncTestFactory()
    # Factory hiện tại sẽ làm việc trên instance được inject
    yield factory

# Tùy chỉnh pytest-asyncio để dùng loop mặc định
@pytest.fixture(scope="session")
def event_loop():
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()

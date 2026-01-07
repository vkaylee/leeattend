import pytest
from models import ZKTRawAttendance
from database import SessionLocal
from datetime import datetime, timedelta

@pytest.fixture(autouse=True)
def setup_db():
    """Dọn dẹp và nạp dữ liệu mẫu vào DB trước mỗi test"""
    db = SessionLocal()
    db.query(ZKTRawAttendance).delete()
    
    # Tạo dữ liệu mẫu
    logs = [
        ZKTRawAttendance(device_id="DEV1", user_id="101", timestamp=datetime.now() - timedelta(days=1), status=1, punch=0, is_synced=1),
        ZKTRawAttendance(device_id="DEV2", user_id="102", timestamp=datetime.now() - timedelta(hours=5), status=1, punch=0, is_synced=0),
        ZKTRawAttendance(device_id="DEV1", user_id="101", timestamp=datetime.now(), status=1, punch=0, is_synced=0),
    ]
    db.add_all(logs)
    db.commit()
    db.close()
    yield
    db = SessionLocal()
    db.query(ZKTRawAttendance).delete()
    db.commit()
    db.close()

@pytest.mark.asyncio
async def test_query_all_raw_logs(test_agent):
    """Test logic truy vấn dữ liệu từ repository trực tiếp"""
    res = test_agent.repository.query_logs()
    assert len(res) == 3

@pytest.mark.asyncio
async def test_filter_by_device(test_agent):
    res = test_agent.repository.query_logs(device_id="DEV2")
    assert len(res) == 1
    assert res[0].device_id == "DEV2"

@pytest.mark.asyncio
async def test_filter_by_user(test_agent):
    res = test_agent.repository.query_logs(user_id="101")
    assert len(res) == 2

@pytest.mark.asyncio
async def test_filter_by_date_range(test_agent):
    start = datetime.now() - timedelta(hours=10)
    res = test_agent.repository.query_logs(start_date=start)
    assert len(res) == 2
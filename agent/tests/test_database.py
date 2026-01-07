import pytest
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from datetime import datetime
from models import Base, ZKTRawAttendance

# Thiết lập database in-memory để test
SQLALCHEMY_DATABASE_URL = "sqlite:///:memory:"
engine = create_engine(SQLALCHEMY_DATABASE_URL, connect_args={"check_same_thread": False})
TestingSessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

@pytest.fixture
def db_session():
    # Tạo bảng
    Base.metadata.create_all(bind=engine)
    db = TestingSessionLocal()
    try:
        yield db
    finally:
        db.close()
        # Xóa bảng sau khi test xong
        Base.metadata.drop_all(bind=engine)

def test_create_raw_attendance(db_session):
    # 1. Arrange
    timestamp = datetime(2026, 1, 7, 10, 30, 0)
    new_log = ZKTRawAttendance(
        device_id="DEV01",
        user_id="101",
        timestamp=timestamp,
        status=1,
        punch=0
    )

    # 2. Act
    db_session.add(new_log)
    db_session.commit()
    db_session.refresh(new_log)

    # 3. Assert
    assert new_log.id is not None
    assert new_log.device_id == "DEV01"
    assert new_log.user_id == "101"
    assert new_log.timestamp == timestamp
    assert new_log.status == 1
    assert new_log.punch == 0

def test_query_attendance_logs(db_session):
    # Arrange
    logs = [
        ZKTRawAttendance(device_id="D1", user_id="U1", timestamp=datetime.now(), status=1, punch=0),
        ZKTRawAttendance(device_id="D1", user_id="U2", timestamp=datetime.now(), status=1, punch=0),
    ]
    db_session.add_all(logs)
    db_session.commit()

    # Act
    queried_logs = db_session.query(ZKTRawAttendance).all()

    # Assert
    assert len(queried_logs) == 2
    assert queried_logs[0].user_id == "U1"
    assert queried_logs[1].user_id == "U2"

import pytest
from fastapi.testclient import TestClient
from sqlalchemy import create_engine, StaticPool
from sqlalchemy.orm import sessionmaker
from datetime import datetime

from main import app
from database import get_db
from models import Base, ZKTRawAttendance

# Setup SQLite in-memory with StaticPool for sharing connection
SQLALCHEMY_DATABASE_URL = "sqlite://"
engine = create_engine(
    SQLALCHEMY_DATABASE_URL,
    connect_args={"check_same_thread": False},
    poolclass=StaticPool,
)
TestingSessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

# Create tables once for all tests
Base.metadata.create_all(bind=engine)

@pytest.fixture
def db_session():
    connection = engine.connect()
    transaction = connection.begin()
    session = TestingSessionLocal(bind=connection)
    
    # Seed data
    logs = [
        ZKTRawAttendance(device_id="DEV1", user_id="101", timestamp=datetime(2026, 1, 1, 8, 0), status=1, punch=0),
        ZKTRawAttendance(device_id="DEV1", user_id="102", timestamp=datetime(2026, 1, 2, 8, 0), status=1, punch=0),
        ZKTRawAttendance(device_id="DEV2", user_id="101", timestamp=datetime(2026, 1, 3, 8, 0), status=1, punch=0),
    ]
    session.add_all(logs)
    session.commit()
    
    yield session
    
    session.close()
    transaction.rollback()
    connection.close()

@pytest.fixture
def client(db_session):
    def override_get_db():
        yield db_session
    app.dependency_overrides[get_db] = override_get_db
    yield TestClient(app)
    app.dependency_overrides.clear()

def test_query_all_raw_logs(client):
    response = client.get("/attendance/raw")
    assert response.status_code == 200
    data = response.json()
    assert len(data) == 3

def test_filter_by_device(client):
    response = client.get("/attendance/raw?device_id=DEV2")
    assert response.status_code == 200
    data = response.json()
    assert len(data) == 1
    assert data[0]["device_id"] == "DEV2"

def test_filter_by_user(client):
    response = client.get("/attendance/raw?user_id=101")
    assert response.status_code == 200
    data = response.json()
    assert len(data) == 2

def test_filter_by_date_range(client):
    response = client.get("/attendance/raw?start_date=2026-01-02T00:00:00")
    assert response.status_code == 200
    data = response.json()
    assert len(data) == 2
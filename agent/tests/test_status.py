import pytest
from fastapi.testclient import TestClient
from main import app, agent_stats, services
import sys
import os
sys.path.append(os.path.dirname(__file__))
from ZKService_mock import MockZKService

client = TestClient(app)

def test_health_endpoint():
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json()["status"] == "healthy"

def test_status_endpoint_initial():
    # Setup some dummy data in services to see it in status
    response = client.get("/status")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "running"
    assert "uptime_seconds" in data
    assert data["total_processed"] >= 0
    assert "devices" in data

def test_status_reflects_stats():
    # Gán giá trị giả lập
    agent_stats["total_processed"] = 99
    response = client.get("/status")
    assert response.json()["total_processed"] == 99

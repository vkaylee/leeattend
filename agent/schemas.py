from pydantic import BaseModel, ConfigDict
from datetime import datetime
from typing import Optional

class ZKTRawAttendanceBase(BaseModel):
    device_id: str
    user_id: str
    timestamp: datetime
    status: Optional[int] = None
    punch: Optional[int] = None

class ZKTRawAttendanceResponse(ZKTRawAttendanceBase):
    model_config = ConfigDict(from_attributes=True)
    
    id: int
    created_at: datetime

class DeviceStatus(BaseModel):
    device_id: str
    status: str
    last_sync: Optional[datetime] = None
    error: Optional[str] = None

class AgentStatus(BaseModel):
    status: str
    uptime_seconds: float
    total_processed: int
    queue_size: int
    devices: list[DeviceStatus]

from sqlalchemy import Column, Integer, String, DateTime, func
from sqlalchemy.orm import declarative_base

Base = declarative_base()

class ZKTRawAttendance(Base):
    """
    Lưu trữ dữ liệu chấm công thô (raw) trực tiếp từ máy ZKTeco.
    Dữ liệu này chưa qua xử lý logic nghiệp vụ (vào/ra, ca kíp).
    """
    __tablename__ = "zkt_raw_attendance"

    id = Column(Integer, primary_key=True, autoincrement=True)
    device_id = Column(String(50), nullable=False, index=True)
    user_id = Column(String(50), nullable=False, index=True)
    timestamp = Column(DateTime, nullable=False, index=True)
    status = Column(Integer, nullable=True) # Loại xác thực (vân tay, khuôn mặt, thẻ...)
    punch = Column(Integer, nullable=True)  # Trạng thái punch (Check-in, Check-out...)
    is_synced = Column(Integer, server_default="0", nullable=False, index=True) # 0: False, 1: True
    created_at = Column(DateTime, server_default=func.now())

    def __repr__(self):
        return f"<ZKTRawAttendance(user_id={self.user_id}, device_id={self.device_id}, timestamp={self.timestamp})>"
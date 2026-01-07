import logging
from typing import List, Optional
from datetime import datetime
from sqlalchemy.orm import Session
from models import ZKTRawAttendance

logger = logging.getLogger("zkteco-repository")

class AttendanceRepository:
    def __init__(self, session_factory):
        self.session_factory = session_factory

    def save_attendance(self, device_id: str, log_data: dict) -> bool:
        """Lưu một bản ghi chấm công vào database"""
        session: Session = self.session_factory()
        try:
            # Chuyển đổi string timestamp thành datetime object nếu cần
            ts = log_data['timestamp']
            if isinstance(ts, str):
                ts = datetime.strptime(ts, "%Y-%m-%d %H:%M:%S")
            
            db_log = ZKTRawAttendance(
                device_id=device_id,
                user_id=log_data['user_id'],
                timestamp=ts,
                status=log_data['status'],
                punch=log_data['punch'],
                is_synced=0
            )
            session.add(db_log)
            session.commit()
            return True
        except Exception as e:
            logger.error(f"Failed to save log to DB: {e}")
            session.rollback()
            return False
        finally:
            session.close()

    def get_unsynced_logs(self, device_ids: List[str], limit: int = 100) -> List[ZKTRawAttendance]:
        """Lấy danh sách các log chưa đồng bộ của các thiết bị đang hoạt động"""
        session: Session = self.session_factory()
        try:
            return session.query(ZKTRawAttendance).filter(
                ZKTRawAttendance.is_synced == 0,
                ZKTRawAttendance.device_id.in_(device_ids)
            ).order_by(ZKTRawAttendance.timestamp.asc()).limit(limit).all()
        finally:
            session.close()

    def mark_as_synced(self, log_ids: List[int]):
        """Đánh dấu danh sách các log đã đồng bộ thành công"""
        session: Session = self.session_factory()
        try:
            session.query(ZKTRawAttendance).filter(
                ZKTRawAttendance.id.in_(log_ids)
            ).update({"is_synced": 1}, synchronize_session=False)
            session.commit()
        except Exception as e:
            logger.error(f"Failed to mark logs as synced: {e}")
            session.rollback()
        finally:
            session.close()

    def query_logs(self, device_id=None, user_id=None, start_date=None, end_date=None, page=1, page_size=100):
        """Truy vấn nâng cao cho API"""
        session: Session = self.session_factory()
        try:
            query = session.query(ZKTRawAttendance)
            if device_id:
                query = query.filter(ZKTRawAttendance.device_id == device_id)
            if user_id:
                query = query.filter(ZKTRawAttendance.user_id == user_id)
            if start_date:
                query = query.filter(ZKTRawAttendance.timestamp >= start_date)
            if end_date:
                query = query.filter(ZKTRawAttendance.timestamp <= end_date)
            
            return query.order_by(ZKTRawAttendance.timestamp.desc())\
                        .offset((page - 1) * page_size)\
                        .limit(page_size).all()
        finally:
            session.close()

import asyncio
import logging
import os
from sqlalchemy.orm import Session
from database import SessionLocal
from models import ZKTRawAttendance
from central_client import central_client

logger = logging.getLogger("zkteco-sync")

class SyncWorker:
    def __init__(self):
        # Đọc cấu hình từ environment
        self.batch_size = int(os.getenv("SYNC_BATCH_SIZE", 100))
        self.interval = float(os.getenv("SYNC_INTERVAL", 0.1))
        self._stop_event = asyncio.Event()

    async def start(self):
        logger.info(f"Sync worker started (Batch: {self.batch_size}, Interval: {self.interval}s).")
        while not self._stop_event.is_set():
            if central_client.is_connected:
                try:
                    # Nếu có dữ liệu đã đồng bộ, có thể chạy tiếp nhanh (0.1s)
                    # Nếu không có gì, nghỉ 5s để đỡ tốn CPU
                    has_data = await self.sync_batch()
                    wait_time = self.interval if has_data else 5.0
                except Exception as e:
                    logger.error(f"Error during sync batch: {e}")
                    wait_time = 5.0
            else:
                logger.debug("Central client not connected. Skipping sync.")
                wait_time = 5.0
            
            await asyncio.sleep(wait_time)

    async def sync_batch(self) -> bool:
        """Gửi nguyên một batch logs về server. Trả về True nếu có dữ liệu đã xử lý."""
        db: Session = SessionLocal()
        try:
            # Lấy các bản ghi chưa sync
            unsynced_logs = db.query(ZKTRawAttendance).filter(
                ZKTRawAttendance.is_synced == 0
            ).order_by(ZKTRawAttendance.timestamp.asc()).limit(self.batch_size).all()

            if not unsynced_logs:
                return False

            logger.info(f"Syncing batch of {len(unsynced_logs)} logs...")
            
            # Chuẩn bị dữ liệu cho cả batch
            batch_data = []
            for log_record in unsynced_logs:
                batch_data.append({
                    "id": log_record.id, # Để map lại khi nhận ACK
                    "device_id": log_record.device_id,
                    "user_id": log_record.user_id,
                    "timestamp": log_record.timestamp.strftime("%Y-%m-%d %H:%M:%S"),
                    "status": log_record.status,
                    "punch": log_record.punch
                })
            
            try:
                # Gửi cả batch (Server sẽ nhận v["data"] là một List)
                # Lưu ý: Chúng ta giả định device_id của log đầu tiên đại diện cho cả batch hoặc gửi rỗng
                future = central_client.push_attendance_sync("BATCH", batch_data)
                
                # Đợi ACK cho cả batch (Tối đa 10s vì batch lớn)
                await asyncio.wait_for(future, timeout=10.0)
                
                # Nếu thành công, đánh dấu tất cả trong batch đã sync
                ids = [log.id for log in unsynced_logs]
                db.query(ZKTRawAttendance).filter(ZKTRawAttendance.id.in_(ids)).update({"is_synced": 1}, synchronize_session=False)
                db.commit()
                logger.info(f"Successfully synced batch of {len(unsynced_logs)} logs.")
                return True
            except asyncio.TimeoutError:
                logger.warning("Timeout waiting for batch ACK. Will retry later.")
                return False
            except Exception as e:
                logger.error(f"Error during batch sync: {e}")
                return False

        finally:
            db.close()

    def stop(self):
        self._stop_event.set()

sync_worker = SyncWorker()
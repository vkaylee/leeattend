import asyncio
import logging
import os
import random
from repository import AttendanceRepository

logger = logging.getLogger("zkteco-sync")

class SyncWorker:
    def __init__(self, repository: AttendanceRepository, central_client):
        self.repository = repository
        self.central_client = central_client
        self.batch_size = int(os.getenv("SYNC_BATCH_SIZE", 100))
        self.interval = float(os.getenv("SYNC_INTERVAL", 0.1))
        self._stop_event = asyncio.Event()
        self.get_active_devices_callback = None 
        self.max_backoff = 300
        self.base_backoff = 5

    def get_next_wait_time(self, consecutive_errors: int, has_data: bool) -> float:
        if consecutive_errors > 0:
            backoff = min(self.max_backoff, self.base_backoff * (2 ** (consecutive_errors - 1)))
            return random.uniform(self.base_backoff, backoff)
        return self.interval if has_data else 5.0

    async def start(self):
        logger.info(f"Sync worker started (Batch: {self.batch_size}, Interval: {self.interval}s).")
        consecutive_errors = 0
        
        while not self._stop_event.is_set():
            active_ids = self.get_active_devices_callback() if self.get_active_devices_callback else []
            
            if self.central_client.is_connected and active_ids:
                try:
                    has_data = await self.sync_batch(active_ids)
                    wait_time = self.get_next_wait_time(0, has_data)
                    consecutive_errors = 0 
                except Exception as e:
                    consecutive_errors += 1
                    wait_time = self.get_next_wait_time(consecutive_errors, False)
                    logger.error(f"Error during sync batch: {e}")
            else:
                wait_time = 5.0
                consecutive_errors = 0
            
            try:
                await asyncio.wait_for(self._stop_event.wait(), timeout=wait_time)
            except asyncio.TimeoutError:
                continue

    async def sync_batch(self, active_ids: list) -> bool:
        unsynced_logs = self.repository.get_unsynced_logs(active_ids, limit=self.batch_size)
        if not unsynced_logs:
            return False

        logger.info(f"Syncing batch of {len(unsynced_logs)} logs...")
        batch_data = []
        for log_record in unsynced_logs:
            batch_data.append({
                "id": log_record.id,
                "device_id": log_record.device_id,
                "user_id": log_record.user_id,
                "timestamp": log_record.timestamp.strftime("%Y-%m-%d %H:%M:%S"),
                "status": log_record.status,
                "punch": log_record.punch
            })
        
        try:
            future = self.central_client.push_attendance_sync("BATCH", batch_data)
            try:
                await asyncio.wait_for(future, timeout=10.0)
            finally:
                if hasattr(future, 'msg_id'):
                    self.central_client.discard_pending_ack(future.msg_id)
            
            ids = [log.id for log in unsynced_logs]
            self.repository.mark_as_synced(ids)
            logger.info(f"Successfully synced batch of {len(unsynced_logs)} logs.")
            return True
        except asyncio.TimeoutError:
            logger.warning("Timeout waiting for batch ACK. Will retry later.")
            return False
        except Exception as e:
            logger.error(f"Error during batch sync: {e}")
            return False

    def stop(self):
        self._stop_event.set()

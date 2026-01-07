import asyncio
import logging
import random
import time
from datetime import datetime
from typing import Dict, Set, List
from pyzk2 import ZK, const

logger = logging.getLogger("zkteco-device-manager")

class ZKService:
    """Wrapper cho PyZK để quản lý kết nối an toàn"""
    def __init__(self, device_id: str, ip: str, port: int = 4370):
        self.device_id = device_id
        self.ip = ip
        self.port = port
        self.zk = ZK(ip, port=port, timeout=5, force_udp=True, ommit_ping=True)
        self.conn = None

    def connect(self):
        if not self.conn:
            self.conn = self.zk.connect()
        return self.conn

    def disconnect(self):
        if self.conn:
            try:
                self.zk.disconnect()
            except:
                pass
            self.conn = None

    def get_attendance_logs(self):
        self.connect()
        # Chuyển đổi object thành dict thuần để dễ serialize/lưu trữ
        raw_logs = self.conn.get_attendance()
        logs = []
        for log in raw_logs:
            logs.append({
                "user_id": log.user_id,
                "timestamp": log.timestamp.strftime("%Y-%m-%d %H:%M:%S"),
                "status": log.status,
                "punch": log.punch
            })
        return logs

    def sync_time(self):
        self.connect()
        now = datetime.now()
        self.conn.set_time(now)
        return now.strftime("%Y-%m-%d %H:%M:%S")

    def get_users(self):
        self.connect()
        users = self.conn.get_users()
        return [{"uid": u.uid, "user_id": u.user_id, "name": u.name} for u in users]

class DeviceManager:
    def __init__(self, log_queue: asyncio.Queue):
        self.log_queue = log_queue
        self.services: Dict[str, ZKService] = {}
        self.device_states: Dict[str, dict] = {}
        self.processed_logs: Set[str] = set()
        self.monitoring_tasks: Dict[str, asyncio.Task] = {}
        self._stop_event = asyncio.Event()

    def add_device(self, device_id: str, ip: str, port: int = 4370):
        if device_id in self.services:
            self.remove_device(device_id)
        
        service = ZKService(device_id, ip, port)
        self.services[device_id] = service
        self.device_states[device_id] = {"last_sync": None, "error": None}
        
        # Start background monitor for this device
        task = asyncio.create_task(self._monitor_device(service))
        self.monitoring_tasks[device_id] = task
        logger.info(f"Started monitoring device {device_id} at {ip}")

    def remove_device(self, device_id: str):
        if device_id in self.monitoring_tasks:
            self.monitoring_tasks[device_id].cancel()
            del self.monitoring_tasks[device_id]
        
        if device_id in self.services:
            self.services[device_id].disconnect()
            del self.services[device_id]
            
        if device_id in self.device_states:
            del self.device_states[device_id]
        logger.info(f"Stopped monitoring device {device_id}")

    async def stop_all(self):
        self._stop_event.set()
        for device_id in list(self.services.keys()):
            self.remove_device(device_id)

    async def _monitor_device(self, service: ZKService):
        """Vòng lặp giám sát cho một thiết bị cụ thể"""
        self.log_queue.put_nowait({
            "type": "system",
            "device_id": service.device_id,
            "message": f"Bắt đầu giám sát tại {service.ip}"
        })
        
        base_delay = 10
        max_delay = 300
        consecutive_errors = 0

        while not self._stop_event.is_set():
            try:
                logs = service.get_attendance_logs()
                new_count = 0
                
                if logs:
                    for log in logs:
                        log_sig = f"{service.device_id}_{log['user_id']}_{log['timestamp']}"
                        if log_sig not in self.processed_logs:
                            self.processed_logs.add(log_sig)
                            new_count += 1
                            self.log_queue.put_nowait({
                                "type": "attendance",
                                "device_id": service.device_id,
                                "log": log
                            })
                
                # Success cleanup
                if consecutive_errors > 0:
                    self.log_queue.put_nowait({
                        "type": "system",
                        "device_id": service.device_id,
                        "message": "Kết nối máy chấm công đã được phục hồi."
                    })
                    consecutive_errors = 0

                self.device_states[service.device_id] = {
                    "last_sync": datetime.now(),
                    "error": None
                }
                
                # Cleanup internal cache if too big
                if len(self.processed_logs) > 20000:
                    list_logs = list(self.processed_logs)
                    self.processed_logs = set(list_logs[-10000:])

                await asyncio.sleep(base_delay)

            except asyncio.CancelledError:
                break
            except Exception as e:
                consecutive_errors += 1
                backoff = min(max_delay, base_delay * (2 ** (consecutive_errors - 1)))
                wait_time = random.uniform(base_delay, backoff)
                
                # Đảm bảo có entry trong device_states
                if service.device_id not in self.device_states:
                    self.device_states[service.device_id] = {"last_sync": None, "error": None}
                
                self.device_states[service.device_id]["error"] = str(e)
                self.log_queue.put_nowait({
                    "type": "system",
                    "level": "WARN",
                    "device_id": service.device_id,
                    "message": f"Lỗi kết nối máy chấm công ({type(e).__name__}). Thử lại sau {wait_time:.1f}s."
                })
                await asyncio.sleep(wait_time)

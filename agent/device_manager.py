import asyncio
import logging
import random
import time
from datetime import datetime
from typing import Dict, Set, List
from collections import deque
from pyzk2 import ZK, const

logger = logging.getLogger("zkteco-device-manager")

class ZKService:
    """Wrapper cho PyZK để quản lý kết nối an toàn (Async Threaded)"""
    def __init__(self, device_id: str, ip: str, port: int = 4370):
        self.device_id = device_id
        self.ip = ip
        self.port = port
        self.zk = ZK(ip, port=port, timeout=5, force_udp=True, ommit_ping=True)
        self.conn = None

    async def connect(self):
        if not self.conn:
            self.conn = await asyncio.to_thread(self.zk.connect)
        return self.conn

    async def disconnect(self):
        if self.conn:
            try:
                await asyncio.to_thread(self.zk.disconnect)
            except:
                pass
            self.conn = None

    async def get_attendance_logs(self):
        conn = await self.connect()
        # Chuyển đổi object thành dict thuần để dễ serialize/lưu trữ
        raw_logs = await asyncio.to_thread(conn.get_attendance)
        logs = []
        for log in raw_logs:
            logs.append({
                "user_id": log.user_id,
                "timestamp": log.timestamp.strftime("%Y-%m-%d %H:%M:%S"),
                "status": log.status,
                "punch": log.punch
            })
        return logs

    async def sync_time(self):
        conn = await self.connect()
        now = datetime.now()
        await asyncio.to_thread(conn.set_time, now)
        return now.strftime("%Y-%m-%d %H:%M:%S")

    async def get_users(self):
        conn = await self.connect()
        users = await asyncio.to_thread(conn.get_users)
        return [{"uid": u.uid, "user_id": u.user_id, "name": u.name} for u in users]

class DeviceManager:
    def __init__(self, log_queue: asyncio.Queue):
        self.log_queue = log_queue
        self.services: Dict[str, ZKService] = {}
        self.device_states: Dict[str, dict] = {}
        self.cache_set: Set[str] = set()
        self.cache_deque: deque = deque()
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
            service = self.services[device_id]
            # Fire and forget disconnect in a task since remove_device is synchronous
            asyncio.create_task(service.disconnect())
            del self.services[device_id]
            
        if device_id in self.device_states:
            del self.device_states[device_id]
        logger.info(f"Stopped monitoring device {device_id}")

    async def stop_all(self):
        self._stop_event.set()
        for device_id in list(self.services.keys()):
            self.remove_device(device_id)

    async def _monitor_device(self, service: ZKService):
        """Vòng lặp giám sát cho một thiết bị cụ thể (Memory Safe Version)"""
        await self.log_queue.put({
            "type": "system",
            "device_id": service.device_id,
            "message": f"Bắt đầu giám sát tại {service.ip}"
        })
        
        base_delay = 10
        max_delay = 300
        consecutive_errors = 0

        while not self._stop_event.is_set():
            try:
                logs = await service.get_attendance_logs()
                new_count = 0
                
                # Smart Polling: Adjust delay based on data volume
                # If logs > 5000, wait longer to reduce load
                current_delay = base_delay
                if len(logs) > 20000:
                    current_delay = 300  # 5 minutes
                    logger.warning(f"Device {service.device_id} has {len(logs)} logs. Slowing down to 300s.")
                elif len(logs) > 5000:
                    current_delay = 60   # 1 minute
                    logger.warning(f"Device {service.device_id} has {len(logs)} logs. Slowing down to 60s.")

                if logs:
                    for log in logs:
                        log_sig = f"{service.device_id}_{log['user_id']}_{log['timestamp']}"
                        if log_sig not in self.cache_set:
                            # Rolling Window Deduplication
                            self.cache_set.add(log_sig)
                            self.cache_deque.append(log_sig)
                            
                            # Maintain fixed size window (20k)
                            while len(self.cache_deque) > 20000:
                                old_sig = self.cache_deque.popleft()
                                self.cache_set.discard(old_sig)

                            new_count += 1
                            # Backpressure: await here blocks if queue is full (maxsize=1000)
                            await self.log_queue.put({
                                "type": "attendance",
                                "device_id": service.device_id,
                                "log": log
                            })
                
                # Success cleanup
                if consecutive_errors > 0:
                    await self.log_queue.put({
                        "type": "system",
                        "device_id": service.device_id,
                        "message": "Kết nối máy chấm công đã được phục hồi."
                    })
                    consecutive_errors = 0

                self.device_states[service.device_id] = {
                    "last_sync": datetime.now(),
                    "error": None
                }

                await asyncio.sleep(current_delay)

            except asyncio.CancelledError:
                break
            except Exception as e:
                consecutive_errors += 1
                backoff = min(max_delay, base_delay * (2 ** (consecutive_errors - 1)))
                wait_time = random.uniform(base_delay, backoff)
                
                if service.device_id not in self.device_states:
                    self.device_states[service.device_id] = {"last_sync": None, "error": None}
                
                self.device_states[service.device_id]["error"] = str(e)
                try:
                    await self.log_queue.put({
                        "type": "system",
                        "level": "WARN",
                        "device_id": service.device_id,
                        "message": f"Lỗi kết nối máy chấm công ({type(e).__name__}). Thử lại sau {wait_time:.1f}s."
                    })
                except:
                    pass # If queue full during error, just ignore error log
                
                await asyncio.sleep(wait_time)

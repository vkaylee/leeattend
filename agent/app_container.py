import asyncio
import logging
import time
import os
import json
from datetime import datetime
from typing import Dict, List, Optional

from repository import AttendanceRepository
from device_manager import DeviceManager
from central_client import CentralClient
from sync_worker import SyncWorker
from database import SessionLocal

logger = logging.getLogger("zkteco-agent")

class AttendanceAgent:
    def __init__(self, agent_id=None, token=None, server_url=None):
        self.start_time = time.time()
        self.log_queue = asyncio.Queue(maxsize=1000)
        
        # Infrastructure
        self.repository = AttendanceRepository(SessionLocal)
        self.central_client = CentralClient(agent_id, token, server_url)
        self.device_manager = DeviceManager(self.log_queue)
        self.sync_worker = SyncWorker(self.repository, self.central_client)
        
        # State
        self.total_processed = 0
        self._bg_tasks = []
        
        # Configure callbacks
        self.central_client.on_config_received = self.apply_central_config
        self.central_client.on_command_received = self.handle_remote_command
        self.central_client.on_error_received = self.handle_central_error
        self.sync_worker.get_active_devices_callback = lambda: list(self.device_manager.services.keys())

    async def start(self):
        """Khởi động toàn bộ Agent và các background services"""
        logger.info("Starting Attendance Agent container...")
        
        # 1. Load local config
        self.load_local_devices()
        
        # 2. Start background tasks
        self._bg_tasks.append(asyncio.create_task(self._main_runner(), name="main_runner"))
        self._bg_tasks.append(asyncio.create_task(self.central_client.connect(), name="central_conn"))
        self._bg_tasks.append(asyncio.create_task(self._push_status_periodically(), name="status_pusher"))
        self._bg_tasks.append(asyncio.create_task(self.sync_worker.start(), name="sync_worker"))
        
        logger.info("Agent started successfully.")

    async def stop(self):
        """Dừng Agent và dọn dẹp tài nguyên"""
        logger.info("Stopping Attendance Agent container...")
        
        # Stop components
        await self.device_manager.stop_all()
        self.central_client.stop()
        self.sync_worker.stop()
        
        # Cancel tasks
        for task in self._bg_tasks:
            task.cancel()
        
        if self._bg_tasks:
            await asyncio.gather(*self._bg_tasks, return_exceptions=True)
        
        logger.info("Agent stopped.")

    def load_local_devices(self):
        """Nạp cấu hình thiết bị từ file devices.json (Offline-first)"""
        if os.path.exists("devices.json"):
            try:
                with open("devices.json", "r", encoding="utf-8") as f:
                    devices = json.load(f)
                    for d in devices:
                        self.device_manager.add_device(d["id"], d["ip"], d.get("port", 4370))
                logger.info(f"Loaded {len(devices)} devices from local cache.")
            except Exception as e:
                logger.error(f"Failed to load local config: {e}")

    async def apply_central_config(self, devices: List[dict]):
        """Cập nhật cấu hình thiết bị từ Central Server"""
        logger.info(f"Applying central config for {len(devices)} devices...")
        current_ids = set(self.device_manager.services.keys())
        new_ids = {d.get("device_id") or d.get("id") for d in devices}

        # 1. Xóa các máy không còn trong cấu hình mới
        for dev_id in current_ids - new_ids:
            self.device_manager.remove_device(dev_id)

        # 2. Thêm hoặc cập nhật
        for d in devices:
            dev_id = d.get("device_id") or d.get("id")
            ip = d.get("ip")
            port = d.get("port", 4370)
            
            existing = self.device_manager.services.get(dev_id)
            if not existing or existing.ip != ip or existing.port != port:
                self.device_manager.add_device(dev_id, ip, port)

    async def handle_remote_command(self, command_data: dict):
        """Điều phối lệnh điều khiển từ xa"""
        method = command_data.get("method")
        device_id = command_data.get("device_id")
        
        try:
            # 1. Kiểm tra các method không cần device_id
            if method == "ping":
                return {"success": True, "data": "pong"}
            
            # 2. Các method còn lại BẮT BUỘC cần device_id
            if not device_id:
                return {"success": False, "error": "device_id is required"}

            # 3. Kiểm tra sự tồn tại của thiết bị
            service = self.device_manager.services.get(device_id)
            if not service:
                return {"success": False, "error": f"Device {device_id} not found"}

            # 4. Thực hiện lệnh
            if method == "sync_time":
                new_time = service.sync_time()
                return {"success": True, "data": {"new_time": new_time}}
            elif method == "get_users":
                users = service.get_users()
                return {"success": True, "data": {"users": users}}
            
            # 5. Nếu đi đến đây tức là method chưa được implement cho device
            return {"success": False, "error": f"Method {method} not implemented"}
        except Exception as e:
            return {"success": False, "error": str(e)}

    async def handle_central_error(self, code, message):
        """Xử lý lỗi từ Central Server"""
        await self.log_queue.put({
            "type": "system",
            "level": "ERROR",
            "message": f"CENTRAL SERVER ERROR [{code}]: {message}"
        })

    async def _main_runner(self):
        """Xử lý hàng đợi log và đẩy vào DB/Client"""
        while True:
            item = await self.log_queue.get()
            try:
                msg_type = item.get("type", "attendance")
                if msg_type == "attendance":
                    if self.repository.save_attendance(item["device_id"], item["log"]):
                        self.total_processed += 1
                elif msg_type == "system":
                    level = item.get("level", "INFO")
                    message = item.get("message", "")
                    device_id = item.get("device_id", "SYSTEM")
                    
                    if "SERVER ERROR" not in message:
                        self.central_client.push_system_event(level, message, device_id)
                    
                    # Console log
                    prefix = "🛑 ERROR" if level == "ERROR" else "⚠️ WARN " if level == "WARN" else "ℹ️ INFO "
                    print(f"[{datetime.now().strftime('%H:%M:%S')}] [{device_id}] {prefix}: {message}")
            finally:
                self.log_queue.task_done()

    async def _push_status_periodically(self):
        while True:
            device_list = []
            for device_id, state in self.device_manager.device_states.items():
                status = "online" if state.get("error") is None else "offline"
                device_list.append({
                    "device_id": device_id,
                    "status": status,
                    "last_sync": state.get("last_sync").isoformat() if state.get("last_sync") else None,
                    "error": state.get("error")
                })
            if device_list:
                self.central_client.push_status_update(device_list)
            await asyncio.sleep(30)

    def get_agent_status(self):
        uptime = time.time() - self.start_time
        devices = []
        for device_id, state in self.device_manager.device_states.items():
            devices.append({
                "device_id": device_id,
                "status": "online" if state.get("error") is None else "offline",
                "last_sync": state.get("last_sync"),
                "error": state.get("error")
            })
        return {
            "status": "running",
            "uptime_seconds": uptime,
            "total_processed": self.total_processed,
            "queue_size": self.log_queue.qsize(),
            "devices": devices
        }

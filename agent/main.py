import os
import logging
import asyncio
import json
from datetime import datetime
from typing import List, Optional, Set, Dict
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException, Depends
from sqlalchemy.orm import Session
from dotenv import load_dotenv
from pyzk2 import ZK, const

from database import SessionLocal, get_db
from models import ZKTRawAttendance
from schemas import ZKTRawAttendanceResponse, AgentStatus, DeviceStatus
from central_client import central_client
from sync_worker import sync_worker

import time

# Load environment variables
load_dotenv()

# Global Agent Stats
agent_stats = {
    "start_time": time.time(),
    "total_processed": 0,
    "device_states": {} # stores last_sync and current error per device
}

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("zkteco-api")

# Configuration
DEVICES_FILE = os.getenv("DEVICES_FILE", "devices.json")
# Default to 0 to use a random available port, avoiding local port conflicts
API_PORT = int(os.getenv("API_PORT", 0))

# Global storage for processed logs (Composite key: device_id:user_id:timestamp)
processed_logs: Set[str] = set()
LOG_CACHE_FILE = "processed_logs.txt"

# Centralized queue for logs
log_queue: asyncio.Queue = asyncio.Queue()

def save_log_cache():
    try:
        with open(LOG_CACHE_FILE, "w") as f:
            # Chỉ lưu 10,000 bản ghi gần nhất để file không quá nặng
            logs_to_save = list(processed_logs)[-10000:]
            f.write("\n".join(logs_to_save))
    except Exception as e:
        logger.error(f"Failed to save log cache: {e}")

def load_log_cache():
    if os.path.exists(LOG_CACHE_FILE):
        try:
            with open(LOG_CACHE_FILE, "r") as f:
                lines = f.read().splitlines()
                processed_logs.update(lines)
            logger.info(f"Loaded {len(processed_logs)} log signatures from cache")
        except Exception as e:
            logger.error(f"Failed to load log cache: {e}")

class ZKService:
    def __init__(self, device_id: str, ip: str, port: int):
        self.device_id = device_id
        self.ip = ip
        self.port = port
        self.zk = ZK(ip, port=port, timeout=5, password=0, force_udp=False, ommit_ping=False)

    def _parse_attendance_log(self, log):
        """Tách riêng logic parse một bản ghi log để dễ unit test"""
        try:
            if hasattr(log.timestamp, 'strftime'):
                ts = log.timestamp.strftime("%Y-%m-%d %H:%M:%S")
            else:
                ts = str(log.timestamp)
        except (ValueError, Exception):
            ts = "Invalid Date"
            
        return {
            "device_id": self.device_id,
            "user_id": str(log.user_id),
            "timestamp": ts,
            "status": log.status,
            "punch": log.punch
        }

    def get_attendance_logs(self):
        conn = None
        try:
            conn = self.zk.connect()
            conn.disable_device()
            
            attendance_list = []
            try:
                logs = conn.get_attendance()
                if logs:
                    for log in logs:
                        parsed_log = self._parse_attendance_log(log)
                        attendance_list.append(parsed_log)
            except ValueError as ve:
                if "day is out of range for month" in str(ve):
                    logger.error(f"[{self.device_id}] Lỗi ngày tháng trên máy. Bỏ qua log lỗi.")
                else:
                    logger.error(f"[{self.device_id}] ValueError: {ve}")
                return []
            except Exception as e:
                logger.error(f"[{self.device_id}] Error reading logs: {e}")
                return []
            
            conn.enable_device()
            return attendance_list
        except Exception as e:
            logger.error(f"[{self.device_id}] Connection error ({self.ip}): {e}")
            raise e
        finally:
            if conn:
                conn.disconnect()

    def get_device_info(self):
        conn = None
        try:
            conn = self.zk.connect()
            return {
                "id": self.device_id,
                "status": "online",
                "device_name": conn.get_device_name(),
                "firmware": conn.get_firmware_version(),
                "users_count": len(conn.get_users())
            }
        except Exception as e:
            return {"id": self.device_id, "status": "offline", "error": str(e)}
        finally:
            if conn:
                conn.disconnect()

    def get_users(self):
        conn = None
        try:
            conn = self.zk.connect()
            users = conn.get_users()
            return [{"uid": u.uid, "user_id": u.user_id, "name": u.name} for u in users]
        except Exception as e:
            logger.error(f"[{self.device_id}] Error getting users: {e}")
            raise e
        finally:
            if conn:
                conn.disconnect()

    def sync_time(self):
        conn = None
        try:
            conn = self.zk.connect()
            now = datetime.now()
            conn.set_time(now)
            return now.strftime("%Y-%m-%d %H:%M:%S")
        except Exception as e:
            logger.error(f"[{self.device_id}] Error syncing time: {e}")
            raise e
        finally:
            if conn:
                conn.disconnect()

# Manager to hold service instances and their tasks
services: Dict[str, ZKService] = {}
monitor_tasks: Dict[str, asyncio.Task] = {}

def load_devices():
    """Load devices from local file (Initial/Fallback)"""
    try:
        if os.path.exists(DEVICES_FILE):
            with open(DEVICES_FILE, "r", encoding="utf-8") as f:
                devices_data = json.load(f)
                rebuild_services(devices_data)
            logger.info(f"Loaded {len(services)} devices from {DEVICES_FILE}")
    except Exception as e:
        logger.error(f"Failed to load devices: {e}")

def rebuild_services(devices_data: List[Dict]):
    """Khởi tạo lại danh sách services từ dữ liệu json một cách an toàn"""
    global services
    
    # Phân biệt giữa dữ liệu lỗi (None) và dữ liệu rỗng hợp lệ ([])
    if devices_data is None:
        logger.warning("Nhận được cấu hình không hợp lệ (None). Giữ nguyên cấu hình cũ.")
        return False

    # So sánh cấu hình mới với cấu hình hiện tại
    def get_config_signature(d_list):
        if not d_list: return []
        sigs = []
        for d in d_list:
            d_id = d.get('id', d.get('device_id'))
            d_ip = d.get('ip')
            d_port = d.get('port', 4370)
            if d_id and d_ip:
                sigs.append(f"{d_id}|{d_ip}|{d_port}")
        return sorted(sigs)

    current_devices_data = [
        {"id": s.device_id, "ip": s.ip, "port": s.port} for s in services.values()
    ]
    
    new_sig = get_config_signature(devices_data)
    old_sig = get_config_signature(current_devices_data)

    if new_sig == old_sig:
        logger.info("Cấu hình thiết bị không đổi. Bỏ qua nạp lại.")
        return False

    # Xử lý trường hợp xóa sạch thiết bị
    if not devices_data:
        services = {}
        logger.info("Danh sách thiết bị trống. Đã dọn dẹp tất cả services.")
        return True

    new_services = {}
    for d in devices_data:
        try:
            d_id = d.get('id', d.get('device_id'))
            d_ip = d.get('ip')
            d_port = d.get('port', 4370)
            if d_id and d_ip:
                new_services[d_id] = ZKService(d_id, d_ip, d_port)
        except Exception as e:
            logger.error(f"Lỗi khi tạo ZKService cho {d.get('id', 'Unknown')}: {e}")
    
    if new_services or not devices_data:
        services = new_services
        logger.info(f"Đã nạp thành công {len(services)} thiết bị: {list(services.keys())}")
        return True
    else:
        logger.error("Không tìm thấy thiết bị hợp lệ nào. Giữ cấu hình cũ.")
        return False


async def start_device_monitors():
    """Bắt đầu giám sát cho tất cả services hiện có"""
    for service in services.values():
        if service.device_id not in monitor_tasks or monitor_tasks[service.device_id].done():
            task = asyncio.create_task(monitor_attendance(service))
            monitor_tasks[service.device_id] = task

async def stop_all_monitors():
    """Dừng tất cả các monitor đang chạy"""
    for task in monitor_tasks.values():
        task.cancel()
    if monitor_tasks:
        await asyncio.gather(*monitor_tasks.values(), return_exceptions=True)
    monitor_tasks.clear()

async def apply_central_config(devices_data: List[Dict]):
    """Callback nhận cấu hình từ Central Server"""
    if rebuild_services(devices_data):
        log_queue.put_nowait({
            "type": "system",
            "message": f"Áp dụng cấu hình mới từ Server ({len(devices_data)} máy)"
        })
        # 1. Dừng các monitor cũ
        await stop_all_monitors()
        # 2. Chạy các monitor mới
        await start_device_monitors()
    else:
        logger.info("Giữ nguyên trạng thái monitor hiện tại.")

async def handle_remote_command(command_data: Dict):
    """Xử lý lệnh điều khiển từ Central Server"""
    method = command_data.get("method")
    device_id = command_data.get("device_id")
    
    try:
        if method == "sync_time":
            if device_id in services:
                new_time = services[device_id].sync_time()
                return {"success": True, "data": {"new_time": new_time}}
            return {"success": False, "error": f"Device {device_id} not found"}
            
        elif method == "get_users":
            if device_id in services:
                users = services[device_id].get_users()
                return {"success": True, "data": {"users": users}}
            return {"success": False, "error": f"Device {device_id} not found"}
            
        elif method == "ping":
            return {"success": True, "data": "pong"}
            
        return {"success": False, "error": f"Method {method} not implemented"}
    except Exception as e:
        logger.error(f"Error executing remote command {method}: {e}")
        return {"success": False, "error": str(e)}

def format_attendance_display(device_id, log):
    """Định dạng chuỗi hiển thị log ra console"""
    return (
        f"\n[🔔 NEW LOG - {device_id}] - {datetime.now().strftime('%H:%M:%S')}\n"
        f" 👤 User ID  : {log['user_id']}\n"
        f" ⏰ Time     : {log['timestamp']}\n"
        f" 📍 Status   : {log['status']}\n"
        f"{'-' * 35}"
    )

def cleanup_processed_logs():
    """Dọn dẹp bộ nhớ đệm nếu vượt quá giới hạn"""
    global processed_logs
    if len(processed_logs) > 20000:
        logger.info("Cleaning up processed logs cache...")
        list_logs = list(processed_logs)
        processed_logs.clear()
        processed_logs.update(list_logs[-10000:])
        save_log_cache()

async def main_runner():
    """Runner chính duy nhất xử lý mọi đầu ra (Log chấm công & Thông báo hệ thống)"""
    print("\n" + "="*50)
    print("🚀 TỔNG TRẠM QUẢN LÝ MÁY CHẤM CÔNG ĐÃ SẴN SÀNG")
    print("="*50 + "\n")
    
    try:
        while True:
            item = await log_queue.get()
            try:
                msg_type = item.get("type", "attendance")
                
                if msg_type == "attendance":
                    device_id = item["device_id"]
                    log_data = item["log"]
                    print(format_attendance_display(device_id, log_data))
                    
                    # NOTE: We no longer push directly. 
                    # SyncWorker will pick up records from DB with is_synced=0
                    
                    # Save to Database
                    try:
                        db = SessionLocal()
                        # Chuyển đổi string timestamp thành datetime object
                        ts_obj = datetime.strptime(log_data['timestamp'], "%Y-%m-%d %H:%M:%S")
                        
                        db_log = ZKTRawAttendance(
                            device_id=device_id,
                            user_id=log_data['user_id'],
                            timestamp=ts_obj,
                            status=log_data['status'],
                            punch=log_data['punch']
                        )
                        db.add(db_log)
                        db.commit()
                        db.close()
                        agent_stats["total_processed"] += 1
                    except Exception as db_err:
                        logger.error(f"Failed to save log to DB: {db_err}")
                
                elif msg_type == "system":
                    level = item.get("level", "INFO")
                    message = item.get("message", "")
                    device_id = item.get("device_id", "SYSTEM")
                    timestamp = datetime.now().strftime('%H:%M:%S')
                    
                    # Push to Central Server
                    central_client.push_system_event(level, message, device_id)
                    
                    prefix = "ℹ️ INFO" if level == "INFO" else "⚠️ WARN"
                    print(f"[{timestamp}] [{device_id}] {prefix}: {message}")

            except Exception as e:
                logger.error(f"Error in Main Runner: {e}")
            finally:
                log_queue.task_done()
    except asyncio.CancelledError:
        print("\n👋 Đang dừng hệ thống...")

async def monitor_attendance(service: ZKService):
    """Background task for a specific device - Silent Producer"""
    # Gửi thông báo khởi động về Main Runner
    log_queue.put_nowait({
        "type": "system",
        "device_id": service.device_id,
        "message": f"Bắt đầu giám sát tại {service.ip}"
    })
    
    base_delay = 10
    max_delay = 300
    current_delay = base_delay

    try:
        while True:
            try:
                logs = service.get_attendance_logs()
                new_count = 0
                
                # Update last sync time
                agent_stats["device_states"][service.device_id] = {
                    "last_sync": datetime.now(),
                    "error": None
                }

                if logs:
                    for log in logs:
                        log_sig = f"{service.device_id}_{log['user_id']}_{log['timestamp']}"
                        
                        if log_sig not in processed_logs:
                            processed_logs.add(log_sig)
                            new_count += 1
                            log_queue.put_nowait({
                                "type": "attendance",
                                "device_id": service.device_id,
                                "log": log
                            })
                    
                    if new_count > 0:
                        save_log_cache()

                if current_delay != base_delay:
                    log_queue.put_nowait({
                        "type": "system",
                        "device_id": service.device_id,
                        "message": "Kết nối đã được phục hồi."
                    })
                    current_delay = base_delay

                cleanup_processed_logs()

            except Exception as e:
                agent_stats["device_states"][service.device_id] = {
                    "last_sync": agent_stats["device_states"].get(service.device_id, {}).get("last_sync"),
                    "error": str(e)
                }
                log_queue.put_nowait({
                    "type": "system",
                    "level": "WARN",
                    "device_id": service.device_id,
                    "message": f"Không thể kết nối. Thử lại sau {current_delay}s."
                })
                await asyncio.sleep(current_delay)
                current_delay = min(current_delay * 2, max_delay)
                continue
                
            await asyncio.sleep(current_delay)
            
    except asyncio.CancelledError:
        pass

async def push_status_periodically():
    """Gửi trạng thái tất cả các máy lên Central Server mỗi 30s"""
    while True:
        try:
            device_list = []
            for device_id, service in services.items():
                state = agent_stats["device_states"].get(device_id, {})
                status = "online" if state.get("error") is None else "offline"
                device_list.append({
                    "device_id": device_id,
                    "status": status,
                    "last_sync": state.get("last_sync").isoformat() if state.get("last_sync") else None,
                    "error": state.get("error")
                })
            
            if device_list:
                central_client.push_status_update(device_list)
        except Exception as e:
            logger.error(f"Error in push_status_periodically: {e}")
        
        await asyncio.sleep(30)

@asynccontextmanager
async def lifespan(app: FastAPI):
    # 1. Khởi tạo môi trường
    load_log_cache()
    load_devices() # Nạp cấu hình local để chạy ngay
    
    # 2. Đăng ký callbacks
    central_client.on_config_received = apply_central_config
    central_client.on_command_received = handle_remote_command
    
    tasks = []
    # 3. Chạy các service background
    tasks.append(asyncio.create_task(main_runner()))
    tasks.append(asyncio.create_task(central_client.connect()))
    tasks.append(asyncio.create_task(push_status_periodically()))
    tasks.append(asyncio.create_task(sync_worker.start()))
    
    # 4. Bắt đầu giám sát thiết bị (Offline-first)
    await start_device_monitors()
    
    # Gửi thông báo hệ thống sau khi khởi động xong
    log_queue.put_nowait({
        "type": "system",
        "message": "Agent đã khởi động và đang giám sát thiết bị local."
    })
        
    yield
    
    # 5. Dọn dẹp khi tắt máy
    await stop_all_monitors()
    for task in tasks:
        task.cancel()
    await asyncio.gather(*tasks, return_exceptions=True)
    save_log_cache()

app = FastAPI(title="ZKTeco Multi-Device Attendance API", lifespan=lifespan)

@app.get("/ping")
def ping():
    return {"message": "pong"}

@app.get("/health")
def health_check():
    """Đơn giản để check service có đang chạy hay không"""
    return {"status": "healthy", "timestamp": datetime.now()}

@app.get("/status", response_model=AgentStatus)
def get_agent_status():
    """Trả về chi tiết trạng thái vận hành của Agent và các máy chấm công"""
    uptime = time.time() - agent_stats["start_time"]
    
    device_list = []
    for device_id, service in services.items():
        state = agent_stats["device_states"].get(device_id, {})
        device_list.append(DeviceStatus(
            device_id=device_id,
            status="online" if state.get("error") is None else "offline",
            last_sync=state.get("last_sync"),
            error=state.get("error")
        ))

    return AgentStatus(
        status="running",
        uptime_seconds=uptime,
        total_processed=agent_stats["total_processed"],
        queue_size=log_queue.qsize(),
        devices=device_list
    )

@app.get("/attendance/raw", response_model=List[ZKTRawAttendanceResponse])
def get_raw_attendance(
    device_id: Optional[str] = None,
    user_id: Optional[str] = None,
    start_date: Optional[datetime] = None,
    end_date: Optional[datetime] = None,
    page: int = 1,
    page_size: int = 100,
    db: Session = Depends(get_db)
):
    query = db.query(ZKTRawAttendance)
    
    if device_id:
        query = query.filter(ZKTRawAttendance.device_id == device_id)
    if user_id:
        query = query.filter(ZKTRawAttendance.user_id == user_id)
    if start_date:
        query = query.filter(ZKTRawAttendance.timestamp >= start_date)
    if end_date:
        query = query.filter(ZKTRawAttendance.timestamp <= end_date)
        
    # Sort by timestamp descending (newest first)
    query = query.order_by(ZKTRawAttendance.timestamp.desc())
    
    # Pagination
    offset = (page - 1) * page_size
    results = query.offset(offset).limit(page_size).all()
    
    return results

@app.get("/devices")
def list_devices():
    return [s.get_device_info() for s in services.values()]

@app.get("/devices/{device_id}/logs")
def fetch_logs(device_id: str):
    if device_id not in services:
        raise HTTPException(status_code=404, detail="Device not found")
    try:
        logs = services[device_id].get_attendance_logs()
        return {"device_id": device_id, "count": len(logs), "logs": logs}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/devices/{device_id}/users")
def list_users(device_id: str):
    if device_id not in services:
        raise HTTPException(status_code=404, detail="Device not found")
    try:
        users = services[device_id].get_users()
        return {"device_id": device_id, "count": len(users), "users": users}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/devices/{device_id}/sync-time")
def sync_time(device_id: str):
    if device_id not in services:
        raise HTTPException(status_code=404, detail="Device not found")
    try:
        new_time = services[device_id].sync_time()
        return {"device_id": device_id, "message": "Time synced", "new_time": new_time}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=API_PORT)
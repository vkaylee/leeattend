import asyncio
import json
import logging
import os
import websockets
from datetime import datetime
from dotenv import load_dotenv

load_dotenv()

logger = logging.getLogger("zkteco-central")

class CentralClient:
    def __init__(self):
        self.url = os.getenv("CENTRAL_SERVER_URL")
        self.token = os.getenv("AGENT_TOKEN")
        self.agent_id = os.getenv("AGENT_ID")
        self.send_queue = asyncio.Queue()
        self.is_connected = False
        self.on_config_received = None # Callback function
        self.on_command_received = None # Callback function
        self.pending_acks = {} # Map msg_id -> asyncio.Future

    async def connect(self):
        if not self.url:
            logger.warning("CENTRAL_SERVER_URL not set. Skipping central connection.")
            return

        headers = {
            "Authorization": f"Bearer {self.token}",
            "X-Agent-ID": self.agent_id
        }

        retry_delay = 5
        max_retry_delay = 300

        while True:
            try:
                logger.info(f"Connecting to central server at {self.url}...")
                async with websockets.connect(self.url, additional_headers=headers) as websocket:
                    self.is_connected = True
                    retry_delay = 5 # Reset delay on success
                    logger.info("Successfully connected to central server.")
                    
                    # Send handshake
                    await websocket.send(json.dumps({
                        "type": "handshake",
                        "agent_id": self.agent_id,
                        "token": self.token, # Thêm token vào handshake
                        "timestamp": datetime.now().isoformat()
                    }))

                    # Run sender and receiver concurrently
                    await asyncio.gather(
                        self._sender(websocket),
                        self._receiver(websocket),
                        self._heartbeat(websocket)
                    )
            except Exception as e:
                self.is_connected = False
                logger.error(f"Connection to central server failed: {e}. Retrying in {retry_delay}s...")
                await asyncio.sleep(retry_delay)
                retry_delay = min(retry_delay * 2, max_retry_delay)

    async def _sender(self, websocket):
        while True:
            msg = await self.send_queue.get()
            try:
                await websocket.send(json.dumps(msg))
                self.send_queue.task_done()
            except Exception as e:
                logger.error(f"Failed to send message: {e}")
                # Re-queue the message to try again later
                await self.send_queue.put(msg)
                break

    async def _receiver(self, websocket):
        async for message in websocket:
            try:
                data = json.loads(message)
                msg_type = data.get("type")
                
                # Xử lý Cấu hình (Config Push từ Server)
                if msg_type == "config_push" or msg_type == "config":
                    config_data = data.get("data", data) # Hỗ trợ cả 2 định dạng
                    devices = config_data.get("devices", [])
                    logger.info(f"Received {len(devices)} devices from central server.")
                    
                    # Lưu vào local cache (devices.json)
                    self._save_local_config(devices)
                    
                    if self.on_config_received:
                        await self.on_config_received(devices)
                
                elif msg_type == "command":
                    logger.info(f"Received command from central: {data}")
                    if self.on_command_received:
                        result = await self.on_command_received(data)
                        # Send response back
                        response = {
                            "type": "response",
                            "id": data.get("id"),
                            "agent_id": self.agent_id,
                            "status": "success" if result.get("success", True) else "error",
                            "data": result.get("data"),
                            "error": result.get("error"),
                            "timestamp": datetime.now().isoformat()
                        }
                        await self.send_queue.put(response)
                
                elif msg_type == "ack":
                    msg_id = data.get("msg_id")
                    if msg_id in self.pending_acks:
                        future = self.pending_acks.pop(msg_id)
                        if not future.done():
                            future.set_result(data)

                elif msg_type == "system":
                    code = data.get("code")
                    if code == "SERVER_SHUTDOWN":
                        logger.warning("Central server is shutting down. Waiting for reconnect...")

                elif msg_type == "error":
                    code = data.get("code")
                    message = data.get("message")
                    if code == "DUPLICATE_ID":
                        logger.critical(f"CONNECTION REJECTED: {message}")
                        logger.critical("Another agent with the same ID is already running. Exiting...")
                        os._exit(1)
                    else:
                        logger.error(f"Received error from central: [{code}] {message}")

            except Exception as e:
                logger.error(f"Error processing message from central: {e}")

    def _save_local_config(self, devices):
        """Lưu danh sách thiết bị vào file devices.json để dùng khi offline"""
        try:
            # Map dữ liệu từ Server (device_id, ip, port) sang format cũ của Agent (id, name, ip, port)
            # Nếu Server dùng device_id, ta map nó thành id.
            local_devices = []
            for d in devices:
                local_devices.append({
                    "id": d.get("device_id", d.get("id")),
                    "name": d.get("name", d.get("device_id", "Unknown")),
                    "ip": d.get("ip"),
                    "port": d.get("port", 4370)
                })

            with open("devices.json", "w", encoding="utf-8") as f:
                json.dump(local_devices, f, indent=2, ensure_ascii=False)
            logger.info("Successfully updated local cache: devices.json")
        except Exception as e:
            logger.error(f"Failed to save local config: {e}")

    async def _heartbeat(self, websocket):
        while True:
            try:
                await websocket.ping()
                await asyncio.sleep(30)
            except:
                break

    def push_attendance_sync(self, device_id, log):
        """Đẩy log chấm công và trả về Future để chờ ACK"""
        import uuid
        msg_id = str(uuid.uuid4())
        msg = {
            "type": "attendance",
            "msg_id": msg_id,
            "agent_id": self.agent_id,
            "device_id": device_id,
            "data": log,
            "timestamp": datetime.now().isoformat()
        }
        
        loop = asyncio.get_event_loop()
        future = loop.create_future()
        self.pending_acks[msg_id] = future
        
        self.send_queue.put_nowait(msg)
        return future

    def push_log(self, device_id, log):
        """Đẩy log chấm công vào hàng đợi để gửi lên server trung tâm"""
        msg = {
            "type": "attendance",
            "agent_id": self.agent_id,
            "device_id": device_id,
            "data": log,
            "timestamp": datetime.now().isoformat()
        }
        self.send_queue.put_nowait(msg)

    def push_system_event(self, level, message, device_id="SYSTEM"):
        """Đẩy thông báo hệ thống lên server trung tâm"""
        msg = {
            "type": "system",
            "agent_id": self.agent_id,
            "level": level,
            "device_id": device_id,
            "message": message,
            "timestamp": datetime.now().isoformat()
        }
        self.send_queue.put_nowait(msg)

    def push_status_update(self, devices: list):
        """Đẩy trạng thái các thiết bị lên server trung tâm"""
        msg = {
            "type": "status_update",
            "agent_id": self.agent_id,
            "devices": devices,
            "timestamp": datetime.now().isoformat()
        }
        self.send_queue.put_nowait(msg)

central_client = CentralClient()
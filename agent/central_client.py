import asyncio
import json
import logging
import os
import random
import websockets
from datetime import datetime
from dotenv import load_dotenv

load_dotenv()

logger = logging.getLogger("zkteco-central")

class CentralClient:
    def __init__(self, agent_id=None, token=None, url=None):
        self.url = url or os.getenv("CENTRAL_SERVER_URL")
        self.token = token or os.getenv("AGENT_TOKEN")
        self.agent_id = agent_id or os.getenv("AGENT_ID")
        self.send_queue = asyncio.Queue(maxsize=1000)
        # ... rest of __init__
        self.last_error_code = None # Track the last error from server
        self.on_config_received = None # Callback function
        self.on_command_received = None # Callback function
        self.on_error_received = None # Callback function
        self.pending_acks = {} # Map msg_id -> asyncio.Future
        self.attempts = 0 # Track retry attempts across connection sessions
        self._stop_event = asyncio.Event()
        self._connection_task = None # Track the active connection task
        
        # Retry configuration
        self.base_retry_delay = 5
        self.max_retry_delay = 300
        self.auth_error_min_delay = 60

    def get_next_retry_delay(self, attempts: int, is_auth_error: bool) -> float:
        """Tính toán thời gian chờ tiếp theo dựa trên số lần thử và loại lỗi"""
        # Exponential Backoff
        full_delay = min(self.max_retry_delay, self.base_retry_delay * (2 ** attempts))
        jitter_delay = random.uniform(self.base_retry_delay, full_delay)
        
        if is_auth_error:
            return max(self.auth_error_min_delay, jitter_delay)
        return jitter_delay

    async def connect(self):
        """Entry point công khai để quản lý kết nối tới Central Server"""
        if not self.url:
            logger.warning("CENTRAL_SERVER_URL not set. Skipping central connection.")
            return

        # Ngăn chặn việc khởi tạo nhiều connection task cùng lúc
        if self._connection_task and not self._connection_task.done():
            logger.debug("Connection task already running.")
            return
        
        self._connection_task = asyncio.current_task()
        self._stop_event.clear()
        
        while not self._stop_event.is_set():
            await self._connection_cycle()

    async def _connection_cycle(self):
        """Xử lý một chu kỳ kết nối đơn lẻ bao gồm cả xử lý lỗi"""
        start_time = asyncio.get_event_loop().time()
        headers = {
            "Authorization": f"Bearer {self.token}",
            "X-Agent-ID": self.agent_id
        }

        try:
            self.last_error_code = None
            current_attempt = self.attempts + 1
            logger.info(f"Connecting to central server at {self.url} (Attempt {current_attempt})...")
            
            async with websockets.connect(self.url, additional_headers=headers) as websocket:
                self.is_connected = True
                logger.info("TCP Connection established. Performing handshake...")
                
                # 1. Thực hiện Handshake
                await websocket.send(json.dumps({
                    "type": "handshake",
                    "agent_id": self.agent_id,
                    "token": self.token,
                    "timestamp": datetime.now().isoformat()
                }))
                
                # 2. Chạy các services ngầm
                await self._run_services(websocket)
                
        except (websockets.exceptions.ConnectionClosed, ConnectionResetError, BrokenPipeError) as e:
            await self._handle_connection_error(e)
        except Exception as e:
            await self._handle_unexpected_error(e)
        finally:
            self.is_connected = False
            # Bảo vệ: Đảm bảo không bao giờ quay lại loop nhanh hơn 1s
            elapsed = asyncio.get_event_loop().time() - start_time
            if elapsed < 1.0 and not self._stop_event.is_set():
                await asyncio.sleep(1.0 - elapsed)

    async def _run_services(self, websocket):
        """Quản lý vòng đời của các task xử lý tin nhắn và heartbeat"""
        tasks = [
            asyncio.create_task(self._receiver(websocket), name="cc_receiver"),
            asyncio.create_task(self._sender(websocket), name="cc_sender"),
            asyncio.create_task(self._heartbeat(websocket), name="cc_heartbeat"),
            asyncio.create_task(self._stop_event.wait(), name="cc_stop_waiter")
        ]
        
        done, pending = await asyncio.wait(tasks, return_when=asyncio.FIRST_COMPLETED)
        
        # Thu hồi kết quả/lỗi từ các task đã xong
        error_to_raise = None
        for task in done:
            try:
                if not task.cancelled():
                    task.result()
            except (websockets.exceptions.ConnectionClosed, ConnectionResetError, BrokenPipeError) as e:
                error_to_raise = e
            except Exception as e:
                logger.error(f"Task {task.get_name()} failed: {e}")
                error_to_raise = e

        # Cleanup: Hủy các task còn lại
        for task in pending:
            task.cancel()
            
        if error_to_raise:
            raise error_to_raise

    async def _handle_connection_error(self, e):
        """Xử lý các lỗi mất kết nối mạng thông thường"""
        if self._stop_event.is_set(): return
        
        self.attempts = min(self.attempts + 1, 10)
        is_auth_error = self.last_error_code in ["INVALID_TOKEN", "AGENT_NOT_TRUSTED", "UNAUTHORIZED_AGENT"]
        retry_delay = self.get_next_retry_delay(self.attempts, is_auth_error)
        
        # Phân loại lỗi để log
        is_reset = isinstance(e, (ConnectionResetError, BrokenPipeError))
        if not is_reset and hasattr(e, 'code'):
            is_reset = e.code in [1006, 1011]

        if is_reset:
            logger.warning(f"Central connection reset (Attempt {self.attempts}). Retrying in {retry_delay:.1f}s...")
        elif is_auth_error:
            logger.warning(f"Connection rejected ([{self.last_error_code}]). Retrying in {retry_delay:.1f}s...")
        else:
            logger.warning(f"WebSocket closed ({type(e).__name__}). Retrying in {retry_delay:.1f}s...")

        await asyncio.sleep(retry_delay)

    async def _handle_unexpected_error(self, e):
        """Xử lý các lỗi logic không dự kiến"""
        if self._stop_event.is_set(): return
        
        self.attempts = min(self.attempts + 1, 10)
        is_auth_error = self.last_error_code in ["INVALID_TOKEN", "AGENT_NOT_TRUSTED", "UNAUTHORIZED_AGENT"]
        
        if is_auth_error:
            retry_delay = self.get_next_retry_delay(self.attempts, True)
            logger.warning(f"Authorization error detected in unexpected path. Retrying in {retry_delay:.1f}s...")
            await asyncio.sleep(retry_delay)
        else:
            logger.error(f"Unexpected connection error: {e}. Retrying in 5s...")
            await asyncio.sleep(5.0)

    async def _sender(self, websocket):
        while not self._stop_event.is_set():
            try:
                # Đợi message từ queue với timeout để có thể check stop_event
                msg = await asyncio.wait_for(self.send_queue.get(), timeout=1.0)
                try:
                    await websocket.send(json.dumps(msg))
                    self.send_queue.task_done()
                except Exception as e:
                    logger.error(f"Failed to send message: {e}")
                    # Trả lại queue để gửi lại sau khi reconnect
                    await self.send_queue.put(msg)
                    raise # Bắn lỗi để thoát task và trigger reconnect
            except asyncio.TimeoutError:
                continue
            except Exception:
                raise

    async def _receiver(self, websocket):
        try:
            async for message in websocket:
                if self._stop_event.is_set():
                    break
                try:
                    data = json.loads(message)
                    msg_type = data.get("type")
                    
                    # Successful response or message from server means we are authorized and connected
                    if msg_type in ["config", "config_push", "ack", "response"]:
                        if self.attempts > 0:
                            logger.info(f"Handshake verified after {self.attempts} attempts. Resetting retry counter.")
                            self.attempts = 0 
                    
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
                        self.last_error_code = code # Store for connect loop
                        
                        if self.on_error_received:
                            asyncio.create_task(self.on_error_received(code, message))

                        if code == "DUPLICATE_ID":
                            logger.critical(f"CONNECTION REJECTED: {message}")
                            logger.critical("Another agent with the same ID is already running. Exiting...")
                            os._exit(1)
                        else:
                            logger.error(f"Received error from central: [{code}] {message}")

                except Exception as e:
                    logger.error(f"Error processing message from central: {e}")
        except (websockets.exceptions.ConnectionClosed, ConnectionResetError, BrokenPipeError):
            raise
        except Exception as e:
            if not self._stop_event.is_set():
                logger.error(f"Receiver loop crashed: {e}")
            raise

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
        """Duy trì kết nối WebSocket và cập nhật trạng thái Online qua WS Message"""
        while not self._stop_event.is_set():
            try:
                # 1. Gửi tin nhắn Heartbeat (Cập nhật DB tại Server)
                await websocket.send(json.dumps({
                    "type": "heartbeat",
                    "agent_id": self.agent_id,
                    "timestamp": datetime.now().isoformat()
                }))
                
                # 2. WebSocket Ping (Duy trì socket tầng thấp)
                await websocket.ping()
                
                await asyncio.sleep(30)
            except Exception as e:
                logger.warning(f"Heartbeat failed: {e}")
                raise

    def stop(self):
        """Dừng kết nối Central Server"""
        self._stop_event.set()
        self.is_connected = False

    def discard_pending_ack(self, msg_id):
        """Xóa một Future khỏi danh sách chờ mà không cần set result (dùng khi timeout)"""
        if msg_id in self.pending_acks:
            self.pending_acks.pop(msg_id)

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
        future.msg_id = msg_id # Attach msg_id for cleanup
        self.pending_acks[msg_id] = future
        
        try:
            self.send_queue.put_nowait(msg)
        except asyncio.QueueFull:
            logger.warning("Send queue is full. Dropping message.")
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
        try:
            self.send_queue.put_nowait(msg)
        except asyncio.QueueFull:
            logger.warning("Send queue is full. Dropping message.")

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
        try:
            self.send_queue.put_nowait(msg)
        except asyncio.QueueFull:
            logger.warning("Send queue is full. Dropping message.")

    def push_status_update(self, devices: list):
        """Đẩy trạng thái các thiết bị lên server trung tâm"""
        msg = {
            "type": "status_update",
            "agent_id": self.agent_id,
            "devices": devices,
            "timestamp": datetime.now().isoformat()
        }
        try:
            self.send_queue.put_nowait(msg)
        except asyncio.QueueFull:
            logger.warning("Send queue is full. Dropping message.")

central_client = CentralClient()
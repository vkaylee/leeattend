import asyncio
import json
import logging
import sys
from datetime import datetime
from pyzk2 import ZK, const

logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s")
logger = logging.getLogger("zk-worker")

class ZKService:
    def __init__(self, ip: str, port: int = 4370):
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

    async def get_time(self):
        conn = await self.connect()
        dt = await asyncio.to_thread(conn.get_time)
        return dt.strftime("%Y-%m-%d %H:%M:%S")

    async def sync_time(self):
        conn = await self.connect()
        now = datetime.now()
        await asyncio.to_thread(conn.set_time, now)
        return now.strftime("%Y-%m-%d %H:%M:%S")

class ZKWorkerClient:
    def __init__(self, master_addr="127.0.0.1", master_port=5000, max_parallel=1):
        self.master_addr = master_addr
        self.master_port = master_port
        self.service = None
        self.writer = None
        self.max_parallel = max_parallel
        self.semaphore = asyncio.Semaphore(max_parallel)
        self.write_lock = asyncio.Lock()

    async def run(self):
        while True:
            try:
                logger.info(f"🔗 Connecting to Master at {self.master_addr}:{self.master_port}...")
                reader, self.writer = await asyncio.open_connection(self.master_addr, self.master_port)
                logger.info("✅ Connected to Master")
                await self.handle_requests(reader)
            except Exception as e:
                logger.error(f"❌ Connection error: {e}. Retrying in 5s...")
                await asyncio.sleep(5)

    async def handle_requests(self, reader):
        tasks = set()
        try:
            while True:
                data = await reader.readline()
                if not data: break
                line = data.decode().strip()
                if not line: continue

                await self.semaphore.acquire()
                task = asyncio.create_task(self.process_request(line))
                tasks.add(task)
                task.add_done_callback(lambda t: (tasks.discard(t), self.semaphore.release()))
        except Exception as e:
            logger.error(f"❌ Error in handle_requests: {e}")
        finally:
            if tasks:
                logger.info("Waiting for pending tasks to complete...")
                await asyncio.gather(*tasks, return_exceptions=True)

    async def process_request(self, line):
        try:
            request = json.loads(line)
            method = request.get("method")
            params = request.get("params", {})
            request_id = request.get("id")
            logger.info(f"📩 RPC Request: {method}")
            result = None
            error = None
            try:
                if method == "common.ping":
                    result = "pong"
                elif method == "device.connect":
                    ip = params["ip"]
                    port = params.get("port", 4370)
                    self.service = ZKService(ip, port)
                    await self.service.connect()
                    result = True
                elif method == "device.get_logs":
                    if not self.service: raise Exception("Device not connected")
                    result = await self.service.get_attendance_logs()
                elif method == "device.get_time":
                    if not self.service: raise Exception("Device not connected")
                    result = await self.service.get_time()
                elif method == "device.sync_time":
                    if not self.service: raise Exception("Device not connected")
                    result = await self.service.sync_time()
                else:
                    error = {"code": -32601, "message": f"Method {method} not found"}
            except Exception as e:
                logger.error(f"❌ Error executing method {method}: {e}")
                error = {"code": -32603, "message": str(e)}
            
            response = {"jsonrpc": "2.0", "id": request_id}
            if error: response["error"] = error
            else: response["result"] = result
            
            async with self.write_lock:
                if self.writer:
                    self.writer.write((json.dumps(response) + "\n").encode())
                    await self.writer.drain()
        except json.JSONDecodeError as e:
            logger.error(f"❌ Error parsing request: {e}")
        except Exception as e:
            logger.error(f"❌ Unexpected error in process_request: {e}")

if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 5000
    worker = ZKWorkerClient(master_port=port)
    asyncio.run(worker.run())

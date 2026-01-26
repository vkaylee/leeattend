import asyncio
import json
import logging
import sys
from datetime import datetime
from pyzk2 import ZK, const

logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s")
logger = logging.getLogger("zk-worker")

class ZKService:
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

class ZKWorkerClient:
    def __init__(self, master_addr="127.0.0.1", master_port=5000):
        self.master_addr = master_addr
        self.master_port = master_port
        self.services = {}
        self.writer = None

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
        while True:
            data = await reader.readline()
            if not data:
                break
            
            try:
                request = json.loads(data.decode())
                method = request.get("method")
                params = request.get("params", {})
                request_id = request.get("id")

                logger.info(f"📩 RPC Request: {method}")
                
                result = None
                error = None

                try:
                    if method == "zk.ping":
                        result = "pong"
                    
                    elif method == "zk.connect":
                        device_id = params["device_id"]
                        ip = params["ip"]
                        port = params.get("port", 4370)
                        if device_id not in self.services:
                            self.services[device_id] = ZKService(device_id, ip, port)
                        await self.services[device_id].connect()
                        result = True
                    
                    elif method == "zk.get_attendance":
                        device_id = params["device_id"]
                        result = await self.services[device_id].get_attendance_logs()
                    
                    elif method == "zk.get_users":
                        device_id = params["device_id"]
                        result = await self.services[device_id].get_users()
                    
                    elif method == "zk.sync_time":
                        device_id = params["device_id"]
                        result = await self.services[device_id].sync_time()
                    
                    elif method == "zk.disconnect":
                        device_id = params["device_id"]
                        if device_id in self.services:
                            await self.services[device_id].disconnect()
                            del self.services[device_id]
                        result = True
                    
                    else:
                        error = {"code": -32601, "message": "Method not found"}
                
                except Exception as e:
                    logger.exception("🔥 Error handling RPC method")
                    error = {"code": -32000, "message": str(e)}

                response = {
                    "jsonrpc": "2.0",
                    "result": result,
                    "error": error,
                    "id": request_id
                }
                
                self.writer.write(json.dumps(response).encode() + b"\n")
                await self.writer.drain()

            except Exception as e:
                logger.error(f"Error parsing request: {e}")

if __name__ == "__main__":
    # Get port from command line arguments if provided
    port = 5000
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            logger.error(f"Invalid port provided: {sys.argv[1]}. Using default 5000.")

    worker = ZKWorkerClient(master_port=port)
    try:
        asyncio.run(worker.run())
    except KeyboardInterrupt:
        pass

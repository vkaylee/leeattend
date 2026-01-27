import asyncio
import json
import logging
import sys
from datetime import datetime

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger('mock-worker')

class MockWorkerClient:
    def __init__(self, master_addr='127.0.0.1', master_port=5000):
        self.master_addr = master_addr
        self.master_port = master_port
        self.writer = None

    async def run(self):
        while True:
            try:
                logger.info(f'🔗 Connecting to Master at {self.master_addr}:{self.master_port}...')
                reader, self.writer = await asyncio.open_connection(self.master_addr, self.master_port)
                logger.info('✅ Connected to Master')
                await self.handle_requests(reader)
            except Exception as e:
                logger.error(f'❌ Connection error: {e}. Retrying in 5s...')
                await asyncio.sleep(5)

    async def handle_requests(self, reader):
        while True:
            data = await reader.readline()
            if not data: break
            line = data.decode().strip()
            if not line: continue
            try:
                request = json.loads(line)
                method = request.get('method')
                params = request.get('params', {})
                request_id = request.get('id')
                logger.info(f'📩 RPC Request: {method}')
                result = None
                if method == 'common.ping': result = 'pong'
                elif method == 'device.connect': result = True
                elif method == 'device.get_logs':
                    result = [{
                        'user_id': '999',
                        'timestamp': datetime.now().strftime('%Y-%m-%d %H:%M:%S'),
                        'status': 1,
                        'punch': 1
                    }]
                elif method == 'device.get_time': result = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
                elif method == 'device.sync_time': result = True
                
                response = {'jsonrpc': '2.0', 'result': result, 'id': request_id}
                self.writer.write((json.dumps(response) + '\n').encode())
                await self.writer.drain()
            except Exception as e:
                logger.error(f'❌ Error handling request: {e}')

if __name__ == '__main__':
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 5000
    worker = MockWorkerClient(master_port=port)
    asyncio.run(worker.run())

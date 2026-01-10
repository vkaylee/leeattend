import asyncio
from zk_worker import ZKWorker

if __name__ == "__main__":
    worker = ZKWorker()
    asyncio.run(worker.run())
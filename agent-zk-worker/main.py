import asyncio
from zk_worker import ZKWorker

async def start_worker():
    worker = ZKWorker()
    await worker.run()

if __name__ == "__main__":
    asyncio.run(start_worker())
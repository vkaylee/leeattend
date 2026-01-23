import asyncio
import signal
import sys
from zk_worker import ZKWorker

async def start_worker():
    stop_event = asyncio.Event()
    worker = ZKWorker()

    loop = asyncio.get_running_loop()
    
    def handle_signal():
        print("\nShutdown signal received (Python Worker)...")
        stop_event.set()

    for sig in (signal.SIGINT, signal.SIGTERM):
        try:
            loop.add_signal_handler(sig, handle_signal)
        except NotImplementedError:
            # Signal handlers are not implemented on Windows for some signals
            pass

    # Start the worker and wait for it or the stop event
    worker_task = asyncio.create_task(worker.run())
    
    await stop_event.wait()
    print("Stopping Python Worker...")
    
    worker_task.cancel()
    try:
        await worker_task
    except asyncio.CancelledError:
        pass

if __name__ == "__main__":
    try:
        asyncio.run(start_worker())
    except KeyboardInterrupt:
        pass

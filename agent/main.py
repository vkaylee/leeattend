import asyncio
import logging
import signal
import os
from app_container import AttendanceAgent
from dotenv import load_dotenv

# Setup Logging
logging.basicConfig(
    level=logging.INFO, 
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("zkteco-agent-cli")

# Load environment variables
load_dotenv()

# Singleton instance for production
agent_instance = None

async def main():
    global agent_instance
    
    logger.info("🚀 Starting ZKTeco Attendance Agent (Pure CLI Mode)")
    
    # 1. Initialize Agent
    agent_instance = AttendanceAgent()
    
    # 2. Setup Signal Handlers for Graceful Shutdown
    stop_event = asyncio.Event()
    loop = asyncio.get_running_loop()
    
    def shutdown_handler():
        logger.info("\nStopping agent...")
        stop_event.set()

    # Windows support: signal.SIGINT is available, but add_signal_handler is not always
    try:
        for sig in (signal.SIGINT, signal.SIGTERM):
            loop.add_signal_handler(sig, shutdown_handler)
    except NotImplementedError:
        # Fallback for systems without signal handler support (like Windows)
        pass

    # 3. Start Agent
    await agent_instance.start()
    
    # 4. Wait for stop event
    try:
        await stop_event.wait()
    except asyncio.CancelledError:
        pass
    finally:
        # 5. Stop Agent
        await agent_instance.stop()
        logger.info("Agent stopped cleanly.")

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
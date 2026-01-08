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
    
    # 2. Register Error Handler for prominent console output
    async def on_central_error(code, message):
        auth_errors = ["INVALID_TOKEN", "AGENT_NOT_TRUSTED", "UNAUTHORIZED_AGENT"]
        if code in auth_errors:
            print("\n" + "!" * 60)
            print(f"🛑 AUTH ERROR: [{code}]")
            print(f"📢 {message}")
            print("!" * 60 + "\n")
            
    # Wrap existing handler to keep both functionalities
    original_handler = agent_instance.central_client.on_error_received
    async def combined_error_handler(code, msg):
        await on_central_error(code, msg)
        if original_handler:
            await original_handler(code, msg)
            
    agent_instance.central_client.on_error_received = combined_error_handler

    # 3. Setup Signal Handlers for Graceful Shutdown
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
        logger.debug("Signal handlers not supported, relying on KeyboardInterrupt")

    # 3. Start Agent
    agent_task = asyncio.create_task(agent_instance.start())
    
    # 4. Wait for stop event or task completion
    try:
        await stop_event.wait()
    except (asyncio.CancelledError, KeyboardInterrupt):
        logger.info("Interrupt received...")
    finally:
        # 5. Stop Agent
        logger.info("Shutting down agent...")
        await agent_instance.stop()
        agent_task.cancel()
        logger.info("Agent stopped cleanly.")

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
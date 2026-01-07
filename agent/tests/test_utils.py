import asyncio
import pytest
from unittest.mock import patch, MagicMock
import main
import central_client

class LoopBreak(Exception):
    """Exception raised to safely break out of an infinite loop in tests."""
    pass

class AsyncTestFactory:
    def __init__(self):
        self.backups = {}
        self.sleep_patcher = None
        self.max_loops = 1
        self.loop_count = 0

    async def cancel_all_tasks(self):
        """Cancel all running tasks except the current one"""
        current = asyncio.current_task()
        tasks = [t for t in asyncio.all_tasks() if t is not current]
        for t in tasks:
            t.cancel()
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)

    def clear_globals(self):
        """Xóa triệt để trạng thái của agent instance hiện tại"""
        if main.agent_instance:
            # Clear log_queue
            while not main.agent_instance.log_queue.empty():
                try:
                    main.agent_instance.log_queue.get_nowait()
                except asyncio.QueueEmpty:
                    break
            
            # Clear services
            main.agent_instance.device_manager.services.clear()
            main.agent_instance.device_manager.device_states.clear()
            main.agent_instance.device_manager.processed_logs.clear()
            main.agent_instance.total_processed = 0

    def mock_infinite_loop(self, max_loops=1):
        """
        Context manager để bẻ gãy vòng lặp while True.
        """
        self.max_loops = max_loops
        self.loop_count = 0
        
        async def safe_sleep(delay=0):
            self.loop_count += 1
            if self.loop_count > self.max_loops:
                raise LoopBreak("Bẻ gãy vòng lặp vô hạn (sleep)")
            return

        async def safe_get():
            if not main.agent_instance.log_queue.empty():
                return await main.agent_instance.log_queue.get()
            
            self.loop_count += 1
            if self.loop_count > self.max_loops:
                raise LoopBreak("Bẻ gãy vòng lặp vô hạn (queue.get - empty)")
            
            raise LoopBreak("Queue trống trong môi trường test")

        from contextlib import ExitStack
        stack = ExitStack()
        stack.enter_context(patch("asyncio.sleep", side_effect=safe_sleep))
        if main.agent_instance:
            stack.enter_context(patch.object(main.agent_instance.log_queue, "get", side_effect=safe_get))
        
        return stack

    def create_mock_service(self, device_id="TEST_DEV"):
        """Tạo và đăng ký một ZKService giả lập vào agent hiện tại"""
        from device_manager import ZKService
        from unittest.mock import AsyncMock
        mock = AsyncMock(spec=ZKService)
        mock.device_id = device_id
        mock.ip = "127.0.0.1"
        if main.agent_instance:
            main.agent_instance.device_manager.services[device_id] = mock
        return mock

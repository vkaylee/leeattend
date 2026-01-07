import pytest
import asyncio
from unittest.mock import MagicMock, patch
from main import monitor_attendance, main_runner, log_queue, ZKService, processed_logs

@pytest.mark.asyncio
async def test_system_event_on_startup():
    """Xác nhận monitor gửi thông báo hệ thống khi khởi động"""
    # Xóa hàng chờ
    while not log_queue.empty():
        log_queue.get_nowait()

    mock_service = MagicMock(spec=ZKService)
    mock_service.device_id = "SYS_TEST"
    mock_service.ip = "1.1.1.1"
    
    # Dừng ngay sau vòng lặp đầu
    async def mock_sleep_stop(delay):
        raise asyncio.CancelledError()

    with patch("asyncio.sleep", side_effect=mock_sleep_stop):
        try:
            await monitor_attendance(mock_service)
        except (asyncio.CancelledError, Exception):
            pass

    # Kiểm tra queue: Phải có ít nhất 1 tin nhắn 'system'
    found_sys_msg = False
    while not log_queue.empty():
        item = log_queue.get_nowait()
        if item.get("type") == "system" and "Bắt đầu giám sát" in item.get("message"):
            found_sys_msg = True
            break
    
    assert found_sys_msg, "Không tìm thấy thông báo khởi động hệ thống trong Queue"

@pytest.mark.asyncio
async def test_system_event_on_failure():
    """Xác nhận monitor gửi cảnh báo WARN khi mất kết nối"""
    # Xóa hàng chờ trước khi test
    while not log_queue.empty():
        log_queue.get_nowait()

    mock_service = MagicMock(spec=ZKService)
    mock_service.device_id = "FAIL_TEST"
    mock_service.ip = "1.1.1.1"
    mock_service.get_attendance_logs.side_effect = Exception("Hardware Error")
    
    # Chỉ cho chạy 1 chu kỳ sleep rồi dừng
    sleep_count = 0
    async def mock_sleep_count(delay):
        nonlocal sleep_count
        sleep_count += 1
        if sleep_count >= 1:
            raise asyncio.CancelledError()

    with patch("asyncio.sleep", side_effect=mock_sleep_count):
        try:
            await monitor_attendance(mock_service)
        except asyncio.CancelledError:
            pass

    # Kiểm tra queue
    items = []
    while not log_queue.empty():
        items.append(log_queue.get_nowait())
    
    found_warn = any(item.get("type") == "system" and item.get("level") == "WARN" for item in items)
    assert found_warn, f"Không tìm thấy cảnh báo lỗi kết nối trong Queue. Items found: {items}"

@pytest.mark.asyncio
async def test_main_runner_handles_system_msg():
    """Kiểm tra main_runner in đúng định dạng cho tin nhắn hệ thống"""
    test_msg = {
        "type": "system",
        "level": "INFO",
        "device_id": "DEV_01",
        "message": "Test System Message"
    }
    await log_queue.put(test_msg)

    with patch("builtins.print") as mock_print:
        # Chạy runner xử lý 1 item
        original_get = log_queue.get
        async def get_once():
            item = await original_get()
            asyncio.get_event_loop().call_soon(lambda: runner_task.cancel())
            return item
        
        with patch.object(log_queue, "get", side_effect=get_once):
            runner_task = asyncio.create_task(main_runner())
            try:
                await runner_task
            except asyncio.CancelledError:
                pass

        # Kiểm tra xem có in dòng chứa 'INFO' và 'Test System Message' không
        print_calls = [str(call) for call in mock_print.call_args_list]
        assert any("INFO" in call and "Test System Message" in call for call in print_calls)
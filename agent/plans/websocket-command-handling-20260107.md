### # Implementation Plan: WebSocket Command Handling (Agent & Server)

### ## Approach
- **Why this solution:** Tận dụng kết nối WebSocket (Outbound) hiện có của Agent để cho phép Server gửi lệnh điều khiển (Remote Procedure Call - RPC) mà không cần Agent phải mở port (vượt NAT).
- **Protocol:** Sử dụng JSON-based messaging với `command_id` để map kết quả trả về.

### ## Steps

1. **Define Protocol Specification** (5 min)
   - Request: `{"type": "command", "id": "uuid", "method": "sync_time", "device_id": "M1"}`
   - Response: `{"type": "response", "id": "uuid", "status": "success", "data": {...}}`

2. **Rust Server: Implement WebSocket Handler** (30 min)
   - Thêm `axum::extract::ws` và `tokio::sync::mpsc` để quản lý kết nối.
   - Files to modify: `apiserver/src/main.rs`, `apiserver/src/handlers.rs`.
   - Files to create: `apiserver/src/agent_manager.rs` (Quản lý các Agent đang online).

3. **Agent (Python): Command Dispatcher** (20 min)
   - Cập nhật `agent/central_client.py` để nhận type `command`.
   - Implement mapping từ `method` name sang function call trong `ZKService`.
   - Files to modify: `agent/central_client.py`, `agent/main.py`.

4. **Integration: Round-trip Testing** (15 min)
   - Server gửi lệnh `sync-time`.
   - Agent thực hiện và trả về kết quả.
   - Đảm bảo `command_id` được giữ nguyên để Server nhận diện đúng task.

### ## Implementation Details

#### Rust Server WebSocket (Axum)
```rust
async fn ws_handler(ws: WebSocketUpgrade, ...) {
    ws.on_upgrade(|socket| handle_socket(socket, ...))
}

// Agent Manager sẽ lưu trữ Sender channel cho mỗi agent_id
type AgentMap = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>;
```

#### Python Agent Dispatcher
```python
# In CentralClient._receiver
if msg_type == "command":
    result = await self.execute_command(data)
    await self.send_queue.put({
        "type": "response",
        "id": data["id"],
        "status": "success",
        "data": result
    })
```

### ## Timeline
| Phase | Duration |
|-------|----------|
| Protocol & Setup | 5 min |
| Rust Server WS | 30 min |
| Python Agent Dispatch | 20 min |
| Integration Test | 15 min |
| **Total** | **70 min** |

### ## Rollback Plan
- Revert commit và quay lại cơ chế polling cũ nếu WebSocket gateway gây quá tải CPU (hiếm gặp).
- Tắt tính năng command trong `.env` (`ENABLE_COMMANDS=false`).

### ## Security Checklist
- [ ] **Token Validation:** Kiểm tra Agent Token ngay khi handshake WebSocket.
- [ ] **Command Sanitization:** Chỉ cho phép các method có trong whitelist (`sync_time`, `get_users`).
- [ ] **Timeouts:** Server sẽ hủy request nếu Agent không phản hồi sau 30s.

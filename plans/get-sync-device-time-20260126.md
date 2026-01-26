# Implementation Plan: Get & Sync Device Time

### ## Approach
- **Pattern**: Manual On-Demand (Approach A). Quản trị viên chủ động yêu cầu lấy giờ hoặc đồng bộ giờ.
- **Protocol**: 
    1. UI -> REST API (Apiserver)
    2. Server -> WebSocket AgentCommand (Agent)
    3. Agent -> JSON-RPC (Python Sidecar)
    4. Sidecar -> Physical ZKTeco Device.
- **Why**: Tránh việc query phần cứng quá thường xuyên gây treo máy chấm công, đồng thời tận dụng hạ tầng command-response đã có của Agent.

### ## Steps

1. **Python Sidecar Implementation** (15 min)
   - Modify `agent-zk-worker/zk_worker.py`:
     - Add `get_time()` method using `zk.get_time()` from `pyzk`.
     - Update JSON-RPC dispatcher to handle `zk.get_time`.

2. **Rust Agent Protocol** (15 min)
   - Modify `agent-main/src/models.rs`:
     - Add `GetDeviceTime` and `SyncDeviceTime` to `AgentCommand` enum (nếu chưa có).
   - Modify `agent-main/src/main.rs`:
     - Handle incoming WebSocket commands from server and route to Sidecar.

3. **API Server Implementation** (20 min)
   - Modify `apiserver/src/models/agent.rs`:
     - Define `AgentCommand` variant for Time operations.
   - Modify `apiserver/src/handlers/agents.rs`:
     - Add `POST /api/agents/:id/devices/:device_id/get-time`.
     - Add `POST /api/agents/:id/devices/:device_id/sync-time`.
   - Update `AgentManager` service to dispatch these commands.

4. **Integration & Frontend (Brief)** (10 min)
   - Cập nhật trang Device Management để hiển thị cột "Device Time" kèm nút Refresh và Sync.

5. **Testing** (20 min)
   - Create integration test in `agent-main/tests/device_time_test.rs` (using mock sidecar).
   - Create API test in `apiserver/tests/device_api_tests.rs`.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Driver (Python) | 15 min |
| Agent (Rust) | 15 min |
| API Server | 20 min |
| Testing | 20 min |
| **Total** | **~1.5 hours** |

### ## Rollback Plan
- Revert commit and restart Agent process. Feature is stateless, no DB migration required.

### ## Security Checklist
- [x] **Auth**: Chỉ Admin/Owner mới được phép gọi API Get/Sync Time.
- [x] **Validation**: Validate `device_id` thuộc quyền quản lý của Agent.
- [x] **Timeout**: Giới hạn timeout 10s cho lệnh RPC tránh treo thread.

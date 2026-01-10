# Code Standards & Conventions

## 🦀 Rust (Backend)

### 1. Linter & Quality
- **Warnings as Errors**: Dự án được cấu hình để từ chối tất cả các warnings (`warnings = "deny"` trong `Cargo.toml`). Code không được phép merge nếu còn tồn tại warning (unused variables, dead code, etc.).
- **Method Size (Rule of 30)**: Một hàm không nên dài quá 30 dòng code. Nếu dài hơn, hãy cân nhắc tách nhỏ ra các private methods hoặc helper functions.
- **Single Responsibility (SRP)**: Một hàm chỉ làm một việc duy nhất. Tên hàm phải phản ánh đúng hành động (ví dụ: `validate_agent_token` thay vì `check_agent`).

### 2. Architecture Layers (Handler-Service-Repository)
Để đảm bảo tính linh hoạt và dễ kiểm thử (Mockable), dự án tuân thủ mô hình 3 lớp:
- **Handlers (API Layer)**: Chỉ trích xuất tham số từ Request, gọi Service và trả về Response.
- **Services (Domain Layer)**: Chứa logic nghiệp vụ. Services nhận các Repositories thông qua Dependency Injection (DI) để tương tác với dữ liệu.
- **Repositories (Data Layer)**: Định nghĩa các Trait (sử dụng `#[async_trait]`) để trừu tượng hóa việc truy vấn dữ liệu (SQLx, In-memory, etc.).

### 3. Dependency Injection
Sử dụng `Arc<dyn RepositoryTrait>` để inject vào Services. Khởi tạo toàn bộ dependencies tại `main.rs` và đưa vào `AppState`.

## 🐍 Python (Worker Agents)

### 1. Environment Management
- **Venv Required**: Luôn sử dụng môi trường ảo (`venv`) để phát triển và chạy test. Thư mục chuẩn là `venv/` bên trong module.
- **Dependencies**: Quản lý qua `requirements.txt`.
- **Formatting**: Sử dụng `black` hoặc `ruff` để format code.

### 2. Testing
- **Pytest**: Sử dụng `pytest` kết hợp với `pytest-asyncio` cho các task bất đồng bộ.
- **Execution**: Luôn chạy test thông qua môi trường ảo: `source venv/bin/activate && pytest`.

## 🐍 Python (Worker Agents)

### 1. Environment Management (Linux Priority)
- **Linux Venv**: Bắt buộc sử dụng `python3 -m venv venv`. Kích hoạt bằng `source venv/bin/activate`.
- **Windows Note**: Tránh dùng chung thư mục `venv` giữa Linux và Windows do sự khác biệt về cấu trúc script (`bin/` vs `Scripts/`) và đường dẫn thư viện.
- **Dependencies**: Luôn cập nhật `requirements.txt` sau khi cài package mới.

### 2. Testing & Execution
- **Pytest**: Chạy test trong môi trường đã kích hoạt: `pytest -v`.
- **Hardware Access**: Trên Linux, cần kiểm tra quyền truy cập cổng Serial/USB (thường thuộc group `dialout`).

## Testing Standards

### Integration Tests (Rust)

We use a **Dynamic Test Database** pattern to ensure test isolation and allow parallel execution.

1. **Isolation**: Every integration test file (or test case) MUST use `TestDatabase::new().await`. This creates a unique Postgres database for that specific session.
2. **Migrations**: The helper automatically runs `sqlx::migrate!()`. No manual setup required.
3. **Parallelism**: Do NOT use `#[serial]` unless absolutely necessary for non-database resources. Tests are safe to run in parallel.
4. **Cleanup**: Databases are automatically dropped on `Drop` via a detached background shell process. This allows the test to finish immediately while the database is cleaned up shortly after.
5. **Signal Safety**: A global **Signal Trap** (SIGINT/SIGTERM) is initialized on the first `TestDatabase` creation. If the test process is interrupted (e.g., Ctrl+C), it will attempt to clean up all currently active test databases.
6. **Robustness**: The cleanup logic terminates active database backends before dropping to prevent "database is being accessed by other users" errors.

**Example Pattern:**
```rust
#[tokio::test]
async fn my_feature_test() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    
    // ... test logic ...
}
```

### Mocking vs Fakes
- Prefer **Fakes** (like `TestFactory` for DB data) over Mocks.
- Use `ZKService_mock.py` for external hardware simulation.


---

## ⚛️ React (Frontend)

### 1. Development
- **Vite**: Sử dụng Vite làm build tool.
- **Server State (React Query)**: Sử dụng `@tanstack/react-query` để quản lý Server State. Toàn bộ logic fetching, caching và mutation phải nằm trong **Domain-specific Custom Hooks** (ví dụ: `useAgents`, `useDevices`).
- **Component Design**: Component chỉ nên chịu trách nhiệm hiển thị (UI) và nhận sự kiện từ người dùng. Tránh sử dụng `useEffect` và `useState` thủ công để fetch dữ liệu.
- **Tailwind CSS 4**: Sử dụng các tiện ích của Tailwind 4 để styling.
- **Lucide React**: Thư viện icon chuẩn.

### 📅 Logic Lịch trình & Ca làm việc

Khi tính toán ca làm việc hiệu lực của nhân viên tại một thời điểm, **PHẢI** tuân thủ thứ tự ưu tiên sau:

1.  **Mức 1: Lịch trực tuần (Weekly Roster)** - `shift_rosters`. Bản ghi gán cho một ngày cụ thể. Nếu gán là NULL, nghĩa là nhân viên ĐƯỢC NGHỈ (Day Off) bất chấp các mức dưới.
2.  **Mức 2: Gán cố định (Fixed Assignment)** - `employee_shifts`. Lấy bản ghi có ngày hiệu lực (`start_date`) thỏa mãn và có thời điểm tạo (`created_at`) mới nhất (Temporal Overriding).
3.  **Mức 3: Ca mặc định phòng ban (Department Default)** - `department_default_shifts`. Lấy ca mặc định của phòng ban nhân viên đang thuộc về tại thời điểm đó.

**Lưu ý**: Tuyệt đối không tính toán logic này ở Frontend. Luôn sử dụng API `/api/workspaces/:id/schedules` để lấy kết quả đã tính toán từ Backend.

---

### 🎨 Frontend: Thông báo (Toasts)
- **No Browser Dialogs**: Tuyệt đối **KHÔNG** sử dụng `window.alert()`, `window.confirm()` hoặc `window.prompt()` cho các thông báo hành động thông thường.
- **Global Toast**: Sử dụng custom hook `useToast()` để hiển thị thông báo.
  - `toast.success()`: Khi hành động thành công (Lưu, Cập nhật).
  - `toast.error()`: Khi có lỗi API hoặc lỗi logic nghiêm trọng.
  - `toast.warning()`: Khi người dùng nhập thiếu dữ liệu.
- **Transitions**: Sử dụng Tailwind `animate-in` cho tất cả các Modals và Toast items để tạo cảm giác mượt mà.

---

## 🛡️ Security

- **Multi-Tenancy**: Luôn kiểm tra `workspace_id` trong mọi query.
- **RBAC**: Sử dụng `MemberContext` để xác thực quyền hạn. Không hard-code check role ở tầng handler nếu có thể dùng chung logic kiểm tra quyền.

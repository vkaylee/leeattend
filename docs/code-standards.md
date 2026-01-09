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

### 4. Testing
- **Unit Testing**: Viết unit test cho Services bằng cách sử dụng Mock Repositories.
- **Test Factory Pattern**: Sử dụng `TestFactory` trong `tests/common/factory.rs` để tạo dữ liệu mẫu cho Integration tests. 
- **Timeout Wrapping (Anti-Hanging)**: Tất cả các bài test bất đồng bộ hoặc thao tác khởi tạo dữ liệu (Factory) **PHẢI** được bọc trong một khối `timeout`. Điều này ngăn chặn việc bộ test suite bị treo vô hạn do deadlock hoặc network issues.
  ```rust
  tokio::time::timeout(Duration::from_secs(5), async { ... }).await.expect("Timed out");
  ```
- **Fluent API**: Builder pattern được khuyến khích để tạo data test sạch sẽ.
- **Cleanup**: Sử dụng `factory.cleanup().await` ở đầu mỗi hàm `spawn_app` hoặc setup test.

---

## ⚛️ React (Frontend)

### 1. Development
- **Vite**: Sử dụng Vite làm build tool.
- **Server State (React Query)**: Sử dụng `@tanstack/react-query` để quản lý Server State. Toàn bộ logic fetching, caching và mutation phải nằm trong **Domain-specific Custom Hooks** (ví dụ: `useAgents`, `useDevices`).
- **Component Design**: Component chỉ nên chịu trách nhiệm hiển thị (UI) và nhận sự kiện từ người dùng. Tránh sử dụng `useEffect` và `useState` thủ công để fetch dữ liệu.
- **Tailwind CSS 4**: Sử dụng các tiện ích của Tailwind 4 để styling.
- **Lucide React**: Thư viện icon chuẩn.

### 3. UI/UX Conventions
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

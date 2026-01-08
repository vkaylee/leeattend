# Code Standards & Conventions

## 🦀 Rust (Backend)

### 1. Linter & Quality
- **Warnings as Errors**: Dự án được cấu hình để từ chối tất cả các warnings (`warnings = "deny"` trong `Cargo.toml`). Code không được phép merge nếu còn tồn tại warning (unused variables, dead code, etc.).
- **Method Size (Rule of 30)**: Một hàm không nên dài quá 30 dòng code. Nếu dài hơn, hãy cân nhắc tách nhỏ ra các private methods hoặc helper functions.
- **Single Responsibility (SRP)**: Một hàm chỉ làm một việc duy nhất. Tên hàm phải phản ánh đúng hành động (ví dụ: `validate_agent_token` thay vì `check_agent`).

### 2. Architecture Layers (Handler-Service-Model)
Để tránh phình to code, chúng ta tuân thủ mô hình phân lớp:
- **Handlers (API Layer)**: Chỉ làm nhiệm vụ trích xuất tham số, gọi Service và trả về Response. Tuyệt đối không viết logic nghiệp vụ phức tạp hoặc tính toán DB trực tiếp tại đây.
- **Services (Logic Layer)**: Chứa logic nghiệp vụ có thể tái sử dụng. Các hàm Service nhận input là dữ liệu thô và trả về kết quả logic.
- **Extractors (Auth/Context)**: Sử dụng Axum Extractors (như `MemberContext`) để tách biệt logic kiểm tra quyền và danh tính ra khỏi luồng chính.

### 3. Testing
- **Test Factory Pattern**: Sử dụng `TestFactory` trong `tests/common/factory.rs` để tạo dữ liệu mẫu. 
- **Fluent API**: Builder pattern được khuyến khích để tạo data test sạch sẽ:
  ```rust
  let user = factory.user().with_email("test@example.com").create().await;
  ```
- **Cleanup**: Sử dụng `factory.cleanup().await` ở đầu mỗi hàm `spawn_app` hoặc setup test.

---

## ⚛️ React (Frontend)

### 1. Development
- **Vite**: Sử dụng Vite làm build tool.
- **Logic Separation**: Tách logic xử lý data và gọi API ra khỏi Component bằng **Custom Hooks**. Component chỉ nên chịu trách nhiệm hiển thị (UI) và nhận sự kiện từ người dùng.
- **Tailwind CSS 4**: Sử dụng các tiện ích của Tailwind 4 để styling.
- **Lucide React**: Thư viện icon chuẩn.

### 2. Testing
- **Vitest**: Framework testing chính.
- **Functional Factories**: Sử dụng các factory function trong `src/test/factories.ts` để sinh dữ liệu mẫu cho component tests.
  ```typescript
  const agent = agentFactory({ is_trusted: false });
  ```

---

## 🛡️ Security

- **Multi-Tenancy**: Luôn kiểm tra `workspace_id` trong mọi query.
- **RBAC**: Sử dụng `MemberContext` để xác thực quyền hạn. Không hard-code check role ở tầng handler nếu có thể dùng chung logic kiểm tra quyền.

# Code Standards & Conventions

## 🦀 Rust (Backend)

### 1. Linter & Quality
- **Warnings as Errors**: Dự án được cấu hình để từ chối tất cả các warnings (`warnings = "deny"` trong `Cargo.toml`). Code không được phép merge nếu còn tồn tại warning (unused variables, dead code, etc.).
- **Formatting**: Sử dụng `cargo fmt` trước khi commit.
- **Async/Await**: Sử dụng `tokio` làm runtime. Ưu tiên sử dụng `axum` extractors cho security context (`MemberContext`).

### 2. Testing
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

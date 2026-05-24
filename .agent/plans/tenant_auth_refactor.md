# Khóa chặt Tầng Dữ liệu bằng Rust Type-System (Option B)

Hiện tại, bất kỳ ai cũng có thể lấy được database connection pool của một Tenant bằng cách gọi `diesel_pools.tenant_conn(workspace_id)`. Điều này cho phép các API Handler kết nối thẳng tới DB mà không cần qua bước kiểm tra quyền (Authorization), mở ra rủi ro cực lớn về IDOR/DoS.

Để khắc phục, chúng ta sẽ bắt buộc hàm `tenant_conn` phải nhận vào một Context đã được xác thực (chứa một Token bí mật), thay vì nhận thẳng chuỗi `Uuid`.

## User Review Required

> [!WARNING]
> Mặc dù phương án này tạo ra cấu trúc cực an toàn, nhưng vì Codebase hiện tại đang truyền `workspace_id` đi khắp nơi, việc refactor MỘT LÚC toàn bộ các hàm trong Repositories để nhận `TenantAuthToken` sẽ mất rất nhiều thời gian và dễ gây lỗi đứt gãy. 
> 
> **Cách tiếp cận đề xuất (Tiếp cận dần dần - Gradual Refactoring):**
> 1. Đổi tên hàm cũ thành `tenant_conn_bypass_auth_DANGER(workspace_id)`. Tôi sẽ dùng script đổi toàn bộ các file cũ sang hàm này để đảm bảo code compile thành công.
> 2. Cung cấp hàm mới `tenant_conn(token: &TenantAuthToken)` cực an toàn.
> 3. Cập nhật `MemberContext` và `EmployeeContext` để tạo ra `TenantAuthToken` khi verify thành công.
> 4. (Tương lai) Các Dev khi viết tính năng mới hoặc review code cũ sẽ phải tìm cách truyền Token này xuống Repository để xóa dần chữ `DANGER` đi.
> 
> Bạn có đồng ý với hướng tiếp cận **Bóp nghẹt dần dần (Strangler Pattern)** này không, hay bạn muốn tôi phải sửa TẤT CẢ chữ ký của Repositories ngay bây giờ?

## Proposed Changes

### `apiserver/src/diesel_pool.rs`
Định nghĩa một Struct đại diện cho bằng chứng xác thực thành công (Capability Token) và sửa hàm lấy pool:
#### [MODIFY] [diesel_pool.rs](file:///home/elt1541/lee/leeattend/apiserver/src/diesel_pool.rs)
```rust
#[derive(Clone, Copy)]
pub struct TenantAuthToken(Uuid);

impl TenantAuthToken {
    /// Chỉ được phép khởi tạo bởi hệ thống auth (Extractors) hoặc System Bypass
    pub(crate) fn new_authorized(workspace_id: Uuid) -> Self {
        Self(workspace_id)
    }

    pub fn workspace_id(&self) -> Uuid {
        self.0
    }
}

impl DieselPools {
    // Hàm an toàn (Khuyên dùng)
    pub async fn tenant_conn(&self, token: &TenantAuthToken) -> Result<DieselConnection, AppError> {
        // ...
    }

    // Hàm nguy hiểm, chỉ dùng cho System (Workers/Migrations) hoặc legacy code
    pub async fn tenant_conn_bypass_auth_DANGER(&self, workspace_id: Uuid) -> Result<DieselConnection, AppError> {
        // ...
    }
}
```

### `apiserver/src/handlers/common.rs`
#### [MODIFY] [common.rs](file:///home/elt1541/lee/leeattend/apiserver/src/handlers/common.rs)
Cập nhật các Extractor (`MemberContext`, `EmployeeContext`) để chứa thêm `TenantAuthToken`. 
```rust
pub struct MemberContext {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub auth_token: TenantAuthToken, // <-- NEW
    // ...
}
```

### Script Refactoring Toàn Hệ Thống
#### [NEW] [refactor.py](file:///home/elt1541/lee/leeattend/scratch/refactor.py)
Tôi sẽ viết một Python script lướt qua toàn bộ thư mục `apiserver/src` và thay thế mọi chuỗi `tenant_conn(workspace_id)` thành `tenant_conn_bypass_auth_DANGER(workspace_id)`.

## Verification Plan

### Automated Tests
- Chạy `cargo check` để đảm bảo hệ thống type-safe.
- Chạy `cargo nextest run` để đảm bảo việc rename không làm hỏng logic test.

### Manual Verification
- Review code để chắc chắn chữ `DANGER` nổi bật lên ở mọi Repositories.

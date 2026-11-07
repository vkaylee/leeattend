# Kế Hoạch Bổ Sung Navigation Gaps & Drift-Detection Test Chống Sót

Status: Implemented; verification blocked by isolated test environment startup

## Mục Tiêu
Đăng ký đầy đủ các module đã hoàn thiện gồm **Worker & Queue Monitor** (`/infra/monitor`), **Nhật ký kiểm toán** (`/audit-logs`) và **Tài liệu & Biểu mẫu** (`/documents`) vào Sidebar navigation, đồng thời bổ sung **Automated Drift-Detection Test** đảm bảo không bao giờ bỏ sót bất kỳ chức năng nào giữa App Registry và Navigation Menu trong tương lai.

---

## Ràng Buộc & Hợp Đồng Giao Diện
1. **Phạm vi bảo toàn**:
   - Chỉ thay đổi tầng Navigation, Route Metadata và Test suite của Frontend (`webdashboard`).
   - Không thay đổi schema DB hay endpoint backend Go.
   - Giữ nguyên cơ chế kiểm tra phân quyền RBAC: `infra:monitor`, `audit_logs:read`, `payroll:read`.
2. **Nguyên tắc Chống Sót (Drift-Detection Test Contract)**:
   - Tạo bộ test tự động đối soát `APP_REGISTRY` vs `useNavMenuItems()`.
   - Nếu có bất kỳ App nào được đăng ký trong hệ thống mà không có trong Sidebar (hoặc không nằm trong danh sách ngoại lệ có chủ đích như `/dashboard` catalog), test suite sẽ báo lỗi ngay lập tức.
3. **Tuân thủ Chuẩn Thiết Kế & Accessibility (WCAG 2.1 AA)**:
   - Các link menu mới phải có `testId` chuẩn (`sidebar-link-infra-monitor`, `sidebar-link-audit-logs`, `sidebar-link-documents`).
   - Tên icon và nhãn dịch nhất quán theo i18n (`common.json`, `settings.json`, `audit_logs.json`).

---

## Chi Tiết Nhiệm Vụ (Tasks)

### Task 1: Bổ sung Hằng số Route & Metadata
- **File**: `webdashboard/src/lib/routes.ts`
  - Thêm `DOCUMENTS: "/documents"` vào đối tượng `ROUTES`.
- **File**: `webdashboard/src/lib/app-registry.ts`
  - Cập nhật mục `documents` dùng `to: ROUTES.DOCUMENTS` thay vì chuỗi hardcode.
- **File**: `webdashboard/src/lib/route-metadata.ts`
  - Thêm metadata cho `[ROUTES.DOCUMENTS]` với `breadcrumbKey: "common.my_documents"`, `defaultBreadcrumb: "Tài liệu & Biểu mẫu"`.
- **Verify**: `npx tsc --noEmit` hoặc type check pass.

### Task 2: Đăng ký Worker Monitor, Audit Logs & Documents vào Sidebar
- **File**: `webdashboard/src/hooks/use-nav-menu-items.ts`
  - Import icon `FileText`, `ShieldCheck` từ `lucide-react` (đã có sẵn `Activity`).
  - Trong nhóm `payroll`: Thêm mục `ROUTES.DOCUMENTS` (icon `FileText`, label `common.my_documents`, permissions `["payroll:read"]`).
  - Trong nhóm `system`: Thêm mục `ROUTES.INFRA_MONITOR` (icon `Activity`, label `monitor.title`, permissions `["infra:monitor"]`).
  - Trong nhóm `system`: Thêm mục `ROUTES.AUDIT_LOGS` (icon `ShieldCheck`, label `audit_logs.title`, permissions `["audit_logs:read"]`).
  - Trong hàm `getActiveNavGroup`:
    - Thêm `ROUTES.DOCUMENTS` vào mảng `PAYROLL_PATHS`.
    - Thêm `ROUTES.AUDIT_LOGS` vào điều kiện nhóm `system`.
- **Verify**: Gọi hook `useNavMenuItems()` trả về đầy đủ 3 items mới với đúng `testId` và permissions.

### Task 3: Viết Automated Drift-Detection Test (Bảo Đảm Không Sót)
- **File**: `webdashboard/src/hooks/__tests__/use-nav-menu-items.test.tsx`
  - **Parity Test**: So khớp toàn bộ danh sách `APP_REGISTRY` với danh sách items trong `useNavMenuItems()`.
    - Lấy danh sách đường dẫn `to` từ tất cả các app trong `APP_REGISTRY`.
    - Danh sách loại trừ hợp lệ duy nhất:
      - `ROUTES.HOME` (`/`): Đã được ghim cố định ở vị trí top `dashboard`.
      - `ROUTES.SETTINGS.ROOT` (`/workspace/settings`): Đã được ghim cố định ở vị trí top `settings`.
      - `ROUTES.DASHBOARD` (`/dashboard`): Trang catalog biểu đồ phụ trợ mở từ màn hình chính.
    - Mọi route còn lại trong `APP_REGISTRY` **BẮT BUỘC** phải tồn tại trong `useNavMenuItems().groups`.
  - **Permission Tests**:
    - Khi user có quyền `infra:monitor` → Hiển thị `/infra/monitor`; khi không có → Ẩn.
    - Khi user có quyền `audit_logs:read` → Hiển thị `/audit-logs`; khi không có → Ẩn.
    - Khi user có quyền `payroll:read` → Hiển thị `/documents`; khi không có → Ẩn.
  - **Active Group Tests**:
    - `getActiveNavGroup("/infra/monitor")` → `"system"`.
    - `getActiveNavGroup("/audit-logs")` → `"system"`.
    - `getActiveNavGroup("/documents")` → `"payroll"`.
- **File**: `webdashboard/src/lib/__tests__/routes.test.ts`
  - Thêm `"/documents"` vào mảng `EXPECTED_APP_ROUTES`.
- **Verify**: Chạy test kiểm tra pass 100% các assertion mới.

---

## Ma Trận Kiểm Thử (Test Impact Matrix)
- **Changed behavior**:
  - Người dùng có vai trò `admin` hoặc quyền tương ứng sẽ thấy:
    - Mục "Giám Sát Worker & Hàng Đợi" (`/infra/monitor`) trong menu Hệ thống.
    - Mục "Nhật Ký Kiểm Toán" (`/audit-logs`) trong menu Hệ thống.
    - Mục "Quản Lý Tài Liệu & Biểu Mẫu" (`/documents`) trong menu Lương & Tài liệu.
  - Thanh chỉ báo nhóm menu tích cực (Active Nav Group) sáng đúng nhóm `system` khi vào `/infra/monitor` hoặc `/audit-logs`, và nhóm `payroll` khi vào `/documents`.
- **Safety & Boundary**:
  - Người dùng không có quyền `infra:monitor` (ví dụ nhân viên tiêu chuẩn) sẽ không thấy mục Worker Monitor.
  - Người dùng không có quyền `audit_logs:read` sẽ không thấy mục Audit Logs.
  - Tự động bắt lỗi nếu có lập trình viên thêm app mới vào `app-registry.ts` mà quên thêm vào `use-nav-menu-items.ts`.

---

## Tiêu Chí Hoàn Thành (Done When)
- [x] `ROUTES.DOCUMENTS` được định nghĩa chuẩn và sử dụng trong toàn bộ webdashboard.
- [x] 3 items (`/infra/monitor`, `/audit-logs`, `/documents`) xuất hiện đầy đủ trong `useNavMenuItems()`.
- [x] `getActiveNavGroup` nhận diện chính xác nhóm `system` và `payroll`.
- [x] Test tự động Drift-Detection giữa `APP_REGISTRY` và `useNavMenuItems()` được bổ sung và chạy kiểm tra.
- [x] Các test permission cụ thể cho 3 item mới pass.
- [ ] LeeDevKit isolated test environment verification: Bị chặn do môi trường test container khởi động không hoàn tất trong môi trường hiện tại (đã ghi nhận từ lần chạy trước).

# Kế Hoạch Chuyển Đổi Quản Lý Thành Viên Vào Cài Đặt (Members to Settings Migration)

Status: Approved

## 🎯 Mục Tiêu
Quy hoạch tập trung toàn bộ khối quản trị danh tính & truy cập (IAM: Thành viên, Vai trò, Phân quyền) vào **Settings Hub (`/workspace/settings/members`)**, đồng thời chuẩn hóa Menu chính (Sidebar) tinh gọn, chuyên biệt cho nghiệp vụ nhân sự hàng ngày.

---

## 📋 Rule Compliance Check (MANDATORY)
1. **Rule ID: `coding-standards.md#2-clean-code-standards`**: Áp dụng Single Responsibility (SRP); quy hoạch đúng ranh giới IAM vào Settings; loại bỏ các định nghĩa route trùng lặp và dead code.
2. **Rule ID: `project-structure.md#1-source-of-truth`**: Duy trì Single Source of Truth cho `ROUTES` constants (`ROUTES.SETTINGS.MEMBERS = "/workspace/settings/members"`) và `route-metadata.ts`.
3. **Rule ID: `design-rules.md#2-accessibility--wcag-21-aa-standards`**: Đảm bảo layout trong Settings hiển thị mượt mà, đầy đủ focus rings, WCAG 2.1 AA và test IDs chuẩn mực.

---

## 🏛️ Thiết Kế Chi Tiết

### 1. Route & Navigation Architecture
- **Canonical Route mới:** `/workspace/settings/members`
  - File route: `webdashboard/src/routes/_authenticated/_workspace/workspace/settings/members.tsx`
  - File lazy: `webdashboard/src/routes/_authenticated/_workspace/workspace/settings/members.lazy.tsx`
- **Chuyển hướng tương thích (Backwards Compatibility):**
  - `/workspace/people` → redirect `301` về `/workspace/settings/members`
  - `/members` → redirect `301` về `/workspace/settings/members`
- **Menu chính (Sidebar):**
  - Gỡ bỏ `ROUTES.PEOPLE` khỏi nhóm `org` trên Sidebar chính để tránh trùng lặp với `Nhân viên` (`/employees`).
  - Gỡ bỏ `ROUTES.SETTINGS.ROLES` khỏi nhóm `system` trên Sidebar chính vì toàn bộ nhóm Quản trị truy cập đã quy tụ tập trung trong Settings Hub.
- **Settings Hub (Cài đặt):**
  - Nhóm `Truy cập & Phân quyền` (`getAccessGroup` trong `use-settings-nav.tsx`) sở hữu trọn vẹn bộ ba:
    1. *Vai trò & Chức danh* (`/workspace/settings/roles`)
    2. *Phân quyền người dùng* (`/workspace/settings/permissions`)
    3. *Tài khoản người dùng* (`/workspace/settings/members`)
- **Metadata & Catalog:**
  - `routes.ts`: `ROUTES.SETTINGS.MEMBERS = "/workspace/settings/members"`.
  - `route-metadata.ts`: Đăng ký `[ROUTES.SETTINGS.MEMBERS]` đầy đủ breadcrumb và title.
  - `app-registry.ts`: Đặt `people` trỏ về `ROUTES.SETTINGS.MEMBERS`, `group: "settings"`.

---

## 📋 Nhiệm Vụ Triển Khai (Tasks)

- [ ] **Task 1: Tạo route `/workspace/settings/members` và cấu hình redirect**
  - Tạo `routes/.../workspace/settings/members.tsx` và `members.lazy.tsx`.
  - Cập nhật `workspace/people.tsx` & `.lazy.tsx`, `members.tsx` & `.lazy.tsx` thực hiện redirect về `ROUTES.SETTINGS.MEMBERS`.
- [ ] **Task 2: Tinh chỉnh layout `MembersPage` cho khung Settings**
  - Thay `<PageContainer>` bằng `<div className="space-y-8 animate-in fade-in duration-500">` để tương thích hoàn hảo với `SettingsPageLayout`.
- [ ] **Task 3: Cập nhật `ROUTES`, `route-metadata.ts` và `app-registry.ts`**
  - Đặt `ROUTES.SETTINGS.MEMBERS = "/workspace/settings/members"`.
  - Cập nhật `route-metadata.ts` và `app-registry.ts`.
- [ ] **Task 4: Tinh gọn Sidebar chính (`use-nav-menu-items.ts`) và Settings Nav (`use-settings-nav.tsx`)**
  - Gỡ bỏ `PEOPLE` khỏi `org` và `ROLES` khỏi `system` trên Sidebar chính.
  - Đảm bảo `use-settings-nav.tsx` trỏ đúng `ROUTES.SETTINGS.MEMBERS`.
- [ ] **Task 5: Đồng bộ Unit Tests**
  - `routes.test.ts`: Thêm `/workspace/settings/members` vào `EXPECTED_APP_ROUTES`.
  - `use-nav-menu-items.test.tsx`: Cập nhật active group và drift-detection tests.
  - `use-settings-nav.test.tsx`: Kiểm tra `accessPaths` chứa `/workspace/settings/members`.
- [ ] **Task 6: Rebuild Vite & Kiểm thử toàn diện**
  - Chạy `vite build:fast` tạo lại `route-tree.gen.ts`.
  - Chạy `leedevkit test web --unit-only` xác minh 100% test suites pass.

# 📋 Modern HR Admin UI Migration - Detailed Implementation Plan

- **Status:** Approved / Ready for Execution
- **Task Slug:** `modern-hr-admin-ui-plan`
- **Owner:** @frontend-specialist
- **Rule Compliance:**
  - `RULE-UI-001` (Design System: Tận dụng các component chuẩn `components/ui/{sheet,smart-table,table-density-toggle,page-header}`)
  - `RULE-UI-002` (WCAG 2.1 AA & Accessibility: Visible focus rings, keyboard accessibility `Cmd+B` toggle sidebar, `Cmd+K` command bar, focus trap trên Slide-over Sheet, HTML5 semantic `<aside>`, `<header>`, `<main>`, `<nav>`)
  - `RULE-CODE-002` (Clean Code & Dead Code Policy: Xóa bỏ 100% mã nguồn DSM thừa, không để lại code chết, Type-safe TypeScript strict)
  - `RULE-RESP-001` (Responsive & Mobile: Sidebar tự động chuyển thành drawer trượt trên màn hình di động `< 768px`)

---

## 🎯 Mục tiêu Kiến trúc (Target Architecture)

Chuyển đổi giao diện `LeeAttend` từ mô hình Desktop ảo hoá (Synology DSM 7) sang mô hình **Enterprise SaaS HR Admin Tiêu Chuẩn**:
1. **Collapsible Left Sidebar**: Cột điều hướng bên trái hỗ trợ 2 trạng thái: `expanded` (240px) và `collapsed` (64px icon-only), lưu `localStorage`, phím tắt `Cmd+B` / `Ctrl+B`.
2. **Global Topbar**: Breadcrumb động, Command Palette trigger (`Cmd+K`), Thông báo real-time, Workspace & Profile switcher, Theme/i18n switcher.
3. **Restored Router Viewport**: Đưa `<Outlet />` của TanStack Router ra làm trung tâm điều hướng, hiển thị trọn vẹn màn hình cho các bảng dữ liệu chuyên sâu (Employees, Attendance Logs, Shifts, Payroll).
4. **Slide-over Sheet Detail View**: Tích hợp `Sheet` từ mép phải cho tác vụ xem/sửa chi tiết bản ghi (Hồ sơ NV, Đơn nghỉ) mà không làm mất ngữ cảnh cuộn hay bộ lọc của bảng danh sách.
5. **Clean Cutover DSM Purge**: Gỡ bỏ hoàn toàn `useWindowManagerStore`, `DsmDesktopCanvas`, `DsmWindowFrame`, `DsmTopTaskbar` và `dsm-app-registry.tsx`.

---

## 🛠️ Chi tiết các Phase triển khai

### Phase 1: Xây dựng Layout Components mới (`Sidebar` & `Topbar`)
- **Mục tiêu:** Xây dựng 2 component điều hướng chuẩn mực SaaS enterprise.
- **Files:**
  - `webdashboard/src/components/layout/sidebar.tsx` (Mới)
  - `webdashboard/src/components/layout/topbar.tsx` (Mới)
  - `webdashboard/src/components/layout/__tests__/sidebar.test.tsx` (Mới)
  - `webdashboard/src/components/layout/__tests__/topbar.test.tsx` (Mới)
- **Chi tiết kỹ thuật:**
  - `sidebar.tsx`:
    - Phân nhóm module:
      - *Tổng quan*: Dashboard (`/workspace/dashboard`)
      - *Nhân sự*: Nhân viên (`/workspace/employees`), Phòng ban (`/workspace/departments`), Sơ đồ tổ chức (`/workspace/org-chart`)
      - *Chấm công & Ca*: Bảng chấm công (`/workspace/attendance-logs`), Xếp ca (`/workspace/shifts`), Đơn nghỉ (`/workspace/leave-requests`), Đổi ca (`/workspace/shift-swaps`)
      - *Tiền lương & Hợp đồng*: Bảng lương (`/workspace/payroll`), Hợp đồng (`/workspace/econtracts`)
      - *Vận hành*: Phê duyệt (`/workspace/approvals`), Báo cáo (`/workspace/reports`), Audit Logs (`/workspace/audit-logs`)
      - *Cài đặt*: Cài đặt hệ thống (`/workspace/settings`)
    - Hỗ trợ tooltip khi ở chế độ collapsed (`64px`).
    - Nút toggle và hook bắt sự kiện bàn phím `Cmd+B` / `Ctrl+B`.
    - Trạng thái mobile: bọc trong Sheet/Drawer khi viewport `< 768px`.
  - `topbar.tsx`:
    - Tích hợp Breadcrumbs lấy từ đường dẫn hiện tại của TanStack Router.
    - Button kích hoạt Command Search (`Cmd+K`).
    - Notifications Popover, Language Switcher, Theme Switcher, Avatar Profile.
- **Verification:** Unit tests Vitest kiểm tra collapse/expand, click chuyển route, và responsive toggle.

---

### Phase 2: Refactor `AppShell` & Khôi phục Router `<Outlet />`
- **Mục tiêu:** Ráp `Sidebar` và `Topbar` vào `AppShell`, khôi phục `<Outlet />` hiển thị toàn diện dữ liệu.
- **Files:**
  - `webdashboard/src/components/layout/app-shell.tsx`
  - `webdashboard/src/components/layout/__tests__/app-shell.test.tsx`
- **Chi tiết kỹ thuật:**
  - Gỡ bỏ hoàn toàn `DsmTopTaskbar`, `DsmDesktopCanvas`, `DsmLauncherOverlay`.
  - Layout DOM:
    ```tsx
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      <Sidebar isCollapsed={isCollapsed} onToggle={toggleCollapse} />
      <div className="flex flex-1 flex-col overflow-hidden">
        <Topbar onToggleSidebarMobile={toggleMobileSidebar} />
        <main className="flex-1 overflow-y-auto bg-muted/20">
          <ErrorBoundary>
            <Outlet />
          </ErrorBoundary>
        </main>
      </div>
    </div>
    ```
- **Verification:** Kiểm tra render đầy đủ `<Outlet />` và các route con hiển thị bình thường.

---

### Phase 3: Chuẩn hóa Command Palette (`Cmd+K`)
- **Mục tiêu:** Tận dụng bộ từ điển tính năng `feature-registry.ts` để làm hộp thoại tìm kiếm nhanh (Command Bar) chuẩn SaaS, thay thế full-screen DSM launcher.
- **Files:**
  - `webdashboard/src/components/layout/command-palette.tsx` (Mới)
  - `webdashboard/src/components/layout/__tests__/command-palette.test.tsx` (Mới)
- **Chi tiết kỹ thuật:**
  - Modal kích thước vừa vặn (`max-w-2xl`), mở bằng `Cmd+K` hoặc click vào ô tìm kiếm trên Topbar.
  - Cho phép tìm nhanh nhân viên, phân hệ, cài đặt (gõ "wifi", "ot", "nghỉ phép" -> enter chuyển ngay tới route tương ứng).
- **Verification:** Unit test tìm kiếm và phím mũi tên `ArrowUp`/`ArrowDown`/`Enter`.

---

### Phase 4: Tích hợp Slide-over Sheet cho Detail View
- **Mục tiêu:** Áp dụng Sheet từ mép phải cho tác vụ xem chi tiết bản ghi, giữ nguyên context bảng dữ liệu.
- **Files:**
  - `webdashboard/src/pages/employees-page.tsx` (Tích hợp Sheet hiển thị hồ sơ chi tiết khi click hàng)
  - `webdashboard/src/pages/leave-requests-page.tsx` (Tích hợp Sheet phê duyệt/xem lý do đơn nghỉ)
- **Chi tiết kỹ thuật:**
  - Dùng `components/ui/sheet.tsx` (`side="right"`, width `480px - 600px`).
  - Đồng bộ trạng thái mở qua URL Search Params (ví dụ: `?employeeId=xxx`), giúp người dùng có thể bookmark hoặc gửi link trực tiếp.
  - Phím `Esc` đóng Sheet và xóa param trên URL mà không reload bảng.
- **Verification:** Unit test kiểm tra click mở Sheet, hiển thị đúng thông tin và đóng giữ nguyên scroll.

---

### Phase 5: Dọn dẹp Dead Code DSM (Clean Cutover Purge)
- **Mục tiêu:** Xóa triệt để các file DSM đã lỗi thời theo chính sách Clean Code.
- **Files cần xóa:**
  - `webdashboard/src/stores/use-window-manager-store.ts`
  - `webdashboard/src/stores/__tests__/use-window-manager-store.test.ts`
  - `webdashboard/src/components/layout/dsm-desktop-canvas.tsx`
  - `webdashboard/src/components/layout/dsm-window-frame.tsx`
  - `webdashboard/src/components/layout/dsm-top-taskbar.tsx`
  - `webdashboard/src/components/layout/__tests__/dsm-desktop-canvas.test.tsx`
  - `webdashboard/src/components/layout/__tests__/dsm-window-frame.test.tsx`
  - `webdashboard/src/components/layout/__tests__/dsm-top-taskbar.test.tsx`
  - `webdashboard/src/components/features/dsm-launcher/*`
  - `webdashboard/src/lib/dsm-app-registry.tsx`
- **Files cần cập nhật liên kết:**
  - `webdashboard/src/components/features/desktop/desktop-header.tsx`
  - `webdashboard/src/components/features/desktop/widgets/quick-shortcuts-widget.tsx`
  - Gỡ bỏ các event `open-dsm-launcher` cũ, chuyển sang kích hoạt Command Palette.
- **Verification:** Kiểm tra grep không còn sót import mồ côi.

---

### Phase 6: Tổng kiểm thử & Nghiệm thu (Testing & Verification Gate)
- **Mục tiêu:** Đảm bảo toàn bộ hệ thống vượt qua mọi bài test, zero regression.
- **Commands:**
  1. Frontend Unit Tests: `leedevkit test web --unit-only`
  2. Type Check: `cd webdashboard && bun run type-check`
  3. Linter: `cd webdashboard && bun run lint`
  4. Backend Gate: `leedevkit test api --unit-only`
  5. Full Regression Pass: `leedevkit test web`
- **Definition of Done:**
  - [ ] AppShell render Sidebar (collapsible 64px/240px) + Topbar + Outlet.
  - [ ] Tất cả các route trong `_workspace/` truy cập mượt mà qua URL.
  - [ ] Phím tắt `Cmd+B` toggle sidebar, `Cmd+K` mở Command Palette.
  - [ ] Chi tiết nhân viên mở trong Slide-over Sheet từ mép phải.
  - [ ] 100% test suites frontend và backend đều PASSED.
  - [ ] Không còn bất kỳ file code chết nào liên quan đến DSM window manager.

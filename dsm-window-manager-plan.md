# 📋 DSM Multi-Window Desktop Management System - Detailed Implementation Plan

- **Status:** Completed
- **Task Slug:** `dsm-window-manager-plan`
- **Owner:** @frontend-specialist & @backend-specialist
- **Rule Compliance:**
  - `RULE-DB-001` (Multi-Tenancy Isolation: Bắt buộc filter `workspace_id` và `user_id` khi đọc/ghi `dashboard_preferences`)
  - `RULE-DB-004` (Schema Migration: Non-destructive, có đủ `up.sql` và `down.sql`, rollback an toàn)
  - `RULE-UI-002` (WCAG 2.1 AA & Accessibility: Full keyboard drag/focus trap, visible focus ring, aria-label trên mọi nút window control)
  - `RULE-CODE-002` (Clean Code & Type Safety: Không dùng `any`, định nghĩa schema Zod & Go struct chặt chẽ, structured `AppError`)

---

## 🎯 Mục tiêu
Chuyển đổi giao diện LeeAttend từ mô hình định tuyến trang đơn (Single-Page View) sang mô hình **Synology DSM 7 Desktop ảo hoá**:
1. Top Taskbar chuẩn DSM: Tích hợp App Launcher + Quản lý các tab app đang mở + System tray góc phải.
2. Multi-Window Canvas: Cửa sổ nổi tự do (kéo thả header, resize 8 hướng, snap boundary, minimize, maximize, close, z-index focus stacking).
3. DOM Keep-Alive & Lazy Loading: App nạp lười (`React.lazy`), ẩn bằng CSS khi minimize để bảo toàn form/scroll/filter, giới hạn LRU tối đa 8 app mở đồng thời.
4. Server-synced Preferences: Tự động lưu và khôi phục tọa độ, kích thước, trạng thái cửa sổ lên backend PostgreSQL (`dashboard_preferences.window_layout`).

---

## 🛠️ Danh sách công việc chi tiết (Execution Tasks)

### Task 1: Database Migration - Thêm cột `window_layout` vào `dashboard_preferences`
- **Mục tiêu:** Thêm cột `window_layout JSONB DEFAULT '{}'::jsonb NOT NULL` vào bảng `dashboard_preferences`.
- **Files:**
  - `apiserver/migrations/20261107000000_add_window_layout_to_dashboard_preferences/up.sql`
  - `apiserver/migrations/20261107000000_add_window_layout_to_dashboard_preferences/down.sql`
- **Chi tiết:**
  - `up.sql`: `ALTER TABLE public.dashboard_preferences ADD COLUMN IF NOT EXISTS window_layout jsonb DEFAULT '{}'::jsonb NOT NULL;`
  - `down.sql`: `ALTER TABLE public.dashboard_preferences DROP COLUMN IF EXISTS window_layout;`
- **Verification:** Chạy migration qua `./scripts/_db-ops.sh` hoặc test migration script, kiểm tra schema bảng qua PostgreSQL CLI.

### Task 2: Backend API & Go Repository Update
- **Mục tiêu:** Mở rộng struct `DashboardPreferences`, `UpdateDashboardPreferences`, và câu lệnh SQL repository.
- **Files:**
  - `apiserver/internal/api/dashboard_preferences.go`
  - `apiserver/internal/repository/dashboard_preferences.go`
  - `apiserver/internal/api/dashboard_preferences_test.go`
- **Chi tiết:**
  - Thêm trường `WindowLayout json.RawMessage` vào struct.
  - Cập nhật hàm `scanDashboardPreferences`, `GetDashboardPreferences`, `UpdateDashboardPreferences`.
  - Viết unit tests kiểm tra GET/PUT preferences có `window_layout`.
- **Verification:** Chạy test backend Go bằng hermetic wrapper: `./scripts/_go-gates.sh test ./internal/api/...`.

### Task 3: Window Manager Store (`useWindowManagerStore`)
- **Mục tiêu:** Xây dựng Zustand/React store quản lý vòng đời toàn bộ cửa sổ desktop.
- **Files:**
  - `webdashboard/src/stores/use-window-manager-store.ts`
  - `webdashboard/src/stores/__tests__/use-window-manager-store.test.ts`
- **State & Actions:**
  - `windows: Record<string, WindowState>`: `{ id, appId, title, icon, position: {x, y}, size: {w, h}, zIndex, state: 'normal'|'minimized'|'maximized', isFocused }`
  - `openApp(appId: string)`: Mở app (nếu đã mở -> focus & restore nếu minimized; nếu vượt quá 8 app -> đóng LRU app).
  - `closeWindow(id: string)`: Đóng cửa sổ.
  - `minimizeWindow(id: string)`: Thu nhỏ xuống taskbar.
  - `maximizeWindow(id: string)`: Phóng to toàn màn hình / khôi phục kích thước cũ.
  - `focusWindow(id: string)`: Đưa z-index lên đỉnh cao nhất.
  - `updateBounds(id: string, bounds: { x, y, w, h })`: Cập nhật vị trí/kích thước khi kéo thả/resize.
  - `syncToServer()`: Debounce 1000ms gọi `PUT /api/v1/dashboard/preferences`.
- **Verification:** Vitest test suite kiểm tra đầy đủ các action, LRU pruning và debounce sync.

### Task 4: UI Component: `DsmTopTaskbar`
- **Mục tiêu:** Tạo thanh Taskbar chuẩn DSM 7 ở đỉnh màn hình, thay thế `Navbar` và loại bỏ `FloatingDock`.
- **Files:**
  - `webdashboard/src/components/layout/dsm-top-taskbar.tsx`
  - `webdashboard/src/components/layout/__tests__/dsm-top-taskbar.test.tsx`
- **Chi tiết:**
  - Khu vực trái: Nút DSM App Menu (kích hoạt `DsmLauncherOverlay`).
  - Khu vực giữa: Danh sách các app đang mở (Icon + Tên rút gọn + Đèn báo active indicator + click toggle minimize/focus + nút close nhanh).
  - Khu vực phải: Chuyển đổi ngôn ngữ, Dark mode, Thông báo, Profile, Nút Show Desktop (minimize all).
- **Verification:** Unit test render taskbar, active state indicator và sự kiện click toggle window.

### Task 5: UI Component: `DsmWindowFrame`
- **Mục tiêu:** Khung cửa sổ nổi đa năng có header kéo thả, 8 handle resize, và CSS Container Queries.
- **Files:**
  - `webdashboard/src/components/layout/dsm-window-frame.tsx`
  - `webdashboard/src/components/layout/__tests__/dsm-window-frame.test.tsx`
- **Chi tiết:**
  - Header: Drag handle bằng pointer events / native physics, double-click to maximize/restore.
  - Controls: Nút Minimize (`-`), Maximize/Restore (`□`), Close (`✕`).
  - Resize handles: 8 hướng (N, S, E, W, NE, NW, SE, SW) với min width 360px, min height 240px.
  - Đóng Radix Popover/Dropdown đang mở khi bắt đầu di chuyển cửa sổ.
  - Bọc phần thân bằng class `@container` để các component con tự thích ứng kích thước cục bộ.
- **Verification:** Kiểm tra drag bounds không bay ra khỏi viewport, keyboard navigation (phím tắt `Alt+W` đóng, `Alt+M` minimize).

### Task 6: Desktop Canvas & App Registry Integration
- **Mục tiêu:** Tạo canvas chứa wallpaper DSM và tích hợp dynamic import các app hiện có của LeeAttend.
- **Files:**
  - `webdashboard/src/components/layout/dsm-desktop-canvas.tsx`
  - `webdashboard/src/components/layout/app-shell.tsx`
  - `webdashboard/src/lib/dsm-app-registry.tsx`
- **Chi tiết:**
  - Khai báo App Registry mapping từ `appId` (`employees`, `attendance`, `leave`, `payroll`, `settings`, `overview`, `monitor`) sang React lazy component tương ứng.
  - Thay thế Outlet trong `AppShell` bằng `DsmDesktopCanvas`.
  - Trạng thái `minimized`: sử dụng CSS `hidden` (giữ nguyên DOM tree và state component).
- **Verification:** Mở đồng thời 3 app, nhập text vào form của app 1, minimize app 1, mở app 2, restore app 1 -> text trong form vẫn giữ nguyên 100%.

### Task 7: Full Test Suite & Verification (Gate)
- **Mục tiêu:** Chạy toàn bộ test suite từ backend đến frontend đảm bảo không có bất kỳ regression nào.
- **Commands:**
  - Backend: `./leedevkit test apiserver`
  - Frontend: `cd webdashboard && bun run test && bun run type-check`
  - E2E / Integration: `./leedevkit test`
- **Verification:** 100% tests pass, bundle budget không vượt ngưỡng cảnh báo.

---

## ⏱️ Thứ tự thực hiện (Execution Order)
`Task 1 (DB)` ➔ `Task 2 (Backend API)` ➔ `Task 3 (Store)` ➔ `Task 4 (Top Taskbar)` ➔ `Task 5 (Window Frame)` ➔ `Task 6 (Canvas & Apps)` ➔ `Task 7 (Verification)`

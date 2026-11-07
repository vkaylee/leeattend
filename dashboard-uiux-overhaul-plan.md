# 📋 Dashboard UI/UX & Accessibility Overhaul Plan

- **Status:** Approved
- **Task Slug:** `dashboard-uiux-overhaul-plan`
- **Owner:** @frontend-specialist
- **Rule Compliance:**
  - `RULE-UI-001`: Design System & Phân cấp thị giác nhất quán (Không trùng lặp search, loại bỏ khối màu lạc quẻ, bo góc và shadow đồng bộ)
  - `RULE-UI-002`: WCAG 2.1 AA Accessibility (100% icon button có `aria-label`, độ tương phản text >= 4.5:1, visible focus rings)
  - `RULE-UI-004`: Edge Cases & States (Bổ sung Empty State hoàn chỉnh cho widget Việc cần xử lý ngay, Skeleton loader khi đang fetch)
  - `RULE-I18N-001`: Đa ngôn ngữ và Bản địa hóa tiền tệ (Đồng bộ i18n key vi/en/zh, định dạng tiền tệ `0 ₫` qua `useCurrency`)

---

## 🎯 Mục tiêu (Goal)
Khắc phục triệt để các tồn tại UI/UX trên Dashboard máy tính để bàn (Desktop View): loại bỏ ô tìm kiếm trùng lặp, mở rộng nhãn nút thao tác nhanh để không bị cắt chữ, bổ sung Empty State cho danh sách việc cần làm, chuẩn hoá tiền tệ theo locale, và đảm bảo đạt chuẩn Accessibility WCAG 2.1 AA.

---

## 🔒 Ràng buộc & Giao diện (Constraints & Interfaces)
- **Constraints:**
  - Không phá vỡ các bài test hiện có của Vitest trong `webdashboard`.
  - Giữ nguyên cấu trúc widget registry (`DESKTOP_WIDGET_REGISTRY`) và cơ chế customize/hide widget.
  - Tuân thủ quy tắc Clean Code: Không để lại dead code hay CSS thừa.
- **Interfaces:**
  - `DesktopHeaderProps`, `DesktopWidgetDefinition`, `useCurrency().formatCurrency`.
  - i18n namespace `common:desktop.*` trong `vi`, `en`, `zh`.

---

## 🛠️ Danh sách công việc (Tasks)

- [x] **Task 1: Loại bỏ ô tìm kiếm trùng lặp trên Desktop Header**
  - *Action:* Trong `desktop-header.tsx`, ẩn ô tìm kiếm spotlight trigger trên màn hình `>= sm` (`hidden` hoàn toàn vì Topbar đã có ô `⌘K` trung tâm), chỉ hiển thị nút "Gắn Widget" và lời chào.
  - *Files:* `webdashboard/src/components/features/desktop/desktop-header.tsx`
  - *Verify:* Inspect DOM xác nhận chỉ tồn tại duy nhất 1 ô tìm kiếm `⌘K` trên header ở màn hình desktop.

- [x] **Task 2: Tối ưu hoá bố cục thẻ "Thao tác nhanh" (Quick Shortcuts Widget)**
  - *Action:* Căn chỉnh layout dạng 2 cột thẻ rộng rãi, loại bỏ việc cắt chữ (`truncate`) không cần thiết trên nhãn và mô tả; nhãn hiển thị đầy đủ: "Bấm giờ cá nhân", "Tạo đơn xin nghỉ", "Đăng ký tăng ca", "Xem phiếu lương".
  - *Files:* `webdashboard/src/components/features/desktop/widgets/quick-shortcuts-widget.tsx`
  - *Verify:* Chụp ảnh UI kiểm tra các nút hiển thị trọn vẹn văn bản, không còn dấu `...`.

- [x] **Task 3: Bổ sung Empty State cho Widget "Việc cần xử lý ngay" (Pending Tasks)**
  - *Action:* Khi `tasks.length === 0` và không có lỗi, thay vì trả về `null` khiến widget bị rỗng toang, render Empty State tinh tế với icon `CheckCircle2` màu xanh lá nhẹ, tiêu đề "Không có công việc tồn đọng" và chú thích "Bạn đã hoàn thành tất cả thủ tục và quy trình cần xử lý".
  - *Files:* `webdashboard/src/components/features/dashboard/pending-tasks-widget.tsx`
  - *Verify:* Khi danh sách công việc rỗng, widget hiển thị giao diện empty state trực quan thay vì ô trắng trơn.

- [x] **Task 4: Định dạng tiền tệ theo Locale trên Widget "Chỉ số điều hành"**
  - *Action:* Trong `executive-kpi-widget.tsx`, sử dụng `formatCurrency(estimatedPayroll)` từ `useCurrency()` thay vì hiển thị cứng ký hiệu `$`.
  - *Files:* `webdashboard/src/components/features/desktop/widgets/executive-kpi-widget.tsx`
  - *Verify:* Kiểm tra hiển thị `0 ₫` khi locale là tiếng Việt.

- [x] **Task 5: Khắc phục triệt để Accessible Name & WCAG 2.1 AA**
  - *Action:*
    1. Bổ sung `aria-label` cho tất cả icon button (nút ẩn widget `EyeOff`, nút ghim, nút toggle sidebar, nút chuyển đổi ngôn ngữ/theme).
    2. Nâng màu text mô tả phụ từ `text-slate-400` lên `text-slate-500 dark:text-slate-400` để đạt tỷ lệ tương phản tối thiểu 4.5:1 với nền trắng.
    3. Thêm `focus-visible:ring-2 focus-visible:ring-primary` cho tất cả các thẻ tương tác.
  - *Files:* `webdashboard/src/components/features/desktop/desktop-widget-card.tsx`, `webdashboard/src/components/features/desktop/widgets/*.tsx`
  - *Verify:* Scan DOM xác nhận số lượng missing aria buttons giảm về 0.

- [x] **Task 6: Kiểm thử tự động & Nghiệm thu giao diện (Verification Gate)**
  - *Action:*
    1. Chạy Type Check: `cd webdashboard && bun run type-check`
    2. Chạy Linter: `cd webdashboard && bun run lint`
    3. Chạy Unit Tests: `./leedevkit test web --unit-only`
    4. Chạy script Playwright chụp ảnh nghiệm thu lại màn hình `admin_dashboard.png`.
  - *Verify:* Toàn bộ bài test vượt qua 100%, ảnh chụp màn hình đạt chuẩn thẩm mỹ cao cấp.

---

## ✅ Tiêu chí hoàn thành (Done When)
- [x] Giao diện Desktop Dashboard chỉ có 1 thanh tìm kiếm duy nhất trên Topbar.
- [x] Các nhãn nút trong Thao tác nhanh hiển thị đầy đủ, không bị cắt cụt.
- [x] Widget Việc cần xử lý ngay có Empty State chuyên nghiệp khi không có việc tồn đọng.
- [x] Dự toán quỹ lương hiển thị đúng định dạng tiền tệ theo locale.
- [x] 100% button có `aria-label` hợp lệ, đạt chuẩn WCAG 2.1 AA.
- [x] Vitest unit test suite frontend chạy PASSED hoàn toàn.

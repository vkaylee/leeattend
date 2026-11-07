# Kế hoạch Tối ưu Hóa UI/UX & Khả năng Truy cập (WCAG 2.1 AA) Settings Hub

## 📋 Rule Compliance Check (MANDATORY)
1. **Rule ID: `design-rules.md#2-accessibility--wcag-21-aa-standards`**: Đảm bảo tỷ lệ tương phản chữ (Color Contrast Ratio >= 4.5:1), hỗ trợ bàn phím (`focus-visible:ring-2`) và đầy đủ ARIA labels.
2. **Rule ID: `design-rules.md#3-responsive--motion`**: Tối ưu Mobile-first, layout mượt mà không vỡ hạt, tự động đóng navigation drawer sau khi chọn mục.
3. **Rule ID: `coding-standards.md#6-production-readiness`**: Hoàn thiện đa ngôn ngữ (i18n), không trộn lẫn ngôn ngữ (VI/EN), đảm bảo trạng thái loading, skeleton, empty state.
4. **Rule ID: `coding-standards.md#5-boy-scout-rule`**: Sửa ngay lỗi lân cận liên quan: Bug `isActive` trên FloatingDock cho Settings route (`/workspace/settings`).

---

## 🎯 Mục tiêu
Nâng cấp trải nghiệm người dùng toàn diện cho phân hệ Settings Hub của LeeAttend:
- Khắc phục triệt để lỗi độ tương phản và kẹt menu trên Mobile.
- Đồng bộ đa ngôn ngữ 100% giữa Sidebar danh mục và Form nội dung.
- Loại bỏ hiện tượng tràn/cắt cụt chữ trên Sidebar header ở desktop.
- Đồng bộ hóa trạng thái active trên thanh điều hướng FloatingDock.
- Chuẩn hóa định dạng hiển thị cho các trường lựa chọn (Timezone, Date format).

---

## 🏗️ Chi tiết các giai đoạn triển khai (Phased Execution)

### Giai đoạn 1: Sửa lỗi khẩn cấp Mobile UX & Khả năng truy cập (Critical Fixes)
1. **Sửa độ tương phản nút Menu Selector trên Mobile (`settings.tsx`)**:
   - Thay `<Button className="...">` đang bị ghi đè màu chữ tối trên nền xanh (`bg-primary-600 text-slate-700`) thành nút có độ tương phản cao, phong cách Inset Card `variant="outline"` hoặc phong cách Apple Card với text rõ ràng (`text-slate-800 dark:text-slate-100`).
   - Đảm bảo tỷ lệ tương phản đạt chuẩn WCAG 2.1 AA (>= 4.5:1 ở cả Light và Dark mode).
2. **Sửa lỗi kẹt Menu Drawer khi điều hướng trên Mobile (`settings-sidebar.tsx` & `settings.tsx`)**:
   - Bổ sung prop `onItemClick?: () => void` vào `SettingsSidebarProps`.
   - Gắn `onClick={onItemClick}` vào các thẻ `<Link>` của danh mục trong `SettingsSidebar`.
   - Truyền `onItemClick={() => setMobileMenuOpen(false)}` từ `SettingsPageLayout`.
   - Kết quả: Khi người dùng chạm vào bất kỳ danh mục nào trên điện thoại, trang lập tức đóng menu và hiển thị pane chi tiết tương ứng.
3. **Sửa bug `isActive` của Settings trên `FloatingDock` (`floating-dock.tsx`)**:
   - Sửa `pathname.startsWith("/settings")` thành `pathname.startsWith(ROUTES.SETTINGS.ROOT)`.
   - Đảm bảo icon Settings trên FloatingDock sáng đúng trạng thái active khi truy cập `/workspace/settings/...`.

### Giai đoạn 2: Chuẩn hóa Đa ngôn ngữ (i18n Polish)
1. **Đồng bộ hóa nhãn danh mục trong `use-settings-nav.tsx`**:
   - Bổ sung và cập nhật key dịch tiếng Việt trong `webdashboard/src/i18n/locales/vi/settings.json`:
     - *General* -> *Cài đặt chung*
     - *Work Locations* -> *Địa điểm làm việc*
     - *Custom Fields* -> *Trường tùy chỉnh*
     - *Roles* -> *Vai trò & Chức danh*
     - *Permissions* -> *Phân quyền người dùng*
     - *Organization* -> *Cơ cấu tổ chức*
     - *Time & Attendance* -> *Thời gian & Chấm công*
     - *Payroll* -> *Bảng lương & Công thức*
     - *Leave Management* -> *Chính sách nghỉ phép*
     - *Public Holidays* -> *Ngày nghỉ lễ*
     - *Overtime Rules* -> *Quy định tăng ca*
     - *Approval Workflows* -> *Quy trình phê duyệt*
     - *Onboarding Lifecycle* -> *Quy trình tiếp nhận*
     - *Working Conditions* -> *Điều kiện làm việc*
     - *Document Storage* -> *Lưu trữ tài liệu*
     - *E-Contracts* -> *Hợp đồng điện tử*
2. **Đồng bộ Form General Settings (`general-settings.tsx`)**:
   - Khắc phục các nhãn viết hoa cứng (Hardcoded Uppercase): "SAVE CONFIGURATION", "General Configuration", "DATE FORMAT", "TIME FORMAT", "SYSTEM TIMEZONE", "DEFAULT LANGUAGE".
   - Đưa về `t("settings.general...")` với fallback song ngữ chuẩn.

### Giai đoạn 3: Tinh chỉnh Bố cục & Typography (Layout & Visual Refinement)
1. **Chống cắt cụt chữ Sidebar Header (`settings-sidebar.tsx`)**:
   - Tái cấu trúc Header của `SettingsSidebar`: Chuyển huy hiệu `ADMIN` thành dạng badge gọn nhẹ đặt ngay cạnh tiêu đề hoặc dưới dạng tag nhỏ, giải phóng không gian cho dòng subtitle "Bảng điều khiển" / "Control Panel" không bao giờ bị cắt thành `CONTROL P...`.
2. **Format hiển thị Timezone người dùng**:
   - Thay giá trị raw `ASIA/HO_CHI_MINH` bằng nhãn thân thiện: `(GMT+07:00) Asia/Ho Chi Minh`.
3. **Tối ưu tỷ lệ không gian Detail Pane**:
   - Tinh chỉnh khoảng cách padding và độ rộng tối đa của các Card cấu hình để tận dụng hiệu quả diện tích màn hình lớn mà không làm mất tính cân đối.

---

## 🧪 Kiểm thử & Nghiệm thu (Verification & Acceptance Criteria)
1. **Unit Tests**:
   - Cập nhật test suite `settings-layout.test.tsx` kiểm tra việc đóng menu mobile khi click item.
   - Bổ sung test kiểm tra `isActive` cho Settings trên `floating-dock.test.tsx`.
   - Chạy `./leedevkit test web --unit-only` đảm bảo 100% test case pass.
2. **Visual & Browser E2E Verification**:
   - Sử dụng Playwright chụp lại ảnh màn hình Mobile (390×844) kiểm tra tương phản nút selector menu.
   - Chụp lại ảnh màn hình Desktop ở cả Light mode và Dark mode kiểm tra header sidebar, nhãn tiếng Việt và định dạng timezone.
   - Không có console error hay cảnh báo layout shift.

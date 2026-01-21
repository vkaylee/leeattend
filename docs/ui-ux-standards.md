# Tiêu chuẩn UI/UX LeeAttend

Tài liệu này quy định các nguyên tắc thiết kế và trải nghiệm người dùng (UX) cho hệ thống LeeAttend nhằm đảm bảo tính chuyên nghiệp, thẩm mỹ và hiệu suất vận hành cao.

---

## 1. Triết lý thiết kế (Design Philosophy)

### 1.1. Zero-Distraction (Không xao nhãng)
Giao diện phải tập trung vào dữ liệu nghiệp vụ. Loại bỏ các yếu tố trang trí dư thừa.
- **Header:** Chỉ chứa Breadcrumbs tĩnh và thông báo (nếu có). Không đặt các menu điều hướng chính tại đây.
- **Sidebar:** Sử dụng cơ chế "Single-open accordion" (chỉ mở một nhóm menu duy nhất). Nhóm menu "Hệ thống" luôn đóng mặc định.

### 1.2. Premium Aesthetics (Thẩm mỹ cao cấp)
- Sử dụng các góc bo tròn lớn (`rounded-2xl` hoặc `rounded-3xl`) cho Card và Modal.
- Hệ thống Shadow (đổ bóng) nhẹ nhàng, ưu tiên sử dụng màu Shadow cùng tông với màu chủ đạo (ví dụ: `shadow-violet-100`).
- Sử dụng font chữ không chân (Sans-serif) với khoảng cách dòng thoáng đãng.

---

## 2. Hệ thống màu sắc (Color System)

### 2.1. Dynamic Themeing
Hệ thống sử dụng các biến CSS để điều khiển màu sắc chủ đạo:
- `--primary-50` đến `--primary-950`.
- Màu mặc định: **Violet (#7c3aed)**.

### 2.2. Thứ tự ưu tiên màu sắc (Color Priority)
1. **User Override:** Màu do người dùng tự chọn trong trang Hồ sơ (Lưu tại LocalStorage).
2. **Workspace Theme:** Màu đại diện cho thương hiệu của doanh nghiệp (Lưu tại Database).
3. **System Default:** Màu Violet mặc định.

---

## 3. Quy tắc Layout & Responsive

### 3.1. Phân cấp khu vực
- **Sidebar (Trái):** Điều hướng chính và Chuyển đổi Workspace.
- **Header (Trên):** Chỉ dẫn vị trí (Breadcrumbs).
- **Main Content (Giữa):** Khu vực hiển thị dữ liệu và thao tác.

### 3.2. Responsive (Màn hình nhỏ)
- **Dưới 1024px (lg):** Sidebar chuyển thành dạng Drawer.
- **Backdrop:** Phải có lớp phủ mờ (`backdrop-blur-sm`) khi mở Sidebar trên mobile.
- **Touch Targets:** Các phần tử tương tác (Button, Input) phải có chiều cao tối thiểu 44px.

---

## 4. Tương tác & Hiệu ứng (Interactions)

### 4.1. Chuyển động (Animations)
Sử dụng Tailwind Animate cho mọi thành phần mới xuất hiện:
- Trang mới: `animate-in fade-in duration-500`.
- Modal/Dropdown: `animate-in zoom-in-95 slide-in-from-top-2`.

### 4.2. Phản hồi trạng thái (Feedback)
- **Thao tác thành công:** Luôn hiển thị Toast màu xanh lá.
- **Thao tác lỗi:** Toast màu đỏ kèm mô tả ngắn gọn nguyên nhân.
- **Loading:** Ưu tiên dùng **Skeleton Loaders** thay vì Spinner xoay tròn để giảm cảm giác chờ đợi.

---

## 5. Quy chuẩn linh kiện (Component Standards)

### 5.1. Buttons
- **Primary:** Nền màu chủ đạo, chữ trắng, shadow theo màu chủ đạo.
- **Secondary:** Nền xám nhạt hoặc trắng, viền mảnh.
- **Danger:** Chỉ dùng màu đỏ cho các hành động không thể hoàn tác (Xóa vĩnh viễn).

### 5.2. Tables
- Sử dụng nền trắng, viền cực mảnh hoặc không viền (chỉ kẻ dòng).
- Hỗ trợ trạng thái **Hover** trên từng dòng.
- Header của bảng luôn viết hoa, font size nhỏ (`text-xs`) và đậm.

### 5.3. Modals & Dialogs
- **React Portals:** Phải sử dụng `createPortal` để render ra ngoài cấu trúc DOM của trang, đảm bảo không bị che khuất bởi Header/Sidebar hoặc bị giới hạn bởi `overflow` của thẻ cha.
- **Backdrop:** Sử dụng lớp phủ đen trong suốt mỏng (`bg-black/20`) kết hợp làm mờ sâu (`backdrop-blur-md`) để tạo hiệu ứng tập trung.
- **Z-Index:** Luôn duy trì `z-index` cực cao (`z-[9999]`) để đảm bảo phủ lên toàn bộ giao diện.
- **Header:** Luôn có tiêu đề rõ ràng, icon đại diện và nút đóng (X) ở góc phải.
- **Animation:** Sử dụng hiệu ứng `fade-in` cho nền và `zoom-in` hoặc `slide-in-from-bottom` cho nội dung Modal.

---

## 6. Cá nhân hóa (Personalization)

Người dùng có quyền can thiệp vào giao diện mà không ảnh hưởng đến người khác.
- Cho phép thay đổi màu sắc chủ đạo.
- Tự động lưu và nhận diện màu sắc theo trình duyệt (`localStorage`) để đảm bảo tốc độ phản hồi ngay khi vừa tải trang.

---

## 7. Đa ngôn ngữ (Internationalization - i18n)

Hệ thống hỗ trợ đa ngôn ngữ để phục vụ các doanh nghiệp đa quốc gia.

### 7.1. Ngôn ngữ hỗ trợ
- **Tiếng Việt (vi)**: Ngôn ngữ mặc định.
- **Tiếng Anh (en)**: Ngôn ngữ quốc tế.
- **Tiếng Trung (zh)**: Phục vụ các doanh nghiệp sản xuất.

### 7.2. Đồng bộ hóa trạng thái
- **Frontend-First**: Khi người dùng thay đổi ngôn ngữ trên giao diện, `i18next` sẽ cập nhật ngay lập tức mà không cần tải lại trang.
- **Backend Sync**: Ngôn ngữ được chọn sẽ được tự động gửi lên server để lưu vào `preferred_language` của User.
- **Fallback Logic**: 
    1. Ưu tiên ngôn ngữ trong `localStorage`.
    2. Nếu không có, sử dụng `preferred_language` từ Backend.
    3. Nếu là khách (chưa login), sử dụng `default_language` của Workspace.
    4. Cuối cùng, fallback về trình duyệt hoặc Tiếng Việt.

### 7.3. Quy tắc dịch thuật & Cấu trúc (Structure)
Để đảm bảo tính mở rộng và dễ bảo trì, cấu trúc tệp tin ngôn ngữ (`.json`) phải tuân thủ:
- **Feature-First Sections**: Các module lớn (auth, ess, shifts, payroll, patterns, rules...) phải được đặt ở **top-level** của tệp JSON. Điều này giúp các component truy cập key ngắn gọn hơn (ví dụ: `t("auth.login")`).
- **Common Section**: Chỉ dùng cho các nhãn UI dùng chung (Save, Cancel, Close, Search...).
- **Key Parity**: Tất cả các tệp ngôn ngữ (`en.json`, `vi.json`, `zh.json`) **PHẢI** có bộ keys giống hệt nhau 100%.

### 7.4. Kiểm soát tự động (Automated Audit)
Hệ thống thực thi các bài kiểm tra tự động (`npm run test`) để đảm bảo chất lượng dịch thuật:
1.  **Usage Check**: Quét toàn bộ mã nguồn để đảm bảo mọi key được gọi trong `t()` đều tồn tại trong tệp ngôn ngữ.
2.  **Parity Check**: Đảm bảo không có key nào bị thiếu giữa các ngôn ngữ khác nhau.
3.  **JSX Literal Check**: Cảnh báo (Warning) nếu phát hiện chuỗi ký tự thô (Hardcoded strings) trong JSX mà chưa qua hàm `t()`.

---

## 8. Các mẫu thiết kế đặc thù (Special Patterns)

### 8.1. Tab-based Management (Quản lý theo Tab)
Khi một thực thể (ví dụ: Nhân viên) có nhiều loại dữ liệu liên quan (Lịch sử phiếu lương vs Cấu hình lương), sử dụng hệ thống Tab switch ngay trong vùng nội dung chi tiết.
- **Vị trí:** Đặt dưới tên thực thể.
- **Visual:** Sử dụng text-only tabs với dấu phân cách (Bullet dot) và màu sắc nhấn (Primary color) cho tab đang hoạt động.
- **Trải nghiệm:** Chuyển đổi tab không được làm tải lại trang (Single Page navigation).

### 8.2. Selection-First Bulk Actions (Chọn trước - Thao tác sau)
Pattern dùng cho các bảng dữ liệu lớn (Roster, Logs):
1. Người dùng chọn các bản ghi (Checkbox hoặc Click chọn).
2. Thanh công cụ nổi (Floating Action Bar) xuất hiện ở đáy màn hình.
3. Người dùng chọn hành động để áp dụng cho toàn bộ mục đã chọn.
- **Mục tiêu:** Giảm nhiễu giao diện trên từng dòng dữ liệu.

### 8.3. Unified Employee Search (Tìm kiếm nhân viên tập trung)
Để đảm bảo trải nghiệm tìm kiếm nhất quán trên toàn hệ thống:
- **Component**: Luôn sử dụng Combobox `EmployeeSearch` tích hợp sẵn.
- **Search-as-you-type**: Hiển thị kết quả ngay khi người dùng nhập liệu (sau 400ms debounce).
- **Rich Results**: Hiển thị đầy đủ Họ tên kèm Mã nhân viên (Employee ID) để tránh nhaming lẫn.
- **Persistent Selection**: Khi một nhân viên đã được chọn, nhãn (label) của họ phải luôn hiển thị rõ ràng ngay cả khi kết quả tìm kiếm đã thay đổi.
- **Multi-select basket**: Đối với các báo cáo, cho phép chọn nhiều nhân viên bằng cách hiển thị các "chips" (thẻ) có thể xóa được dưới thanh tìm kiếm.

### 8.4. Settings Hub (Trung tâm cấu hình)
Hệ thống sử dụng một mô hình quản lý tập trung cho toàn bộ cấu hình Workspace:
- **Centralized Navigation**: Một điểm truy cập duy nhất tại Sidebar dẫn đến trang tổng quan Settings.
- **Categorized Card Layout**: Các nhóm cài đặt (People, Payroll, Connectivity, v.v.) được hiển thị dưới dạng các Card lớn có Icon đại diện, tiêu đề và mô tả ngắn gọn.
- **Sidebar-driven Detail View**: Khi vào một nhóm cài đặt cụ thể, sử dụng layout 2 cột: Sidebar trái chứa menu danh mục con, vùng bên phải hiển thị form cấu hình chi tiết.
- **Consistent Forms**: Mọi form cài đặt phải sử dụng cùng một kiểu Save/Cancel button đặt ở góc dưới bên phải hoặc Toolbar cố định.

### 8.5. Date & Period Selection (Chọn ngày và Kỳ lương)
Để tối ưu hóa thao tác người dùng khi làm việc với các dữ liệu theo chu kỳ (Lương, Báo cáo):
- **Component**: Luôn sử dụng `MonthYearPicker` cho các trường chọn Tháng/Năm.
- **Visual**: Ưu tiên hiển thị lưới 12 tháng (`3x4` hoặc `4x3`) để chọn nhanh bằng 1 lần click thay vì dùng dropdown list dài.
- **Defaulting Strategy**: Đối với các nghiệp vụ kết thúc kỳ (Tính lương, Xem báo cáo tháng), mặc định luôn chọn **Tháng trước** (`Last Month`) thay vì tháng hiện tại, vì đây là khoảng thời gian người dùng thường xuyên thao tác nhất.
- **Shortcuts**: Luôn cung cấp các nút chọn nhanh "Tháng này" và "Tháng trước" để reset bộ lọc ngay lập tức.

---

---

## 10. Debug Suite & Traceability (New)

Thiết kế dành cho quản trị viên và kỹ thuật viên để kiểm toán (audit) logic hệ thống.

### 10.1. Developer Toolbar
Một floating action button ở góc dưới bên phải màn hình (`z-[9999]`). Toolbar cung cấp các công cụ:
- **Identity**: Chuyển đổi nhanh vai trò (Owner/Admin/Member) hoặc giả lập (impersonate) một user cụ thể.
- **Time Travel**: Giả lập thời gian hệ thống để kiểm tra logic tính lương/phép trong tương lai.
- **Database Utilities**: Các thao tác dọn dẹp nhanh (Wipe logs/payroll) để phục vụ testing.

### 10.2. Traceability UI (BSON Visibility)
Nguyên tắc hiển thị nguồn gốc dữ liệu mà không làm rối người dùng thông thường.
- **Pattern**: Thêm một cột "Debug Info" (ẩn mặc định) trong các bảng báo cáo quan trọng (Attendance Reports).
- **Detail Level**: Sử dụng cơ chế **Expandable Rows**. Khi nhấn vào icon Debug, một vùng nội dung mới sẽ mở ra bên dưới dòng hiện tại.
- **Visual Style**: Sử dụng Terminal-style box (nền `slate-900`, chữ `emerald-400`, font `mono`) để hiển thị JSON metadata thô.
- **Activation**: Chỉ hiển thị cột Debug và các thành phần liên quan khi "Global Debug Mode" được kích hoạt từ Developer Toolbar.

---

## 11. Time-Tracking UI Components (Phase 1)

Thiết kế dành cho giao diện chấm công (Clock) và quản lý thời gian làm việc.

### 11.1. PunchCard Component
**Purpose**: Primary interface for active time session display and punch actions.

**Visual Hierarchy**:
- Large status indicator (24x24 icon in rounded container with contextual color)
- Employee name (bold, 2xl font)
- Status badge (Working/On Break/Off Duty) with animated dot
- Current session timestamp in monospace font (when active)
- Action buttons with distinct colors:
  - `bg-primary-600` for "Start Work"
  - `bg-amber-500` for "Start Break"
  - `bg-emerald-600` for "Return to Work"
  - `bg-rose-600` for "Punch Out"
- Subtle "Forgot to punch?" link at bottom

**Container**: Rounded white card (`rounded-[40px]`) with shadow and decorative background icon.

### 11.2. TimeEntryRow Component
**Purpose**: Standardized table row for displaying individual time entry records.

**Columns**:
1. **Date + Type Badge**: Entry date with colored badge (Amber for Break, Primary for Work)
2. **Punch In Time**: Monospace formatted time
3. **Punch Out Time**: Either formatted time or animated status badge if session ongoing
4. **Adjust Button** (optional): Appears on hover, opens adjustment modal

**Row Styling**: Light hover effect (`hover:bg-slate-50/50`), smooth transitions, grouped "group" container for hover-reveal buttons.

### 11.3. AdjustmentRequestModal Component
**Purpose**: Form for submitting attendance corrections with validation.

**Form Fields**:
- Date picker (single date selection)
- Punch In time input (HH:MM format)
- Punch Out time input (HH:MM format)
- Reason textarea (multi-line, required)
- Submit/Cancel buttons

**Validation**:
- Punch In must be before Punch Out
- Both times required
- Reason required (min 10 chars recommended)
- UTC conversion applied based on workspace timezone

**Modal Styling**: Standard modal with backdrop blur, high z-index, fade-in animation.

### 11.4. Real-time Status Updates
**Behavior**: 30-second auto-refresh via React Query to maintain synchronized clock status.
- Smooth UI updates without disrupting user interaction
- Active entry indicator updates on server changes
- Prevents manual refresh fatigue

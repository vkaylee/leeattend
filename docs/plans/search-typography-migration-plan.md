# Kế Hoạch Chuyển Đổi Đồng Bộ SearchInput & Typography (VitePress Clean Standard)

## 1. Bối cảnh & Mục tiêu
- **Hiện trạng:**
  - Sau khi xây dựng thành công `ThemeSwitch`, `SearchInput` và các biến thể `Typography` (`hero`, `tagline`, `<GradientText />`) lấy cảm hứng từ chuẩn Clean & Minimalist của VitePress (`nginxproxymanager.com`), thanh Topbar và lời chào Dashboard đã đạt độ hoàn thiện cao.
  - Tuy nhiên, trên toàn bộ hệ thống (`webdashboard`), các trang danh sách (Reports, Payroll, E-Contracts, Shifts, Settings) vẫn đang sử dụng các thẻ `<input>` thô sơ với class CSS phân tán, không đồng bộ về padding, bo góc, icon kính lúp và trạng thái focus.
- **Mục tiêu:**
  - Thay thế toàn bộ các ô tìm kiếm rời rạc bằng component dùng chung `SearchInput` (`src/components/ui/search-input.tsx`).
  - Chuẩn hóa các tiêu đề trang và phụ đề bằng `Typography` / `<GradientText />`.
  - Giữ vững 100% test contract hiện có, đảm bảo tương thích WCAG 2.1 AA (hỗ trợ phím tắt, nút xóa nhanh, viền focus bàn phím rõ nét).

---

## 2. Bảng Thống Kê & Phân Loại Các Vị Trí Cần Thay Thế

### Nhóm A: Thanh Lọc & Tìm Kiếm Trên Bảng Dữ Liệu Chính (Data Table Filters)
| File | Vị trí / Tính năng | Hiện trạng | Kế hoạch chuyển đổi |
|---|---|---|---|
| `src/components/features/reports/report-filters.tsx` | Bộ lọc tìm kiếm nhân viên theo tên trong Báo cáo | Thẻ `<input type="text">` thô + icon tuyệt đối + nút X thủ công | Thay bằng `<SearchInput size="sm" shortcut="⌘K" onClear={...} />` |
| `src/components/payroll/summary-table/summary-search-header.tsx` | Ô tìm kiếm nhân viên trên Bảng tổng hợp lương | Thẻ `<input>` bo góc tròn to `rounded-[1.5rem]`, padding 14 | Chuẩn hóa bằng `<SearchInput size="md" />`, đồng bộ trạng thái focus |
| `src/components/features/employees/econtracts-filters.tsx` | Ô tìm kiếm hợp đồng lao động & mã nhân sự | Thẻ `<input>` class cứng `h-10 pl-10 pr-4` | Thay bằng `<SearchInput size="sm" />` |
| `src/components/common/navigable-list/navigable-list-search.tsx` | Ô tìm kiếm danh sách điều hướng chung | Thẻ `<input>` tự chế `rounded-[2rem] py-5` | Tích hợp `<SearchInput size="md" />`, bảo toàn phím điều hướng `listNav.onKeyDown` |

### Nhóm B: Hộp Thoại & Modal Chọn Lọc (Modals & Dialog Pickers)
| File | Vị trí / Tính năng | Hiện trạng | Kế hoạch chuyển đổi |
|---|---|---|---|
| `src/components/shifts/bulk-employee-picker-modal.tsx` | Modal phân ca hàng loạt cho nhân viên | Thẻ `<input>` kèm nhãn số lượng đã chọn | Thay bằng `<SearchInput size="sm" />` |
| `src/components/shifts/shift-picker-modal.tsx` | Modal chọn ca trực trong lịch làm việc | Thẻ `<input>` tìm kiếm ca trực | Thay bằng `<SearchInput size="sm" />` |
| `src/components/settings/permission-management-modal.tsx` | Modal phân quyền & tìm kiếm quyền hệ thống | Thẻ `<input>` tìm kiếm mã quyền | Thay bằng `<SearchInput size="sm" />` |
| `src/components/leave/leave-policy-components/template-gallery.tsx` | Modal chọn mẫu chính sách nghỉ phép | Thẻ `<input>` tìm template mẫu | Thay bằng `<SearchInput size="sm" />` |
| `src/components/features/dashboard/chart-catalog-sidebar.tsx` | Sidebar tìm kiếm biểu đồ trong kho widget | Thẻ `<input>` tìm kiếm widget | Thay bằng `<SearchInput size="sm" />` |

### Nhóm C: Tiêu Đề Phân Hệ & Typography (Page Headers & Taglines)
| File | Vị trí / Tính năng | Hiện trạng | Kế hoạch chuyển đổi |
|---|---|---|---|
| `src/components/ui/page-header.tsx` | Header tiêu đề dùng chung các trang | Thẻ `h1` và `p` cơ bản | Hỗ trợ prop `gradientTitle` và style `tagline` cho subtitle |
| `src/pages/employees/components/employees-page-header.tsx` | Header trang Nhân sự | Chữ plain text thông thường | Nhấn gradient tinh tế cho từ khóa chính |
| `src/pages/payroll/components/payroll-header.tsx` | Header trang Tính lương | Chữ plain text thông thường | Nhấn gradient tinh tế cho từ khóa chính |
| `src/pages/reports/components/reports-header.tsx` | Header trang Báo cáo chấm công | Chữ plain text thông thường | Nhấn gradient tinh tế cho từ khóa chính |

---

## 3. Lộ Trình Triển Khai (4 Giai Đoạn)

### Giai đoạn 1: Bảng dữ liệu cốt lõi (Core Data Tables)
- **Mục tiêu:** Nâng cấp trải nghiệm tìm kiếm ở 3 màn hình nghiệp vụ có tần suất sử dụng cao nhất: Báo cáo (`report-filters.tsx`), Lương (`summary-search-header.tsx`), Hợp đồng (`econtracts-filters.tsx`).
- **Thực hiện:**
  1. Thay thế markup `<input>` bằng `<SearchInput>`.
  2. Kết nối prop `value`, `onChange`, `onClear`, `placeholder`.
  3. Chạy regression test:
     `./leedevkit test web --unit-only --pattern "report-filters|payroll-summary|econtracts"`
  4. Chụp màn hình Playwright MCP để kiểm tra giao diện bảng.

### Giai đoạn 2: Hộp thoại & Picker Modals (Dialogs & Selectors)
- **Mục tiêu:** Chuẩn hóa ô tìm kiếm bên trong các modal phân ca và cài đặt.
- **Thực hiện:**
  1. Thay thế trong `bulk-employee-picker-modal.tsx`, `shift-picker-modal.tsx`, `permission-management-modal.tsx`, `template-gallery.tsx`.
  2. Đảm bảo hỗ trợ keyboard focus (`autoFocus` hoặc phím tắt).
  3. Chạy unit test:
     `./leedevkit test web --unit-only --pattern "bulk-employee|shift-picker|permission-management"`

### Giai đoạn 3: Danh sách điều hướng & Widget Catalog
- **Mục tiêu:** Đồng bộ `navigable-list-search.tsx` và `chart-catalog-sidebar.tsx`.
- **Thực hiện:**
  1. Thay thế input nhưng bảo toàn `searchInputRef` và `listNav.onKeyDown` (ArrowUp, ArrowDown, Enter).
  2. Chạy test `navigable-list.test.tsx` và `chart-catalog-sidebar.test.tsx`.

### Giai đoạn 4: Đồng bộ Typography & Page Headers
- **Mục tiêu:** Cập nhật `PageHeader` để các trang phân hệ (Nhân viên, Lương, Báo cáo, Ca làm) có tiêu đề sắc nét với tùy chọn `GradientText` và tagline chuẩn VitePress.
- **Thực hiện:**
  1. Thêm prop `gradientTitle?: boolean` vào `src/components/ui/page-header.tsx`.
  2. Kiểm tra trực quan bằng Playwright MCP cả Light Mode và Dark Mode.

---

## 4. Tiêu Chuẩn Nghiệm Thu & Kiểm Chứng (Quality Gates)
1. **Accessibility (WCAG 2.1 AA):**
   - Mọi `SearchInput` phải có `aria-label` hoặc `placeholder` rõ ràng.
   - Khi có dữ liệu nhập, nút X (`Clear`) phải có `aria-label="Clear search input"`.
   - Vòng focus bàn phím rõ nét, không bị clipping viền.
2. **Khả năng tương thích ngược (Zero Regressions):**
   - Giữ nguyên toàn bộ `data-testid` hiện có (`report-search-input`, `payroll-search-input`, ...).
   - Toàn bộ unit tests liên quan phải pass 100%.
3. **Hiệu năng & Bundle Size:**
   - Không cài thêm bất kỳ dependency nào từ bên ngoài. Tận dụng tối đa `SearchInput` và `Typography` vừa tạo.

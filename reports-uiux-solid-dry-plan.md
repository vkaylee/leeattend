# Kế Hoạch Tối Ưu UI/UX Báo Cáo Chấm Công (/reports) Theo Nguyên Lý SOLID & DRY

Status: Approved

## Mục Tiêu
Tối ưu hóa toàn diện giao diện, trải nghiệm người dùng, khả năng tiếp cận (WCAG 2.1 AA) và bản địa hóa tiếng Việt cho trang `/reports` bằng cách chuẩn hóa các thành phần theo nguyên lý SOLID và loại bỏ trùng lặp theo DRY.

## Tuân Thủ Quy Chuẩn Hệ Thống (Rule Compliance)
- **`ARCH-001` (Clean Architecture & SOLID):** Tách bạch trách nhiệm giữa thanh điều khiển/bộ lọc (Page level) và các bảng dữ liệu (Presentation level); áp dụng Dependency Inversion cho các cell dùng chung.
- **`CODE-001` (DRY - Don't Repeat Yourself):** Loại bỏ trùng lặp ô tìm kiếm, tái sử dụng `EmployeeIdentityCell` cho cả 2 view, và chuẩn hóa dùng chung `EmptyState`.
- **`TYPO-001` (Typography Standards):** Chuẩn hóa toàn bộ nhãn, tiêu đề, nút bấm từ ALL-CAPS sang Sentence case thân thiện, loại bỏ cảm giác căng thẳng ("shouting UI").
- **`A11Y-001` (WCAG 2.1 AA & Semantic Contrast):** Sửa lỗi tương phản ngữ nghĩa (không bôi màu cam cảnh báo cho giá trị 0 ngày nghỉ, không bôi xanh link cho 0h bình thường).
- **`UI-001` (Design System Consistency):** Giảm thiểu độ dày các dải công cụ xếp tầng từ 350px xuống dưới 180px, tối ưu không gian hiển thị cho dữ liệu chính.

---

## Kiến Trúc & Thiết Kế SOLID / DRY

### 1. Phân Tích SOLID
| Nguyên Lý | Hiện Trạng (Smell) | Thiết Kế Tối Ưu |
|:---|:---|:---|
| **S - Single Responsibility** | `ReportsTable` vừa quản lý bảng tổng hợp vừa tự render thanh tìm kiếm và stats badges, khiến `TimesheetMatrixGrid` bị thiếu ô tìm kiếm. | `ReportsTable` chỉ chịu trách nhiệm render dữ liệu bảng tổng hợp. Ô tìm kiếm được đưa lên `ReportFilters` tại tầng trang, dùng chung cho cả 2 chế độ xem. |
| **O - Open/Closed** | `ReportsHeader` chứa các chuỗi điều kiện inline để chọn nhãn nút xuất và hành động. | Nhóm các cấu hình chế độ xem (viewMode, exportLabel, exportAction) thành contract rõ ràng, dễ dàng mở rộng khi thêm chế độ xem mới. |
| **L - Liskov Substitution** | Hai chế độ xem (Ma trận 31 ngày & Bảng tổng hợp) nhận dữ liệu lọc từ hai nguồn khác nhau. | Cả 2 view đều tiêu thụ chung danh sách `filteredEmployees` từ pipeline lọc thống nhất tại `useReportsPage`. |
| **I - Interface Segregation** | `ReportFiltersProps` khai báo các props tìm kiếm (`searchQuery`, `onSearchQueryChange`) nhưng không render mà để lọt xuống `NavigableList`. | `ReportFiltersProps` tích hợp và render trực quan ô tìm kiếm; loại bỏ các props thừa không dùng đến. |
| **D - Dependency Inversion** | Cột nhân viên trong `columns.tsx` tự dựng HTML riêng biệt không đồng bộ. | `columns.tsx` phụ thuộc trực tiếp vào component trừu tượng `EmployeeIdentityCell` đã xây dựng. |

### 2. Phân Tích DRY
* **Ô tìm kiếm nhân viên:** Thay vì viết 2 ô tìm kiếm cục bộ hoặc thiếu trong ma trận, dùng 1 ô tìm kiếm duy nhất tại `ReportFilters`.
* **Cột định danh nhân viên:** Tái sử dụng `EmployeeIdentityCell` cho cả Bảng tổng hợp giờ lẫn Ma trận 31 ngày.
* **Empty State:** Thay thế các chuỗi text viết hoa tự do (`NO DATA FOUND FOR THIS PERIOD.`) bằng component chuẩn `EmptyState`.
* **Từ điển i18n:** Chuẩn hóa các key bị lỗi (`reports.view_raw_logs`, `reports.table.employee`).

---

## Ràng Buộc & Giao Diện Toàn Cục (Constraints & Interfaces)
- **Bảo toàn chức năng:** Không làm ảnh hưởng đến logic tính công, xuất file CSV ma trận BOM UTF-8 hay hộp thoại xuất Excel.
- **Ngôn ngữ:** Mặc định tiếng Việt chuẩn nhân sự (Sentence case); tương thích song ngữ hoàn hảo khi chuyển sang tiếng Anh.
- **Kiểm thử tự động:** Mọi thay đổi phải vượt qua 100% unit tests (`./leedevkit test web --unit-only`) và xác minh qua ảnh chụp Playwright MCP.

---

## Danh Sách Nhiệm Vụ (Tasks)

- [ ] **Task 1: Sửa lỗi i18n và chuẩn hóa Sentence case cho từ điển**
  - *Nội dung:* Bổ sung key `reports.view_raw_logs: "Nhật ký thô"`, `reports.table.employee: "Nhân viên"` trong `vi/common.json` và `en/common.json`. Chuẩn hóa các nhãn từ ALL-CAPS sang Sentence case (ví dụ: "Bảng công tháng", "Tổng hợp giờ", "Xuất Excel", "Xuất ma trận (CSV)").
  - *Ràng buộc:* Giữ nguyên cấu trúc JSON hiện có, không xóa key đang dùng.
  - *Giao diện:* `webdashboard/src/i18n/locales/{vi,en}/common.json`.
  - *Xác minh:* `npm test -- --pattern "locales|reports"` và không còn nút nào hiển thị `REPORTS.VIEW_RAW_LOGS`.

- [ ] **Task 2: Tích hợp ô tìm kiếm vào `ReportFilters` và tinh gọn bố cục thanh công cụ (SRP & DRY)**
  - *Nội dung:* Di chuyển ô tìm kiếm nhân sự vào `ReportFilters` (cùng hàng hoặc tích hợp gọn gàng với bộ lọc trạng thái và phòng ban). Xóa ô tìm kiếm trùng lặp trong `NavigableList` của `ReportsTable`.
  - *Ràng buộc:* Tìm kiếm phải phản hồi tức thì trên cả 2 view (Matrix & Summary), hỗ trợ phím tắt và xóa nhanh.
  - *Giao diện:* `webdashboard/src/components/features/reports/report-filters.tsx`, `webdashboard/src/pages/reports/components/reports-table.tsx`.
  - *Xác minh:* Gõ từ khóa tìm kiếm khi đang xem `MONTHLY TIMESHEET` thì danh sách nhân sự ma trận tự động lọc theo.

- [ ] **Task 3: Tái sử dụng `EmployeeIdentityCell` và chuẩn hóa màu sắc trong bảng Tổng hợp (DIP & DRY)**
  - *Nội dung:* Cập nhật `columns.tsx` của `ReportsTable` để render `EmployeeIdentityCell` (avatar badge viết tắt + tên in đậm chuẩn + mã nhân viên monospace). Sửa màu sắc: `0 ngày` chuyển thành màu xám trung tính (`text-slate-400`); `0h` chuyển thành text số thường không bôi xanh link; bỏ màu cam đơn lẻ ở header cột `Tăng ca`.
  - *Ràng buộc:* Giữ nguyên tính năng click vào dòng để mở `EmployeeDailyDrawer`.
  - *Giao diện:* `webdashboard/src/pages/reports/components/reports-table/columns.tsx`.
  - *Xác minh:* Cột nhân sự ở cả 2 bảng có cùng visual identity; số 0 ngày không còn màu cam cảnh báo.

- [ ] **Task 4: Chuẩn hóa `EmptyState` và giao diện `EmployeeDailyDrawer` (Design System & Typography)**
  - *Nội dung:* Thay thế đoạn text thô `NO DATA FOUND FOR THIS PERIOD.` trong `daily-report-table.tsx` bằng `EmptyState` component có icon lịch, tiêu đề thân thiện: *"Chưa có dữ liệu chấm công"*, mô tả hướng dẫn. Chuyển typography của header drawer về Sentence case.
  - *Ràng buộc:* Đảm bảo drawer hiển thị đẹp mắt, không tràn viền trên mobile 375x812.
  - *Giao diện:* `webdashboard/src/components/features/reports/daily-report-table.tsx`, `webdashboard/src/components/features/reports/employee-daily-drawer.tsx`.
  - *Xác minh:* Mở drawer của nhân viên chưa có dữ liệu chấm công, màn hình hiển thị `EmptyState` chuẩn, không có lỗi pointer-events hay vỡ layout trên mobile.

- [ ] **Task 5: Tối ưu thanh Legend ký hiệu công và phản hồi trên Mobile (Responsive UX)**
  - *Nội dung:* Thu gọn bảng giải thích *Ký hiệu công* trên mobile để không chiếm 4 dòng chắn ngang màn hình (sử dụng popover hoặc modal sheet khi màn hình nhỏ < 640px). Đưa các nhãn ký hiệu về Sentence case.
  - *Ràng buộc:* Người dùng mobile vẫn dễ dàng tra cứu ký hiệu công mà không làm che bảng.
  - *Giao diện:* `webdashboard/src/components/features/attendance/timesheet-matrix-grid.tsx`.
  - *Xác minh:* Chuyển viewport sang 375x812, nút ký hiệu công mở popover/sheet gọn gàng và đóng mở mượt mà.

- [ ] **Task 6: Kiểm thử tự động hồi quy và kiểm tra trực quan Playwright MCP**
  - *Nội dung:* Chạy toàn bộ test suites web & api bằng hermetic wrapper: `./leedevkit test web --unit-only` và `./leedevkit test api --unit-only`. Chụp ảnh màn hình Desktop & Mobile để xác nhận các cải tiến trực quan.
  - *Ràng buộc:* 100% test pass; không có console error nào trong runtime.
  - *Giao diện:* Toàn bộ trang `/reports`.
  - *Xác minh:* Log test báo `PASSED`, ảnh chụp giao diện sạch sẽ, chuyên nghiệp, không lỗi layout.

---

## Tiêu Chí Hoàn Thành (Done When)
- [ ] Không còn bất kỳ text bung mã i18n nào trên giao diện (đặc biệt là nút `Nhật ký thô`).
- [ ] Ô tìm kiếm nhân sự hoạt động đồng bộ và hiển thị ở cả 2 chế độ xem (Bảng công tháng & Tổng hợp giờ).
- [ ] Cả 2 bảng đều sử dụng chung `EmployeeIdentityCell` với avatar badge và định danh nhất quán.
- [ ] Không còn màu cam cảnh báo cho `0 ngày nghỉ` hay màu xanh link cho `0h`.
- [ ] Loại bỏ tình trạng ALL-CAPS trên toàn bộ nhãn, nút và cột bảng.
- [ ] Drawer chi tiết hiển thị `EmptyState` chuẩn thiết kế khi chưa có dữ liệu.
- [ ] Toàn bộ unit tests web và api đều PASSED 100%.

## Điểm Cần Lưu Ý Khi Review (Review Focus)
1. **Hiệu năng lọc khi danh sách nhân sự lớn:** Khi đưa ô tìm kiếm lên toolbar dùng chung, đảm bảo có debounce nhẹ hoặc lọc tối ưu qua `useMemo` để không gây giật lag trên bảng ma trận 31 cột x N nhân viên.
2. **Khả năng bấm dòng trên bảng:** Khi nhúng `EmployeeIdentityCell` vào `ReportsTable`, đảm bảo sự kiện click mở drawer trên cả hàng không bị xung đột với các phần tử con.

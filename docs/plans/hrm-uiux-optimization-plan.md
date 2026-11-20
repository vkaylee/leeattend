# Kế Hoạch Tối Ưu Hóa UI/UX Chuẩn Hóa Nghiệp Vụ HRM & Áp Dụng DRY, SOLID

Status: Approved

## Mục Tiêu
Chuẩn hóa giao diện (UI) và trải nghiệm người dùng (UX) của LeeAttend từ góc nhìn vận hành Nhân sự (HRM) chuyên nghiệp, tối ưu hóa quy trình nghiệp vụ cho 5 vai trò On-Premise (`admin`, `hr_manager`, `shift_manager`, `team_lead`, `employee`), tuân thủ Luật Lao động Việt Nam 2019, đồng thời tái cấu trúc kiến trúc mã nguồn theo các nguyên lý Clean Code (DRY & SOLID).

---

## Các Ràng Buộc & Giao Diện Toàn Cục (Constraints & Interfaces)
- **Ràng buộc pháp lý & nghiệp vụ:** Tuân thủ Bộ luật Lao động 2019 (khoảng cách nghỉ giữa ca >= 12h, trần làm thêm 40h/tháng, cơ chế chốt/khóa bảng lương bất biến).
- **Ràng buộc trợ năng & an toàn:** WCAG 2.1 AA (độ tương phản >= 4.5:1, `aria-label` cho nút icon-only, nhãn ngữ nghĩa `label`), che giấu PII tuyệt đối (`user_***@domain.com`).
- **Ràng buộc thực thi (PTY Safety):** Mọi lệnh terminal phải chuyển hướng log `> logs/run_{command_slug}.log 2>&1 </dev/null`.
- **Giao diện & Module cốt lõi:**
  1. `webdashboard/src/pages/payroll-page.tsx` & `webdashboard/src/components/payroll/`
  2. `webdashboard/src/pages/attendance-logs-page.tsx` & `webdashboard/src/components/features/attendance/`
  3. `webdashboard/src/pages/approvals/unified-inbox-page.tsx` & `components/approval-detail-drawer.tsx`
  4. `webdashboard/src/pages/ess/my-profile-page.tsx` & `webdashboard/src/pages/members-page.tsx`

---

## Kiến Trúc DRY & SOLID Được Áp Dụng

| Nguyên Lý | Hiện Trạng Cần Tối Ưu | Thiết Kế Mới Theo Tiêu Chuẩn |
|---|---|---|
| **Single Responsibility (SRP)** | Các trang lớn (`payroll-page.tsx`, `attendance-logs-page.tsx`) đang ôm đồm cả logic fetch dữ liệu, tính toán ma trận, lọc thời gian và điều khiển modal/drawer. | Tách các custom hooks riêng biệt: `use-payroll-lifecycle.ts` (quản lý trạng thái khóa/mở kỳ), `use-timesheet-matrix.ts` (tính toán lưới công tháng), `use-schedule-impact.ts` (kiểm tra xung đột ca khi duyệt đơn). |
| **Open/Closed (OCP)** | Bảng lương và loại công đang dùng hardcoded switch-case khi thêm cột mới hoặc loại hình nghỉ phép. | Chuyển sang mô hình Metadata-driven Column Definitions: cấu hình cột được định nghĩa linh hoạt theo schema danh mục công (`attendance-code-registry.ts`), mở rộng loại công mới mà không sửa logic render của Grid. |
| **Liskov Substitution (LSP)** | Các loại đơn duyệt (`leave`, `overtime`, `shift_swap`, `attendance_adjustment`) có cấu trúc hiển thị khác nhau gây phân mảnh component chi tiết. | Toàn bộ 4 loại đơn thực thi chung giao diện `UnifiedApprovalItemContract`; component `ApprovalDetailDrawer` và `ApprovalImpactCard` hiển thị hoán đổi mượt mà bất kể loại đơn nào. |
| **Interface Segregation (ISP)** | Nhiều component nhận vào các prop object nhân viên cồng kềnh chứa cả trường nhạy cảm. | Tách thành các interface tinh gọn: `EmployeeSummaryBadgeProps` (chỉ nhận `id, name, avatar, department`), `PayslipSummaryProps` (chỉ nhận dữ liệu lương cần render). |
| **Dependency Inversion (DIP)** | Các component con gọi trực tiếp API client hoặc mutation cục bộ. | Component phụ thuộc vào abstraction thông qua custom hook facades (`useTimesheetActions`, `usePayrollActions`), cho phép dễ dàng mock dữ liệu khi viết Unit Test và E2E. |
| **DRY (Don't Repeat Yourself)** | Code định dạng tiền tệ VND, định dạng ngày công, badge trạng thái phê duyệt và hàng bảng rỗng (`TableEmptyRow`) bị viết lặp lại ở nhiều nơi. | Chuẩn hóa thư viện tiện ích dùng chung: `CurrencyBadge`, `AttendanceStatusPill`, `TimesheetMatrixGrid`, `TableEmptyRow`. |

---

## Danh Sách Nhiệm Vụ Triển Khai (Tasks)

- [ ] **Task 1: Chuẩn hóa Quy trình Chốt & Khóa Kỳ Lương (Payroll Lock Workflow)**
  - *Mô tả:* Bổ sung trạng thái vòng đời kỳ lương (`DRAFT` - Bản nháp, `CALCULATED` - Đã tính toán, `LOCKED` - Đã khóa chốt sổ) vào `PayrollHeader` và `payroll-summary-table.tsx`. Khi kỳ lương ở trạng thái `LOCKED`, vô hiệu hóa các nút chỉnh sửa/tính lại và hiển thị badge "Đã khóa sổ bởi [Tên HR] vào [Ngày/Giờ]".
  - *Nguyên tắc áp dụng:* SRP (tách hook `use-payroll-lifecycle.ts`), DRY (dùng chung `ConfirmDialog`).
  - *Ràng buộc:* C&B có thể xem lịch sử audit ai đã khóa/mở kỳ lương; chặn nhân viên sửa công kỳ đã khóa.
  - *Xác minh:* Kiểm tra trạng thái kỳ lương trên UI; click nút "Khóa kỳ lương" xuất hiện modal xác nhận và chuyển đổi trạng thái giao diện sang Read-only.

- [ ] **Task 2: Xây dựng Chế độ Xem Bảng Tổng Hợp Công Tháng (Monthly Timesheet Matrix Grid)**
  - *Mô tả:* Bổ sung tab mới `timesheet-matrix` trong `attendance-logs-page.tsx`. Giao diện dạng ma trận: Cột cố định bên trái (Mã NV, Họ tên, Phòng ban), Khối cuộn ngang 31 ngày (hiển thị ký hiệu công: `X`, `P`, `Ro`, `OT`, `Ô`), Cột tổng hợp bên phải (Tổng công chuẩn, Tổng giờ OT, Số ngày nghỉ phép).
  - *Nguyên tắc áp dụng:* OCP (sử dụng dynamic columns registry), ISP (component `TimesheetCell` siêu nhẹ tối ưu hiệu năng ảo hóa Virtual Scrolling).
  - *Ràng buộc:* Hỗ trợ bàn phím di chuyển giữa các ô; tooltip giải thích chi tiết giờ vào/ra khi hover ô công.
  - *Xác minh:* Truy cập tab "Bảng công tháng", bảng lưới render đầy đủ 31 ngày, cuộn mượt mà không vỡ layout trên desktop 1280x800.

- [ ] **Task 3: Nâng cấp Phân tích Tác Động Lịch Trực Khi Phê Duyệt (Team Schedule Impact Card)**
  - *Mô tả:* Tích hợp component `TeamScheduleImpactCard` vào `approval-detail-drawer.tsx` trong Hộp thư đến (`/approvals/inbox`). Khi quản lý xem xét duyệt đơn nghỉ phép/đổi ca, hiển thị ngay số lượng nhân sự cùng phòng ban đang đi làm trong ngày đó và cảnh báo nếu tỷ lệ đi làm dưới ngưỡng tối thiểu (< 70%).
  - *Nguyên tắc áp dụng:* LSP (thực thi thống nhất trên mọi loại đơn xin nghỉ hoặc đổi ca), DIP (truy vấn qua hook `use-schedule-impact.ts`).
  - *Ràng buộc:* Trực quan hóa bằng thanh tiến độ màu sắc (Xanh: an toàn, Vàng: thiếu người nhẹ, Đỏ: vi phạm định mức ca).
  - *Xác minh:* Mở chi tiết một đơn xin nghỉ phép trong inbox, kiểm tra thẻ thông tin lịch trực hiển thị trực quan và cảnh báo đúng số lượng nhân sự.

- [ ] **Task 4: Thiết kế Phiếu Lương Điện Tử Chi Tiết & Bảo Mật (Interactive Payslip View)**
  - *Mô tả:* Tối ưu hóa ngăn chi tiết phiếu lương (`payroll-detail-drawer.tsx`) và trang tự phục vụ của nhân viên (`/ess/profile` -> Phiếu lương). Phân nhóm minh bạch theo 4 khối:
    1. Thu nhập chịu thuế & Lương cơ bản
    2. Các khoản phụ cấp không chịu thuế (Ăn trưa, xăng xe, điện thoại)
    3. Các khoản khấu trừ bắt buộc (BHXH 8%, BHYT 1.5%, BHTN 1%, Thuế TNCN)
    4. Lương thực lĩnh (Net Take-home Pay) nổi bật kèm nút tải PDF.
  - *Nguyên tắc áp dụng:* ISP (giao diện `PayslipSectionProps`), DRY (dùng chung cho cả HR xem và Nhân viên xem).
  - *Ràng buộc:* Tự động ẩn số tiền (Masking) với nút bấm toggle con mắt để bảo mật nơi công cộng; WCAG contrast >= 4.5:1.
  - *Xác minh:* Mở xem phiếu lương trên tài khoản `employee`, kiểm tra phân nhóm thu nhập - khấu trừ rõ ràng, nút ẩn/hiện số tiền hoạt động chính xác.

- [ ] **Task 5: Tối ưu Phân Loại Trạng Thái Nhân Sự & Cảnh Báo Hết Hạn Hợp Đồng**
  - *Mô tả:* Bổ sung bộ lọc trạng thái lao động theo luật (`Thử việc`, `Chính thức`, `Tạm hoãn HĐLĐ`, `Đã nghỉ việc`) và widget cảnh báo trên `members-page.tsx`:
    - "Cần đánh giá thử việc trong 7 ngày tới (N nhân sự)"
    - "Hợp đồng lao động sắp hết hạn trong 30 ngày (N nhân sự)"
  - *Nguyên tắc áp dụng:* OCP (danh sách cảnh báo khai báo theo quy tắc), SRP (tách component `ContractExpirationAlerts`).
  - *Ràng buộc:* Nhãn trạng thái phân biệt bằng cả màu sắc và biểu tượng (tuân thủ WCAG sensory characteristics).
  - *Xác minh:* Danh sách nhân viên lọc chuẩn theo trạng thái; widget cảnh báo hiển thị chính xác các trường hợp cần xử lý tái ký.

- [ ] **Task 6: Kiểm Thử Tự Động, Hồi Quy & Xác Minh Trực Quan MCP Playwright**
  - *Mô tả:*
    - Chạy toàn bộ test suites frontend và backend qua hermetic wrappers (`leedevkit test web --unit-only`, `leedevkit test api --unit-only`).
    - Dùng Playwright MCP xác minh trực quan trên 3 vai trò chính: `hr_manager` (xem payroll & timesheet), `team_lead` (xem impact card khi duyệt), `employee` (xem payslip cá nhân).
  - *Ràng buộc:* Tỷ lệ pass 100%, không phát sinh cảnh báo accessibility hoặc lỗi console mới.

---

## Tiêu Chí Hoàn Thành (Done When)
- [ ] Bảng lương có cơ chế chốt/khóa kỳ minh bạch, ngăn ngừa sai lệch số liệu sau kỳ chốt.
- [ ] Bảng công tháng hiển thị dạng lưới ma trận 31 ngày chuẩn nghiệp vụ C&B.
- [ ] Người quản lý nắm được tác động lịch trực trước khi phê duyệt đơn.
- [ ] Phiếu lương nhân viên minh bạch các khoản trích nộp theo Luật Lao động Việt Nam.
- [ ] Toàn bộ unit tests và kiểm tra giao diện qua Playwright MCP đều vượt qua 100%.

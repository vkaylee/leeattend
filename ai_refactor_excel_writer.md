# Kế Hoạch Safe Refactoring: `excel_writer.rs` God Module

## 1. 🎯 Mục Tiêu (Objective)
- **Vấn đề:** File `apiserver/src/services/excel_writer.rs` hiện tại đang là một "God Module" với kích thước ~1200 dòng, đảm nhiệm quá nhiều công việc (từ định nghĩa format, query/tính toán dữ liệu, đến thao tác ghi file trực tiếp). Việc này vi phạm Single Responsibility Principle (SRP) và khiến bảo trì, viết test rất khó.
- **Giải pháp:** Phân rã `excel_writer.rs` thành các sub-module chuyên biệt (Styles, Extractors, Generators) áp dụng Facade & Strategy Pattern, đồng thời tuân thủ chuẩn Enterprise Grade Premium (Type-safe, dễ test).

---

## 2. 🛡 Protect (Bảo Vệ)
- [ ] **Khóa tính năng (Freeze):** Đảm bảo không có commit nào thêm tính năng liên quan đến Excel/Export trong khi tiến hành refactor.
- [ ] **Kiểm tra trạng thái (Baseline Check):** Chạy `./run-tests.sh --api` để đảm bảo hệ thống đang Pass toàn bộ tests. Lấy snapshot log.
- [ ] **Test Coverage cho Export:** Kiểm tra xem đã có Integration Test (E2E API) cho module Export chưa. Nếu chưa, bổ sung một test `export_smoke_test.rs` đơn giản (kiểm tra status code 200, header trả về file `xlsx`).

---

## 3. 🏗 Refactor (Tái Cấu Trúc)
Không thay đổi logic nghiệp vụ, chỉ tổ chức lại code.

- [ ] **Bước 3.1: Tạo Cấu Trúc Thư Mục Mới**
  - Tạo folder `apiserver/src/services/excel/` và `mod.rs`.
  - Khai báo các module con trong `mod.rs`.

- [ ] **Bước 3.2: Tách Styling (style_utils.rs)**
  - Di chuyển toàn bộ logic khởi tạo Format (`title_format`, `header_format`, `number_format`, v.v.) sang `apiserver/src/services/excel/style_utils.rs`.
  - Cung cấp hàm `get_enterprise_formats(workbook)` để tái sử dụng ở mọi sheet.

- [ ] **Bước 3.3: Tách Data Extractor (data_extractor.rs)**
  - Tách logic chuẩn bị dữ liệu (`summaries`, data fetching logic từ request) ra khỏi phần write file. Trả về cấu trúc DTO thuần (không liên quan đến thư viện `rust_xlsxwriter`).

- [ ] **Bước 3.4: Tách Sheet Generators (generators/attendance_sheet.rs, ...)**
  - Tách logic vòng lặp `worksheet.write_string`, `write_number` ra một struct hoặc trait `SheetGenerator`. Mỗi generator chịu trách nhiệm ghi dữ liệu DTO lên một worksheet cụ thể.

- [ ] **Bước 3.5: Cấu trúc lại Facade (excel_writer.rs)**
  - Sửa file `excel_writer.rs` hiện tại để đóng vai trò làm Coordinator/Facade:
    - Khởi tạo Workbook.
    - Gọi `data_extractor` để lấy DTO.
    - Gọi `style_utils` lấy config.
    - Truyền vào các `SheetGenerator` tương ứng.
    - Lưu file và trả về stream/bytes.

---

## 4. ✅ Verify (Kiểm Chứng)
- [ ] **Type-Safety & Linter:** Chạy `./scripts/_cargo.sh clippy -- -D warnings`. Đảm bảo code 100% không có cảnh báo mới và giải quyết các lỗi lifetime (nếu có do việc truyền file tham chiếu).
- [ ] **Hồi Quy (Regression Tests):** Chạy `./run-tests.sh --all` trong vùng chứa Podman, kiểm tra API export xem file Excel trả về có giữ nguyên cấu trúc cũ không.
- [ ] **Auditing (Checklist):** Chạy `python .agent/scripts/checklist.py .` để AI đánh giá tổng quan codebase sau khi refactor.

---
*Ghi chú: Nếu gặp trục trặc về test E2E cho chức năng export, chuyển sang chế độ fallback hoặc dừng lại và báo cáo để fix.*

# PLAN: Đưa WebDashboard Linter về con số 0 (Zero Lint Debt)

## 🛡️ Rule Compliance Check
- **Rule 4.2 (Clean Code Standards)**: Đảm bảo SOLID & DRY. Các component vi phạm `max-lines-per-function` và `complexity` sẽ được tách nhỏ để đảm bảo Single Responsibility và dễ đọc hiểu.
- **Rule 4.3 (Testing Standards)**: Yêu cầu bắt buộc phải pass `./test.sh web </dev/null` trước khi hoàn thành bất kỳ thay đổi nào.
- **Rule 5.1 & 5.4 (Verification Pipeline)**: Bắt buộc dùng hermetic scripts (`./scripts/_npm.sh run lint -- src/` hoặc `./test.sh web </dev/null`) để chạy kiểm tra linter thay vì chạy trực tiếp `npm` trên host.
- **Rule 4.8 (Performance & Resource Optimization)**: Loại bỏ các function bị rò rỉ bộ nhớ (leaked renders), tối ưu rendering để pass các check liên quan.
- **PTY Safety**: Không sử dụng lệnh ngầm (`&`) hoặc pipes mà thiếu `</dev/null >/dev/null 2>&1` để tránh bị treo (Deadlocks).

---

## 🎯 Mục Tiêu
Giải quyết toàn bộ các vi phạm ESLint, TypeScript và React trong WebDashboard (dựa trên kết quả linting trả về) để đạt được **0 error, 0 warning**. Tổng số lỗi hiện đang rơi vào khoảng trên 100 lỗi trải dài khắp các component (đặc biệt trong thư mục `src/components/features` và `src/components/payroll`).

## 📊 Phân tích lỗi hiện tại
Từ kết quả linting, các lỗi tập trung vào 4 nhóm chính:
1. **Complexity & Size (Khó nhất):** `max-lines-per-function` (> 100 dòng), `complexity` (> 10). Phổ biến ở các component lớn như `MemberTable`, `ConnectivityTopology`, `ReportExportDialog`, `EmployeeDailyDrawer`.
2. **TypeScript Strictness:** `@typescript-eslint/no-explicit-any`, `no-misused-promises`, `no-unnecessary-type-conversion` (dùng `!!`), `require-await`, `prefer-nullish-coalescing`.
3. **React/UX Rules:** `react/no-unescaped-entities`, `react-hooks/exhaustive-deps`.
4. **Code Style/Deprecation:** `perfectionist/sort-imports`, `@typescript-eslint/no-deprecated` (Recharts `Cell`).

---

## 🗺️ Lộ Trình Xử Lý (4 Giai Đoạn)

### Phase 1: Trái cây treo thấp (Low-hanging Fruits) & Auto-fixes
**Mục tiêu:** Giảm nhanh số lượng lớn các lỗi cú pháp và code style không làm thay đổi logic.
- Chạy tự động sửa lỗi (nếu có thể): `./scripts/_npm.sh run lint -- src/ --fix </dev/null`
- Xử lý `perfectionist/sort-imports` (đảo thứ tự import).
- Xử lý `react/no-unescaped-entities` (sửa `"` thành `&quot;`).
- Xử lý `@typescript-eslint/no-unnecessary-type-conversion` (xóa các dấu `!!` dư thừa).
- Sửa các hàm async không có await (`@typescript-eslint/require-await`).

### Phase 2: Nâng cấp TypeScript Strictness (Type-safety)
**Mục tiêu:** Thay thế các kiểu dữ liệu không an toàn và xử lý sai logic Promise.
- Thay thế triệt để các vị trí dùng `any` (`@typescript-eslint/no-explicit-any`) bằng `interface` hoặc `type` chính xác.
- Sửa lỗi `@typescript-eslint/no-misused-promises` và `@typescript-eslint/no-confusing-void-expression`.
- Thay các toán tử `||` bằng `??` (`@typescript-eslint/prefer-nullish-coalescing`) để tránh rủi ro logic với số `0` hoặc chuỗi rỗng `""`.

### Phase 3: "Giải cứu" Monolith Components (De-monolithizing)
**Mục tiêu:** Tách các component khổng lồ (vi phạm max-lines & complexity) thành sub-components/hooks.
- **Tách Logic:** Tạo các Custom Hooks (`use...`) để di chuyển state và logic xử lý ra khỏi các file `*.tsx`.
- **Tách UI:** Chia nhỏ các component lớn như:
  - `ConnectivityTopology`
  - `BatchCalculationModal` & `BonusPolicyManager`
  - `ReportExportDialog`
  - `EmployeeFormModal` & `EmployeeDailyDrawer`
- **Tái cấu trúc biểu đồ:** Disable hoặc sửa lỗi deprecation của `Recharts Cell` (`@typescript-eslint/no-deprecated`) ở các biểu đồ (turnover, recruitment).

### Phase 4: Final Verification (Xác nhận 100% sạch lỗi)
**Mục tiêu:** Chạy lại toàn bộ CI suite trên hermetic toolbox và xác nhận con số 0.
- Command kiểm tra: `./test.sh web --lint-only </dev/null`
- Đảm bảo terminal output thông báo: `[webdashboard-lint] PASSED`.
- Quét lại bằng `checklist.py` (nếu cần thiết theo yêu cầu Final Check).

# Kế Hoạch Refactor WebDashboard: Xử lý Technical Debt (Complexity & Max Lines)

## 🛡️ Rule Compliance Check

1. **Rule 4.2 (Clean Code Standards):** Phân chia Single Responsibility cho các Component lớn (chia nhỏ `assignment-form-modal.tsx`, `payslip-modal.tsx`, `leave-request-row.tsx`).
2. **Rule 4.6 (Boy Scout Rule):** Khi refactor component, kết hợp dọn dẹp các rule bị vi phạm nhỏ lân cận nếu chúng gây ảnh hưởng cấu trúc.
3. **Rule 5.4 (Verification Pipeline):** Xác minh sự thay đổi qua hermetic tool (`./scripts/_npm.sh run lint` và test pipeline) đảm bảo ZERO regression cho các phần đã sửa.

---

## 📅 Các Giai Đoạn (Phases)

### Giai Đoạn 1: Phân tách các UI Component Khổng Lồ (Monolithic Components)
Tách các components hiện đang vượt quá `max-lines-per-function` (100) và `complexity` (10) thành các sub-components gọn gàng hơn.
- `src/components/payroll/rule-assignment/assignment-form-modal.tsx` (336 lines)
- `src/components/payroll/payslip-modal.tsx` (246 lines)
- `src/components/features/leave/leave-request-row.tsx` (complexity 26)

*Giải pháp:* 
- Rút trích các phần như header, section form, action buttons thành các component tĩnh bên ngoài.
- Gom các phần tử rendering danh sách thành components tách biệt thay vì map nội tuyến (inline mapping).

### Giai Đoạn 2: Xử Lý Logic Phức Tạp trong Hooks
Giảm tải cyclomatic complexity của các hooks điều phối dữ liệu lớn.
- `src/components/features/reports/use-employee-daily-drawer.ts` (257 lines)
- `src/hooks/use-report-stream.ts` (152 lines, complexity 11)

*Giải pháp:*
- Tách state orchestration (quản lý trạng thái nội bộ) và API interactions (giao tiếp bên ngoài) thành 2 hooks hoặc tạo utility function hỗ trợ parsing/format dữ liệu.

### Giai Đoạn 3: Dọn Dẹp Các Component Bậc 2
- `src/components/features/employees/employee-form-modal.tsx`
- `src/components/payroll/payroll-summary-table.tsx`
- `src/components/payroll/batch-calculation-modal.tsx`
- ...

### Giai Đoạn 4: Validation (Kiểm tra quy chuẩn)
Chạy command hermetic để verify kết quả:
```bash
./scripts/_npm.sh run lint
```
Đảm bảo các file đã sửa không còn cảnh báo về max-lines hoặc complexity, đồng thời không phá vỡ logic render (no-leaked-renders).

---

# WORKSTREAM C: UI Refactoring - ESS, Approvals & Settings

## 🛡️ Rule Compliance Check
1. **Rule 4.11 Hermetic Toolbox**: Sử dụng `./scripts/_npm.sh` cho các lệnh npm/bun để chạy trong container cách ly.
2. **Rule 4.8 Enterprise-Grade Premium Standard**: Đảm bảo UI giữ nguyên UX, các components được tách nhỏ sạch sẽ, props rõ ràng.
3. **Rule 4.3 Testing Standards**: Xác minh bằng `./test.sh --web </dev/null` sau mỗi phase.
4. **Rule 4.10 PTY Safety**: Luôn sử dụng `</dev/null` khi chạy lệnh shell.

## 📋 Tình trạng hiện tại (Baseline)
- `holidays-page.tsx`: 426 lines (Max allowed: 100).
- `time-settings.tsx`: Complexity 27 (Max: 10).
- `my-profile-page.tsx`: 231 lines, Complexity 14.
- `ot-approvals-page.tsx`: 177 lines, Unescaped entities.
- Tổng cộng 40 lỗi/cảnh báo linter trong scope.

## 🚀 Các giai đoạn thực hiện

### Phase 1: Approvals Module (`src/pages/approvals/`)
- [ ] **Task 1.1**: Refactor `ot-approvals-page.tsx`.
    - Trích xuất `OtApprovalsTable`, `OtApprovalsFilters`.
    - Sửa lỗi `react/no-unescaped-entities`.
- [ ] **Task 1.2**: Kiểm tra lint module Approvals.

### Phase 2: ESS Module (`src/pages/ess/`)
- [ ] **Task 2.1**: Refactor `my-profile-page.tsx`.
    - Tách các Section thành components: `PersonalInfoSection`, `SecuritySection`, etc.
    - Giảm Complexity và fix `@typescript-eslint/no-unnecessary-type-conversion`.
- [ ] **Task 2.2**: Refactor `my-attendance-page.tsx`.
    - Tách `AttendanceSummary`, `AttendanceTable`.
- [ ] **Task 2.3**: Refactor `my-clock-page.tsx`.
    - Fix non-null assertions.
- [ ] **Task 2.4**: Kiểm tra lint module ESS.

### Phase 3: Settings Module (`src/pages/settings/`)
- [ ] **Task 3.1**: Refactor `holidays-page.tsx` (TRỌNG TÂM).
    - Tách `HolidayTable`, `HolidayFormModal`.
    - Loại bỏ non-null assertions.
- [ ] **Task 3.2**: Refactor `time-settings.tsx` (TRỌNG TÂM).
    - Tách các Tab/Section nội dung thành sub-components.
    - Refactor logic tính toán phức tạp ra ngoài component.
- [ ] **Task 3.3**: Refactor `audit-log-page.tsx`.
- [ ] **Task 3.4**: Refactor các file còn lại: `settings-layout.tsx`, `leave-settings.tsx`, `payroll-settings.tsx`.
- [ ] **Task 3.5**: Kiểm tra lint module Settings.

### Phase 4: Final Verification
- [ ] **Task 4.1**: Chạy toàn bộ linter cho webdashboard.
- [ ] **Task 4.2**: Chạy smoke test UI (nếu có thể) hoặc xác minh build thành công.

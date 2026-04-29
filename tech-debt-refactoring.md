# Technical Debt Refactoring Plan

## Goal
Phục hồi chất lượng codebase đạt chuẩn Enterprise Grade Premium bằng cách xử lý: lỗ hổng Security Auth Context (Backend), đồng bộ Test Suite (Frontend), và chuẩn hóa UI Components (Frontend).

## Tasks
- [ ] Task 1: Sửa `apiserver/src/handlers/employment_phases.rs`, trích xuất `user_id` từ `Extension<User>` và truyền vào `evaluated_by`, xóa dòng `TODO`. → Verify: `./scripts/_cargo.sh clippy </dev/null` không báo lỗi.
- [ ] Task 2: Chạy Backend Integration Tests. → Verify: `./run-tests.sh --api </dev/null` chạy thành công.
- [ ] Task 3: Sửa `webdashboard/src/pages/__tests__/MembersPage.test.tsx`, thay đổi logic mock từ `window.confirm` sang `useConfirm` hook và xóa `TODO`. → Verify: `./scripts/_npm.sh run test MembersPage </dev/null` Passed.
- [ ] Task 4: Thay thế thẻ `<button>` và `<input>` thô bằng `<Button>` và `<Input>` từ Design System trong `webdashboard/src/pages/WorkingPatternsPage.tsx`. → Verify: UI hiển thị đúng, form hoạt động bình thường.
- [ ] Task 5: Thực hiện thay thế tương tự (HTML -> UI Components) cho `webdashboard/src/pages/LeaveRequestsPage.tsx`. → Verify: Chạy lại `./scripts/_npm.sh run lint </dev/null` không có lỗi.
- [ ] Task 6: Thực hiện kiểm chứng toàn hệ thống. → Verify: `./run-tests.sh --all </dev/null` Passed toàn bộ.

## Done When
- [ ] Không còn comment `TODO` trong `employment_phases.rs` và `MembersPage.test.tsx`.
- [ ] File `WorkingPatternsPage.tsx` và `LeaveRequestsPage.tsx` không còn sử dụng thẻ `<button>` và `<input>` nguyên thủy.
- [ ] Toàn bộ hệ thống test (Backend & Frontend) báo Passed.

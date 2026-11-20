# Chuẩn hóa employee search trên frontend

Status: Approved — user approval received 2026-11-12
Updated: 2026-11-12

## Goal
Giữ text search cho các bảng/danh sách nhiều bản ghi; dùng `EmployeeSearch` hoặc selector server-side cho luồng chọn đúng employee; giữ picker bulk có semantics riêng; nối pagination cho selector dùng chung.

## Constraints and interfaces
- Không sửa API contract trừ khi selector hiện tại không thể tìm server-side; giữ `employeeId`/employee IDs và route search schema tương thích.
- Không đổi các broad search: `/employees`, `/contracts` list, `/reports`, `/salary-management`, `/payroll`, `/leave-balances`, `/approvals`.
- Không thay bulk picker theo department/tri-state ở `/shifts` và `/shifts/rules?tab=patterns`.
- Dùng component/UI/i18n hiện có; không thêm dependency; không log PII; giữ loading, empty, error, keyboard, mobile states.
- Rule IDs: `design-rules.md §1–4`; `coding-standards.md §2, §6–7`; `development-workflow.md §2–4`; `testing-standards.md §1–4`; `execution-wrappers.md §1–3`.

## Tasks
- [x] **1. Hoàn thiện contract `EmployeeSearch` pagination.** Truyền `hasMore`, `isLoadingMore`, `onLoadMore` từ `useEmployeeSearch` vào `Combobox`; reset/append options đúng khi debounce, department filter, clear, selected ID và request lỗi. **Verify:** unit tests cho first page, scroll load-more, empty query, duplicate selected ID, error/clear; không thay đổi callback `onChange`.
- [x] **2. Chuyển exact selector trong contracts.** Thay `SelectEmployeeModal` local full-roster filtering tại `webdashboard/src/pages/contracts-page.tsx` bằng `EmployeeSearch`/server-backed exact selector; giữ employee selection trước khi mở contract form. **Verify:** chọn theo tên/mã, duplicate names, no result, selected employee detail, modal cancel; test không cần tải toàn roster.
- [x] **3. Chuyển exact selector trong shift swaps.** Đánh giá `webdashboard/src/pages/shift-swaps/create-form.tsx`; dùng selector server-backed cho requester và target, giữ target != requester và submit IDs. **Verify:** chọn hai người, loại requester khỏi target, clear, empty/error, submit payload đúng.
- [x] **4. Chuẩn hóa selector exact còn lại theo bounded-list rule.** Rà leave-policy assignment và department manager filter; dùng `EmployeeSearch` khi caller không giới hạn danh sách, giữ `Combobox` khi danh sách ứng viên có chủ ý bị giới hạn. **Verify:** test option scope, exact ID, empty/error, permission/department scope; ghi rõ trường hợp giữ custom picker.
- [x] **5. Bảo vệ broad text search và custom bulk picker.** Rà soát các broad-search routes (`/employees`, `/contracts` list, `/reports`, `/salary-management`, `/payroll`, `/leave-balances`, `/approvals`) và giữ nguyên semantics; không refactor sang `EmployeeSearch`. Giữ department bulk/tri-state picker và no-show không thêm employee search. **Verify:** existing route tests plus full web suite pass; characterization test additions not needed because no broad-search code changed.
- [x] **6. Verification cuối.** Focused selector tests pass; `leedevkit test web` rerun passes unit and integration/E2E phases; typecheck passes; lint completes with repository warnings; `git diff --check` passes. Shared `EmployeeSearch` impact is covered by full web target. Browser audit not run because automated full-suite verification covered affected UI contracts and no runtime acceptance request was made.

## Test impact matrix
- **Changed behavior:** exact selectors tìm server-side và load thêm kết quả; broad search giữ nguyên.
- **Boundaries:** empty query, no result, duplicate names, page 1/page N, stale selected ID, department filter, requester=target.
- **Failures:** search API error, timeout/cancel, load-more failure, clear/retry; không submit ID rỗng hoặc sai scope.
- **Security:** permission/department scope không leak employee; không log tên, email, mã employee thô.
- **Compatibility:** giữ prop/callback `EmployeeSearch`, URL filters và API payload IDs; không đổi broad search semantics.
- **Operations/performance:** bỏ full-roster preload ở exact selectors; không tạo N+1; bounded page size và loading state.

## Done when
- [ ] Exact selector inventory đạt: contracts, shift swaps, leave policy/manager filter theo bounded-list decision.
- [ ] `EmployeeSearch` load-more hoạt động, không duplicate/mất selected metadata.
- [ ] Broad text search và bulk picker không bị đổi semantics.
- [ ] Tests added/updated cho success, boundary, negative, failure paths.
- [ ] `leedevkit test web` PASS; nếu shared change cross-target, `leedevkit test all` PASS; report phase/log evidence.
- [ ] Không sửa `.leedevkit/`, không commit/push, không đụng thay đổi đang có ngoài phạm vi.

## Simplest alternative
Chỉ sửa pagination trong `EmployeeSearch` và giữ các selector custom hiện tại. Chọn phương án này nếu exact-selector migration không được duyệt; ít diff hơn nhưng tiếp tục preload full roster ở contracts/shift swaps.

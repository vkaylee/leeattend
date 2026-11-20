# RBAC role normalization beyond admin

Status: Approved

## Goal
Chuẩn hóa quyền của `hr_manager`, `shift_manager`, `team_lead`, `employee` theo canonical permission catalog; loại bỏ lệch tên legacy; giới hạn department scope đúng vai trò; không mở rộng quyền ngoài nghiệp vụ.

## Rule IDs
- `access-control.md §1-2`: default deny, handler authorization, granular permissions, scope boundary.
- `database-rules.md §3-4`: dữ liệu RBAC idempotent, migration có rollback, không destructive cleanup thiếu phê duyệt.
- `development-workflow.md §2, §4, §6`: test impact matrix, authorization/migration/compatibility coverage.
- `testing-standards.md §1-2, §4`: focused tests rồi full applicable LeeDevKit target.
- `migration-and-rollback.md §1-6`: expand/migrate/verify/contract, rollback và mixed-version safety.

## Constraints and interfaces
- `admin` giữ đủ 97 canonical permissions; không thay đổi scope admin.
- Không xóa legacy rows trong bước đầu. Mapping phải additive và idempotent.
- Không dùng wildcard như `admin:all`; chỉ grant permission canonical cụ thể.
- `employee` chỉ giữ self-service; không nhận department-management permission.
- `team_lead` phải bị chặn khi truy cập department ngoài phạm vi được quản lý.
- Fresh-baseline policy hiện tại không hỗ trợ upgrade chain cũ; nếu cần cập nhật DB hiện hữu, dùng bootstrap reconciliation hoặc migration path được phê duyệt, không tạo migration version mà runner không áp dụng.
- Không đổi public API permission names nếu chưa có compatibility mapping cho frontend, tests, seed và persisted rows.

## Tasks
- [x] 1. Inventory quyền hiện tại → đối chiếu `auth.AllPermissions()`, role seed, handler checks, frontend route guards và docs role matrix; lập bảng legacy → canonical với trạng thái `exact`, `split`, `retired`, `no-safe-mapping`. → Verify: không còn permission legacy chưa được phân loại; mọi mapping split/no-safe có owner và quyết định.
- [ ] 2. Chốt role × permission matrix → định nghĩa tối thiểu cho `hr_manager`, `shift_manager`, `team_lead`, `employee`; phân biệt read/manage/approve/on-behalf; giữ least privilege. → Interface: seed catalog và `/me/permissions` cùng một canonical contract. → Verify: review diff matrix; test deny cho quyền ngoài role.
- [ ] 3. Chốt scope model → chỉ role cần quản trị toàn tổ chức dùng `global`; `team_lead` dùng `scoped`; xác định rõ `hr_manager`/`shift_manager` có global hay giới hạn theo department dựa trên nghiệp vụ hiện tại. → Interface: `roles.scope_type`, `GetManagedDepartmentIDs`, `HasScopedPermission`. → Verify: cross-department request trả 403; same-department request được phép; global role không bị false deny.
- [ ] 4. Thêm additive reconciliation → cập nhật `catalog.go` bằng canonical grants cho role matrix; thêm mapping/runtime compatibility cho legacy names nếu persisted rows hoặc old clients còn dùng; không xóa rows. → Constraints: `ON CONFLICT DO NOTHING`, transaction-safe, fail closed. → Verify: chạy bootstrap hai lần không duplicate; old legacy rows vẫn tồn tại; effective permissions chỉ chứa quyền được chốt.
- [ ] 5. Chuẩn hóa consumers → cập nhật handler permission literals, frontend route/nav guards, mocks, i18n permission labels và dev-account expectations sang canonical names; giữ compatibility seam có thời hạn nếu cần. → Verify: grep active source không còn legacy literal ngoài migration/compatibility fixture; API/UI permission responses thống nhất.
- [ ] 6. Bổ sung test bảo vệ contract → role matrix parity, unknown/retired permission rejection, bootstrap idempotency, legacy compatibility, scoped allow/deny, unauthorized 403 và no side effect. → Verify: test fail khi grant thừa hoặc thiếu; test không phụ thuộc thứ tự DB rows.
- [ ] 7. Rollout và cleanup gate → deploy additive grants trước, quan sát `/me/permissions` và 403 metrics, sau compatibility window mới đề xuất xóa legacy rows/aliases bằng migration có `down.sql`; không tự cleanup production data. → Verify: reconciliation query, rollback procedure, owner/removal date cho compatibility code.
- [ ] 8. Verification → chạy focused API tests, `leedevkit test api`, kiểm tra phase logs; nếu backend/shared contract thay đổi thì chạy `leedevkit test all`. → Verify: report pass/fail/skip/warnings và remaining risks; `git diff --check`.

## Done When
- [ ] Mỗi non-admin role có matrix canonical được review.
- [ ] Không còn legacy permission không phân loại hoặc dùng ngầm.
- [ ] `team_lead` và mọi scoped role bị chặn ngoài department scope.
- [ ] Bootstrap/reconciliation idempotent và giữ dữ liệu legacy trong compatibility window.
- [ ] Handler, frontend guard, seed, tests và docs dùng cùng canonical names.
- [ ] Applicable API/integration/full-suite verification pass.

## Review Focus
- Mapping một legacy permission sang canonical permission rộng hơn làm tăng quyền ngoài ý muốn.
- `global` scope hiện tại có thể bypass department boundary.
- Old persisted role rows và old frontend clients có thể không hiểu canonical names.
- Permission grants đúng nhưng handler vẫn dùng literal legacy, gây 403 sai.
- Rollback sau khi đã cấp additive grants không được làm mất quyền hợp lệ của admin.

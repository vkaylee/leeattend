# Squash migrations thành single-organization baseline

Status: Approved

## Goal
Gộp migration chain hiện tại thành một baseline PostgreSQL single-organization; database mới không tạo `workspaces`, `workspace_*`, hoặc `workspace_id`; reset database dev thay vì hỗ trợ nâng cấp dữ liệu cũ.

## Quyết định đã chốt
- Database dev: reset/recreate.
- Không hỗ trợ migrate database đã chạy chain cũ.
- Xoá compatibility artifacts khỏi active runtime/schema: table, column, query, route, fixture, script.
- Keep only active Go migration and schema documentation in repository context.
- Không commit hoặc chạy thao tác xoá database thật trước khi kiểm tra target và xác nhận volume dev.

## Rule IDs áp dụng
- `database-rules.md §4`: migration phải có rollback artifact; không drop production schema thiếu phê duyệt.
- `migration-and-rollback.md §1-6`: squash, backup boundary, integrity verification, recovery evidence.
- `development-workflow.md §2, §6`: test impact matrix; persistence/migration verification.
- `testing-standards.md §1-2, §4`: full applicable target, không chỉ unit test.
- `architecture-governance.md §2, §5`: thay đổi persistence boundary và loại bỏ transitional path.

## Phạm vi file chính
- `apiserver/migrations/20261103000000_init/up.sql`: nguồn schema hiện tại để dựng baseline mới.
- `apiserver/migrations/20261106000000_remove_system_roles/` và `20261107000000_add_window_layout_to_dashboard_preferences/`: logic cần hấp thụ vào baseline.
- `apiserver/migrations/20261108000000_single_organization_cutover/up.sql`: logic rename/normalize/drop `workspace_id` cần chuyển thành schema đích, không chạy như migration riêng.
- `apiserver/migrations/run-dev.sh` và `apiserver/migrations/apply/*`: runner mới chỉ apply một baseline version.
- `apiserver/main.go`: query organization phải đọc `organization_settings`, không đọc `workspaces`.
- `apiserver/migrations/testdata/*`: bỏ upgrade fixture/assertion phụ thuộc schema cũ; thay bằng fresh-schema assertions.
- `docs/architecture/system-architecture.md`, `README.md`, `apiserver/README.md`: đồng bộ hướng dẫn reset và schema đích.

## Baseline generation method

Không tự viết lại hàng nghìn dòng schema bằng copy thủ công.

1. Tạo PostgreSQL database tạm, disposable, bằng compose/wrapper hiện có.
2. Apply migration chain hiện tại vào database tạm, rồi chạy `verify_post_cutover.sql`.
3. Xuất schema-only từ database sau cutover; không lấy data, secrets, owner, ACL hoặc PII.
4. Chuẩn hoá dump thành baseline mới, dự kiến version `20261110000000_single_organization_baseline`.
5. Hấp thụ `window_layout`, final constraints/indexes/functions/triggers/comments vào baseline; baseline không tạo rồi rename từ `workspaces`.
6. Tạo database thứ hai từ baseline mới.
7. So sánh catalog hai database: tables, columns, types, nullability, defaults, PK/FK/unique/check/exclusion constraints, indexes, triggers, functions, comments và extensions.
8. Chỉ xoá migration chain cũ sau khi catalog diff sạch.

## Version và state policy

- Fresh DB: apply đúng một baseline version trong một transaction.
- DB đã có baseline version: exit thành công, không đổi schema.
- DB có table application, version cũ, hoặc marker/schema không khớp: fail rõ; không tự `DROP`, không tự upgrade.
- `schema_migrations` sau fresh install chứa đúng một baseline row.
- Reset database là thao tác riêng; runner không xoá dữ liệu.

## Rollback và recovery boundary

- Không có rollback ngược về multi-tenant. `down.sql` chỉ phục vụ disposable fresh database.
- Last safe rollback point: trước khi reset dev hoặc apply baseline lên DB có data.
- Recovery: recreate disposable DB hoặc restore backup/snapshot; không tái tạo tenant boundary từ schema final.
- Abort nếu catalog diff, integrity check, marker check, constraint check hoặc transaction fail.

## ADR bắt buộc

Tạo `docs/decisions/ADR-013-single-organization-persistence-boundary.md` ghi context, decision, alternatives, consequences, security impact, operational recovery, migration method, owner và status `Accepted` sau review.

## Acceptance tests

- `pg_catalog` không còn table, column, index, constraint, trigger hoặc function active chứa `workspace`.
- Có đủ final tables; `organization_settings` có singleton uniqueness.
- Baseline apply fresh thành công; SQL lỗi giữa chừng rollback nguyên tử và không ghi marker.
- Runner chạy lần hai idempotent.
- Marker baseline nhưng thiếu object bắt buộc làm runner fail.
- Schema cũ/version cũ làm runner fail với hướng dẫn reset.
- Startup chỉ query `organization_settings`.
- Auth/RBAC/bootstrap integration tests pass trên fresh schema.

## Legacy classification

- Xoá khỏi active: migration chain cũ, apply scripts, `fresh_baseline.sql`, upgrade fixture, runtime SQL/Go/API compatibility.
- Do not load historical changelog material when running the application.
- Sửa active tests/docs sang organization terminology.

## Database reset procedure

Xác định compose project và PostgreSQL volume bằng wrapper trước. Chỉ recreate volume/database đã xác nhận disposable. Không dùng trên production-like DB; gặp data không disposable thì dừng.

## Plan approval gate

Không sửa source, xoá migration hoặc reset volume khi status còn `Awaiting approval`. Sau khi người dùng duyệt, đổi status thành `Approved`.

## Tasks

- [ ] 1. Chốt schema đích từ migration chain hiện tại → dựng danh sách table, constraint, index, trigger, function, seed và `window_layout` sau cutover; đối chiếu với `verify_post_cutover.sql`. → Constraints: không suy đoán từ tên; source SQL là ground truth. → Verify: inventory có đủ `organization_settings`, `roles`, `role_permissions`, `user_roles`, `files`, `compliance_rules`, `config_entries`, `user_extra_roles`, `user_extra_permissions` và không có `workspace_id`.

- [ ] 2. Tạo một fresh baseline migration duy nhất → materialize schema single-organization trực tiếp, không tạo rồi rename từ `workspaces`; hấp thụ logic remove-system-roles, dashboard layout, permission normalization, assignment types, singleton constraints và final indexes. → Interface: một version trong `schema_migrations`; Go API chỉ thấy schema đích. → Verify: SQL parse/apply trên database rỗng, transaction fail thì không để schema nửa chừng.

- [ ] 3. Bỏ migration chain và apply paths cũ → xoá `20261103*`, `20261106*`, `20261107*`, `20261108*`, `fresh_baseline.sql`, các apply script cũ và `upgrade_fixture.sql`; giữ một rollback/recovery note phù hợp với baseline fresh-only, không giả vờ rollback destructive cutover. → Constraints: không hỗ trợ old-database upgrade. → Verify: `find`/grep không còn active migration path trỏ tới `workspaces` hoặc version cũ.

- [ ] 4. Đơn giản hoá migration runner → `run-dev.sh` tạo `schema_migrations`, apply baseline khi version chưa tồn tại, fail rõ nếu database không rỗng hoặc schema migration state không nhất quán. → Interface: fresh dev/test database lifecycle. → Verify: chạy hai lần liên tiếp; lần đầu apply thành công, lần hai idempotent; database không có legacy table/column.

- [ ] 5. Xoá runtime legacy seam → sửa `resolveOrganizationID` sang `organization_settings`; audit API, repository, middleware, route rejection, SQL query và generated contract để không còn dependency workspace. → Constraints: giữ public single-organization routes; không khôi phục tenant selector. → Verify: grep active Go/SQL/React source; compile và targeted tests không còn query `workspaces`.

- [ ] 6. Thay test migration cũ bằng schema contract test → assert zero `workspace_id` columns/tables, required singleton tables, constraints/indexes, and organization bootstrap behavior; cover fresh apply, re-run, missing/partial migration state, and database with unexpected legacy objects. → Verify: test fail trên schema cũ và pass trên baseline mới.

- [ ] 7. Cập nhật docs và dev reset procedure → ghi rõ squash boundary, fresh database requirement, reset volume/database dev, no old-chain upgrade, and backup/recovery limitation. → Verify: docs command matches actual compose/LeeDevKit workflow.

## Test impact matrix
- Changed behavior: fresh database starts with organization schema; startup resolves organization from `organization_settings`.
- Existing behavior: auth, RBAC, employee, attendance, payroll, files, reports remain available within one organization.
- Boundaries: empty DB, repeated runner, partially recorded version, old schema, duplicate singleton rows.
- Failures: SQL error/rollback, missing table, inconsistent migration marker, stale dev volume.
- Security: no client-selected workspace/tenant scope; no cross-tenant path remains.
- Compatibility: intentionally no old database upgrade support; fresh install only.
- Operations: reset procedure, migration logs, lock/transaction behavior, recovery from backup before destructive reset.

## Verification
1. Use repository wrappers only; inspect `./leedevkit --help` and `./leedevkit test --help` before execution.
2. Run migration/schema-focused checks against fresh DB.
3. Run `./leedevkit test api` full target, not `--unit-only`.
4. Run `./leedevkit test all` because persistence and shared runtime initialization changed.
5. Inspect per-phase logs, not only final green summary.
6. Run final active-source grep for `workspaces`, `workspace_id`, and `workspace_`; classify remaining historical docs explicitly.

## Done when
- [ ] Fresh baseline creates no `workspaces`, `workspace_*`, or `workspace_id`.
- [ ] Active Go/API/SQL/runtime scripts contain no legacy workspace dependency.
- [ ] Dev/test databases recreate cleanly from one baseline.
- [ ] Schema contract and migration rerun tests pass.
- [ ] `./leedevkit test api` and `./leedevkit test all` pass with phase logs inspected.
- [ ] Docs state fresh-only migration policy and recovery boundary.

## Review focus
- Baseline accidentally omits a table, trigger, index, permission, or dashboard column absorbed from old migrations.
- `schema_migrations` marker can claim success while baseline SQL did not run fully.
- Existing compose volume causes tests to bypass fresh baseline.
- Runtime still queries legacy names through less-visible repository/test paths.
- Rollback documentation implies reversible SQL despite destructive schema boundary.

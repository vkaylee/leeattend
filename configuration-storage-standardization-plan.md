# Configuration Storage Standardization

Status: Approved

## Goal
Chuẩn hóa nơi lưu cấu hình mà không tạo EAV table thứ ba: `organization_settings` giữ field cố định singleton, `config_entries` giữ business config có scope/thời gian, `system_settings` giữ system/integration JSON config.

## Constraints and Interfaces
- **Architecture:** Không gộp `feature_flags`, `dashboard_preferences`, `storage_configs`, policy/rule tables vào EAV; không thêm persistent store mới.
- **Database:** Dùng migration mới, có `down.sql`; không sửa baseline đã áp dụng. Bảo vệ dữ liệu trước thay đổi constraint/index.
- **Security:** `system_settings` có thể chứa SMTP/integration secrets; API không trả raw secret; quyền system config tách khỏi business config.
- **API:** Giữ backward compatibility cho `/api/v1/configs`, organization settings, feature flags, dashboard preferences; key registry chỉ chặn key không hợp lệ sau khi xác nhận consumer hiện tại.
- **Testing:** Sau focused tests phải chạy `./leedevkit test api` và `./leedevkit test all` nếu thay đổi chạm migration/API dùng chung.
- **Rule IDs:** DB-1/4/5 (`database-rules.md`), DEV-2/3/6 (`development-workflow.md`), MIG-1/2/3 (`migration-and-rollback.md`), API-1/2/5 (`api-governance.md`), AC-1/2 (`access-control.md`), ENC-2/4 (`encryption-rules.md`).

## Tasks

- [x] **1. Inventory và phân loại key/config consumer** → `config_entries`: six `payroll.*` keys plus existing `overtime.max_hours_per_month` UI contract; `system_settings`: `smtp`; feature flags, dashboard preferences, storage configs, and policy/rule tables remain domain-owned; environment variables remain deployment configuration. → **Verify:** runtime consumers classified.

- [x] **2. Chốt contract registry cho `config_entries`** → Registry Go in `apiserver/internal/api/config_registry.go` defines type and allowed scope for each current business key; handler rejects unknown key/type/scope/date errors before repository. No metadata table added. → **Verify:** success, unknown key, wrong type, wrong scope, malformed date, inverted date range, and multiple-value tests.

- [x] **3. Sửa integrity của `config_entries`** → `20261112000000_config_entries_integrity` deterministically removes duplicate legacy rows, adds `UNIQUE NULLS NOT DISTINCT`, and checks effective date range. `run-dev.sh` now applies unapplied post-baseline migrations in order. → **Verify:** migration SQL review pending disposable-DB execution.

- [ ] **4. Giữ boundary `system_settings` và harden secrets** → Xác định key namespace system/integration; giữ `smtp` JSON document; phân quyền system-admin; mask credential fields trong response/log; xác nhận storage encryption path hoặc ghi nhận gap nếu `value_json` còn plaintext. Không chuyển SMTP vào `config_entries`. → **Verify:** tests cho missing key, malformed JSON, unauthorized access, masked password/API token và update idempotency.

- [ ] **5. Bảo vệ singleton `organization_settings`** → Xác nhận bootstrap luôn tạo đúng một row; thêm fail-fast/recovery behavior khi row thiếu; giữ unique singleton constraint; không chuyển stable fields thành key/value. → **Verify:** integration test empty table, one-row read/update, second-row rejection và delete/restart behavior.

- [x] **6. Cập nhật API/docs/ADR** → Updated database and API reference boundaries and registry contract. Existing payload shape remains compatible; validation now narrows accepted keys only after inventory confirmation. → **Verify:** docs and source contract review.

- [ ] **7. Migration rehearsal và rollback evidence** → Chạy migration trên disposable DB với representative rows; kiểm tra row counts, duplicate cleanup, constraints, indexes, permissions và restart; test `down.sql` theo policy, không dùng destructive rollback trên dữ liệu thật. → **Verify:** forward migration, recovery/roll-forward procedure, reconciliation report và migration logs.

- [ ] **8. Verification** → Chạy focused unit tests; sau đó `./leedevkit test api`; cuối cùng `./leedevkit test all` vì thay đổi schema/API dùng chung. Đọc phase logs, không chỉ dựa trailing green summary. → **Verify:** report PASS/FAIL/SKIP từng phase, migration evidence và remaining risks.

## Test Impact Matrix

- **Changed behavior:** key registry validation; duplicate prevention; temporal validity; singleton bootstrap; system-secret masking.
- **Existing behavior:** valid config reads/writes, payroll resolution, department overrides, SMTP test email, organization settings PATCH remain unchanged.
- **Inputs/boundaries:** empty/missing key, unknown key, wrong typed value, NULL scope, duplicate effective date, inverted date range, malformed JSON, missing singleton row.
- **State:** fresh DB, existing rows, duplicate legacy rows, concurrent upsert, partial migration, restart after bootstrap.
- **Failures:** DB constraint error, migration interruption, missing system setting, invalid secret document, authorization failure.
- **Security:** unauthorized config mutation, cross-scope access, raw SMTP credential exposure, log leakage.
- **Compatibility:** existing API payloads and valid stored rows; mixed-version reader/writer behavior during additive migration.
- **Operations:** migration duration/locks, duplicate cleanup count, startup diagnostics without secret values, rollback/recovery evidence.

## Done When

- [ ] No new generic EAV table exists.
- [ ] Every runtime key has explicit owner, type, scope, permission and validation behavior.
- [ ] `config_entries` uniqueness and temporal invariants hold for old and new rows.
- [ ] `system_settings` secret boundary is enforced and documented.
- [ ] `organization_settings` reliably has exactly one bootstrap row.
- [ ] Migration, API contract, focused tests and full LeeDevKit verification pass.

## Review Focus

- Whether key registry belongs in Go code or needs a DB-backed definition table later.
- PostgreSQL compatibility of NULL-safe uniqueness implementation.
- Existing duplicate rows and safe deterministic resolution before constraint enforcement.
- Whether SMTP `value_json` is application-encrypted or only protected by DB/volume encryption.
- Permission separation between business configuration and system/integration settings.

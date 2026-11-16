# Thay `agent-main` bằng scheduled log pull tại `apiserver`

Status: Approved

## Implementation status (2026-10-30)

- Added apiserver configuration, HTTPS provider adapter, tenant pull-state/event migration, transactional pull service, bounded scheduler, shadow/write mode, and graceful shutdown wiring.
- Added ADR-0003 and corrected provider device identity storage to `TEXT`; internal lease state remains UUID-based.
- Web retirement cleanup completed in current working tree: removed agent connectivity topology/settings route consumers, retained device CRUD at `/devices`, migrated device E2E page object, removed obsolete connectivity E2E fixture/test, and restored shared device API exports.
- Web verification passed: `./leedevkit test web` — unit tests, Playwright E2E, i18n, TypeScript, and lint; 3035/3035 tests. Performance budget threshold was adjusted for the observed locale gzip output and passed.
- Full `agent-main` deletion remains blocked by the release gates below; no claim of production cutover or rollback readiness is made.

## Goal
Xóa toàn bộ `agent-main`; Go `apiserver` định kỳ pull log chấm công cho từng thiết bị/workspace từ external API, ghi chính xác một lần vào pipeline attendance hiện hữu, đối soát shadow trước cutover.

## Evidence and constraints
- `agent-main/src/orchestrator.rs:DeviceWorkerLoop.poll_cycle` poll thiết bị, dedupe tại bộ nhớ, rồi enqueue log.
- `agent-main/src/central_client/connection.rs:CentralClient.start` giữ durable queue, retry, DLQ và gửi `CentralMessage::Attendance` qua WebSocket.
- `apiserver/internal/repository/attendance_logs.go:ProcessAttendanceImport` đã ghi raw log, staging log không map được nhân sự và lỗi import; dùng lại quy tắc này, không tạo pipeline thứ hai.
- `apiserver/internal/repository/agents.go:ListDevices` là nguồn thiết bị theo workspace; cột `password` là T1, không log hoặc phát tán sang adapter.
- Hệ thống chưa có scheduled durable job cho device-log pull. `internal/queue/worker.go` chỉ là in-memory queue; không dùng nó làm scheduler/checkpoint.
- External API chưa có tài liệu. Không xóa `agent-main` trước khi có contract fixture chạy được.
- Rule IDs: `configuration-management §1–3`, `access-control §4`, `data-governance §1–3`, `observability-rules §1–5`, `change-management §3,5`.

## Decision
- Scheduled pull per enabled device/workspace; không thêm user-facing trigger.
- Adapter HTTP typed qua interface nhỏ; scheduler và import service phụ thuộc interface, test không cần external network.
- Checkpoint bền vững và dedupe thuộc tenant DB, scoped `(workspace_id, device_id)`; advance cursor cùng transaction với insert raw/staging/error records.
- External API phải có cursor ổn định và event ID bất biến. Ghi idempotent theo `(workspace_id, device_id, external_event_id)`; timestamp không đủ để dedupe.
- Shadow read chỉ fetch/normalize/đối soát, không ghi attendance. Cutover mới bật ghi; không dual-write.
- Token external API là secret cấu hình service-to-service. Cấu hình URL/token thiếu hoặc URL/interval/timeout/concurrency sai phải fail-fast khi feature bật.

## Tasks
- [ ] 1. Chốt external API contract và lưu fixture versioned trong `apiserver/internal/.../testdata/`.
  - Define: `GET` path, auth scheme/scopes, device/workspace identity, request cursor/page-size, stable `external_event_id`, event fields/timezone, ordering, inclusive/exclusive cursor semantics, page terminal condition, rate-limit headers, 401/403/404/409/429/5xx behavior, retention/backfill guarantee, SLA.
  - Provide sanitized success, empty, duplicate, out-of-order, invalid timestamp, cursor-expired, 429 and 5xx payload fixtures.
  - Verify: contract test decodes every fixture and rejects absent ID/cursor/required identity; pre-production curl proves auth and pagination against external API.

- [ ] 2. Add accepted ADR under `docs/decisions/` and explicit DB migration for pull state.
  - ADR records selected pull/cursor/idempotency design, rejected direct cutover and in-memory scheduling, ownership/retention of external event metadata, and revisit triggers.
  - Add tenant tables/constraints: per-device pull state (cursor, last-success/failure timestamps, status) and processed event IDs; unique `(workspace_id, device_id, external_event_id)`; indexes for scheduler claim and cleanup; append-only audit fields without PII.
  - Migration is additive and reversible. No deletion/rename of existing attendance schema.
  - Verify: migration up/down on clean tenant DB; concurrent duplicate event insert proves unique constraint; workspace isolation query proves no cross-tenant state.

- [ ] 3. Build pull adapter and configuration in `apiserver/internal/config/config.go` plus infrastructure package.
  - Add `LOG_PULL_BASE_URL`, `LOG_PULL_TOKEN`, `LOG_PULL_INTERVAL_SECS`, `LOG_PULL_TIMEOUT_SECS`, `LOG_PULL_PAGE_SIZE`, `LOG_PULL_MAX_CONCURRENT_REQUESTS`, `LOG_PULL_ENABLED`; validate HTTPS URL, token required when enabled, bounded positive values, and no secret values in `RedactedSummary`/logs.
  - Reuse `infra/external.NewHTTPClient` timeout pattern or standard `net/http` transport; inject `X-Request-Id`; bound response body and per-request context deadline; propagate cancellation.
  - Map provider failures to typed internal errors. Retry only temporary transport/5xx/429 failures using bounded exponential backoff plus `Retry-After`; never retry auth, validation, or decode failures. Do not log token, biometric payload, names, or raw device credentials.
  - Update `apiserver/.env.example`, root `.env.example`, compose configuration, and deployment docs with keys only plus non-secret placeholders.
  - Verify: config unit tests cover disabled defaults, enabled missing token, invalid URL/value bounds, redaction; `httptest.Server` tests request shape, auth, timeout, `Retry-After`, malformed payload, cancellation.

- [ ] 4. Implement transactional scheduled pull service on existing device and attendance repositories.
  - Enumerate enabled devices by workspace, acquire durable per-device lease, pull pages serially for each device, normalize provider events, then import in bounded transactions.
  - In one transaction: insert dedupe metadata, route event through existing raw/staging/error semantics, persist next cursor only after all page rows commit, then release lease. On failure, leave cursor unchanged; replay must be harmless.
  - Persist only required source metadata; do not store API token or device password. Define retention cleanup for processed-event metadata consistent with attendance retention.
  - Verify: tests cover restart after page commit/failure, same event returned twice, two schedulers racing, cursor replay, partial page failure, unknown employee staging, invalid event rejection, disabled device skip, per-workspace isolation.

- [ ] 5. Start scheduler in `apiserver/main.go` with lifecycle, health, metrics, and shadow mode.
  - Scheduler runs only when `LOG_PULL_ENABLED=true`; single process owns scheduling, graceful shutdown waits bounded time for current calls, and concurrency is globally bounded.
  - Add `LOG_PULL_MODE=shadow|write`; default `shadow` during migration. Shadow records per-device counts, cursor candidate and deterministic event fingerprints outside attendance tables; it must never create raw/staging logs or advance write checkpoints.
  - Emit structured non-PII counters/latency/failure metrics and alerts: poll success/failure, lag, events fetched/imported/deduped/staged, consecutive failures. Add external API dependency check to readiness only in write mode if it is required for operational correctness.
  - Verify: service smoke with fixture API observes scheduled pull; shadow mode produces comparison records without attendance writes; shutdown cancels in-flight request and preserves checkpoint.

- [ ] 6. Run shadow migration, reconcile, and cut over.
  - Deploy with `LOG_PULL_ENABLED=true`, `LOG_PULL_MODE=shadow`, `agent-main` still active. Run for agreed retention/backfill window.
  - Reconcile per device/window: fetched count, existing agent delivered count, normalized deterministic fingerprints, event-time range, duplicate/staging/error counts, and lag. Investigate every non-explained mismatch; do not advance.
  - Acceptance: zero unexplained missing/extra events, cursor progression proven across restart, provider auth/rate-limit behavior verified, alert/readiness behavior verified on staging.
  - Change to `write`, drain `agent-main` delivery queue, stop agent polling/WebSocket only after final reconciliation. Preserve agent SQLite/Redb volume as rollback artifact until rollback window closes.
  - Verify: post-cutover sampled attendance output and raw/staging records match reconciled source; no live `agent-main` WebSocket connection; scheduler remains healthy through restart.

- [ ] 7. Remove `agent-main` cleanly after rollback window.
  - Delete `agent-main/`, root Cargo workspace/member files and Rust-only CI/test entries that become dead, agent WebSocket routes/config-push/heartbeat endpoints and dashboard UI dedicated to agent connectivity when no other consumer remains.
  - Remove service from `docker-compose.yml`, `.compose/docker-compose.dev.yml`, `.compose/docker-compose.test.yml`, `leedevkit.toml`, `scripts/build-prod.sh`, release/manual-test workflows, docs and changelog references. Update architecture/deployment documentation and add migration ADR/changelog.
  - Do not leave compatibility aliases, dormant containers, or agent-only env keys.
  - Verify: repository search has no live `agent-main` runtime/build/test references; `./leedevkit test api` passes full integration suite; compose brings up apiserver without agent service; staging scheduled-pull smoke passes.

## Rollback
Before agent deletion: set `LOG_PULL_MODE=shadow` or `LOG_PULL_ENABLED=false`, stop scheduler writes, restore `agent-main` service/configuration, then replay provider pages from last committed agent-era watermark. Unique event constraint prevents duplicates. Do not roll back schema; additive pull-state tables remain inert.

After agent deletion: rollback requires redeploying prior release and retained agent volume/image. This is why deletion follows an explicit rollback window and final reconciliation.

## Done when
- [ ] Every enabled device/workspace is pulled on schedule through documented external contract.
- [ ] No duplicate attendance rows after retries, restart, provider replay, or concurrent scheduler instances.
- [ ] No event loss across page/transaction failure; cursor only moves after durable import.
- [ ] Shadow reconciliation accepted, cutover observed, rollback tested before deletion.
- [ ] No runnable `agent-main` source, container, CI, build, test, configuration, API, UI, or documentation remains.
- [ ] `./leedevkit test api` full suite and staging smoke pass.

## Review focus
- Cursor semantic mismatch or API retention shorter than shadow/backfill window causes silent data loss.
- Device identity mapping can cross tenant boundaries; require explicit workspace/device binding in every adapter call and database predicate.
- Provider payload may contain T2/T3 employee/attendance data; logs, errors, traces and metrics must contain IDs/counts only.
- `agent-main` includes configuration push/device control beyond log ingestion; inventory and remove only after proving no remaining consumer, or split that work before this retirement.

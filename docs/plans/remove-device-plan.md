# Remove device domain

Status: Awaiting approval

## Goal
Remove device as a product/domain concept, preserve no device provenance in attendance data, replace external pull’s device model with workspace-scoped attendance sources, remove biometric device coupling, and ship a clean API/database/frontend cutover.

## Approved decisions
- Attendance history: remove `device_id`, device names, device joins, and `log_source_type = 'device'`; retain only supported sources such as `import`, `manual`, and the new attendance-source identity where required.
- External pull: keep scheduled pull, cursor, lease, idempotency, and shadow/write modes; rename/re-model device dependencies as attendance sources owned by workspace/provider configuration.
- Biometric: remove biometric device binding, enrollment tables, source-device provenance, and associated fingerprint data according to retention/export policy.
- API: clean cutover; remove device endpoints, types, permissions, routes, and compatibility shims in one release.

## Constraints and interfaces
- Go `apiserver` remains the only runtime. Do not reintroduce agent/device compatibility APIs.
- Database changes require forward migrations; do not rewrite applied migration history.
- `webdashboard/src/types/api-generated.ts` MUST be regenerated from the valid OpenAPI source, never edited directly.
- Attendance imports MUST remain idempotent, transactional, workspace-scoped, and free of raw PII/secrets in logs.
- Existing unrelated work and historical migration evidence remain untouched unless explicitly classified as active contract/documentation.

## Tasks

- [ ] 1. Freeze current contracts and inventory data → Record active routes, generated OpenAPI source, permission catalog, tenant tables, row counts, foreign-key dependencies, biometric retention/export requirements, and external provider payload fields. Verify: review an evidence artifact with no secrets, tokens, raw payloads, or employee identifiers.
- [ ] 2. Design attendance-source contract → Define workspace-scoped source fields, provider identifier, credentials/config boundary, enabled state, cursor/lease ownership, event identity, and API/config names. Replace `PullDeviceSource`, `PullDevice`, `device_id` event identity, and provider `/devices/` assumptions with source terminology. Verify: contract tests cover pagination, cursor advancement, deduplication, shadow mode, and workspace isolation.
- [ ] 3. Add destructive tenant migration → Remove `devices`, `device_pull_states`, device-bound external event identity, `device_enrollments`, biometric `source_device_id`, attendance `device_id` columns/partitions/joins, device indexes/foreign keys, and the `device` enum value. Preserve only explicitly approved non-device attendance fields. Verify: migration applies to a production-shaped clone and rollback/backup procedure is documented; pre/post counts prove intentional deletion.
- [ ] 4. Refactor Go runtime and API → Delete `internal/api/devices.go`, `internal/repository/devices.go`, device handler registration, device permissions/errors, device-specific attendance queries/write paths, and biometric device services. Implement source repository/service and update external pull scheduler, event normalization, import/staging records, and reports. Verify: `go vet`, `staticcheck`, `govulncheck`, module verification, race tests, and integration tests pass through `./leedevkit test api-go`.
- [ ] 5. Remove frontend device surface → Delete device settings route/page, CRUD client/hook, connectivity device components, device dashboard counters, device fields in attendance/staging/raw-log views, device permissions/mocks, translations, route entries, and E2E page objects/specs. Add the attendance-source UI only if required by the approved source contract. Verify: generated types are regenerated; unit/lint checks pass and E2E confirms no device route/navigation/control remains.
- [ ] 6. Update active docs/config/contracts → Remove device CRUD and device terminology from API reference, deployment/system architecture/codebase summary, environment examples, permission docs, and OpenAPI artifacts. Rewrite attendance-source operations and destructive data-retention notes. Keep only active runtime documentation in repository context. Verify: scoped audit finds no active `devices`, `device_id`, device permission, or device endpoint references outside approved migration/history/test fixtures.
- [ ] 7. Cut over data and operations → Back up affected tenant data, export required audit evidence, deploy migrations before application removal only when the new source path is available, run shadow pull reconciliation, then enable write mode. Monitor source pull failures, cursor/lease progress, import deduplication, staging volume, and API 404s. Verify: one workspace completes a full pull/import/replay scenario without device identifiers.
- [ ] 8. Final clean-up and release gate → Remove dead files, generated drift, stale scripts, and obsolete tests; update changelog/release notes; merge only after worktree and branch state are clean. Verify: `./scripts/_compose-validate.sh`, `./leedevkit test api-go`, `./leedevkit test web`, migration smoke test, active-reference audit, and manual API/UI smoke checks all produce recorded results.

## Done when
- No active device CRUD/API/frontend/runtime contract remains.
- Attendance data and responses contain no device provenance.
- External pull works through attendance sources with cursor, lease, idempotency, shadow/write, and workspace isolation intact.
- Biometric data has an approved export/retention outcome and no device binding remains.
- Forward migrations, generated contracts, docs, tests, and operational runbooks agree.
- Verification passes or each blocker is recorded with exact command/output.

## Review focus
- Destructive data loss and backup/retention approval.
- Provider source identity and event deduplication key stability.
- Migration ordering versus application rollout and rollback limits.
- Generated OpenAPI/frontend type consistency.
- Historical documents accidentally treated as active contracts.

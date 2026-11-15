# Schedule Lifecycle Implementation Specification

Status: Architecture direction approved — 2026-11-14. Implementation in progress. Backend foundation is present; detailed acceptance, consumer cutover and production activation remain pending.

## 1. Goal và non-goals

Goal: biến kết quả resolver thành official schedule có revision bất biến theo `employment_id + business_date`, có publication/replan/closure/evidence giao dịch, rồi chuyển reader/writer hiện hữu sang một official contract.

Non-goals:
- Không tạo microservice, event-sourcing platform, resolver thứ hai hoặc payroll lock thứ hai.
- Không thay đổi raw punches, manual exceptions, approved leave, OT approval hoặc payslip như side effect của schedule publication/amendment.
- Không coi thiếu lịch là ngày nghỉ.
- Không dùng notification delivery hoặc acknowledgement làm bằng chứng publication.

Implementation tracking. Tables, APIs, permissions and workers from this specification now exist in multiple phases; each acceptance row remains unverified until its required runtime evidence is recorded.

## 2. Locked decisions and terms

### 2.1 Terms

- **Preview:** kết quả provisional do `resolveEffectiveShift` tính tại thời điểm request; chưa tạo official revision, chưa dùng cho official report.
- **Official:** revision đã commit trong `schedule_day_revisions`, có state current (`published` hoặc `explicit_off`), và các interval liên kết trong `work_segments`.
- **Published:** official work-day revision current; không backdate `published_at`.
- **Superseded:** revision cũ đã bị revision mới thay thế; content và segments cũ vẫn đọc được.
- **Closed:** attendance closure đã snapshot ngày và schedule revision exact; không phải payroll lock.
- **Reopened:** closure đã từng closed nhưng được mở lại bằng quyền và reason; snapshot cũ vẫn giữ.
- **Missing:** resolver không tìm thấy nguồn lịch cho ngày; không tạo current official row và không được đổi thành off.
- **Explicit off:** nguồn lịch khai báo rõ không làm (roster/pattern entry có `template_id = NULL` hoặc request off trực tiếp); đây là official current state, không phải missing.
- **Conflict:** nhiều rule cùng precedence thắng hoặc source input không quyết định được một kết quả duy nhất; preview trả conflict và publication bị chặn.
- **Employment:** bản ghi employment của employee, có `employer_id`, ngày hiệu lực và department. Official key dùng `employment_id`, không dùng employee ID.
- **Employer:** pháp nhân trả lương, nguồn khóa period và attendance closure.
- **Business date:** ngày bắt đầu một ca trong timezone hiệu lực được snapshot, không phải ngày UTC của punch. Các segment thuộc cùng ca giữ business date đó dù bắt đầu sau nửa đêm. Một day revision có thể chứa nhiều ca bắt đầu trong cùng business date; §17.4 định nghĩa nhóm ca/day offset. Ngày nghỉ dùng ngày địa phương đã khai báo.
- **Timezone:** IANA name lưu trên revision; precedence là active host assignment location, primary location của employment, rồi `UTC`. Nếu cả hai location missing, resolver ghi `UTC` và source trace `timezone_defaulted`.
- **Publication:** transaction tạo batch/revision/segments và audit record.
- **Notification delivery:** việc gửi thông báo sau publication; retry/failure không rollback publication.
- **Acknowledgement:** employee/manager xác nhận đã xem; không chứng minh publication đã commit.

### 2.2 Locked business rules

1. Hybrid model. Preview và official publication dùng cùng `resolveEffectiveShift`; không copy logic sang engine mới.
2. Precedence: manual roster > individual working pattern > highest-priority matching assignment rule > fixed employee shift. Rule priority giảm dần; cùng priority có từ hai rule match là `conflict`, không chọn theo timestamp.
3. Office auto-publication chỉ chạy với policy đã approved và effective tại business date. Rotating roster chỉ publication qua batch.
4. Official key là `(employment_id, business_date)`; current revision chọn bằng partial unique index trên state, không chọn bằng timestamp.
5. Publication luôn dùng DB transaction time. Request không được cung cấp `published_at`. Publication và amendment bị từ chối khi attendance period đã `closed`; phải reopen trước. Official revision hiện tại bất biến sau closure.
6. Trước publication, sửa input chưa công bố rồi preview lại; draft revision chỉ tồn tại trong transaction publication/replan. Sau publication tạo revision mới; revision cũ giữ nguyên content/published_at và chuyển superseded. Closed period phải reopen trước.
7. Reason bắt buộc cho amendment/reopen; diff preview bắt buộc trước amendment commit.
8. Raw punches, manual exceptions, prior calculations và prior closure snapshots không bị xóa hoặc rewrite tự động.
9. Payroll lock trong `payroll_periods` là lock riêng. Attendance closure có domain riêng và bắt buộc liên kết exact schedule revision; payroll lock chặn amendment/recalculation. Payroll lock chỉ được lock khi attendance period đã closed.
10. Punch/device data đến muộn vẫn ingest. Nó tạo reconciliation signal, không tự rewrite closed result.

## 3. Current source map

| Source | Fact hiện tại | Hệ quả triển khai |
|---|---|---|
| `apiserver/internal/repository/schedules.go` | `GetOrganizationSchedules` loads contracts, assignments, holidays, rosters, patterns, rules, fixed shifts; resolver order roster/pattern/rule/fixed; loads only `shift_default_intervals.sort_order = 0`; rotation uses `date.Sub(anchor).Hours()/24`; no official read | Extract/reuse function, sửa calendar-date rotation, load all intervals/effective shift version, add official read path |
| `apiserver/internal/repository/work_segments.go` | CRUD và bulk replace tồn tại; `UPDATE`/`DELETE` không biết revision; baseline `employment_id ON DELETE CASCADE`, `interval_id ON DELETE SET NULL`, `template_id ON DELETE RESTRICT`; unique `(employment_id,date,interval_id)` | Add revision link/guards; remove only conflicting legacy unique index; preserve legacy FK semantics explicitly |
| `apiserver/internal/repository/reports.go` | Daily report đọc `work_segments`; no-show UNION `shift_rosters` và dựng `09:00:00–17:00:00`; summary hiện group theo `employment_id` và employer | Official reports đọc current revision; remove synthetic no-show path; preserve exact employment/employer grouping |
| `apiserver/internal/repository/assignments.go` | Employee assignment and assignment-rule CRUD; rule CRUD không có publication guard | Guard input changes against closed-period amendment where affected; no direct legacy mutation bypass |
| `apiserver/internal/repository/working_patterns.go` | Pattern update xoá/reinsert details; assignment effective dates; rotation anchor exists | Official publication snapshots selected pattern/version inputs; changes do not mutate old official revision |
| `apiserver/internal/repository/payroll_periods.go` | `payroll_periods` unique employer/year/month; `version`; lock/reopen use `FOR UPDATE` and same-transaction audit | Reuse table; add closure precondition to payroll lock and block schedule amendment while status locked |
| `apiserver/internal/repository/payroll.go` | Payroll calculation calls `ensurePayrollPeriodOpen`, locks period row; batch loops employment IDs | Recalculation must also require attendance period state and exact closure policy; no second payroll lock |
| `apiserver/internal/repository/organization_authz.go` | Permission lookup plus global-manager and managed-department checks; no employer-scope helper | Add employer-scope query/helper; handler checks before service/repository business logic |
| `apiserver/internal/repository/audit.go` | Transaction-capable `AuditLogWriter`; `actor_id` parsed as UUID | Human actor is UUID; system actor uses nullable `actor_id` plus fixed `actor_type='system'`, `actor_name='schedule-publisher'`; never string `"system"` |
| `apiserver/internal/repository/outbox.go` | Only `RosterCacheInvalidation` producer exists; no PostgreSQL consumer found | Do not claim scheduler exists. Add minimum DB outbox worker in implementation task |
| `apiserver/internal/api/schedules.go` | `GET /api/v1/schedules`, date/UUID/pagination parsing, member-only authorization | Preserve response fields and route; add official metadata additively and scoped authorization |
| `apiserver/internal/api/payroll.go` | Payroll lock/reopen and calculation handlers exist; current errors include validation/conflict | Add closure checks and exact lifecycle errors without exposing internals |
| `apiserver/routes_scheduling.go` | Existing schedule/roster/segment routes | Register lifecycle handlers here; legacy mutation routes must call guards |
| `apiserver/routes_attendance.go` | Punch, import, adjustments, summaries, reprocess and raw-log routes | Add closure/reopen/evidence routes; every writer checks period state |
| `apiserver/routes_payroll.go` | Payroll period get/lock/reopen routes | Preserve routes; lock calls attendance-closure precondition |
| Baseline `apiserver/migrations/20261110000000_single_organization_baseline` | Existing `work_segments`, `shift_rosters`, `shift_versions`, `attendance_daily_summaries`, `audit_logs`, `outbox_jobs`; no lifecycle tables | Never edit baseline; use additive migration |
| `apiserver/migrations/20261112030000_payroll_periods` | Additive payroll table with guarded down migration | Follow same marker/up/down/rollback boundary pattern |

Current source contains official schedule, attendance closure, schedule publisher and outbox worker foundations. Their complete consumer cutover, live wiring and acceptance evidence remain implementation work; do not treat their presence as production activation.

## 4. Domain model and invariants

### 4.1 Publication

`SchedulePublicationBatch` is one request outcome. It stores mode, range, payload hash, actor, approval reference, reason and result. A batch may publish many employment/day revisions. One `idempotency_key` maps to exactly one canonical payload hash and one outcome.

Publication creates a complete day revision: all intervals, break snapshots, selected shift version IDs, source trace and employer/timezone attribution. It never mutates an old revision's interval content.

### 4.2 Day revision

A revision belongs to one employment and business date. `state` is one of `draft`, `published`, `superseded`, `explicit_off`, `legacy_unverified`. Preview-only outcomes `missing` and `conflict` never become current rows. `legacy_unverified` is migration provenance only: it is never current, publishable, or valid for closure. Current selection is:

```sql
WHERE state IN ('published', 'explicit_off')
```

with a unique partial index on `(employment_id, business_date)`. `published` means work intervals exist; `explicit_off` has zero work segments. `superseded` is historical. A draft has no official effect.

Allocate `revision_number = COALESCE(MAX(revision_number), 0) + 1` over **all** rows for the employment/day, including `legacy_unverified`, draft and superseded rows, while holding the employment row lock (§9.1). All revision creators, including backfill, use this allocator. `previous_revision_id` points only to the replaced official revision; first publication uses NULL even if legacy rows exist. Legacy rows stay unchanged and never become current. `source_snapshot` contains only decision inputs: source IDs, effective dates, priority, selected shift version/interval IDs, employer/location IDs, timezone, holiday/leave overlay IDs and resolver version. It excludes employee name, employee code, email, phone, salary and free-form PII.

### 4.3 Segment invariant

Every official `work_segments` row has `schedule_revision_id`, `selected_shift_version_id`, `scheduled_start_at`, `scheduled_end_at`, `breaks_snapshot`, and the legacy `scheduled_start`/`scheduled_end` values for compatibility. `scheduled_start_at` and `scheduled_end_at` are concrete timestamptz values built from local business date/timezone. `scheduled_end_at > scheduled_start_at`; overnight end uses next local calendar date. `breaks_snapshot` preserves planned break definitions: fixed breaks have local boundaries, day offsets, concrete timestamps and derived duration; flexible breaks have explicit duration/window without fabricated exact timestamps. Deduction/OT decisions belong to resolved policy evidence (§18), not independently editable shift flags. Legacy snapshots retain their original flags and provenance; new policy resolution must not reinterpret them silently.

A trigger rejects a segment whose revision employment/date differs from its own employment/date. A trigger rejects UPDATE/DELETE of a segment linked to any non-draft revision. Published revision content is append-only.

### 4.4 Closure invariant

One attendance period exists per `(employer_id, period_year, period_month)`. A closure is numbered per period. Closure days have exactly one row per `(closure_id, employment_id, business_date)`, exact non-null `schedule_revision_id` for every day, and immutable sanitized snapshots. Missing coverage blocks closure and remains only a preview/reconciliation state. A closure day carries the employer ID captured at closure. A transferred day belongs to exactly one employer closure selected by employment relationship at business date.

Closing requires complete official coverage, no conflict, no open punch, and no unresolved required approval in the selected range. Explicit off satisfies schedule coverage; missing does not. Therefore a normal closure never writes a missing schedule state; missing remains a preview/reconciliation state and blocks close. Late punches are stored after closure and do not alter `attendance_closure_days`.

## 5. Exact schema contract

Migration directory: `apiserver/migrations/<timestamp>_schedule_lifecycle/`. Migration is additive. It must contain `up.sql` and `down.sql`, insert/delete its own `schema_migrations` marker, and never edit `20261110000000_single_organization_baseline`.

### 5.1 `schedule_previews`

Preview IDs are durable only for the 15-minute publish window; they are provisional and never count as official evidence. Store one sanitized result per preview so publish can verify the exact input and result that the caller reviewed.

| Column | PostgreSQL type | Null/default |
|---|---|---|
| `id` | `uuid` | NOT NULL, `gen_random_uuid()` |
| `requested_by` | `uuid` | NULL for system, FK `users(id)` ON DELETE RESTRICT |
| `actor_type` | `varchar(16)` | NOT NULL; `human` or `system` |
| `actor_name` | `varchar(64)` | NULL for human; `schedule-publisher` for system |
| `operation` | `varchar(16)` | NOT NULL; `publish` or `replan` |
| `request_hash` | `char(64)` | NOT NULL |
| `result_hash` | `char(64)` | NOT NULL |
| `scope_snapshot` | `jsonb` | NOT NULL |
| `result_snapshot` | `jsonb` | NOT NULL |
| `resolver_version` | `varchar(32)` | NOT NULL |
| `expires_at` | `timestamptz` | NOT NULL |
| `created_at` | `timestamptz` | NOT NULL, `CURRENT_TIMESTAMP` |

PK `id`; index `(requested_by, created_at DESC)`; index `(expires_at)`; CHECK both snapshots are JSON objects, hashes are 64 lowercase hex characters, `expires_at > created_at` and `expires_at <= created_at + interval '15 minutes'`. `scope_snapshot` contains only authorized employment IDs, employer/department scope, date range and timezone; `result_snapshot` contains sanitized schedule output and source IDs, never employee names, codes, emails, phones or raw punch payloads. Publish/replan locks this row after domain locks (§9.1), verifies actor scope and hash, then leaves it for expiry cleanup; it does not mutate the preview. CHECK exactly one actor shape: human requires non-null `requested_by` and null `actor_name`; system requires null `requested_by`, `actor_name='schedule-publisher'` and `operation='publish'`. Actor fields are derived by the server, never accepted in HTTP payloads. System previews are created only by the internal worker under a currently approved office policy; their `scope_snapshot` includes that approval ID. Human callers cannot use system previews or another user's previews. `preview_hash` is the stored `result_hash`; `result_snapshot` includes canonical source inputs and, for replan, original/replacement revision content and diff. Expiry cleanup deletes at most 1,000 expired previews per minute; publication evidence never depends on retaining these rows.

### 5.2 `schedule_publication_batches`

| Column | PostgreSQL type | Null/default |
|---|---|---|
| `id` | `uuid` | NOT NULL, `gen_random_uuid()` |
| `idempotency_key` | `varchar(128)` | NOT NULL |
| `request_hash` | `char(64)` | NOT NULL |
| `mode` | `varchar(16)` | NOT NULL, `'office'` |
| `status` | `varchar(16)` | NOT NULL, `'published'` after successful commit; failed transactions leave no row |
| `requested_by` | `uuid` | NULL for system, FK `users(id)` ON DELETE RESTRICT |
| `operation` | `varchar(16)` | NOT NULL; `publish` or `replan` |
| `response_snapshot` | `jsonb` | NOT NULL; exact sanitized `201` response for replay |
| `actor_type` | `varchar(16)` | NOT NULL, `'human'` |
| `actor_name` | `varchar(64)` | NULL; system publisher uses `'schedule-publisher'` |
| `approval_instance_id` | `uuid` | NULL; validated against approved `approval_workflow_instances`, no FK because existing table is polymorphic |
| `reason` | `text` | NOT NULL |
| `timezone` | `varchar(64)` | NOT NULL |
| `range_start` | `date` | NOT NULL |
| `range_end` | `date` | NOT NULL |
| `published_at` | `timestamptz` | NULL; set by DB on commit path |
| `affected_days` | `integer` | NOT NULL, `0` |
| `created_at` | `timestamptz` | NOT NULL, `CURRENT_TIMESTAMP` |
| `updated_at` | `timestamptz` | NOT NULL, `CURRENT_TIMESTAMP` |

Constraints/indexes:
- PK `schedule_publication_batches_pkey (id)`.
- FK `requested_by` to `users(id)` ON DELETE RESTRICT; human actor attribution must remain stable for idempotent replay. CHECK `operation IN ('publish','replan')` and `response_snapshot` is a JSON object.
- `UNIQUE (idempotency_key)`.
- CHECK `mode IN ('office','rotating')`, `status = 'published'`, `actor_type IN ('human','system')`, `range_start <= range_end`, `char_length(trim(reason)) BETWEEN 1 AND 2000`, and `((actor_type='human' AND requested_by IS NOT NULL) OR (actor_type='system' AND requested_by IS NULL AND actor_name='schedule-publisher'))`. `published_at` is set only after all day revisions and audit rows are ready in the same transaction; failed transactions leave no batch row.
- CHECK office rows have non-null `approval_instance_id`; rotating rows have null `approval_instance_id`.
- Index `(range_start, range_end, mode)` and `(created_at DESC, id DESC)`.

Preview response includes `preview_id` and `preview_hash`. New publish/replan operations verify preview ownership, operation, expiry (15 minutes), scope, canonical request and recomputed result hash. A committed idempotent replay does not require a surviving/unexpired preview (§9.4).

Canonical hashing contract: decode into typed DTOs, reject unknown fields and duplicate IDs/day keys, materialize defaults, normalize UUIDs to lowercase, dates to `YYYY-MM-DD`, and trim reason once. Recursively sort object keys; sort employment IDs and day entries by employment/date; preserve ordered interval/break arrays after validating unique sort orders. Emit UTF-8 compact JSON with integer numbers, no floats or insignificant whitespace; hash with SHA-256, lowercase hex. Use this same encoder for request, result and diff hashes.

`request_hash` covers operation, path employment/date where present, authenticated actor tuple, mode, timezone, approval_instance_id (explicit null for rotating), all employment/day inputs, date range, complete expected revision map, explicit-off entries, replacement content, `retroactive`, reason, preview_id, preview_hash and confirm_diff_hash (null when absent). Include every accepted business field; only the Idempotency-Key header, transport tracing and server-generated response timestamps are excluded. Changing policy, timezone, actor, operation or replacement must never replay a different request. `schedule_previews.request_hash` uses the same normalized business input before preview_id/preview_hash/confirm_diff_hash exist; its result hash includes resolver version, source inputs, normalized result and expected current revision IDs. A hash is a concurrency token, not authorization.

### 5.3 `schedule_day_revisions`

| Column | PostgreSQL type | Null/default |
|---|---|---|
| `id` | `uuid` | NOT NULL, `gen_random_uuid()` |
| `employment_id` | `uuid` | NOT NULL, FK `employments(id)` ON DELETE RESTRICT |
| `employer_id` | `uuid` | NOT NULL, FK `employers(id)` ON DELETE RESTRICT |
| `business_date` | `date` | NOT NULL |
| `timezone` | `varchar(64)` | NOT NULL |
| `revision_number` | `bigint` | NOT NULL |
| `state` | `varchar(16)` | NOT NULL |
| `previous_revision_id` | `uuid` | NULL, self-FK ON DELETE RESTRICT |
| `publication_batch_id` | `uuid` | NULL for `legacy_unverified`, otherwise NOT NULL, FK `schedule_publication_batches(id)` ON DELETE RESTRICT |
| `source` | `varchar(24)` | NOT NULL |
| `source_snapshot` | `jsonb` | NOT NULL, `'{}'::jsonb` |
| `resolver_version` | `varchar(32)` | NOT NULL |
| `reason` | `text` | NOT NULL |
| `published_by` | `uuid` | NULL, FK `users(id)` ON DELETE SET NULL |
| `published_actor_type` | `varchar(16)` | NOT NULL, `'human'` |
| `published_actor_name` | `varchar(64)` | NULL |
| `published_at` | `timestamptz` | NULL; non-NULL for `published`, `explicit_off`, `superseded` |
| `created_at` | `timestamptz` | NOT NULL, `CURRENT_TIMESTAMP` |

Constraints/indexes:
- PK `schedule_day_revisions_pkey (id)`.
- FK behavior exactly as listed above; no cascade from publication to evidence.
- `UNIQUE (employment_id, business_date, revision_number)`.
- Partial unique index `schedule_day_revisions_current_uq ON ... (employment_id,business_date) WHERE state IN ('published','explicit_off')`.
- Partial unique index `schedule_day_revisions_legacy_uq ON ... (employment_id,business_date) WHERE state='legacy_unverified'` for restartable backfill.
- Index `(employment_id,business_date,revision_number DESC)` and `(employer_id,business_date)`.
- CHECK state values, source values `roster|pattern|assignment_rule|fixed|manual|auto|legacy_unverified`, revision number >= 1, `source_snapshot` JSON object, and `published_at IS NOT NULL` exactly when state is `published`, `explicit_off`, or `superseded`; `published_at IS NULL` for `draft` and `legacy_unverified`. `publication_batch_id` is NULL only for `legacy_unverified` rows, and non-NULL for every other revision state. Add a check that `state='legacy_unverified'` requires `source_snapshot->>'provenance'='legacy_unverified'`.
- Cross-row employment/employer attribution is checked in the publication transaction against effective employment relationship; a trigger rejects later mismatch on segment insert.

Current revision query must use `ORDER BY revision_number DESC` only as defensive ordering after the partial unique predicate. It must not use `created_at`, `updated_at`, or `published_at` to choose current.

### 5.4 Additive columns on `work_segments`

Add:

| Column | PostgreSQL type | Null/default |
|---|---|---|
| `schedule_revision_id` | `uuid` | NULL; FK `schedule_day_revisions(id)` ON DELETE RESTRICT |
| `selected_shift_version_id` | `uuid` | NULL; FK `shift_versions(id)` ON DELETE RESTRICT |
| `scheduled_start_at` | `timestamptz` | NULL |
| `scheduled_end_at` | `timestamptz` | NULL |
| `breaks_snapshot` | `jsonb` | NOT NULL, `'[]'::jsonb` |

Add CHECK `schedule_revision_id IS NULL OR (selected_shift_version_id IS NOT NULL AND scheduled_start_at IS NOT NULL AND scheduled_end_at IS NOT NULL AND scheduled_end_at > scheduled_start_at)` and JSON array validation. Add index `(schedule_revision_id, sort_order, id)` and partial unique `(schedule_revision_id, interval_id) WHERE schedule_revision_id IS NOT NULL AND interval_id IS NOT NULL`.

Replace baseline `idx_work_segments_interval_unique` with two indexes: legacy `(employment_id,date,interval_id) WHERE schedule_revision_id IS NULL AND interval_id IS NOT NULL`, and official partial index above. Do not change existing FKs: employment cascade remains blocked for official evidence by the new revision FK; interval set-null remains allowed for legacy metadata but publication snapshot keeps interval details; template restrict remains.

A trigger rejects official segment INSERT unless revision is `draft` or publication transition is active and employment/date match. A trigger rejects UPDATE/DELETE where `schedule_revision_id IS NOT NULL`. Legacy mutation is rejected for a date that has a current official revision while lifecycle feature is active.

### 5.5 `attendance_periods`

| Column | PostgreSQL type | Null/default |
|---|---|---|
| `id` | `uuid` | NOT NULL, `gen_random_uuid()` |
| `employer_id` | `uuid` | NOT NULL, FK `employers(id)` ON DELETE RESTRICT |
| `period_year` | `integer` | NOT NULL |
| `period_month` | `integer` | NOT NULL |
| `status` | `varchar(16)` | NOT NULL, `'open'` |
| `version` | `bigint` | NOT NULL, `1` |
| `current_closure_id` | `uuid` | NULL; FK added after closures table, ON DELETE RESTRICT |
| `created_at` | `timestamptz` | NOT NULL, `CURRENT_TIMESTAMP` |
| `updated_at` | `timestamptz` | NOT NULL, `CURRENT_TIMESTAMP` |

PK `id`; UNIQUE `(employer_id,period_year,period_month)`; CHECK year 2000–2100, month 1–12, status `open|closed|reopened`; index `(employer_id,period_year,period_month)`. `current_closure_id` is the latest closure, never deletes history.

### 5.6 `attendance_closures`

| Column | PostgreSQL type | Null/default |
|---|---|---|
| `id` | `uuid` | NOT NULL, `gen_random_uuid()` |
| `period_id` | `uuid` | NOT NULL, FK `attendance_periods(id)` ON DELETE RESTRICT |
| `employer_id` | `uuid` | NOT NULL, FK `employers(id)` ON DELETE RESTRICT |
| `closure_number` | `integer` | NOT NULL |
| `status` | `varchar(16)` | NOT NULL, `'closed'`; immutable after insert |
| `closed_by` | `uuid` | NULL, FK `users(id)` ON DELETE SET NULL |
| `closed_actor_type` | `varchar(16)` | NOT NULL, `'human'` |
| `closed_at` | `timestamptz` | NOT NULL, `CURRENT_TIMESTAMP` |
| `snapshot_hash` | `char(64)` | NOT NULL |
| `created_at` | `timestamptz` | NOT NULL, `CURRENT_TIMESTAMP` |

PK `id`; FKs all `RESTRICT`/`SET NULL` as listed; UNIQUE `(period_id,closure_number)`; index `(employer_id,closed_at DESC)`; CHECK closure number >= 1, status `closed`, and actor type `human|system`. `attendance_closures` and closure days are immutable snapshots: reopen does not update or delete them. Reopen updates only attendance_periods.status/version/updated_at, retains current_closure_id and inserts an append-only attendance.reopened event into existing audit_logs (§9.3); the next close creates a new closure number.

### 5.7 `attendance_closure_days`

| Column | PostgreSQL type | Null/default |
|---|---|---|
| `id` | `uuid` | NOT NULL, `gen_random_uuid()` |
| `closure_id` | `uuid` | NOT NULL, FK `attendance_closures(id)` ON DELETE RESTRICT |
| `period_id` | `uuid` | NOT NULL, FK `attendance_periods(id)` ON DELETE RESTRICT |
| `employer_id` | `uuid` | NOT NULL, FK `employers(id)` ON DELETE RESTRICT |
| `employment_id` | `uuid` | NOT NULL, FK `employments(id)` ON DELETE RESTRICT |
| `business_date` | `date` | NOT NULL |
| `schedule_revision_id` | `uuid` | NOT NULL, FK `schedule_day_revisions(id)` ON DELETE RESTRICT |
| `schedule_state` | `varchar(16)` | NOT NULL; closure accepts only `published` or `explicit_off` |
| `schedule_snapshot` | `jsonb` | NOT NULL |
| `attendance_snapshot` | `jsonb` | NOT NULL |
| `calculation_snapshot` | `jsonb` | NOT NULL |
| `time_entry_ids` | `uuid[]` | NOT NULL, `'{}'` |
| `raw_attendance_log_ids` | `uuid[]` | NOT NULL, `'{}'` |
| `created_at` | `timestamptz` | NOT NULL, `CURRENT_TIMESTAMP` |

PK `id`; UNIQUE `(closure_id,employment_id,business_date)`; indexes `(employment_id,business_date)` and `(schedule_revision_id)`; CHECK `schedule_state IN ('published','explicit_off')` and `schedule_revision_id IS NOT NULL`, and snapshots are JSON objects. Missing coverage is rejected before closure insert. The close transaction validates `period_id`, `employer_id`, and employment's effective employer together; the FK prevents cross-table deletion but cannot encode date attribution alone.

Snapshots contain calculation inputs and exact schedule revision ID/version, interval timestamps, breaks, policy versions, holiday/leave overlay IDs, summary values and evidence IDs. They contain no names, emails, phone numbers, salary details, raw payloads or PII. Retention follows attendance 5 years after termination; audit 3 years; closure evidence is not hard-deleted while its retention obligation exists.

### 5.8 Existing FK and legacy handling

- `shift_rosters.template_id ON DELETE SET NULL`: nullable roster is preserved as explicit off; it is not treated as missing.
- `shift_rosters.employment_id ON DELETE RESTRICT`: legacy roster remains source input until publication; after official activation, mutation route requires lifecycle guard.
- `work_segments.employment_id ON DELETE CASCADE`: new revision FK and closure FK make published evidence deletion fail; no hard delete of critical evidence.
- `work_segments.interval_id ON DELETE SET NULL`: legacy relation may become null, but official `breaks_snapshot`, selected version and concrete timestamps remain.
- `work_segments.template_id ON DELETE RESTRICT`: unchanged.
- `time_entries.segment_id ON DELETE SET NULL`: unchanged for raw punches; closure stores IDs/snapshot and never relies on the nullable link alone.
- `shift_versions`: protect referenced version content and interval definitions from UPDATE/DELETE, but allow controlled supersession metadata updates. In one transaction lock the template row, then its versions ordered by effective_from/id; insert successor V2 and update only V1.effective_to to V2.effective_from minus one calendar day and V1.superseded_by to V2.id. V2 must belong to the same template and start strictly after V1.effective_from; no other V1 field may change. Validate non-overlapping inclusive effective ranges before commit and audit old/new metadata atomically. Publication/replan takes `FOR SHARE` on selected template rows before reading versions, so supersession cannot race selection. Never rewrite existing official revisions/segments, even if their future business dates now resolve to V2; those changes require replan. Backdated corrections also insert a new version and obey range validation; do not overwrite version content referenced by evidence.
- `audit_logs.actor_id` is nullable. Human audit uses UUID. System audit uses `actor_id=NULL`, `actor_type='system'`, `actor_name='schedule-publisher'`; this is valid for `audit_logs` and never sends a non-UUID string to `AuditLogWriter`.
- `outbox_jobs` remains reused for notification/cache work. No new lifecycle table duplicates it. A future worker must add retry metadata additively and must not be called publication evidence.

### 5.9 Down migration boundary

`down.sql` first refuses rollback if any preview, schedule batch, day revision, official segment link, attendance period, closure or closure-day row exists, or if any legacy row has been marked lifecycle-managed. It then drops guards/indexes/columns/tables in reverse FK order and deletes only its own migration marker. Before activation, rollback is tested on an empty expanded schema. After official evidence exists, rollback is forbidden; roll-forward is required.

## 6. State machines

### 6.1 Day revision

| Transition | Actor/permission | Preconditions | DB effect | Audit action/error |
|---|---|---|---|---|
| none → draft | publication/replan service, never preview | validated first publication or authorized replan; MAX+1 allocator | Insert revision and draft segments inside mutation transaction only | `schedule.revision_drafted`; validation/conflict |
| draft → published | mode publish permission for first publication; amend permission for replan | no conflict/missing; payroll open; attendance open/reopened; preview valid; office authority valid; retroactive rules met | Publish new revision; never supersede via publish endpoint | `schedule.published`; `already_published`/`stale_preview`/`locked_period` |
| draft → explicit_off | same authority and period/preview guards as published | explicit off source, zero work segments | Publish off revision; current unique index | `schedule.explicit_off` |
| published/explicit_off → superseded | amendment permission and expected current revision | period open; reason/diff; payroll unlocked; current row locked | Update only state and create replacement revision; old segments unchanged | `schedule.superseded`; `stale_revision`/`locked_period` |
| published/explicit_off → anything else | nobody | impossible | Trigger rejects | `invalid_transition` |

Missing/conflict are resolver outcomes, not current official states. They block publication and are returned with exact date/source trace.

### 6.2 Publication batch

`requested` is not persisted. One transaction either creates `published` batch plus all rows and audit, or rolls back all writes. A duplicate idempotency key with same hash returns stored batch and revision IDs. Same key with different hash returns `idempotency_conflict` (409). A transaction failure leaves no partial batch; retry may reuse key. A committed batch cannot be republished.

### 6.3 Attendance period/closure

| Transition | Actor/permission | Preconditions | DB effect | Audit/error |
|---|---|---|---|---|
| absent → open | service | employer/month valid | Insert period, version 1 | none |
| open/reopened → closed | global or employer-scoped `attendance:closure:close` | complete official coverage; no missing/conflict; no open punches; required approvals resolved; payroll period exists and is open | Lock period, insert numbered closure and immutable days, set current closure, increment version | `attendance.closed`; `missing_coverage`/`open_punch`/`closure_conflict` |
| closed → reopened | global `attendance:closure:reopen`, reason | payroll period is open; expected period version; no concurrent amendment | Lock period, update only attendance period to reopened, append transition/audit row; keep closure snapshot immutable | `attendance.reopened`; `locked_period`/`stale_period` |
| reopened → closed | close permission, same checks | new closure number; old closure preserved | Insert new closure/days, set current | `attendance.reclosed` |

Reopen never deletes prior closure rows or changes their snapshots. Closing a period does not lock payroll. Payroll lock requires `attendance_periods.status='closed'`; payroll reopen does not automatically reopen attendance.

### 6.4 Payroll period interaction

`payroll_periods` remains the only payroll lock table. `LockPayrollPeriod` follows §9.1 lock order, verifies its attendance period and every source closure used by cross-month valuation slices are closed (§17.4), then performs existing payroll lock audit in the same transaction. Exact source closure IDs must be retained with the calculation; reopening a referenced attendance closure is blocked by any dependent locked payroll period. `ReopenPayrollPeriod` keeps existing reason/version behavior and does not erase closure. Schedule amendment, attendance reprocess, payroll calculation and payslip recalculation reject while payroll status is `locked` with `locked_period`.

## 7. API contract

All lifecycle handlers authenticate and authorize before parsing business payload or querying domain data. UUIDs use canonical UUID syntax. Dates use `YYYY-MM-DD`. Timezones use IANA names and are validated by `time.LoadLocation`. Date range maximum is 62 calendar days for preview/publish and 31 days for legacy GET. Batch employment maximum is 500. Page maximum is 100; default 50; stable order is `(business_date ASC, employment_id ASC, revision_number DESC, id ASC)`.

Error envelope remains `{ "error": { "code": "...", "message": "..." } }` through `apperror.WriteError`; add these keys without exposing PII: `invalid_uuid`, `invalid_date`, `invalid_timezone`, `range_too_large`, `forbidden`, `stale_revision`, `conflict`, `idempotency_conflict`, `locked_period`, `missing_coverage`, `resolver_conflict`, `duplicate_key`, `audit_failure`, `closure_conflict`, `open_punch`, `not_found`, `stale_preview`, `already_published`, `no_current_revision`, `retroactive_confirmation_required`.

### 7.1 Preview

`POST /api/v1/schedules/preview`

Request:

```json
{
  "employment_ids": ["11111111-1111-1111-1111-111111111111"],
  "start_date": "2026-11-16",
  "end_date": "2026-11-16",
  "timezone": "Asia/Ho_Chi_Minh",
  "mode": "office",
  "approval_instance_id": "33333333-3333-3333-3333-333333333333",
  "reason": "Approved office policy publication",
  "retroactive": false,
  "expected_revisions": {
    "11111111-1111-1111-1111-111111111111:2026-11-16": 0
  }
}
```

Response `200`:

```json
{
  "provisional": true,
  "preview_id": "66666666-6666-6666-6666-666666666666",
  "preview_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "expires_at": "2026-11-13T10:15:00Z",
  "resolver_version": "schedule-resolver-v1",
  "data": [{
    "employment_id": "11111111-1111-1111-1111-111111111111",
    "business_date": "2026-11-16",
    "status": "work",
    "source": "pattern",
    "timezone": "Asia/Ho_Chi_Minh",
    "shift_version_id": "22222222-2222-2222-2222-222222222222",
    "intervals": [{
      "sort_order": 0,
      "start_at": "2026-11-16T08:00:00+07:00",
      "end_at": "2026-11-16T17:00:00+07:00",
      "breaks": []
    }],
    "trace": [],
    "warnings": []
  }]
}
```

`status` is `work|explicit_off|missing|conflict`. Preview writes only its provisional schedule_previews row using §9.1, never official schedule/audit rows. Persist normalized request and source/result snapshots before returning preview_id. `400` validation, `403` forbidden, `409` stale expected revision. A diagnostic preview may contain missing/conflict rows; publish rejects them. For a publish preview include the same approval_instance_id, reason and retroactive value intended for publication (null approval for rotating); authorization is rechecked at commit. Expected revision maps must cover every requested employment/day: 0 for no current, otherwise its revision_number. Timezone is a consistency assertion against resolved timezone, not a client override of business date.

### 7.2 Office publication

`POST /api/v1/schedules/publish`

Headers: `Idempotency-Key` required, 1–128 ASCII characters. Request must include `preview_id` and `preview_hash`; server rejects expired or changed previews with `409 stale_preview`.

Request:

```json
{
  "employment_ids": ["11111111-1111-1111-1111-111111111111"],
  "start_date": "2026-11-16",
  "end_date": "2026-11-16",
  "timezone": "Asia/Ho_Chi_Minh",
  "approval_instance_id": "33333333-3333-3333-3333-333333333333",
  "preview_id": "66666666-6666-6666-6666-666666666666",
  "preview_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "reason": "Approved office policy publication",
  "retroactive": false,
  "expected_revisions": {
    "11111111-1111-1111-1111-111111111111:2026-11-16": 0
  }
}
```

Response `201`:

```json
{
  "batch_id": "44444444-4444-4444-4444-444444444444",
  "status": "published",
  "published_at": "2026-11-13T10:00:00Z",
  "revision_ids": ["55555555-5555-5555-5555-555555555555"],
  "affected_days": 1
}
```

First publication only: every resolved employment/day requires expected revision token 0; an existing current work/off revision returns `409 already_published`. Office approval must be an approved `approval_workflow_instances` record with entity type `schedule_policy` and policy effective dates covering the range. Store actual approval and publication times; an effective date is not proof of prior approval/publication.

Request field `retroactive` is boolean, default false. A target is retroactive if business_date precedes local today **or** its earliest scheduled start_at is before the actual publication instant. For explicit off, use start of the local day. If any target is retroactive, require `retroactive=true`, human actor, the normal mode publication permission **and** `schedules:amend`, plus a nonblank reason (1–2000 characters). System auto-publication never publishes retrospectively. Without acknowledgement return `409 retroactive_confirmation_required`; without authority return `403`. Payroll must be open and attendance open/reopened. Persist per-day `source_snapshot.retroactive` boolean, `source_snapshot.late_publication_reason` for late days, and actual published_at; include the same flag in response/evidence. This records a late decision, not evidence that the employee was notified in advance. Request-supplied published_at is rejected as an unknown field.

`400` validation; `403` permission/scope; `409` stale_preview/already_published/locked_period/retroactive_confirmation_required/idempotency_conflict; `422` missing coverage. The range may contain late and future days; classify each day separately and commit all or none.

### 7.3 Rotating batch publication

`POST /api/v1/schedules/publish/rotating-batch`

Same preview, idempotency, first-publication and retroactive contract as office; mode is fixed to rotating by this route and approval_instance_id must be null. Require `schedules:publish:rotating`, reason and complete expected revision map of zeros. A batch contains one or more employment/day entries (maximum 500 employments and 62 days); a one-day recovery is valid and still explicitly published, never automatic. Reject the whole batch if any target already has a current revision.

### 7.4 Legacy schedule read

`GET /api/v1/schedules?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD&employment_ids=uuid,...&page=1&limit=50&view=official`

Default `view=official` after feature activation. Existing fields (`data`, `total`, `page`, `limit`, employee/schedule fields) remain. Additive fields: `revision_id`, `schedule_status`, `provisional`, `timezone`, `intervals`, `published_at`. `view=preview` is not accepted on GET; use POST preview. An official missing day returns `schedule_status='missing'`, `shift_id=null`, no synthetic times. Self-service response contains only owned employment rows; manager scope uses employer/department authorization.

### 7.5 Amend/replan

First call `POST /api/v1/schedules/{employment_id}/{business_date}/replan/preview`. Require `schedules:amend` and target scope. Request contains `expected_revision`, `reason` and `replacement` exactly as below, without preview/hash fields. Server validates shift version and all intervals/breaks, computes concrete timestamps, checks overlap and payroll/attendance state, then saves a preview with operation=replan. Response `200` contains `preview_id`, `preview_hash`, `expires_at`, `original_revision_id`, normalized `replacement`, `diff` and `diff_hash`. `diff` has exact shape `{ "before": <normalized original>, "after": <normalized replacement> }`; both sides contain employment_id, business_date, timezone, status, source, selected version IDs, ordered intervals and breaks with concrete timestamps. Hash `{ "original_revision_id": ..., "expected_revision": ..., "diff": ... }` with §5.2 canonical encoding. UI shows this server diff before confirmation; it never invents a hash. Missing current revision returns `409 no_current_revision`; use first publication for missing days.

Then call `POST /api/v1/schedules/{employment_id}/{business_date}/replan` with required Idempotency-Key and the exact previewed replacement/reason plus preview_id, preview_hash and confirm_diff_hash. Server rechecks preview owner/operation/scope and recomputes result/diff under locks; any changed replacement, source input or hash returns `409 stale_preview`, changed current revision returns `409 stale_revision`. Replan is human-only.

Commit request:

```json
{
  "expected_revision": 3,
  "reason": "Original roster had wrong overnight shift",
  "replacement": {
    "status": "work",
    "source": "roster",
    "shift_version_id": "22222222-2222-2222-2222-222222222222",
    "intervals": [{"sort_order": 0, "start": "22:00", "end": "06:00", "breaks": []}]
  },
  "preview_id": "77777777-7777-7777-7777-777777777777",
  "preview_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "confirm_diff_hash": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
}
```

Response `201` contains batch_id, original_revision_id, replacement_revision_id, revision_number, reason, actor_id, actual_time, retroactive and normalized diff. Record actual publication time even for past-day amendments, and mark retroactive by §7.2. New/revised unpublished drafts are input edits followed by a fresh preview, not a call to this endpoint; no official revision is updated in place. Closed returns `409 locked_period` until authorized reopen. `409 stale_revision` for changed current revision, `409 stale_preview` for changed/expired preview; `400` invalid UUID/date/timezone/reason/hash format; `403` missing amend authority/scope. Examples use illustrative 64-hex hashes; only the values returned by the real preview are valid.

### 7.6 History and evidence

`GET /api/v1/schedules/history?employment_id=uuid&start_date=...&end_date=...&cursor=...&limit=50` requires `schedules:history:read` and returns immutable revisions, state, source snapshot IDs, actor type/name, reason, previous/replacement links and stable cursor. It never returns raw PII.

`GET /api/v1/schedules/evidence/{employment_id}/{business_date}` requires `schedules:evidence:export`, returns exact revision/segment/closure IDs, source snapshot, diff chain, audit actions, notification delivery records if present, and acknowledgement records if present. It distinguishes `publication`, `notification_delivery`, and `acknowledgement`; no delivery/ack is inferred. Export is audit-logged with row count and actor.

### 7.7 Close/reopen

`POST /api/v1/attendance/periods/{year}/{month}/close?employer_id=uuid`

Request `{ "version": 3, "reason": "Monthly attendance close" }`; response `201` includes period ID, closure ID/number, status, version, exact day count and snapshot hash. `409 missing_coverage`, `closure_conflict`, `open_punch`, `stale_period`, `locked_period`; `403` scope; `422` date/employer validation.

`POST /api/v1/attendance/periods/{year}/{month}/reopen?employer_id=uuid`

Request `{ "version": 4, "reason": "Correct approved attendance exception" }`; response `200` includes period status/version and prior closure ID. It never deletes snapshots. `409 locked_period` if payroll is locked; `403` without reopen permission.

## 8. Authorization and scoping

New permission constants in `apiserver/internal/auth/permissions.go` (seven schedule permissions and three closure permissions):

- `schedules:read`
- `schedules:preview`
- `schedules:publish:office`
- `schedules:publish:rotating`
- `schedules:amend`
- `schedules:evidence:export`
- `schedules:history:read`
- `attendance:closure:read`
- `attendance:closure:close`
- `attendance:closure:reopen`

Authorization runs in handler before resolver/repository business logic. Scope function rules:

| Actor scope | Read/preview | Office publish | Rotating batch | Amend | Close | Reopen | Evidence export |
|---|---|---|---|---|---|---|---|
| Global manager | all employments | yes with approved policy | yes | yes | yes | yes | yes |
| Employer-scoped manager | target employer's effective employment dates | yes within employer | yes within employer | yes within employer | yes within employer | no unless global | yes within employer |
| Department-scoped shift manager | managed department only | no | yes within department | yes while period open | no | no | yes within department |
| Department-scoped team lead | managed department only | no | no | no | no | no | no |
| Self-service employee | own employment only | no | no | no | no | no | no |

`Global manager` means `OrganizationAuthorizer.IsGlobalManager`. Employer scope must verify the employment's employer relationship at each business date, not current `employments.employer_id` alone. Department scope must verify effective department at each date. Self-service must verify employee ownership. Deny when scope lookup fails; never fail open. The handler returns `403` before schedule existence checks to avoid authorization leakage.

System office previews/publication have no user actor: `requested_by=NULL`, `actor_type='system'`, `actor_name='schedule-publisher'`; audit `actor_id=NULL`, same actor type/name. Human actions always use authenticated UUID. Internal worker authority is limited to approved policy scope and new future office days; it cannot amend, reopen or perform retroactive publication. Human late first publication requires both the mode's publish permission and schedules:amend within scope, never permission inferred solely from role label.

## 9. Transaction and locking protocol

### 9.1 Lock order and preview transaction

Publish/replan use `READ COMMITTED`, so an idempotency lookup after a waiting lock sees the preceding commit. Acquire locks in this order; acquire all rows of each class before the next class:

1. Transaction advisory lock `pg_advisory_xact_lock(hashtextextended('schedule-lifecycle:' || idempotency_key, 0))`. One key per request. Hash collisions only serialize unrelated requests. Preview/close/reopen do not take this key lock.
2. Lookup committed batch and handle replay under §9.4 **before** any preview expiry/current-state precondition.
3. Employee rows for all affected people ordered by UUID `FOR UPDATE`, then employer rows ordered by UUID `FOR UPDATE`. Determine the complete set before locking; resolve all employments of those people for interval conflict checks, including other employer scopes without disclosing their data. Retry the transaction if the set changes. Every schedule writer and employment/employee reassignment path must share this order; never acquire another employee lock after an employer/period lock.
4. Payroll period rows: insert if absent, then lock ordered by `(employer_id,period_year,period_month)`.
5. Attendance period rows: insert if absent, then lock in the same period order.
6. Employment rows ordered by UUID `FOR UPDATE`; current revisions ordered by employment/date `FOR UPDATE`. Allocate revision numbers over all states while holding these employment locks.
7. Selected shift template rows ordered by UUID `FOR SHARE`, then read effective versions. Shift-version writers use template `FOR UPDATE` before version locks; if they also need lifecycle locks, they acquire steps 3–6 first.
8. Preview row `FOR UPDATE` for a new publish/replan. Revalidate ownership/operation/expiry and current inputs. Cleanup only locks preview rows and never domain rows.
9. Raw attendance/time entries and attendance summaries for close/reprocess, ordered by employment/date/ID `FOR UPDATE`.
10. Domain writes, synchronous audit INSERT, commit. Audit failure rolls back every domain write.

Close/reopen begin at step 3, include the complete affected employee/employer/period set and cross-month closure/payroll dependencies (§17.4), skip inapplicable classes, and recheck period status after locks. Payroll locking and every protected attendance writer use the same relative order. No worker holds an outbox claim lock while entering this sequence.

Preview uses a **read-write `REPEATABLE READ`** transaction: read authorized source inputs and current revisions from one consistent snapshot, compute result/diff, INSERT only `schedule_previews`, commit, then return its ID/hash. It does not create official revisions, segments, publication batches or publication audit events; it takes no domain mutation locks. Preview is advisory: publish/replan reloads inputs and compares hashes. A crash before commit exposes no usable preview ID.

### 9.2 Publish

Authenticate, validate DTO and authorize all target scope; derive actor server-side. Normalize payload/hash, begin transaction and perform steps 1–2. For a new operation, lock domain rows and preview, revalidate policy and source hashes. All targets must have no current official revision and an explicit `expected_revisions[employment_id:date]=0` token (0 means no current, not next revision number). Any existing current revision returns `409 already_published`; the entire batch rolls back. **Publish never supersedes a revision**, regardless of actor's amend permission. Replace published work/off only through replan.

Require payroll open, attendance open/reopened, complete work/off resolution and the retroactive rules in §7.2. Insert batch, allocate revision numbers, insert draft revisions and their complete segments, transition these new drafts to published/explicit_off, insert audit and the immutable response snapshot, commit. `published_at` is the server's actual publication timestamp, never supplied by the client. Copy source/decision snapshots into permanent evidence; preview expiry cannot erase them.

### 9.3 Amend and close

Replan requires `schedules:amend`, reason, Idempotency-Key and a replan preview/diff confirmation (§7.5). Use §9.4 replay first; for a new operation take the same lock sequence, require matching expected current revision and diff/result hashes, payroll open and attendance open/reopened. Allocate MAX+1, insert replacement batch/revision/segments, supersede only the expected current revision without changing its published_at/content, publish replacement, write audit and response snapshot, commit. Inherit mode and approval reference from the original batch; snapshot the human amendment authority separately in source_snapshot. Replan is not a new system office authorization and cannot silently expand an approved policy.

Close determines the complete affected employee set, then locks employee, employer, payroll period, attendance period, all employment/day rows and raw/summary rows in §9.1 order. It checks coverage/open punches/approvals, inserts closure and closure-day snapshots, updates period, inserts audit, commits. It does not update attendance_daily_summaries or raw punches.

Reopen locks the affected employees, employers, all dependent payroll periods and attendance periods in §9.1 order; checks all dependent payroll periods open, independent approval (§17.6) and expected version; updates only period status/version/updated_at, retaining current_closure_id. Insert `attendance.reopened` into existing audit_logs in the same transaction with period ID, closure ID, old/new period versions and reason; do not create another transition table or mutate closure rows. Reclose creates a new numbered immutable closure.

### 9.4 Concurrency and idempotency

- After authorization and key lock, lookup by `idempotency_key`. If found, require the same stored actor tuple, current authorization for the stored scope, and canonical request hash. Another actor receives `403`; same actor/different hash receives `409 idempotency_conflict`. Same actor/hash returns the stored response_snapshot with original `201`, even if preview expired/was deleted, revision was superseded or period was subsequently closed. Replay performs no mutation and does not bypass current access control.
- Only when no committed batch exists may preview expiry, source changes, expected revision and period preconditions be evaluated. Rollback leaves no key reservation; retry starts anew. Unique(idempotency_key) remains a DB safety constraint; do not use a no-op UPDATE to retrieve a result.
- Concurrent same-key requests serialize on the advisory lock and replay one outcome. Different-key first publications serialize on employment locks: one succeeds, the other returns `409 already_published`. Concurrent replans return `409 stale_revision` to the loser. Partial current uniqueness is the final guard.
- Concurrent close/amend/reopen serialize on the shared domain lock order. Payroll locked or attendance closed blocks new schedule mutation; an already committed replay remains a read.
- Raw punch ingestion remains allowed; it does not authorize closed-result recalculation.

## 10. Resolver semantics

1. Reuse `resolveEffectiveShift` in `apiserver/internal/repository/schedules.go`; no second resolver.
2. Source precedence is roster, pattern, assignment rule, fixed. A roster row with null template is explicit off. A pattern detail with null template is explicit off. No matching weekday is missing. Missing is never off.
3. Pattern weekly match uses `business_date.Weekday()`. Rotation uses calendar-day difference from anchor in the selected timezone, computed by date arithmetic; never `elapsed 24-hour` arithmetic across DST. Negative modulo normalizes into `[0, cycle_length)`.
4. Assignment-rule ties at same highest priority are conflict. Lower-priority matches are trace `overridden`. Existing SQL order by priority/created time may remain for trace ordering but cannot resolve a tie.
5. Shift template resolution chooses the `shift_versions` row where `effective_from <= business_date` and `(effective_to IS NULL OR business_date <= effective_to)`, with unique effective coverage validated. More than one valid version is conflict. The selected version ID is stored, not only logical template ID.
6. Employer attribution uses the active employment/contract relationship effective on business date. Transfer days are not copied into another employer closure.
7. Timezone uses active host assignment location, then primary location, then UTC. Every interval converts local date/time to timestamptz before storage.
8. Holiday overlay marks holiday/name/paid policy but does not erase schedule. Approved leave overlay marks attendance obligation exempt according to leave policy but does not mutate the official shift or convert missing to off. The overlay IDs and policy versions enter source snapshot.
9. Load all shift default intervals ordered by `sort_order`; preserve every interval and break configuration. A work template without valid intervals is a resolver conflict, not invented default hours; only an explicitly verified legacy conversion may supply an interval before activation.
10. Resolve intervals using shift group and explicit local-day offsets (§17.4), then store exact `scheduled_start_at`/`scheduled_end_at` and break timestamps. Legacy unambiguous single intervals may derive next-day end when end_time < start_time; equal times require explicit duration/day offset, not an implicit 24-hour shift. Never infer the next segment's start date from end_time alone.
11. Preview returns `provisional=true` and source trace. Official reports consume only current official revision; no-show does not synthesize hours.

Current source does not yet load all intervals, leave overlays, or location timezone in `resolveEffectiveShift`; implementation must add those reads in the same resolver and add tests for the assumptions.

## 11. Publication, amendment and replan rules

- Draft inputs: edit unpublished roster/pattern inputs and create a fresh preview; no official revision or supersession. A schedule_day_revisions draft is transaction-local staging only, never a separately committed editable API resource.
- Published/open: replacement requires expected revision, reason, diff preview confirmation and amendment permission. Transaction creates new revision number, retains original, links `previous_revision_id`, writes complete replacement interval set and audit.
- Closed: caller must reopen attendance with `attendance:closure:reopen` and reason; payroll must be open. Amendment then creates a revision and a later reclose creates a new closure snapshot. Old closure remains.
- Raw punches, time entries, raw attendance logs, manual adjustments, OT approvals, leave approvals, prior calculation and payslips are retained. Replan never approves OT/leave, changes payroll or silently recalculates closed results.
- Missing past-day schedules use controlled first publication under §7.2, with retroactive acknowledgement, human publication+amend authority, reason and actual time. Existing past-day schedules use replan. Neither path fabricates prior publication or delivery; closed attendance must reopen and payroll must be open.
- Evidence shows original revision, replacement revision, actor UUID or system actor tuple, reason, actual publication/replan time, diff hash, source snapshot and linked closure IDs.

## 12. Consumer/writer cutover

### 12.1 Readers

1. `repository/schedules.go`: `GET /schedules` reads current official rows for `view=official`; preview remains provisional.
2. `repository/work_segments.go`: list/get official filters by current revision when official mode; legacy list remains only for migration/reconciliation.
3. `repository/reports.go`: daily and summary reports join exact current revision/segments; no-show reads official current work revisions only and removes the `09:00–17:00` UNION branch. Explicit off is excluded; missing is reported as missing coverage, not absent.
4. Matching/pairing code in `apiserver/internal/domain/matcher/` and attendance reprocess paths consume exact segment timestamps and revision ID.
5. Attendance summaries, OT and payroll calculation use closure snapshot when period is closed; open periods use current official revision. No report can silently fall back to synthetic or legacy schedule after activation.

### 12.2 Writers

Guard before mutation in `repository/rosters.go`, `work_segments.go`, `working_patterns.go`, `assignments.go`, `shifts.go`, `attendance.go`, `attendance_logs.go`, `overtime.go`, adjustment handlers and bulk import/reprocess writers. A writer affecting an official day must call lifecycle guard with employment/date and expected revision. A writer affecting closed/locked period returns `409 locked_period`. Punch/raw-log ingestion bypasses schedule mutation guard and remains accepted.

Legacy `shift_rosters`, direct segment create/update/delete, pattern/rule/shift changes, OT imports, adjustments and bulk writers cannot bypass publication/closure guard. Feature activation is per employer; DB trigger blocks old app mutation for activated official dates. Old app versions are rejected at startup/health gate when lifecycle schema version is newer than supported version.

### 12.3 Outbox and worker

Source proves only `outbox_jobs` producers and an in-memory `internal/queue/Worker`; source does not prove a PostgreSQL outbox consumer or scheduler. Do not use asynchronous audit as transaction evidence.

Minimum office publisher implementation (same Go process; no new service):

**Producer and horizon**
- On startup and every 15 minutes, scan enabled employers and approved, unrevoked office policies. Serialize each employer scan with a transaction advisory lock named `schedule-office-producer:<employer_id>`; bound pages to 500 employments. For each employment's resolved timezone cover local today through the last day of next calendar month, clipped to employment and policy effective dates. The periodic scan is required even when no events arrive; it replenishes coverage across month boundaries and retries after downtime.
- Approval/revocation, lifecycle activation and employment/policy/source changes enqueue `ScheduleOfficeCoverageRefresh` in the same transaction as their existing mutation. Its consumer runs that same scan, not a second scheduler/resolver. Revocation stops new publications; existing official schedules remain unchanged.
- Only enqueue missing current employment/day keys whose winning source is authorized by the approved office policy. Preserve existing official/manual exceptions. Do not turn rotating or missing/conflict results into office schedules. Record those gaps for human action. Already-started work/off days are late coverage gaps: do not auto-publish; expose them in reconciliation for §7.2 recovery.
- Durable unit of work is one employment/day. `dedupe_key = 'schedule-office:' + approval_id + ':' + employment_id + ':' + business_date`; request idempotency key is the stable job UUID. Enqueue with unique dedupe_key and ON CONFLICT DO NOTHING. A completed job is not recreated. A dead-letter job requires explicit authorized retry after diagnosis; scans must not reset its retry budget. A fresh approval has a different key but still cannot replace an existing current revision.

**Outbox columns and execution**
- Add nullable `dedupe_key text`, `lease_token uuid`, `locked_until timestamptz`; add `attempts integer NOT NULL DEFAULT 0`, `available_at timestamptz NOT NULL DEFAULT now()`, `last_error text`, `dead_letter_at timestamptz`. Add unique(dedupe_key) WHERE dedupe_key IS NOT NULL and index(job_type,status,available_at). Preserve other job types and their consumers.
- Claim supported due jobs with FOR UPDATE SKIP LOCKED in a short transaction, set status=processing, increment attempts, issue a new lease_token and locked_until=now()+5 minutes, then commit before any domain lock. Heartbeat each minute with token comparison. Expired processing leases are reclaimable within the attempt budget. Completion/failure UPDATE must compare lease_token so an old worker cannot overwrite a newer claim.
- Before making a preview, lookup any committed batch for the stable job UUID using the internal system principal and current scope rules. If it exists, recover its stored response and finish the job without creating another preview. Otherwise create a system preview, then call the same first-publication service with its stored ID/hash, expected revision 0 and current approved authority. System credentials/actor fields are internal, not HTTP-selectable. Serialize same-key domain commits under §9.4; a concurrent worker that loses this race reads the committed system batch rather than sending a newly generated preview as a replay payload.
- Publication and its audit commit atomically; outbox completion is a separate short transaction. A crash between commits is safe because retry recovers the batch by job UUID. Do not hold a claim transaction across preview/domain work. A race where a human published the target is a completed no-op only after reading the current official revision; never amend it.
- Maximum five attempts total: initial attempt immediately, then delays 1 minute, 5 minutes, 15 minutes and 1 hour after failures 1–4. Failure 5 sets dead_letter and dead_letter_at. A lease expiring after attempt 5 also dead-letters. Fresh preview per uncommitted retry; stale preview does not consume an independent unbounded retry loop. Revoked policy, retroactive gap and resolver conflict require human reconciliation, not repeated publication.
- Job payload contains approval ID, employer ID, employment ID, business date and mode=office; no names, codes or raw data. No batch ID or preview hash is invented at enqueue time. Record committed batch ID in job payload after success using the lease token.
- Metrics: scan last success, coverage horizon, missing/late coverage, claimed, published, retry, dead-letter and conflict. Alert on coverage gaps before shift start. Notifications are separate and cannot change publication outcome.

## 13. Migration, rollout and rollback

1. **Expand:** apply additive migration and guards disabled by feature flag. Verify tables, indexes, FKs, checks, trigger definitions and schema marker.
2. **Backfill:** under employment lock, create at most one immutable `legacy_unverified` revision per employment/day (partial UNIQUE(employment_id,business_date) WHERE state='legacy_unverified'). Use the same MAX+1 allocator as publication; set source=legacy_unverified, resolver_version=legacy-import-v1, publication_batch_id/published_at/published_by=NULL and provenance=legacy_unverified. Copy legacy roster/segment IDs and known values into source_snapshot, marking unknown version/timezone explicitly; never invent selected version IDs. Keep legacy work_segments unlinked (`schedule_revision_id=NULL`), preserving their original IDs and time-entry links, since official segment constraints require proven version/timestamps. On restart, skip existing legacy revision keys; do not overwrite captured evidence. Later first publication gets a higher number, previous_revision_id=NULL and real authority/time. Legacy history never satisfies closure coverage.
3. **Shadow read:** for each employer/date, compute preview and compare legacy output to resolver output. Record mismatch counts without changing official readers. Reconcile all multi-interval, overnight, off, missing and effective-version cases.
4. **Enable per employer:** feature flag enables official readers/writers only after zero schema integrity errors, zero current-key duplicates, zero unauthorized legacy writes, zero official-vs-preview mismatches in activation range, and coverage is complete. Abort if any integrity mismatch occurs or if publisher failures exceed 10 rows or 1% of attempted days in a 24-hour window, whichever is reached first.
5. **Cut over writers/readers:** enable lifecycle guards and official reports together. Block old app versions that lack lifecycle guard support. Monitor lock waits, publication latency, missing coverage, stale conflicts and dead letters.
6. **Rollback before activation:** disable flag, stop publisher, restore legacy reader/writer path, run additive migration down only when no lifecycle/evidence rows exist. Migration down must fail safely when data exists.
7. **Roll-forward after evidence:** once any official publication or closure evidence exists, do not down-migrate or delete compatibility data. Fix forward, reconcile mismatches, and preserve revision history.
8. **Contract:** remove legacy compatibility reads/writes only after all supported employers reconcile, no old app is connected, evidence exports reproduce exact snapshots, and a recorded approval authorizes cleanup.

Abort counts are exact: any FK/check failure = abort; any current-key duplicate = abort; any unauthorized legacy mutation = abort; any shadow mismatch in activation scope = abort; publisher failure >= 10 or >= 1%/24h = abort; dead-letter > 0 for required office publication = abort.

## 14. Ordered implementation tasks

| # | Files/symbols | Work | Verification command |
|---:|---|---|---|
| 1 | `docs/adr/` new ADR; `schedule-lifecycle-spec.md` | Record hybrid decision, boundaries, source facts, API/error/permission contract | `git diff --check -- schedule-lifecycle.md schedule-lifecycle-spec.md` |
| 2 | `apiserver/migrations/<timestamp>_schedule_lifecycle/up.sql`, `down.sql` | Add exact tables/columns/indexes/checks/FKs/triggers; add outbox retry columns; no baseline edit | `./leedevkit test api` migration phase; migration fixture validation via repository wrapper |
| 3 | `apiserver/internal/auth/permissions.go`, role/permission seed source | Add seven schedule and three closure permissions and role grants | `./leedevkit test api --unit-only` authorization tests, then full target |
| 4 | `apiserver/internal/repository/schedules.go` (`resolveEffectiveShift`, official queries) | Calendar-date resolver, all intervals, effective shift version, overlays, preview/current reads | resolver unit tests; `./leedevkit test api` |
| 5 | New `apiserver/internal/repository/schedule_lifecycle.go` and API types | Implement preview/publish/idempotency/amend/history/evidence transaction service | repository transaction tests; API contract tests; `./leedevkit test api` |
| 6 | `apiserver/internal/api/schedules.go`, `apiserver/routes_scheduling.go` | Add exact handlers/routes, validation, scope checks, errors, compatibility fields | API contract/403/409/idempotency tests; `./leedevkit test api` |
| 7 | `apiserver/internal/repository/work_segments.go`, `reports.go`, matcher paths | Link official revision, immutable guards, official reports, remove synthetic no-show | repository/report integration tests; `./leedevkit test api` |
| 8 | `apiserver/internal/repository/attendance.go`, `attendance_logs.go`, `overtime.go`, adjustment/bulk writers | Add closed/locked guards; keep raw ingestion; exact revision in recalculation | negative writer tests and late-punch integration tests; `./leedevkit test api` |
| 9 | New closure repository/API; `apiserver/routes_attendance.go` | Close/reopen/reclose, exact snapshots, employer transfer validation, evidence | closure transaction/concurrency tests; `./leedevkit test api` |
| 10 | `apiserver/internal/repository/payroll_periods.go`, `payroll.go`, `routes_payroll.go` | Require closed attendance before payroll lock; block locked amendment/recalc | payroll/closure integration tests; `./leedevkit test api` |
| 11 | `apiserver/internal/app/app.go`, `repository/outbox.go`, new `repository/schedule_office_jobs.go` and `service/schedule_office_worker.go`; policy/employment mutation wiring | Implement event producer, startup/15-minute horizon scan, dedupe, system previews, leased worker and crash-safe recovery under §12.3 | producer/horizon/retry/dead-letter/crash tests; `./leedevkit test api` |
| 12 | Existing roster/shift/pattern/assignment writers | Enforce lifecycle guard and old-version block | bypass regression tests; `./leedevkit test api` |
| 13 | `webdashboard/src/lib/api/*`, schedules/attendance pages/routes | Show provisional/current/superseded/off/missing/closed, diff/reason/history/evidence; i18n/a11y states | `./leedevkit test web` |
| 14 | Migration test fixtures and DB harness | Up/down/integrity, representative legacy and mixed-version validation | `./leedevkit test api`; migration fixture wrapper; no direct host DB command |
| 15 | All changed packages | Run complete verification and inspect per-phase logs | `./leedevkit test all --json` |

Migration fixture validation must use the repository's existing wrapper around `scripts/validate_migration_fixtures.py`; do not add a new script. No runtime command is claimed as run by this document.

## 15. Runnable acceptance matrix

| ID | Test file/symbol expected | Scenario | Expected result |
|---|---|---|---|
| U1 | `schedule_resolver_test.go/TestResolverPrecedence` | roster, pattern, rule, fixed all match | roster wins; trace records overridden sources |
| U2 | `schedule_resolver_test.go/TestResolverRuleTie` | two highest-priority rules match | preview `conflict`; publication `409 resolver_conflict` |
| U3 | `schedule_resolver_test.go/TestMissingVsExplicitOff` | no weekday entry vs nullable roster/pattern | first `missing`; second current `explicit_off` |
| U4 | `schedule_resolver_test.go/TestRotationUsesCalendarDateDST` | anchor across DST | cycle follows local calendar date, no 23/25-hour drift |
| U5 | `schedule_resolver_test.go/TestEffectiveShiftVersion` | two effective versions overlap/replace | one valid version selected; overlap is conflict |
| U6 | `schedule_resolver_test.go/TestAllIntervalsOvernightBreaks` | two intervals, break, 22:00–06:00 | all rows/timestamps/breaks preserved; end next local date |
| U7 | `schedule_diff_test.go/TestReplanDiff` | changed interval and shift version | diff identifies removed/added values and hash is stable |
| R1 | `schedule_lifecycle_repository_test.go/TestPublishTransaction` | batch + revision + segments + audit | all commit or all rollback |
| R2 | same | audit insert failure | no batch, revision or segment remains |
| R3 | same | same idempotency key/same payload | original outcome returned; no duplicate rows |
| R4 | same | same key/different payload | `409 idempotency_conflict`; original outcome unchanged |
| R5 | same | concurrent first publish same day, distinct keys | one current revision; loser `409 already_published`; no supersession |
| R6 | same | duplicate interval key | `409 duplicate_key`; no partial publication |
| R7 | same | published segment UPDATE/DELETE | DB trigger rejects; history unchanged |
| M1 | migration integration fixture | up on baseline | all tables/FKs/checks/indexes/triggers exist |
| M2 | migration integration fixture | down empty expanded schema | succeeds and removes marker/objects |
| M3 | migration integration fixture | down with official evidence | refuses without deleting data |
| M4 | migration fixture | legacy work segment/roster | backfill is `legacy_unverified`, no false publication |
| A1 | `schedules_api_test.go/TestValidation` | malformed UUID/date/timezone/oversized range | `400` exact error key |
| A2 | `schedules_api_test.go/TestAuthorization` | employee, team lead, department, employer, global | allowed scope succeeds; out-of-scope `403` before business query |
| A3 | same | stale expected revision | `409 stale_revision`; no replacement |
| A4 | same | preview | `200`, `provisional=true`, no official row |
| A5 | same | backward-compatible GET | old fields remain; no synthetic 09:00–17:00 |
| C1 | `closure_repository_test.go/TestCloseSnapshotsExactRevision` | close month with work/off | every day has exact revision or explicit off snapshot |
| C2 | same | missing coverage | `409 missing_coverage`; no closure |
| C3 | same | open punch | `409 open_punch`; no closure |
| C4 | same | concurrent close/amend | one serial outcome; no mixed snapshot |
| C5 | same | reopen/reclose | old closure immutable; new closure number and snapshot |
| C6 | same | employer transfer | day in exactly one employer closure |
| P1 | `payroll_periods_test.go/TestLockRequiresClosedAttendance` | lock open attendance period | `409 closure_conflict`; payroll remains open |
| P2 | `payroll_test.go/TestLockedPayrollBlocksScheduleAmend` | amendment/recalc after payroll lock | `409 locked_period`; raw punch still inserts |
| N1 | `late_punch_test.go/TestLatePunchDoesNotRewriteClosedResult` | raw punch after closure | raw row accepted; old closure snapshot unchanged; reconciliation signal exists |
| N2 | `legacy_writer_guard_test.go/TestLegacyMutationCannotBypass` | direct roster/segment/bulk writer on official/closed day | `409 locked_period` or guard error; no bypass |
| W1 | `outbox_worker_test.go/TestOfficePublisherRetryAndDeadLetter` | failures at attempts 1–5 | exact delays/status; attempt 5 dead-letters |
| W2 | same | successful retry | one idempotent publication; no duplicate revision |
| F1 | `webdashboard` schedule/attendance route tests | loading, empty, provisional, missing, forbidden, error, history | accessible/i18n states and exact API error handling |
| I1 | API integration target | DB publication → report/no-show/matching/payroll | only official revision consumed; totals not duplicated |
| I2 | API integration target | full lifecycle | preview → publish → amend → close → reopen → reclose → evidence reproduces chain |
| X1 | full target | all negative cases above | no raw punch/evidence deletion, no partial transaction |
| D1 | `schedule_lifecycle_repository_test.go/TestPublishCannotAmend` | publish-only actor targets current work/off; mixed new/existing batch | 409 already_published; no new rows, no supersession; replan without amend is 403 |
| D2 | `schedule_lifecycle_repository_test.go/TestPreviewPersistence` | preview success and insert failure | read-write REPEATABLE READ inserts one provisional row; failure returns no usable ID; no official rows/audit |
| D3 | `schedule_lifecycle_repository_test.go/TestSystemPreviewOwnership` | worker preview, forged HTTP system actor, another user's preview | valid system tuple publishes under approved policy; forged/cross-owner use denied |
| D4 | `schedule_lifecycle_repository_test.go/TestReplayBeforePreconditions` | response lost, preview expired/deleted, day later amended or closed | same actor/key/payload returns exact original 201; revoked permission returns 403; no write |
| D5 | `schedules_api_test.go/TestLateFirstPublication` | missed yesterday or today's shift already started | system refuses; authorized human with reason/retroactive publishes actual time; missing authority denied; closure can then complete |
| D6 | `schedule_lifecycle_repository_test.go/TestLegacyRevisionAllocator` | legacy #1, then first publish; concurrent backfill/publication | next number uses MAX+1, one current, legacy unchanged, previous official link NULL |
| D7 | `shifts_test.go/TestReferencedVersionSupersession` | referenced open-ended V1, future V2 | only effective_to/superseded_by change atomically; no overlap, V2 selected for future preview, old official snapshots unchanged; content update rejected |
| D8 | `schedule_office_worker_test.go/TestCoverageProducer` | empty queue after approval, startup after outage, month rollover, employment event | bounded jobs produced through next month; dedupe works; current/manual/rotating days not replaced; late gaps visible |
| D9 | `schedule_lifecycle_repository_test.go/TestRequestHashCoverage` | same key with changed approval, timezone, replacement or operation | 409 idempotency_conflict; other actor denied; reordered equivalent object keys hash identically |
| D10 | `schedules_api_test.go/TestReplanPreviewContract` | changed replacement/source/current after viewing diff | valid preview confirms exact before/after and commits; changed payload/source returns stale_preview, changed current stale_revision; no partial write |
| D11 | `schedule_office_worker_test.go/TestCrashAfterPublication` | publication commits, worker dies before job completion | lease recovery finds committed batch, marks done, no duplicate revision; obsolete lease cannot update job |
| D12 | `schedule_lifecycle_repository_test.go/TestClosedPeriodAllEntrypoints` | first publish, explicit off, replan, system retry with new key on closed period | each new mutation denied; committed replay remains read-only |

Required commands after implementation, in order:

```bash
./leedevkit test api
./leedevkit test web
./leedevkit test all --json
```

Migration up/down/integrity and fixture validation run through existing LeeDevKit/repository wrappers. Unit-only output never completes the task. This specification does not claim any of these commands has run.

## 16. Open risks and verifiable assumptions

- `resolveEffectiveShift` currently uses elapsed-hour rotation and only `sort_order=0`; verify corrected calendar-date/all-interval implementation with U4/U6.
- Existing source does not show leave overlay reads or a location-timezone query in the schedule resolver; identify the authoritative leave/location tables before task 4 and cover with resolver tests. This is a source verification task, not an architectural choice.
- Existing source has outbox producers and an in-memory worker but no proven PostgreSQL consumer; task 11 must prove startup wiring, claim query and dead-letter metrics before enabling office auto-publication.
- Existing `work_segments` bulk replace deletes rows; task 7 must ensure official revisions never call that delete path.
- Existing `attendance_daily_summaries` has unique `(employment_id,date)` and mutable OT fields; closure snapshots must copy values, not make the summary table immutable or duplicate it.
- Existing approval workflow is polymorphic and current approval processing updates generic status; task 5 must validate `entity_type`, status and effective policy dates in the same transaction.
- Existing authorization has global/department checks but no employer helper; task 8 must verify relationship-at-date scope and fail closed.
- Existing `audit_logs` is partitioned and actor fields include nullable UUID-compatible `actor_id`; migration task must inspect all audit partitions before adding any trigger or column.
- Existing baseline is fresh-only and later migrations use guarded down scripts; migration task must preserve this rollback boundary and test representative data volume before activation.

Review corrections are mapped to D1–D12; the approved long-term direction is detailed in §17 with L1–L12 acceptance. Architecture agreement does not authorize implementation or certify the detailed contracts. No runtime verification or legal compliance certification is claimed.

## 17. Long-term HRM refinement — 2026-11-14

### 17.1 Boundaries and approval

Approved direction: effective-dated assignments, one resolver, published schedule revisions, separate actual/valued/closed attendance. This section refines the corresponding earlier sections; schema/API additions below must be reconciled into machine-readable contracts and ADR before implementation gate 1 passes. Reuse existing tables/workflows when they satisfy these invariants; no second resolver, microservice, event-sourcing platform or stand-alone scheduling dashboard.

| Layer | Responsibility | Must not imply |
|---|---|---|
| Employment | Effective person/employer relationship and scope | Permission to schedule outside employment dates |
| Work assignment | Pattern/rotation/fixed rule with effective range and anchor | Every future day is already published |
| Official schedule | Published work/off, exact version, intervals and authority | Presence, approved OT or employee consent |
| Actual attendance | Original punches and traceable paired sessions | Every recorded minute is payable |
| Valuation | Standard/night/paid absence/approved OT with policy provenance | Overwriting original punches to fit payroll |
| Closure/payroll | Reproducible approved outputs and separate payroll lock | Reopening erases previous evidence |

Resolving effective-dated assignments on demand is valid for previews; it is not inherently bad HRM. This project chooses bounded persisted official schedules because swaps, no-show, multiple intervals and historical disputes need a stable shared contract. Vendor examples do not prescribe internal schema or prove statutory compliance.

### 17.2 Onboarding and weekly UX

- Reuse employment/assignment UI: select fixed weekly pattern, rotation with anchor, or assign later; require start date and validate any end date. Missing assignment does not prevent saving employment or ingesting punches.
- Reuse roster view for weekly/monthly and bulk operations. Calendar cells show official work/off, missing, provisional draft or conflict; expose assignment status separately from publication coverage. No permanent `employee.is_scheduled` flag.
- Draft means editable source inputs and provisional preview, not a durable `schedule_day_revisions.state=draft` record. Only publication transaction stages those revisions. If source inputs cannot represent a required draft edit, prove that gap before adding storage.
- Approved leave overlays obligation without deleting the published shift. Missing punch is an unresolved attendance issue, not automatically confirmed unauthorized absence. Employee sees official assignments; any preview exposure is explicitly provisional and role-controlled.
- Filters: missing official coverage, unpublished candidates, conflicts, and upcoming joiners with missing coverage. Provide authorized owner/action on each gap, existing retry/reconciliation status for failed auto-publication, and accessible loading/empty/error/i18n states.
- A worker missing a schedule and a required shift missing workers are different concepts. Demand/headcount, qualifications, open-shift bidding and automatic optimization remain deferred; do not create demand tables for this phase.

### 17.3 Coverage contract and bounded publication

Expected days = requested planning window intersect employment effective dates and the approved scheduling scope. Office generation further intersects policy effective dates; days with no applicable policy remain visible coverage gaps, not silently removed from the operational list. Explicit off covers a day; missing/conflict does not. Outside employment is not applicable, not off/missing. An end date is inclusive, consistent with existing employment date semantics; changing it must reconcile affected future official days without deleting history.

Office producer keeps §12.3's local today through end of next month horizon and approved authority. Do not fabricate publication for elapsed shifts. Rotating managers select an explicit date range, normally using the weekly view; selected batch is atomic and every targeted day must resolve to work/off. A narrower batch is allowed, but gaps outside it remain visible in the selected planning window. Assignment without end date does not authorize infinite generation or certify future compliance.

Extend existing `GET /api/v1/schedules` additively, rather than adding a dashboard endpoint:
- Bounded `start_date`/`end_date`, existing pagination and permission scope remain required. Add optional `coverage=all|missing|complete`; omission means all. Validate incompatible/unknown values. Apply employee/employment coverage filtering before pagination, not to the current page only.
- Each employment row adds `coverage` with `expected_days`, `published_work_days`, `published_off_days`, `missing_official_days` and `complete`. Counts apply to the full requested eligible range, not visible cells only; `expected_days = published_work_days + published_off_days + missing_official_days`. Zero eligible days must not make an employee appear as a scheduled employee; omit out-of-range employment rows as today.
- Official missing includes candidates still only in draft; GET must not silently resolve them into published work. Detailed draft/conflict diagnostics come from the existing preview contract. Existing published days whose inputs changed remain published; amendment-needed diagnostics are separate from missing coverage.
- Publication changes become visible after commit and normal query invalidation. Multi-query coverage reads use one consistent DB snapshot so counts, filters and cells agree. No mutable duplicate coverage table or employee flag is needed initially.
- Self-service exposes owned employments only; team/department/employer filters cannot expand authority. Tests verify total counts and cross-page filtering without PII leakage.

### 17.4 Shift grouping, timestamps and valuation boundaries

One day revision may contain several shift occurrences. Within it, every segment belongs to one `shift_group_key`, stable in that revision and paired with revision ID for identity; this is not a new shift-occurrence service/table. Source trace links the group to its source assignment/template and prior counterpart when applicable. Segment-level matching uses segment IDs, never business_date alone.

Proposed additive schema/API delta, finalized with migration review:
- Add nonnegative `start_day_offset` and `end_day_offset` to versioned interval definitions; zero means business date, one the following local date. Persist these offsets and `shift_group_key` on official segment snapshots alongside existing timestamptz columns. Legacy columns may be nullable until verified; official content must be complete. Include these fields in preview/diff/canonical hashes and evidence export.
- First segment in each group starts on business date (offset 0); later segments can start after midnight. Validate resulting ordered, non-overlapping intervals and contained break ranges; apply policy duration limits rather than accepting arbitrary multi-day offsets. Concrete timestamp conversions must agree with local date/time/offset and IANA timezone.
- Example: business date 30/04, intervals 22:00–02:00 (offsets 0/1) and 02:00–06:00 (1/1) remain one group, total eight elapsed hours in Asia/Ho_Chi_Minh. Sort order alone cannot establish day offsets.
- Nonexistent DST local times fail validation. Ambiguous local times require an explicit valid UTC offset/disambiguation included in source snapshot/hash; never silently choose one. Automatic publication with ambiguity creates a gap requiring resolution. UTC fallback must remain visible in trace and undergo timezone coverage validation before activation.
- Intervals use half-open bounds `[start,end)`. Touching endpoints are not overlaps, but rest-period rules still apply. No automatic equivalence between a break gap and paid standby.
- Attendance grouping uses business date; valuation splits actual eligible intervals at holiday/night/rate and payroll-calendar boundaries. Preserve references to original schedule/segment/calculation; splits conserve minutes and money subject to explicit rounding policy. Cross-month payment slices may reference the preceding month's schedule/closure and must not require a fabricated schedule for the calendar date after midnight. Before payroll lock, all source closures referenced by its slices must be closed and exact IDs recorded; reopening a source closure is blocked while any dependent payroll period remains locked.
- Existing one-employer-per-closure-day remains for supported cases. Mid-shift employer transfer with ambiguous attribution is a blocking reconciliation case until HR/legal approves a slice allocation contract and any required schema change. Do not assign the whole shift twice or silently choose one employer. Finalized/paid payslips are preserved; corrections use approved adjustment/off-cycle mechanisms, not overwrite.

### 17.5 Person-level conflicts and transaction safety

Publication, replan and swaps validate the resulting intervals against all current official intervals for the same employee across employments/employers and business dates, plus every candidate interval in the batch. Exclude only revisions explicitly being atomically replaced; never exclude the other employment just because it is outside the caller's display scope. Compare real timestamps, not only date equality. Load enough history for full week/month limits and configured recovery/rest windows; raw future open-ended assignment is not a validated full future calendar.

Employee locks precede employer/period/employment locks as updated in §9.1. Schedule-affecting employment creation, transfer or employee reassignment must acquire the same employee locks; otherwise a new relationship could evade conflict checking. All code paths lock the complete sorted set before mutation; bulk operations and two-sided swaps use one transaction. Do not use per-employment uniqueness as proof of person-level non-overlap.

Cross-scope conflict evaluation is an internal integrity operation, not a grant to read other schedules. Return a generic scoped conflict code without other employer names, person details or hidden shift payloads. Closure over a month must discover and stabilize its complete employee set before acquiring later locks; if membership changes, abort/retry from the beginning. DB integration tests must prove no deadlock/lost update under opposing transfers and cross-employer publication.

Hard overlaps and applicable statutory prohibitions cannot be bypassed by `allow_warnings` or a reason. Only explicit policy warnings allow override when `AllowManagerOverride` and actor authority both permit it. On-call/standby coexistence, if needed, requires an approved activity compatibility rule; default is to block conflicts, not a generic override escape hatch.

### 17.6 Authority, source changes and closure dependencies

The role matrix in §8 is a maximum scope, not an unconditional grant. Enforce action permissions and state-dependent rules: technical admin alone cannot approve HR changes; started shifts or shifts with punches require authorized HR approval for changes to work obligation. Reopening requires an independently approved request and cannot be self-approved. Reuse generic approval instances only after verifying entity binding, expected revision and transaction behavior; add contract fields before implementation rather than treating generic approval status as execution.

Voluntary swaps require target consent, manager approval and fresh validation of both workers' revisions, employment validity, leave and compliance at execution. Commit both replacements atomically; stale schedule or approval causes rollback. Notification is separate from publication and consent; do not claim delivery/read receipts without evidence. Ordinary schedules do not require blanket employee acknowledgement unless policy/law demands it.

Future-effective assignment/template changes may update previews and enqueue coverage refresh but never amend published content implicitly. Do not block a valid future-only policy change merely because unrelated earlier attendance is closed. Changes that alter an official obligation use replan; changes that affect closed calculations require authorized reopen. Every dependent payroll lock, including a cross-month slice dependency (§17.4), must be checked. Raw device ingestion remains allowed and queues reconciliation.

### 17.7 Implementation mapping and acceptance

Reuse §14 tasks; no separate parallel implementation plan:
- Tasks 1–3: finalize ADR, grouping/offset/API/approval deltas, employee lock order and cross-month dependencies; verify all required existing fields/workflows. Do not activate unresolved cases.
- Task 4: H1/H2/H5 and offsets, person-level conflict inputs, leave/calendar/compensation semantics; cover source validation before publication.
- Tasks 5–6 and 12: shared publication/amend/swap contract, state-sensitive authority, coverage queries and guards, no bypass through legacy writers.
- Tasks 7–10: H3/H6/H7, exact segment pairing, grain-safe reports, approved valuation and closure/payroll dependencies; preserve zero compensation intentionally supplied.
- Task 11: bounded producer, employment/policy events, failed publication visibility and no automatic rewrite of published schedules.
- Task 13: onboarding and weekly coverage UX, bulk operations, authorized gap filters and state labels; no new dashboard.
- Tasks 14–15: real DB, migration, concurrent writes, full API/UI workflows and complete suite. Test source facts again on implementation HEAD; previous static findings are not runtime reproductions.

Add these acceptance scenarios to §15; H1–H7 remain required alongside these extensions. Test names are planned, not files claimed to exist.

| ID | Suggested test / scope | Scenario and expected result |
|---|---|---|
| L1 | `TestOnboardingEffectiveCoverage` / API + UI | Join Wednesday; Monday/Tuesday not expected; weekend explicit off counts; assign later saves employment and exposes missing days without rejecting punches |
| L2 | `TestAssignmentIsNotPublication` / API + UI | Active pattern with no official revision remains missing; draft candidate visibly provisional; published predecessor remains official after source change |
| L3 | `TestCoverageFilterBeforePagination` / DB + API | Missing employee lies on later page; server filter/total selects correctly; counts/cells share snapshot, terminated/out-of-range excluded, no unauthorized rows |
| L4 | `TestCrossEmploymentOverlap` / DB concurrency | Same person, different employers concurrently publish overlapping shifts; only one commit, generic conflict for loser, no scope leakage or partial batch |
| L5 | `TestAdjacentNightShiftAndRest` / domain + DB | 22–06 vs 05–13 conflicts across dates; touching 06 endpoints not overlap but rest rule evaluated; full weekly/monthly history and override prohibition enforced |
| L6 | `TestMultiSegmentDayOffsets` / domain + API | 22–02 + 02–06 retains business date/group, all timestamps and eight hours; invalid equal clock/offset/break/DST inputs rejected or explicitly disambiguated |
| L7 | `TestSplitShiftAttendanceConsumers` / DB integration | Two segments/two sessions give correct totals/pagination; missing second shift still visible; overnight punches matched by segment, no fabricated hours |
| L8 | `TestZeroCompensationAndPartialLeave` / domain + DB | Explicit 0 remains 0; omission alone defaults; Sunday worker, weekday off, partial/overnight leave and multiple breaks use eligible intersections without double deduction |
| L9 | `TestSwapConsentAndAtomicReplan` / API + DB | Missing consent/HR approval where required denied; stale or overlapping swap rolls back both; success updates official reader chain and preserves prior revisions |
| L10 | `TestCrossMonthValuationDependencies` / payroll integration | Holiday/night/month split conserves minutes, references exact originating closure; locked dependent payroll blocks reopening; paid original not overwritten |
| L11 | `TestLateDataAndIndependentReopen` / API + DB | Late/out-of-order/duplicate punches preserved or deduplicated by defined identity, closed snapshot unchanged; self-approved reopening rejected; valid reopen/reclose retains evidence |
| L12 | `TestActivationReconciliation` / migration + role E2E | Source changes, office horizon refill, missing/conflict cases, legacy provenance and all reader/writer paths reconcile; employee/team_lead/shift_manager/hr_manager/admin obey documented scope; unresolved employer-transfer/DST cases block affected scope |

### 17.8 Remaining implementation gates and references

Before code: HR/legal must validate actual jurisdiction-specific limits/rates and special employee categories; resolve mid-shift employer attribution if required for launch. Engineering must reconcile exact grouping/offset constraints, approval fields, coverage DTOs and payroll closure dependency persistence into §§5–9 and machine-readable contracts. These are explicit gates, not permission to guess, omit safeguards or claim unsupported cases work. Future features (staffing demand, skills optimizer, bidding) stay outside this delivery.

External references support distinctions, not mandatory schema or legal certification:
- Microsoft Graph shift resource: draftShift versus sharedShift — https://learn.microsoft.com/en-us/graph/api/resources/shift?view=graph-rest-1.0
- Odoo Planning: templates, recurring shifts, open shifts and publish — https://www.odoo.com/documentation/19.0/applications/services/planning.html
- Frappe HR Shift Assignment: effective ranges and attendance linkage — https://docs.frappe.io/hr/shift-assignment

Documentation verification only: `git diff --check -- schedule-lifecycle.md schedule-lifecycle-spec.md`. Runtime commands in §15 remain unexecuted until implementation; approval of architecture is not a test result.

## 18. Shift/policy ownership refinement — 2026-11-14

Direction approved for planning: shift defines planned work/rest timing; attendance and valuation policies define interpretation, payable time and OT treatment. This section refines prior references to shift-level break/OT configuration; migrate ownership before finalizing lifecycle schema/contracts. It does not authorize code, automatic migration of ambiguous values, legal thresholds or deletion of historical evidence.

### 18.1 Verified entry points and reuse

Read current source again at implementation HEAD; these observations are static, not proof of end-to-end wiring:
- `apiserver/internal/api/shifts.go:34`: interval DTO contains break timing alongside auto-deduction, early grace, early OT, aggregation and night flags.
- `apiserver/internal/api/attendance_policies.go:31`: attendance rules already contain work unit, rounding, early-arrival strategy and grace. Do not introduce a second competing attendance-policy configuration.
- `apiserver/internal/api/overtime.go:41`: OT rule currently exposes rate/time/day/holiday criteria; additional eligibility/aggregation fields are not assumed implemented. Existing request/consent/claim/approval flows remain distinct from policy settings.
- `apiserver/internal/api/work_policy.go:26`: work-policy group references attendance, working-pattern, OT, work-credit and pay-basis policies with effective dates. Reuse this composition and verify resolution/membership semantics before routing all consumers through it; a DTO alone does not prove runtime use or child-policy immutability.
- `webdashboard/src/pages/shifts/components/interval-draft-editor.tsx:73`: UI derives night flag from start > end; `grace-ot-section.tsx:20` groups grace and OT; `break-config-section.tsx:81` presents deduction/no-deduction/OT as exclusive choices. Refactor semantics, not only move UI controls.

### 18.2 Field ownership and target meaning

| Existing field/concept | Owner and target | Migration/validation condition |
|---|---|---|
| Name, report symbol, interval name/order, effective version | Shift template/version | Display metadata never determines monetary rates or payroll employer |
| Start/end time and local-day offsets | Versioned shift intervals | All segments/breaks follow §17.4; do not equate sort order with calendar date |
| Fixed break start/end | Planned break within shift | Derive duration from resolved timestamps; reject supplied contradictory duration |
| Flexible break duration | Planned break within shift | Separate fixed/flexible semantics; optional allowed window, no invented actual break or punch |
| `location_id`, `employer_id` | Template applicability/default context | Actual work location is captured in assigned schedule; payroll employer comes from effective employment. Default is not authorization or immutable payroll ownership |
| `early_punch_in_grace_period_minutes` | Split by proven meaning: matching window versus lateness tolerance | Trace actual consumers and precedence before mapping; similar names are not proof of identical meaning |
| Attendance `early_arrival_strategy` and lateness grace | Attendance policy | Keep matching eligibility, violation tolerance and time valuation separate; preserve raw punches |
| `is_break_auto_deduct` | Attendance/valuation policy | Deduction method is distinct from paid/unpaid entitlement; false must not implicitly mean paid break |
| `allow_early_ot`, `max_early_ot_minutes` | OT eligibility policy | Candidate time is not approved/payable OT; separate configuration cap from statutory limits |
| `break_is_overtime` | OT eligibility plus actual evidence and approval | Paid rest, unrecorded rest and working during rest are different states; never approve OT from template boolean alone |
| `ot_aggregation_mode` | OT valuation policy at defined aggregate scope | Declare segment/shift/day scope and rounding order; reject incompatible policies at a shared aggregate boundary rather than selecting one interval's setting |
| `is_night_shift` | Replace arithmetic role with derived temporal attributes | Cross-midnight from offsets/timestamps; eligible night minutes from policy night window; optional display tag cannot determine pay |
| Segment compensation/gap rate and break/OT overrides | Preserve event-level facts; valuation owns entitlement/rates | Audit consumers; explicit zero remains zero. Legacy free-form override JSON cannot override locked statutory/approval rules; no generic override hierarchy added |

Break definition may classify planned rest type for policy matching; it must not duplicate the pay/deduction/approval algorithm. No standalone break-policy service/table is required unless existing policy structures cannot express an approved requirement. Night rules can reuse the existing valuation/OT domain but must support ordinary night work independently of OT eligibility; night work is not automatically overtime.

### 18.3 Single policy authority, precedence and evidence

- Resolve the effective work-policy membership/group for employment and business date, then effective child policies and any explicitly approved time-of-day/rate boundaries. Reuse existing policy resolution semantics; document and test the interaction with current user/group/default policy assignments before cutover. Overlapping equal-authority bindings are conflicts, not latest-row wins.
- For activated scope, work-policy resolution is the single entry contract. Existing defaults or assignments may remain explicit inputs of that contract, not a second fallback in each consumer. No implicit `shift > employee > department > company` merge and no arbitrary per-field overrides. A demonstrated shift-dependent need should be an explicit policy criterion with deterministic priority/effective dates.
- Missing required valuation policy blocks valuation/closure and surfaces a setup issue, never silently grants zero/full pay. Missing policy required for schedule validation blocks publication. Whether a non-scheduling pay-policy gap permits publication must be explicit in the gate-1 contract; raw data remains ingestible in either case.
- Publication snapshots the schedule version and the policy inputs used to validate/interpret that publication. Valuation separately records exact policy versions/content, source membership, calculation algorithm version, approvals and source schedule revision used for each result. Do not assume a group version ID freezes mutable child policies; snapshot immutable content or reuse/add child versioning only where required.
- Future policy changes do not mutate published shift timing. Revaluation in an open period creates traceable changed results using explicit effective policy selection; differences from publication-time assumptions remain visible. Closed/paid results remain protected by closure/payroll guards and controlled correction. Law/policy changes must not be ignored solely because an earlier schedule was published.
- A shift swap changes schedule, not employee pay-basis or policy-group membership. Re-resolve applicable criteria for the new times under that employee's policy. Every caller (preview, publication validation, matching, attendance summary, reports, payroll) receives the appropriate phase's policy evidence through the shared contract, not a private reinterpretation of legacy flags.
- Permissions stay separate: `shifts:manage` alone must not grant attendance/OT/payroll policy management. Policy links shown from shift UI obey their own read/manage permission and scope; no salary/private policy leakage.

### 18.4 UX and API contract delta

Shift editor keeps identity/applicability, version/effective date/reason, intervals with day offsets, planned breaks and timeline. Rename cross-midnight display accordingly; show derived night overlap only with a known policy context. Distinguish planned span, planned breaks, planned work duration and estimated payable time; estimates are not approved payroll.

Policy controls move to existing attendance/OT/work-policy screens with current role boundaries. In shift editor show an optional read-only applied-policy summary only when employment/date are selected; otherwise show that policy depends on assignment/date. No claim that one policy applies universally to everyone using a template. Do not add a new settings dashboard or one policy copy per shift.

Before implementation, update OpenAPI/DTOs/generated client: fixed/flexible break validation, moved policy fields, cross-midnight/night semantics, version/effective dates, source/provenance and phase-specific error handling. Omitted, false, zero and null must have distinct documented meanings. Legacy payload fields cannot be silently ignored: before cutover use explicit legacy semantics; after cutover reject unsupported writes with documented validation/compatibility response or translate only through an approved mapping with policy permissions and atomic audit. Follow API Governance/Change Management compatibility gates; no arbitrary forced removal of published contracts.

### 18.5 Migration and ordered work (reuse §14)

1. **Inventory before schema freeze (tasks 1–4):** map every old field's read/write callers, actual runtime behavior, defaults, policy precedence and existing tests. Include shift/interval/segment overrides, imports, bulk operations, API clients and historical snapshots. Record unresolved semantics and sample conflicting configurations without PII; never infer grace purpose from its name.
2. **Contract and minimal expansion (tasks 1–3):** finalize §18.2 ownership; extend existing attendance/OT structures only for missing concepts; prove group/child-policy temporal evidence. Separate publication assumptions from valuation evidence in §§5–9. Agree supported policy combinations and required permissions; no universal rule engine.
3. **Explicit mapping (tasks 2,4–6):** generate restartable migration records mapping legacy source/version/field/value to target policy/version/effective range and provenance. Reuse identical policy definitions only when semantics and applicability match; do not assign a migrated policy to unrelated users. Divergent same-day interval aggregation, ambiguous grace or unclear break entitlement needs HR review. Leave raw/closed historical evidence untouched; imported knowledge is marked as migration, not fabricated original approval.
4. **Shared resolution and comparison (tasks 4–10,12):** switch code to one policy entry contract behind controlled rollout. Run old/new calculations on representative same inputs without publishing duplicate results. Require parity for intended-equivalent behavior; each intentional bug correction has an approved expected delta and regression test. Unexplained differences block activation; do not preserve a known bug just to achieve parity.
5. **UI and synchronized cutover (tasks 6,11–13):** update editor/settings and API clients, all writers/readers, background jobs and snapshots together per supported scope. Stable existing official schedule/closed results remain unchanged. After evidence, fix forward; no fallback to legacy writes that bypass policy authority. Before evidence/activation, rollback only if no new data would be lost.
6. **Retire and verify (tasks 14–15):** legacy fields/read adapters stop only after compatibility period, zero active legacy consumer, reconciled mappings and evidence replay. Any physical drop needs separate approval and guarded migration; keep historical snapshot fields. Run unit then full API/web and cross-target suites, migration and role-scoped E2E. No test skip or mock-only mapping test completes cutover.

### 18.6 Acceptance additions

B1–B10 supplement H1–H11/L1–L12 and §15, not replace them. Proposed symbols are implementation targets, not tests already present.

| ID | Planned scenario | Expected evidence |
|---|---|---|
| B1 | `TestShiftPolicyOwnership` / API + UI | Shift form contains timing/planned-rest only; shift-only manager cannot alter pay/OT policy through legacy or new payload; contextual policy summary does not leak restricted data |
| B2 | `TestMatchingGraceAndEarlyTimeAreIndependent` / domain + integration | Distinct punch matching buffer, lateness grace and early-arrival valuation produce independent outputs; raw timestamps unchanged; migration ambiguity rejected |
| B3 | `TestFixedAndFlexibleBreaks` / domain + API | Fixed 12–13 derives 60 minutes; conflicting 45 rejected; flexible 45 has no invented 12–12:45 timestamps; overnight/multiple breaks stay valid |
| B4 | `TestPaidBreakIsNotApprovedOT` / valuation integration | Paid rest/no auto-deduct alone creates no OT; work-during-rest needs evidence and required approval; no double deduction or double payment |
| B5 | `TestOTAggregationScope` / domain + DB | Two intervals use deterministic approved aggregate/rounding scope; incompatible interval/policy settings conflict; early cap enforced without erasing actual time |
| B6 | `TestNightWindowVersusCrossMidnight` / domain + API | Under fixture night window 22–06, 18–23 has 60 night minutes and 00–08 has 360 despite neither crossing midnight; offsets drive day crossing, flag never sets pay |
| B7 | `TestPolicyTemporalEvidence` / DB integration | Child policy changes after publication do not alter closed result or shift timing; open revaluation records exact new policy inputs; group ID alone cannot hide mutable changes |
| B8 | `TestSwapPreservesEmployeeEntitlements` / integration | Employees sharing/swapping shift retain their own effective policy-group/pay basis; new times re-evaluate applicable criteria; shift employer metadata cannot reassign payroll employer |
| B9 | `TestPolicyMigrationParityAndProvenance` / migration + DB | Explicit 0/false versus missing preserved; repeat migration idempotent; ambiguity blocks affected scope; approved corrections show expected delta; legacy snapshots replay unchanged |
| B10 | `TestPolicyCutoverAllConsumers` / API + worker + role E2E | Imports/bulk/jobs/old clients cannot bypass authority; all activated consumers use shared phase-appropriate policy contract; unexplained delta aborts activation, no silent fallback |

Pending detailed-contract review: actual meaning of legacy grace and precedence, flexible break evidence model, valid aggregate combinations and temporal child-policy representation. These are bounded engineering/HR checks before gate 1, not defaults to invent. Documentation-only update; implementation and runtime tests remain unexecuted.

# Schedule Lifecycle Implementation Specification

Status: Approved

## 1. Goal và non-goals

Goal: biến kết quả resolver thành official schedule có revision bất biến theo `employment_id + business_date`, có publication/replan/closure/evidence giao dịch, rồi chuyển reader/writer hiện hữu sang một official contract.

Non-goals:
- Không tạo microservice, event-sourcing platform, resolver thứ hai hoặc payroll lock thứ hai.
- Không thay đổi raw punches, manual exceptions, approved leave, OT approval hoặc payslip như side effect của schedule publication/amendment.
- Không coi thiếu lịch là ngày nghỉ.
- Không dùng notification delivery hoặc acknowledgement làm bằng chứng publication.

Planning only. Chưa có bảng/API/permission nào dưới đây trong source hiện tại trừ các reuse point được ghi rõ.

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
- **Business date:** ngày lịch trong timezone của employment/location, không phải ngày UTC của punch.
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

Current source does not contain official schedule, attendance closure, schedule publisher, or outbox consumer. These are planned additions, not existing capabilities.

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

Every official `work_segments` row has `schedule_revision_id`, `selected_shift_version_id`, `scheduled_start_at`, `scheduled_end_at`, `breaks_snapshot`, and the legacy `scheduled_start`/`scheduled_end` values for compatibility. `scheduled_start_at` and `scheduled_end_at` are concrete timestamptz values built from local business date/timezone. `scheduled_end_at > scheduled_start_at`; overnight end uses next local calendar date. `breaks_snapshot` is an ordered JSON array containing every break's local start/end, concrete timestamps, duration and deduction/overtime flags.

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

`payroll_periods` remains the only payroll lock table. `LockPayrollPeriod` locks employer/year/month, verifies attendance period is closed, then performs existing payroll lock audit in the same transaction. `ReopenPayrollPeriod` keeps existing reason/version behavior and does not erase closure. Schedule amendment, attendance reprocess, payroll calculation and payslip recalculation reject while payroll status is `locked` with `locked_period`.

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
3. Employer rows ordered by UUID: `SELECT id FROM employers WHERE ... AND deleted_at IS NULL ORDER BY id FOR UPDATE`.
4. Payroll period rows: insert if absent, then lock ordered by `(employer_id,period_year,period_month)`.
5. Attendance period rows: insert if absent, then lock in the same period order.
6. Employment rows ordered by UUID `FOR UPDATE`; current revisions ordered by employment/date `FOR UPDATE`. Allocate revision numbers over all states while holding these employment locks.
7. Selected shift template rows ordered by UUID `FOR SHARE`, then read effective versions. Shift-version writers use template `FOR UPDATE` before version locks; if they also need lifecycle locks, they acquire steps 3–6 first.
8. Preview row `FOR UPDATE` for a new publish/replan. Revalidate ownership/operation/expiry and current inputs. Cleanup only locks preview rows and never domain rows.
9. Raw attendance/time entries and attendance summaries for close/reprocess, ordered by employment/date/ID `FOR UPDATE`.
10. Domain writes, synchronous audit INSERT, commit. Audit failure rolls back every domain write.

Close/reopen begin at step 3, skip inapplicable classes, and recheck period status after locks. Payroll locking and every protected attendance writer use the same relative order. No worker holds an outbox claim lock while entering this sequence.

Preview uses a **read-write `REPEATABLE READ`** transaction: read authorized source inputs and current revisions from one consistent snapshot, compute result/diff, INSERT only `schedule_previews`, commit, then return its ID/hash. It does not create official revisions, segments, publication batches or publication audit events; it takes no domain mutation locks. Preview is advisory: publish/replan reloads inputs and compares hashes. A crash before commit exposes no usable preview ID.

### 9.2 Publish

Authenticate, validate DTO and authorize all target scope; derive actor server-side. Normalize payload/hash, begin transaction and perform steps 1–2. For a new operation, lock domain rows and preview, revalidate policy and source hashes. All targets must have no current official revision and an explicit `expected_revisions[employment_id:date]=0` token (0 means no current, not next revision number). Any existing current revision returns `409 already_published`; the entire batch rolls back. **Publish never supersedes a revision**, regardless of actor's amend permission. Replace published work/off only through replan.

Require payroll open, attendance open/reopened, complete work/off resolution and the retroactive rules in §7.2. Insert batch, allocate revision numbers, insert draft revisions and their complete segments, transition these new drafts to published/explicit_off, insert audit and the immutable response snapshot, commit. `published_at` is the server's actual publication timestamp, never supplied by the client. Copy source/decision snapshots into permanent evidence; preview expiry cannot erase them.

### 9.3 Amend and close

Replan requires `schedules:amend`, reason, Idempotency-Key and a replan preview/diff confirmation (§7.5). Use §9.4 replay first; for a new operation take the same lock sequence, require matching expected current revision and diff/result hashes, payroll open and attendance open/reopened. Allocate MAX+1, insert replacement batch/revision/segments, supersede only the expected current revision without changing its published_at/content, publish replacement, write audit and response snapshot, commit. Inherit mode and approval reference from the original batch; snapshot the human amendment authority separately in source_snapshot. Replan is not a new system office authorization and cannot silently expand an approved policy.

Close locks employer, payroll period, attendance period, all employment/day rows and raw/summary rows. It checks coverage/open punches/approvals, inserts closure and closure-day snapshots, updates period, inserts audit, commits. It does not update attendance_daily_summaries or raw punches.

Reopen locks employer, payroll period and attendance period; checks payroll open and expected version; updates only period status/version/updated_at, retaining current_closure_id. Insert `attendance.reopened` into existing audit_logs in the same transaction with period ID, closure ID, old/new period versions and reason; do not create another transition table or mutate closure rows. Reclose creates a new numbered immutable closure.

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
9. Load all shift default intervals ordered by `sort_order`; when no interval rows exist, create one virtual interval from shift version default start/end. Preserve every interval and break configuration.
10. For local `end_time <= start_time`, end date is next local calendar date. Breaks crossing midnight use same local-date rule. Store exact `scheduled_start_at`/`scheduled_end_at` and break timestamps.
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

Review corrections are specified and mapped to D1–D12 acceptance cases. This document remains Awaiting approval; these contracts are not proof of implementation, passing integration tests, or absence of further review findings.

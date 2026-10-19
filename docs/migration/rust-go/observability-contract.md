# Observability Contract — Rust → Go (P2-06)

**Scope:** `apiserver` HTTP requests, tracing context, application logs, PII handling, operational metrics, health probes, and audit events. Rust remains compatibility baseline; this document freezes observable behavior for Go. Unknown or conflicting behavior is listed in §8, not silently selected.

**Dependencies:** P1-10 deployment contract, P1-11 privacy contract, P1-02 middleware contract, `.ai/ADRs/0002-go-technology-stack-apiserver.md`.

## 1. Compatibility baseline

| Surface | Rust evidence | Frozen compatibility rule |
|---|---|---|
| Production log encoding | `apiserver/src/main.rs:88-105` selects `ProductionJsonFormatter` only when `APP_ENV=production`; otherwise default formatter | Go production output MUST be one JSON object per line. Human-readable output MAY be development-only. |
| Request ID middleware | `apiserver/src/lib.rs:351-365`; `apiserver/src/mw/request_id.rs:14-48` | Middleware MUST run outermost, before rate limit/tracing. Every HTTP response, including 404 and health responses, MUST carry `x-request-id`. |
| Health paths | `apiserver/src/routes/auth.rs:23-29`, `apiserver/src/lib.rs:351-352` | Rust public paths are `/api/v1/health` and `/api/v1/health/ready`. Preserve these paths during parity testing. |
| HTTP metrics middleware | `docs/migration/rust-go/middleware-contract.md:339-344` | Rust has no HTTP Prometheus/metrics middleware. Go MUST NOT claim Rust metric parity for a newly exposed `/metrics` endpoint without an explicit decision. |

## 2. Request ID and trace propagation

### 2.1 HTTP request ID

| Rule | Required behavior | Evidence |
|---|---|---|
| Header name | Case-insensitive HTTP header `x-request-id` (`X-Request-Id` on the wire is equivalent) | `apiserver/src/mw/request_id.rs:6,15` |
| Incoming valid ID | Parse incoming value as UUID and retain it | `apiserver/src/mw/request_id.rs:17-23` |
| Missing/invalid ID | Generate UUIDv7 (`Uuid::now_v7()`); invalid values are not echoed | `apiserver/src/mw/request_id.rs:17-25` |
| Downstream request | Rewrite request header to canonical generated/retained UUID | `apiserver/src/mw/request_id.rs:27-30` |
| Response | Rewrite response header to same UUID | `apiserver/src/mw/request_id.rs:43-48` |
| Span/task context | Create `request` span with `request_id`; scope same string in `CURRENT_TRACE_CONTEXT` | `apiserver/src/mw/request_id.rs:32-41`; `apiserver/src/utils/logging.rs:267-275` |
| Background job | Copy current request ID into job `trace_context`; outbox stores `_trace_context`; durable job derives UUID `request_id` and raw `trace_id` | `apiserver/src/queue/job/mod.rs:54-79`; `apiserver/src/services/outbox_job_service.rs:25-35`; `apiserver/src/services/infra_job_service.rs:34-54` |
| Worker continuation | Restore job context before processing; persist it on lifecycle transitions | `apiserver/src/worker/runner.rs:377-389,506-535` |

**Go rule:** `request_id` MUST be stored in `context.Context`, copied to DB/Redis/job/inter-service operations, and returned in every HTTP response. Preserve UUID input/generation semantics. Do not log bearer, cookie, refresh, or access tokens while propagating context.

### 2.2 Span and distributed trace fields

`SpanContextLayer` inherits `request_id`, `workspace_id`, and `actor_id` from parent spans and updates them on span records (`apiserver/src/utils/logging.rs:37-119`). The production formatter emits the current numeric tracing span ID as `span_id` (`apiserver/src/utils/logging.rs:194-214,226-238`). Rust request middleware does **not** parse or emit W3C `traceparent`; durable jobs call the request-context string `trace_id` (`apiserver/src/models/diesel/infra_job.rs:9-30`).

**Frozen Rust-parity rules:**

1. `request_id` is the client-visible correlation ID; preserve it independently from any OTel trace ID.
2. `span_id` is optional and present only while a tracing span is current.
3. `workspace_id` and `actor_id` are conditional context fields; never invent them for unauthenticated/systemless requests.
4. Background jobs MUST retain originating request context when spawned from HTTP.
5. Go MAY add W3C OTel propagation because selected migration architecture requires OTel (`rust-go-migration-plan.md:31-33`; `.ai/ADRs/0002-go-technology-stack-apiserver.md:81-87`), but accepting `traceparent`, choosing `trace_id` wire format, and exposing W3C IDs are unresolved additive decisions (§8). They MUST NOT replace `x-request-id`.

## 3. Structured JSON logs

### 3.1 Required envelope

Production log entry is one JSON object with these fixed keys:

| Key | Requiredness/type | Frozen rule | Evidence |
|---|---|---|---|
| `timestamp` | REQUIRED, UTC ISO-8601 with milliseconds | Generate at event formatting time; Rust uses RFC3339 UTC with millisecond precision | `apiserver/src/utils/logging.rs:191-193,219-224` |
| `level` | REQUIRED, `ERROR`, `WARN`, `INFO`, `DEBUG`, or `TRACE` | Preserve tracing metadata level string; target tests compare exact value | `apiserver/src/utils/logging.rs:216-223`; `.agent/rules/observability-rules.md:21-33` |
| `message` | REQUIRED string | Use event message; absent message serializes as empty string | `apiserver/src/utils/logging.rs:128-149,216-223` |
| `service` | REQUIRED string | Rust value is `apiserver`; Go value MUST identify API service consistently | `apiserver/src/utils/logging.rs:219-224` |
| `request_id` | REQUIRED for request-scoped events; conditional for startup/background events | Use propagated UUID string, never token or raw header dump | `apiserver/src/utils/logging.rs:194-238`; `.ai/ADRs/0002-go-technology-stack-apiserver.md:83-87` |
| `workspace_id` | REQUIRED when tenant context exists; otherwise omitted | Inherit from `SpanContext`; do not use request path text as substitute | `apiserver/src/utils/logging.rs:40-44,199-235` |
| `actor_id` | REQUIRED when authenticated actor exists; otherwise omitted | Inherit from `SpanContext`; use UUID only | `apiserver/src/utils/logging.rs:40-44,199-235` |
| `span_id` | REQUIRED when tracing span exists; otherwise omitted | Rust formatter uses current tracing span numeric ID, not guaranteed W3C ID | `apiserver/src/utils/logging.rs:194-214,226-238` |
| `error.type` | REQUIRED on `ERROR` events when an error field exists | Extract `error` or `error.type`; classify errors, do not serialize secrets | `apiserver/src/utils/logging.rs:240-248`; `.agent/rules/observability-rules.md:18-19` |
| `fields` | OPTIONAL object | All non-envelope event fields go under nested `fields`; field values pass centralized masking | `apiserver/src/utils/logging.rs:123-170,251-255` |

Rust's formatter does not add `request_id`, `workspace_id`, or `actor_id` when no span context exists. Startup configuration is logged after tracing initialization (`apiserver/src/main.rs:108-109`), but events before tracing initialization can use stderr (`apiserver/src/main.rs:66-82`). Go MUST preserve the startup fatal-config distinction: no secret values in stderr or logs.

### 3.2 Levels and production filter

| Level | Use | Required examples | Evidence |
|---|---|---|---|
| `ERROR` | Request/system cannot be fulfilled; human action may be required | Readiness DB/Redis failure; audit persistence failure | `apiserver/src/handlers/common.rs:531-553`; `apiserver/src/services/audit_service.rs:61-68` |
| `WARN` | Degraded, recovered, suspicious, or rejected activity | Locked/failed login; rate-limit or cache fallback | `apiserver/src/handlers/auth.rs:151-170,198-205`; `.agent/rules/observability-rules.md:24-30` |
| `INFO` | Significant lifecycle/business event | Export job created; pool startup; successful state event | `apiserver/src/handlers/report_export.rs:178-189`; `apiserver/src/diesel_pool/core.rs:120-124,177-177` |
| `DEBUG` | Detailed technical diagnostics | HTTP trace layer and cache details | `docs/migration/rust-go/middleware-contract.md:127-135`; `.agent/rules/observability-rules.md:24-33` |
| `TRACE` | Framework-level verbose detail; never production | No production enablement | `.agent/rules/observability-rules.md:29-33` |

Production policy is `INFO` and above; `DEBUG` is targeted-only and `TRACE` is forbidden (`.agent/rules/observability-rules.md:32-33`). Current Rust filter defaults to `apiserver=debug,tower_http=debug` when `RUST_LOG` is invalid/unset (`apiserver/src/main.rs:88-93`); this filter/policy mismatch is unresolved (§8), not silently normalized.

## 4. PII and secret masking

### 4.1 Central log-field behavior

`mask_pii_value` lowercases field keys and applies these exact rules (`apiserver/src/utils/logging.rs:15-34`):

| Field-key match | Output | Frozen compatibility rule |
|---|---|---|
| Contains `email` | `mask_email(value)` | Preserve exact vectors in §4.2 |
| Contains `phone` | `mask_phone(value)` | Preserve exact vectors in §4.2 |
| Contains `bank_account` or `account_number` | `mask_bank_account(value)` | Preserve exact vectors in §4.2 |
| Contains `password`, `hash`, `secret`, `token`, `jwt`, `key`, or `national_id` | `[REDACTED]` | T1 values MUST never be raw at any level |
| Other string key | Original value in Rust | Go MUST apply classification-safe centralized filtering; do not rely on caller discipline |

The formatter applies this visitor to event fields, not the `message` value (`apiserver/src/utils/logging.rs:128-149`). Therefore Go MUST reject or sanitize raw PII before interpolation into `message`; a field-only filter is insufficient for the P1-11 rule that T1 never appears in logs (`docs/migration/rust-go/privacy-contract.md:9-18`).

### 4.2 Exact masking vectors

| Function | Input → output | Evidence |
|---|---|---|
| `mask_email` | `john.doe@example.com` → `joh***@example.com`; `lee@gmail.com` → `l***@gmail.com`; `me@test.com` → `***@test.com`; `a@b.com` → `***@b.com`; `longemailaddress@corp.com` → `lon***@corp.com` | `apiserver/src/utils/masking.rs:1-24,47-58` |
| `mask_email` invalid input | No `@`/not exactly two parts: length ≤2 → `***`; otherwise first two bytes + `***` | `apiserver/src/utils/masking.rs:2-10` |
| `mask_phone` | `0987654321` → `098****321`; `12345` → `***`; `1234567` → `123*567` | `apiserver/src/utils/masking.rs:26-35,60-65` |
| `mask_bank_account` | `1234567890` → `******7890`; `123` → `****`; `12345` → `*2345` | `apiserver/src/utils/masking.rs:37-45,67-72` |

Masking uses byte slicing in Rust (`apiserver/src/utils/masking.rs:9,18-20,31-34,42-44`); Go MUST define UTF-8/non-ASCII behavior before accepting non-ASCII test vectors. This is unresolved (§8).

### 4.3 Existing sanitization outside logger

Worker monitor error summaries replace whitespace-delimited values containing `@` or at least eight ASCII digits with `[REDACTED]`, cap output at 160 characters, and append `…` when truncated (`apiserver/src/handlers/worker_monitor.rs:197-224,392-421`). This behavior is required for monitor API error summaries, not a substitute for central log masking. Existing test asserts raw email/phone are absent and `[REDACTED]` exists (`apiserver/src/handlers/worker_monitor.rs:2551-2557`).

Audit callsites mask actor email before constructing records (`apiserver/src/handlers/workspaces/members.rs:129-140`; `apiserver/src/handlers/workspaces/permissions.rs:98-110`), but service callsites also place unmasked values such as employer phone/email into `details` (`apiserver/src/services/employer_service.rs:67-88,122-143`). Go MUST redact audit details under the same T1/T2 policy; parity must not preserve a raw-PII leak.

## 5. Metrics and operational monitoring

### 5.1 Metric-name freeze

Rust exposes no Prometheus endpoint or HTTP metrics middleware (`docs/migration/rust-go/middleware-contract.md:339-344`). Therefore **baseline-compatible metric name set is empty**: no Rust metric names can be claimed as parity names. The migration plan selects OpenTelemetry Go for tracing/metrics (`rust-go-migration-plan.md:31-33`), but supplies no instrument names.

| Metric surface | Frozen name rule | Verification |
|---|---|---|
| Prometheus `/metrics` | No baseline endpoint/name. Go MUST NOT add one and call it parity without a logged decision. | Route test: `/metrics` is not part of parity surface unless decision exists. |
| HTTP request counters/histograms | Name `UNKNOWN — Rust has no emitted name`; do not normalize by guessing. | Go contract test blocks unapproved names. |
| Alert metrics | Concepts only: 5xx rate, p99 latency, rate-limit triggers, failed auth attempts, background-job failures, DB pool utilization; machine names are not specified | `.agent/rules/observability-rules.md:53-62` |
| Infrastructure metrics | Concepts only: CPU, memory, disk, DB replication lag; machine names are not specified | `.agent/rules/observability-rules.md:64-71` |

### 5.2 Existing monitor JSON metric keys

These are API response fields, not Prometheus/OTel instrument names. Preserve names and nullable/unknown semantics when migrating worker monitoring:

- Queue: `depth_high`, `depth_normal`, `depth_low`, `depth_total`, `dead_letter_depth`, `oldest_queued_age_seconds`, `enqueue_rate_per_second`, `processing_rate_per_second`, `growth_rate_per_second`, `estimated_drain_seconds`, `queue_wait_seconds_p50`, `queue_wait_seconds_p95` (`apiserver/src/handlers/worker_monitor.rs:488-513`).
- Reliability: `success_rate`, `failure_rate`, `consecutive_failures`, `retryable_count`, `dead_letter_count`, `lost_count`, `timeout_count`, `execution_seconds_p50`, `execution_seconds_p95`, `execution_seconds_p99` (`apiserver/src/handlers/worker_monitor.rs:515-527`).
- Every `MonitorMetric` keeps `value`, `unit`, `formula`, `time_window`, `observed_at`, `source`, and optional `unknown`; null means unavailable, not measured zero (`apiserver/src/handlers/worker_monitor.rs:488-497`; `apiserver/src/handlers/worker_monitor.rs:576-604`).
- Monitor correlation accepts exactly one of `request_id` or `trace_id`; both/neither is invalid (`apiserver/src/handlers/worker_monitor.rs:618-628,2231-2261`).

## 6. Health and readiness response

### 6.1 Liveness

`GET /api/v1/health` returns HTTP `200` and JSON `{"message":"API Server is running!"}` (`apiserver/src/handlers/common.rs:511-520`). It performs no DB/Redis check. This route bypasses rate limiting (`apiserver/src/mw/rate_limit.rs:166-170`) and MUST remain usable when DB/Redis is down.

### 6.2 Readiness

`GET /api/v1/health/ready` performs `SELECT 1` using API DB pool (`apiserver/src/handlers/common.rs:522-545`) and Redis `PING` when Redis is configured (`apiserver/src/handlers/common.rs:547-556`). Frozen JSON/status contract:

| Condition | HTTP | JSON fields/values |
|---|---:|---|
| DB `SELECT 1` succeeds and Redis succeeds or is disabled | `200` | `status:"healthy"`; `checks.database:"ok"`; `checks.redis:"ok"` or `"disabled"` |
| DB connection/query fails | `500` | `status:"unhealthy"`; `checks.database:"error"`; Redis reflects its own result |
| Configured Redis `PING` fails | `500` | `status:"unhealthy"`; `checks.redis:"error"`; DB reflects its own result |

Response always includes `version` and non-negative `uptime_seconds` (`apiserver/src/handlers/common.rs:558-579`). No `worker` check exists in this Rust response; the generic rules example includes one but is not Rust evidence (`.agent/rules/observability-rules.md:73-88`). Go MUST not add/remove readiness keys during parity.

### 6.3 System health

`GET /api/v1/system/health` is protected to `PlatformAdmin` or `Support` and returns `{"status":"ok","viewer_user_id":...,"viewer_roles":[...]}` (`apiserver/src/handlers/system_admin.rs:385-404`). Role gate tests cover admin/support allowed and billing denied (`apiserver/src/handlers/system_admin.rs:683-694`). This is distinct from public liveness/readiness.

## 7. Audit events

### 7.1 Record shape and write policy

| Field | Rule | Evidence |
|---|---|---|
| `id` | Server-generated UUIDv7 via `new_id()` | `apiserver/src/services/audit_service.rs:43-48`; `apiserver/src/utils/id.rs:19-21` |
| `workspace_id` | REQUIRED tenant scope | `apiserver/src/domain/audit.rs:6-19`; `apiserver/migrations/tenant/20260609000001_init_tenant/up.sql:586-601` |
| `actor_id`, `target_user_id`, `target_resource_id` | Nullable UUIDs | `apiserver/src/domain/audit.rs:8-15` |
| `actor_type`, `actor_name`, `actor_email` | Nullable actor metadata; actor email MUST be masked before write | `apiserver/src/domain/audit.rs:9-13`; `apiserver/src/handlers/workspaces/members.rs:129-140` |
| `action` | REQUIRED string, max DB length 255 | `apiserver/src/domain/audit.rs:15`; `apiserver/migrations/tenant/20260609000001_init_tenant/up.sql:592-599` |
| `details` | Nullable JSON; MUST contain no T1/raw T2 values | `apiserver/src/domain/audit.rs:15-18`; `docs/migration/rust-go/privacy-contract.md:93-102` |
| `ip_address` | First `X-Forwarded-For` value; fallback `X-Real-IP`; absent otherwise | `apiserver/src/services/audit_service.rs:31-41` |
| `user_agent` | `User-Agent` header value when valid UTF-8 | `apiserver/src/services/audit_service.rs:25-29` |
| `created_at` | UTC timestamp at event creation | `apiserver/src/services/audit_service.rs:43-49`; `apiserver/migrations/tenant/20260609000001_init_tenant/up.sql:596-601` |

`AuditHelper::log` writes asynchronously and is fail-open: repository failure emits an `ERROR` with action/workspace/actor context but does not fail the user request (`apiserver/src/services/audit_service.rs:61-70`; `docs/migration/rust-go/middleware-contract.md:329-335`). Go MUST preserve this request outcome unless a decision changes it.

### 7.2 Observed action names

| Action | Trigger/evidence | Required details rule |
|---|---|---|
| `add_member`, `update_member_role`, `remove_member` | `apiserver/src/handlers/workspaces/members.rs:129-140,195-208,260-271` | Preserve role IDs and target IDs; mask actor email |
| `assign_permission`, `revoke_permission` | `apiserver/src/handlers/workspaces/permissions.rs:98-110,161-173` | Preserve permission/expiry or permission ID |
| `assign_extra_role`, `revoke_extra_role` | `apiserver/src/handlers/workspaces/roles.rs:111-122,171-182` | Preserve role/resource IDs |
| `dashboard.sensitive.read` | Sensitive cost metric response audit (`apiserver/src/handlers/dashboard_metrics.rs:91-109`; `apiserver/src/handlers/dashboard_overtime_breakdown.rs:159-178`) | Details identify period/endpoint/cost metric, never salary/PII values |
| `custom_field.definition_created`, `.definition_updated`, `.definition_deleted` | `apiserver/src/services/custom_field_service.rs:122-138,175-189,209-223` | Field key/type metadata only; redact values |
| `custom_field.value_changed` | `apiserver/src/services/custom_field_service.rs:464-484,506-529` | `old_value`/`new_value` require classification-aware masking; raw values are not allowed by this contract |
| `employer.created`, `.updated`, `.deleted` | `apiserver/src/services/employer_service.rs:67-88,122-143,177-188` | Existing source includes phone/email/tax fields; Go MUST redact T1/T2 before persistence |
| `location.created`, `.updated`, `.deleted` | `apiserver/src/services/location_service.rs:67-78,118-129,173-184` | Preserve resource/name/timezone, subject to classification |
| `monitor_alert.acknowledged`, `monitor_alert.resolved` | `apiserver/src/repositories/diesel_monitor_alert_repository.rs:212-225` | System audit details contain alert ID, reason, event type |

### 7.3 Policy-required events and observed gaps

P1-11 requires audit records for T2+ exports with `actor_id`, action, target resource, `row_count`, format/filter details, IP, and user agent (`docs/migration/rust-go/privacy-contract.md:93-102`). Observability rules additionally require login/logout, permission changes, PII detail reads, configuration changes, and bulk operations in both application logs and `audit_logs` (`.agent/rules/observability-rules.md:90-98`).

Current Rust auth handlers log login attempts/failures (`apiserver/src/handlers/auth.rs:140-142,151-170,198-205`) and revoke sessions on logout (`apiserver/src/handlers/auth.rs:405-429`) but no `AuditHelper::log` call is evidenced there. Report export logs job creation (`apiserver/src/handlers/report_export.rs:178-189`) but no audit write is evidenced. These are **unresolved parity/policy gaps**, not silently declared implemented. Audit retention is three years (`docs/migration/rust-go/privacy-contract.md:59-71`; `.agent/rules/observability-rules.md:98,100-103`).

## 8. Unknowns and decisions required

| ID | Unknown/mismatch | Evidence | Required decision/test consequence |
|---|---|---|---|
| O-01 | Rust accepts UUID `x-request-id`, not arbitrary correlation strings; Go OTel plan mentions W3C trace context, but no Rust `traceparent` behavior exists | `apiserver/src/mw/request_id.rs:17-25`; `rust-go-migration-plan.md:31-33` | Decide whether Go accepts/forwards `traceparent` additively. Tests MUST still require UUID `x-request-id` parity. |
| O-02 | Rust formatter fields are conditional, while rules/ADR describe request/workspace fields as mandatory; startup events have no request context | `apiserver/src/utils/logging.rs:194-238`; `.agent/rules/observability-rules.md:6-19`; `.ai/ADRs/0002-go-technology-stack-apiserver.md:83-87` | Freeze conditional requirement in §3.1 for requestless startup/background events, or log decision before enforcing all-entry fields. |
| O-03 | Rust default fallback filter enables `DEBUG`; policy says production `INFO+` and no `TRACE` | `apiserver/src/main.rs:88-105`; `.agent/rules/observability-rules.md:32-33` | Choose compatibility vs security/noise correction. Add startup filter test. |
| O-04 | No Rust Prometheus/OTel instrument names or `/metrics` route | `docs/migration/rust-go/middleware-contract.md:339-344`; `rust-go-migration-plan.md:31-33` | Name target instruments and endpoint only through explicit decision; until then metric parity set remains empty. |
| O-05 | Generic name/address/salary/custom-field masking output is not defined by Rust logger; key heuristic does not cover all T2 fields | `apiserver/src/utils/logging.rs:15-34`; `docs/migration/rust-go/privacy-contract.md:13-18` | Add classification-key policy and golden vectors before Go implementation. |
| O-06 | Rust byte slicing behavior for non-ASCII masking can differ from Go rune slicing | `apiserver/src/utils/masking.rs:9,18-20,31-34,42-44` | Add UTF-8 test vectors or constrain input contract; do not assume rune-equivalent output. |
| O-07 | Export/login/logout/policy-required audit events are required by P1/rules but not evidenced through `AuditHelper::log` in observed handlers | `apiserver/src/handlers/auth.rs:140-205,405-429`; `apiserver/src/handlers/report_export.rs:178-189`; `docs/migration/rust-go/privacy-contract.md:93-102` | Decide whether Go implements policy-required additive audit events or strict observed parity. Add one-event/no-duplicate and no-PII tests. |
| O-08 | `leedevkit.toml` names Go health check `/health`, while Rust route is `/api/v1/health` | `leedevkit.toml:29-35`; `apiserver/src/routes/auth.rs:23-29` | Decide deployment probe path; parity tests cover Rust path, deployment tests cover selected Go path. |
| O-09 | Public readiness JSON has only DB/Redis checks; generic rules example adds `worker` | `apiserver/src/handlers/common.rs:563-570`; `.agent/rules/observability-rules.md:73-88` | Preserve Rust key set for parity; add worker only after API-contract decision. |

## 9. Verification method and required assertions

Go implementation MUST provide tests that can fail on each frozen rule. Rust reference assertions below are existing evidence and expected parity oracles:

| Assertion | Rust evidence / expected check |
|---|---|
| Missing `x-request-id` yields UUID-shaped response ID; invalid incoming ID is replaced; valid ID is retained; request and response IDs match | Middleware algorithm `apiserver/src/mw/request_id.rs:17-48`; existing response-header tests `apiserver/tests/integration/misc/test_app_builder.rs:411-437` and 404 coverage `:753-773` |
| Health response contains `message` and status `200`; readiness contains `status`, `version`, `uptime_seconds`, and DB/Redis checks | `apiserver/tests/integration/misc/health.rs:7-37`; handler contract `apiserver/src/handlers/common.rs:515-579` |
| Liveness remains `200` without dependency queries; readiness returns `500` on failed DB/Redis and bypasses rate limit | `apiserver/src/handlers/common.rs:522-579`; `apiserver/src/mw/rate_limit.rs:166-170` |
| Production log is parseable JSON with envelope keys; `ERROR` includes `error.type`; custom fields are nested under `fields` | `apiserver/src/utils/logging.rs:179-264` |
| Exact email/phone/bank vectors and secret redaction match | `apiserver/src/utils/masking.rs:47-72`; logger tests `apiserver/src/utils/logging.rs:280-293` |
| Trace context is scoped and absent outside scope | `apiserver/src/utils/logging.rs:295-306` |
| Outbox job stores `_trace_context`; job returns same context | `apiserver/src/services/outbox_job_service.rs:311-359` |
| Worker monitor summaries contain no raw email/long phone and use `[REDACTED]` | `apiserver/src/handlers/worker_monitor.rs:2551-2557` |
| Audit record captures action/actor/target/details and client IP precedence; repository failure does not fail request | `apiserver/src/services/audit_service.rs:24-70,117-188` |
| Audit event action and detail assertions cover representative member/permission/role/sensitive-read actions | `apiserver/tests/integration/rbac/api.rs:202-247`; callsites in §7.2 |
| System health allows PlatformAdmin/Support and rejects BillingAdmin | `apiserver/src/handlers/system_admin.rs:683-694` |
| No raw T1/T2 values appear in logs, audit `details`, monitor summaries, or error messages | P1-11 restrictions `docs/migration/rust-go/privacy-contract.md:9-18`; logger and monitor evidence in §§4.1–4.3 |
| No unapproved Prometheus metric names are emitted; monitor JSON key names remain exact | Absence baseline `docs/migration/rust-go/middleware-contract.md:339-344`; JSON keys `apiserver/src/handlers/worker_monitor.rs:488-527` |

**Evidence status:** Contract complete for observed Rust behavior. O-01 through O-09 remain explicit unknowns; no unsupported behavior is presented as frozen baseline.

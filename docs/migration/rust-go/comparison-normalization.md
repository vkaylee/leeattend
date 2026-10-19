# Rust→Go comparison normalization — P2-08

**Scope:** differential HTTP/API comparisons plus database, Redis, and object-storage side effects during LeeAttend migration. This is a comparison rule, not permission to change public behavior. Source evidence is Rust `apiserver`; Go comparator implementation is not yet present in this repository. Unknown comparator details remain **UNKNOWN** until P3-01 fixtures define them.

## Source evidence

| Concern | Rust evidence | Fixed comparison rule |
|---|---|---|
| Request ID | `apiserver/src/mw/request_id.rs:6-14,17-25,43-48` | Preserve supplied valid UUID request ID. If absent/invalid, Rust generates UUIDv7 and returns same value in `x-request-id`; compare header shape and propagation, but allow generated value to differ between runs. |
| Production IDs | `apiserver/src/utils/id.rs:1-7,15-21`; examples `apiserver/src/services/storage/file_service.rs:98-105` and `:252-274` | `new_id()` is UUIDv7. IDs generated independently by Rust and Go MAY differ; each side MUST still emit valid UUIDs and preserve identity/reference relationships inside its own response and side effects. |
| Test/ephemeral IDs | `apiserver/src/utils/id.rs:15-18`; `apiserver/src/services/storage/key_generator.rs:33-37` | UUIDv4 is used for test fixtures and storage-key randomness. Do not compare literal random value; compare UUID format, key components, and cross-field references. |
| Timestamps | `apiserver/src/services/storage/file_service.rs:96-99,266-274`; `apiserver/src/repositories/storage_config_repo.rs:87-103`; `apiserver/src/agent_manager.rs:161-163` | `created_at`, `updated_at`, `expires_at`, presence timestamps, and equivalent generated times MAY differ only within an explicitly configured test tolerance. Preserve timezone/precision policy from P2-04; do not remove field or compare only date. |
| Presigned URL | `apiserver/src/services/storage/s3_storage.rs:165-183,185-209`; TTL setup `apiserver/src/services/storage/file_service.rs:35-45,57-65`; default `apiserver/src/repositories/storage_config_repo.rs:98-102` | Signature, nonce, host signing details, and absolute expiry MAY differ. Parse URL and compare scheme/host policy, bucket/key/path, operation (GET vs PUT), content-type signing for PUT, and expiry duration against configured TTL. Never accept arbitrary URL or expiry. |
| Response serialization | Errors are serialized by `apiserver/src/errors.rs:362-405`; normal handlers return Axum `Json` (for example `apiserver/src/handlers/workspaces/locations.rs:39-55,76-86`) | Decode JSON before comparison. Compare field presence, null versus missing, types, enum strings, numeric representation, and values exactly after only listed normalization. |
| Status | Error status comes from `AppError::status_code()` and is used at `apiserver/src/errors.rs:362-365,404`; rate-limit status/header at `apiserver/src/mw/rate_limit.rs:204-216` | HTTP status is always exact. No normalization of status family (`201` ≠ `200`, `401` ≠ `403`, `429` ≠ `503`). |
| Headers | CORS and allowed headers configured at `apiserver/src/lib.rs:286-305`; request ID response injection `apiserver/src/mw/request_id.rs:43-45`; rate limit `Retry-After` `apiserver/src/mw/rate_limit.rs:204-216` | Header names case-insensitive; values exact after whitespace normalization required by HTTP parsing. `x-request-id` is the only generally allowed generated value. CORS headers compare exact policy. `Retry-After` is exact (`window_secs`), not a free-form timing field. |
| DB transaction | `docs/migration/rust-go/database-contract.md:19-21` | Compare committed rows, changed columns, constraints, row counts, and transaction outcome exactly. Rollback means no durable row/column side effect. |
| Redis | Cache read/write implementation `apiserver/src/cache/redis_cache.rs:71-90`; key/TTL/failure inventory `docs/migration/rust-go/redis-contract.md:35-60` | Compare key names, command class, encoded value, TTL policy, deletion/invalidation, and failure behavior. A cache hit/miss can differ only when fixture explicitly permits it; durable or auth keys cannot be ignored. |
| Ordering | Roster explicitly documents preserved query ordering at `apiserver/src/cache/roster_cache.rs:110-124`; stable employee/report sort inputs are accepted by `apiserver/src/handlers/employees.rs:219-223` | Preserve documented SQL/order semantics. Treat collection as unordered only when endpoint contract and P1 matrix explicitly say order is unspecified; compare as multisets keyed by full canonical element otherwise. |

## Allowed nondeterminism (closed list)

Every ignored field MUST appear in fixture metadata with `path`, `reason`, `source citation`, and `verification`. No wildcard or type-wide ignore is allowed.

| Field/path | Allowed difference | Reason | Required checks |
|---|---|---|---|
| `response.headers.x-request-id` | UUID value | Rust accepts valid incoming ID or generates UUIDv7 (`request_id.rs:17-25`) | Header exists, UUID parses, supplied valid ID is preserved, generated value is not reused across independent requests unless supplied. |
| `trace_id`, `span_id` in logs/traces (if emitted) | ID value | Per-request telemetry identity; no Rust response contract shown in cited source | Format and propagation only; no API body/header comparison may ignore unrelated IDs. **UNKNOWN:** exact Rust trace fields. |
| Newly created resource ID fields (`id`, and matching FK/URL/key components) | UUID value | `new_id()` uses time-sorted UUIDv7 (`utils/id.rs:1-21`) | UUID version/format; all references agree within run; no collision; ordering properties where contract requires UUIDv7. |
| UUIDv4 storage-key component | UUID value | `generate_storage_key` uses UUIDv4 (`key_generator.rs:33-37`) | Key prefix, `YYYY/MM`, sanitized filename, and UUID segment shape exact. |
| `created_at`, `updated_at`, `expires_at`, `last_seen_at` | Instant value within fixture tolerance | Rust calls `Utc::now()` at creation/update (`file_service.rs:96-99,266-274`; `storage_config_repo.rs:101-102`) | Same timezone/offset semantics, precision policy, and relational constraints (`updated_at >= created_at`, expiry relative to requested TTL). Tolerance MUST be recorded per fixture. |
| Presigned URL signature/query (`X-Amz-*`, equivalent signer fields) | Signature and signing instant | AWS presigning generates request-specific signature (`s3_storage.rs:165-182,193-208`) | URL operation, host/bucket/key, signed content type (PUT), and expiry delta exact; configured TTL is 900 seconds by default (`storage_config_repo.rs:98`). |
| Collection order | Element order only | **Only** endpoint rows explicitly marked order-unspecified in `endpoint-matrix.csv` and P2-02 | Compare canonicalized multiset; duplicate counts and all element fields remain exact. |

Not allowed by this list: request body values, IDs supplied by caller, workspace/user IDs, error codes/messages, pagination metadata, status, headers other than listed generated values, DB keys/values, Redis keys/values, TTLs, or object-storage key semantics.

## Exact normalization pipeline

Apply in this order, independently to Rust and Go captures:

1. **Capture raw exchange and side effects.** Save method, URL, request headers/body, response status/headers/body, SQL observation, Redis command/key/value/TTL observation, and object-storage calls. Do not normalize before capture.
2. **Status:** compare integer status exactly.
3. **Headers:** lowercase names for lookup; trim HTTP parser-equivalent surrounding whitespace only; preserve duplicate-header cardinality. Compare all headers exactly except `x-request-id` and explicitly declared trace headers. Never drop `Retry-After`, `Location`, `Content-Type`, CORS, cache, auth, or content-disposition headers.
4. **Body parse:** decode JSON only when declared JSON. Canonicalize object key order and numeric lexical formatting without changing numeric value. Preserve missing versus `null`, empty string versus missing, empty array versus missing, and string versus number. Binary bodies compare bytes; text compares bytes after no newline conversion unless endpoint contract says otherwise.
5. **Nondeterministic fields:** replace only exact fixture paths listed in the closed list. Record original value, reason, and validator result; replacement token MUST include field path (for example `<generated:id>`), not a global regex.
6. **Ordering:** sort only declared unordered arrays by a full stable key supplied by fixture (prefer immutable ID); otherwise preserve array order. Never sort object arrays by generated timestamp unless endpoint contract defines that order.
7. **DB side effects:** canonicalize rows by primary key only for comparison display. Compare table, operation, transaction outcome, row count, primary-key relationships, all non-allowed values, and NULL state. Generated IDs/times use field-level rules above. A Rust rollback and Go commit is a mismatch.
8. **Redis side effects:** compare exact command semantics and key strings. Parse JSON values and apply body rules; compare scalar values, TTL bounds, invalidations, and failure result. TTL tolerance MUST be explicit (recommended observation jitter only when measurement latency is documented); do not erase TTL.
9. **Verdict:** pass only when normalized HTTP result and all required side effects match. Any unlisted difference is drift, not “noise.”

## Forbidden normalization

- Do not replace every UUID, every timestamp, or every URL with a placeholder. This hides caller-supplied IDs, workspace isolation bugs, wrong foreign keys, expiry defects, and leaked URLs.
- Do not ignore status, error code, localized message selection, retry metadata, `Retry-After`, `Location`, `Content-Type`, CORS, authorization, or cache headers.
- Do not compare only HTTP 2xx/4xx families; exact status is contract.
- Do not drop `null`, omitted, empty, unknown, or extra JSON fields. Serde `Option`/missing rules are tracked in `docs/migration/rust-go/type-mapping.md` and must remain observable.
- Do not sort all arrays or deduplicate rows. Ordering may be contract; duplicates may expose missing joins or pagination bugs.
- Do not ignore DB writes because response matches. Do not ignore Redis writes, invalidation, auth revocation, rate-limit counters, queue entries, or TTLs because they are “internal.”
- Do not accept a presigned URL after stripping its entire query string. Validate operation, resource, signer host, signed headers, and expiry.
- Do not widen time tolerances until a failing capture proves clock/measurement jitter; wider tolerance belongs in fixture metadata and review.

## Verification examples

### Example A — generated ID/time allowed, contract drift rejected

```json
{
  "request": {"method": "POST", "path": "/api/v1/workspaces/W/files"},
  "normalize": [
    {"path": "$.body.id", "kind": "generated_uuid", "reason": "Rust new_id UUIDv7"},
    {"path": "$.body.created_at", "kind": "generated_time", "tolerance_ms": 1000,
     "reason": "Rust Utc::now() at file creation"}
  ],
  "assert": ["$.body.workspace_id == fixture.workspace_id", "$.body.file_size == fixture.bytes"]
}
```

Rust `201` versus Go `200`, `id` not UUID-shaped, wrong `workspace_id`, or `created_at` outside tolerance fails. `updated_at` omitted in one response also fails; it is not covered by `created_at` normalization.

### Example B — presigned URL

```text
GET/PUT operation same
scheme + signing host same policy
bucket/key path same after UUIDv4 path segment normalization
PUT signed Content-Type == request Content-Type
expiry(Rust), expiry(Go) = configured TTL (default 900s), with recorded clock jitter only
```

Signature query values may differ. A different key prefix, bucket, method, content type, or TTL fails.

### Example C — side effects

```text
Rust: BEGIN; INSERT file(id=<uuid>, created_at=<time>); COMMIT
Go:   BEGIN; INSERT file(id=<uuid>, created_at=<time>); ROLLBACK
```

Fails despite normalized response equality: committed DB state and transaction outcome differ. Likewise, `SET cache:workspace:<ws> ... EX 300` versus no Redis write fails when endpoint matrix marks cache write/invalidation as required.

## Unresolved decisions

| Decision | Current evidence | Required resolution |
|---|---|---|
| Exact trace/span response/log field names | Request middleware emits `request_id` span field (`apiserver/src/mw/request_id.rs:32-39`); no stable trace ID wire contract is established in cited Rust source | P2-06/P3-01 fixture owner must list exact trace fields before permitting normalization; until then, no trace field is ignored by default. |
| Per-fixture timestamp tolerance | Rust uses wall clock (`apiserver/src/services/storage/file_service.rs:96-99,266-274`; `apiserver/src/repositories/storage_config_repo.rs:101-102`) | Fixture must set tolerance and clock-observation method; no global tolerance. |
| Endpoint order unspecified | Some cache vectors document explicit ordering (`apiserver/src/cache/roster_cache.rs:110-124`), but matrix rows vary | P2-02 must mark each unordered collection; absent mark means order-sensitive. |
| Differential runner/artifact schema | Migration plan requires differential fixtures (`rust-go-migration-plan.md:61,75`) but no runner exists in repository | P3-01 defines capture format and executable runner; until then, this document is normative, not executable. |

## Verification method and acceptance

1. Build one fixture per nondeterministic category using captured Rust response plus DB/Redis/object-storage observations; fixture records every normalization path and non-empty reason.
2. Replay identical request/seed against Rust reference and Go candidate. Capture raw artifacts before normalization.
3. Run strict normalization pipeline above. Verify positive examples pass and mutation tests (status, header, null/missing, foreign ID, DB rollback, Redis key/TTL, URL key) fail.
4. Cross-check endpoint status/header/body/side-effect expectations against `docs/migration/rust-go/endpoint-matrix.csv`, type/nullability rules against `docs/migration/rust-go/type-mapping.md`, transaction rules against `docs/migration/rust-go/database-contract.md`, and Redis rules against `docs/migration/rust-go/redis-contract.md`.
5. **Acceptance:** every ignored field has exact path + reason + citation + validator; no REQUIRED comparison column is empty; unlisted differences fail closed. **UNKNOWN:** repository currently contains no dedicated Rust-vs-Go differential comparator or P3 fixture runner, so executable command name and artifact schema await P3-01.

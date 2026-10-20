# Differential Harness Specification — P3-12

**Scope:** Run one canonical fixture against Rust reference `apiserver` and Go candidate, normalize only approved nondeterminism, compare HTTP and state outcomes, and emit machine-readable reports.  
**Ledger Task:** `P3-12`  
**Dependencies:** `P3-01` (`README.md`, `fixture.schema.json`), `P3-11` (`state-comparators.md`), `P2-08` (`comparison-normalization.md`).  
**Implementation constraint:** This document defines a runner interface. It MUST NOT require a Go package, Go binary, or Go module to exist; any HTTP-capable runner can implement it.

---

## 1. Contract

Harness accepts one or more fixture paths and two base URLs:

```text
reference_base_url = http://127.0.0.1:3000   # Rust Axum reference
candidate_base_url = http://127.0.0.1:3001    # Go candidate
```

For each fixture, harness MUST:

1. Validate JSON against `fixture.schema.json`.
2. Resolve `request.path_params`, append `request.query`, and construct identical HTTP requests for both servers.
3. Apply the fixture identity using the declared transport (`Bearer`, cookie, service token, or debug `X-User-Id`) without changing request payload or tenant path.
4. Reset isolated database, Redis, and external-stub state to fixture `setup` before each server run.
5. Capture raw request, raw HTTP response, database observations, Redis observations, outbox rows, audit rows, and transaction outcome before normalization.
6. Normalize each capture independently using the exact ordered pipeline in `comparison-normalization.md` (P2-08).
7. Compare normalized Rust and Go captures plus each capture against fixture expectations.
8. Emit one machine-readable report per fixture and one aggregate report.
9. Return a non-zero exit status for schema, transport, expectation, comparator, normalization, or parity failure.

Harness MUST run Rust and Go as separate executions against isolated state. It MUST NOT feed Rust output into the Go request, copy generated IDs from one run into the other, or normalize before raw capture.

---

## 2. Runner Interface

### 2.1 CLI Contract

A conforming implementation MAY use another command name, but MUST expose equivalent inputs and outputs:

```text
parity-harness run \
  --fixture docs/migration/rust-go/fixtures/example_fixture.json \
  --reference-base-url http://127.0.0.1:3000 \
  --candidate-base-url http://127.0.0.1:3001 \
  --report-dir artifacts/parity \
  --state-reset-command <optional-command> \
  --external-stub-config <optional-config>
```

Required options:

| Option | Requirement |
|---|---|
| `--fixture` | One fixture JSON file or repeatable fixture paths. |
| `--reference-base-url` | Rust base URL; default `http://127.0.0.1:3000`. |
| `--candidate-base-url` | Go base URL; default `http://127.0.0.1:3001`. |
| `--report-dir` | Directory for per-fixture and aggregate JSON reports. |

Optional implementation adapters:

| Adapter | Purpose |
|---|---|
| `state-reset` | Drops/resets isolated DB and Redis namespace, then applies fixture seeds. A shell command, HTTP endpoint, or library adapter MAY implement it. |
| `external-stubs` | Starts/configures S3, SMTP, ClamAV, and other declared stubs. |
| `capture` | Collects SQL/Redis/object-storage observations. |
| `clock` | Supplies a frozen or observed test clock where server supports it. |

No adapter may silently become a no-op. If fixture requires an unavailable adapter, report `setup_error` and exit non-zero.

### 2.2 Process and Run Identity

Harness creates a unique `run_id` (UUID) and `fixture_id` for every execution. It MUST record:

```json
{
  "run_id": "018d0000-0000-7000-8000-0000000000aa",
  "fixture_id": "WS-LOC-001",
  "reference_base_url": "http://127.0.0.1:3000",
  "candidate_base_url": "http://127.0.0.1:3001",
  "started_at": "2026-10-19T10:00:00.000Z"
}
```

`run_id` is report metadata only. It MUST NOT be injected into the request unless fixture explicitly declares it. Each server run receives a unique execution ID internally, while the request itself remains identical.

---

## 3. Request Construction

### 3.1 URL Resolution

Given:

```json
{
  "path": "/api/v1/workspaces/{ws_id}/locations",
  "path_params": {"ws_id": "018d0000-0000-7000-8000-000000000001"},
  "query": {"page": 1, "limit": 50}
}
```

Harness MUST produce:

```text
http://127.0.0.1:3000/api/v1/workspaces/018d0000-0000-7000-8000-000000000001?page=1&limit=50
http://127.0.0.1:3001/api/v1/workspaces/018d0000-0000-7000-8000-000000000001?page=1&limit=50
```

Rules:

- Substitute every `{name}` placeholder exactly once. Missing or unused path parameters are `fixture_error`.
- Encode path values using RFC 3986 path-segment encoding; do not encode slashes supplied by the route template.
- Encode query values using standard URL query encoding, preserving repeated values and declared order.
- Preserve method, body bytes, content type, and all fixture headers across both requests.
- Base URL MUST be origin-only. Reject a base URL with a non-empty path, query, or fragment to prevent accidental route drift.

### 3.2 Identity Injection

`identity` provides the principal and its credentials. Harness selects one transport:

| `identity.auth_type` | Request action |
|---|---|
| `bearer` | Set `Authorization: Bearer <auth_token>`. |
| `cookie` | Set `Cookie: access_token=<auth_token>`; preserve any fixture cookie attributes only as declared. Rust extractor reads `access_token`. |
| `service_token` | Set the service-token header required by the endpoint contract; missing header mapping is a setup error, not an anonymous fallback. |
| `header_impersonation` | Set `X-User-Id: <user_id>` only in Rust debug/test mode; candidate MUST expose equivalent isolated test configuration. |
| `anonymous` | Send no auth credential. |

If `request.headers` already declares the selected auth header, identity and request values MUST match exactly; conflicting values are a fixture error. `identity.user_id`, `workspace_id`, `role`, and `permissions` describe expected context and MUST NOT be trusted as a substitute for a real credential.

### 3.3 Header Capture

Capture request and response headers as ordered name/value pairs, retaining duplicate-header cardinality. Comparison lowercases names for lookup and trims only HTTP parser-equivalent surrounding whitespace. Do not drop `Content-Type`, `Location`, `Retry-After`, CORS, cache, authorization, or content-disposition headers.

---

## 4. State Isolation and Capture

### 4.1 Per-Server Lifecycle

For each fixture, execute this sequence independently for Rust then Go:

```text
reset isolated DB + Redis + external stubs
apply setup.db_seeds, setup.redis_seeds, setup.external_stubs
capture pre-state snapshots
send exactly one request
capture raw response and post-state snapshots
run DB/Redis/outbox/audit/rollback comparators
```

A fresh isolated tenant database or transactionally reset schema is REQUIRED. Reusing Rust-mutated state for Go invalidates parity. System database seeds and tenant database seeds MUST remain separated; every tenant query is scoped to fixture `identity.workspace_id`.

### 4.2 Raw Capture Envelope

Each side stores a raw capture before normalization:

```json
{
  "side": "rust",
  "request": {
    "method": "POST",
    "url": "http://127.0.0.1:3000/api/v1/workspaces/018d0000-0000-7000-8000-000000000001/locations",
    "headers": [{"name": "content-type", "value": "application/json"}],
    "body_bytes_sha256": "<sha256>"
  },
  "response": {
    "status": 201,
    "headers": [{"name": "content-type", "value": "application/json"}],
    "body_bytes_base64": "<base64>"
  },
  "database": {"before": [], "after": [], "transaction": "committed"},
  "redis": {"before": [], "after": [], "commands": []},
  "outbox_events": [],
  "audit_events": [],
  "captured_at": "2026-10-19T10:00:00.100Z"
}
```

Body bytes, not parsed JSON, remain authoritative for non-JSON responses. Raw bodies MAY be redacted in a human-readable report, but machine-readable artifacts MUST retain a secure hash and enough structured data to reproduce comparison. Credentials, password hashes, cookies, and secrets MUST be encrypted or redacted according to repository security policy.

### 4.3 State Capture Requirements

Capture:

- All tables named by `setup.db_seeds`, `expected_side_effects.db_mutations`, `outbox_events`, or `audit_events`.
- Relevant rows before and after, including NULL state, primary keys, foreign keys, all changed columns, and transaction outcome.
- Redis command class, exact key, type, encoded value, TTL before/after, deletion/invalidation, and error result.
- External calls needed to prove file, S3, SMTP, ClamAV, or queue behavior.
- Audit and outbox rows even when HTTP response is successful; response equality alone is insufficient.

The `state-comparators.md` contract defines row, value, TTL, and rollback verdicts.

---

## 5. Normalization Pipeline (P2-08)

Apply independently to Rust and Go captures, in this order. Store each replacement and validator result in report `normalizations`; never mutate raw captures.

1. **Capture raw exchange and side effects.** No early normalization.
2. **Compare status exactly.** `201` differs from `200`; `401` differs from `403`; `429` differs from `503`.
3. **Normalize header lookup only:** lowercase names and trim HTTP-parser whitespace. Preserve values, duplicates, and all non-permitted headers.
4. **Parse body by declared content type:** JSON becomes an AST with canonical object key ordering and numeric lexical form; missing, `null`, empty, and wrong types remain distinct. Binary bodies compare bytes.
5. **Apply only exact fixture paths:** `expected_response.body_normalizations` and `normalization_rules.ignored_fields` MUST include path, reason, source citation, and validator. No wildcard, global UUID replacement, global timestamp replacement, or unscoped regular expression.
6. **Validate generated values:** Generated IDs retain UUID version/format and internal references; timestamps retain timezone/precision and relationship constraints; presigned URLs retain method, host/bucket/key, content type, and configured expiry; request IDs retain UUID shape and propagation.
7. **Reorder only declared unordered arrays:** Use fixture-provided stable key and preserve duplicate counts. Arrays without a declaration remain order-sensitive.
8. **Compare DB side effects:** Compare tables, operations, transaction result, row count, primary-key relationships, all non-allowed values, and NULL state.
9. **Compare Redis side effects:** Compare exact command semantics and keys, decoded values, invalidations, TTL policy, and failure behavior. TTL jitter is permitted only when fixture declares it.
10. **Emit verdict:** Pass requires normalized HTTP output, expected side effects, and Rust-vs-Go state parity. Any unlisted difference fails closed.

Allowed categories and evidence are closed by `comparison-normalization.md`. In particular, caller-supplied IDs, workspace/user IDs, statuses, error envelopes, pagination, `Retry-After`, CORS, and Redis key/value semantics MUST NOT be ignored.

---

## 6. Comparison Algorithm

For each fixture, produce three verdicts:

```text
expectation_verdict = compare(each side, fixture.expected_response, fixture.expected_side_effects)
parity_verdict      = compare(normalized Rust capture, normalized Go capture)
rollback_verdict    = state-comparator rollback result when request fails
fixture_verdict     = PASS only if all three verdicts PASS
```

A positive parity result requires:

- Same integer HTTP status.
- Same header set, values, and duplicate cardinality except explicitly validated generated paths.
- Same body JSON shape, field presence, nullability, types, enum values, numeric values, and values after declared normalization.
- Same committed DB mutation set and transaction outcome.
- Same Redis command/key/value/TTL mutation set and failure behavior.
- Same audit events and outbox events, including payload and status.
- Same rollback behavior on expected errors.

Harness MUST report Rust and Go expectation failures separately. A fixture may fail because both implementations agree on behavior that contradicts its expected contract; that is still `expectation_failed`, not `PASS`.

---

## 7. Machine-Readable Report

### 7.1 Per-Fixture Report Schema

Reports are UTF-8 JSON named `<fixture-id>.<run-id>.json`. Required top-level fields:

```json
{
  "report_version": "1.0.0",
  "run_id": "018d0000-0000-7000-8000-0000000000aa",
  "fixture_id": "WS-LOC-001",
  "fixture_path": "docs/migration/rust-go/fixtures/example_fixture.json",
  "reference_base_url": "http://127.0.0.1:3000",
  "candidate_base_url": "http://127.0.0.1:3001",
  "status": "passed",
  "failure_codes": [],
  "rust": {
    "transport": "ok",
    "expectation": "passed",
    "normalized": {},
    "state": {},
    "raw_artifact": "WS-LOC-001.<run-id>.rust.raw.json"
  },
  "go": {
    "transport": "ok",
    "expectation": "passed",
    "normalized": {},
    "state": {},
    "raw_artifact": "WS-LOC-001.<run-id>.go.raw.json"
  },
  "parity": {
    "http": "passed",
    "database": "passed",
    "redis": "passed",
    "outbox": "passed",
    "audit": "passed",
    "rollback": "passed",
    "diffs": []
  },
  "normalizations": [],
  "timing_ms": {"setup": 0, "rust": 0, "go": 0, "compare": 0}
}
```

`status` MUST be one of `passed`, `failed`, `setup_error`, `transport_error`, `schema_error`, or `normalization_error`. `failure_codes` MUST use stable machine-readable values:

| Code | Meaning |
|---|---|
| `fixture_schema_invalid` | Fixture fails `fixture.schema.json`. |
| `path_resolution_failed` | Path parameter or query construction invalid. |
| `setup_error` | State reset, seed, or external stub setup failed. |
| `reference_unavailable` | Rust `:3000` cannot be reached. |
| `candidate_unavailable` | Go `:3001` cannot be reached. |
| `http_status_mismatch` | Exact status differs. |
| `http_header_mismatch` | Header set/value/cardinality differs. |
| `http_body_mismatch` | Body differs after permitted normalization. |
| `database_mismatch` | DB rows, columns, FK relationships, or transaction outcome differ. |
| `redis_mismatch` | Redis key, type, command, value, TTL, or invalidation differs. |
| `outbox_mismatch` | Outbox event or payload differs. |
| `audit_mismatch` | Audit event or details differ. |
| `rollback_leak` | Failed request caused undeclared durable state. |
| `normalization_rejected` | Declared normalization lacks evidence or validator failed. |
| `expectation_failed` | Both sides may agree, but fixture expected contract failed. |

Each diff entry MUST include:

```json
{
  "scope": "http|database|redis|outbox|audit|rollback",
  "path": "response.body.workspace_id",
  "kind": "value_mismatch|missing|unexpected|type_mismatch|ttl_mismatch|transaction_mismatch",
  "expected": "<value-or-null>",
  "rust": "<value-or-null>",
  "go": "<value-or-null>",
  "normalization": null,
  "source": "comparison-normalization.md:43-49"
}
```

### 7.2 Aggregate Report

For multiple fixtures, emit `summary.<run-id>.json`:

```json
{
  "report_version": "1.0.0",
  "run_id": "018d0000-0000-7000-8000-0000000000aa",
  "status": "failed",
  "fixtures_total": 2,
  "fixtures_passed": 1,
  "fixtures_failed": 1,
  "reports": [
    {"fixture_id": "WS-LOC-001", "status": "passed", "report": "WS-LOC-001.<run-id>.json"},
    {"fixture_id": "AUTH-001", "status": "failed", "report": "AUTH-001.<run-id>.json"}
  ]
}
```

Aggregate `status` is `passed` only when every fixture passes. Reports MUST be deterministic in field ordering and use UTC timestamps; generated run IDs and raw timestamps are metadata, not comparison inputs.

---

## 8. Exit Codes and CI Integration

| Exit code | Meaning |
|---:|---|
| `0` | All requested fixtures pass schema, expectation, state, rollback, and Rust-vs-Go parity checks. |
| `1` | One or more fixtures fail parity or expected behavior. |
| `2` | Harness invocation, fixture schema, path resolution, or configuration error. |
| `3` | Reference/candidate server or state dependency unavailable. |
| `4` | Normalization declaration rejected or validator failed. |

CI MUST archive per-fixture raw captures, normalized reports, aggregate report, and comparator logs on non-zero exit. Secrets MUST be redacted before artifact upload.

---

## 9. Verification Procedure

A P3-12 implementation is accepted when the following checks pass:

1. Run `example_fixture.json` against Rust on `:3000` and Go on `:3001` using isolated state.
2. Confirm both requests have identical method, resolved path, query, headers, and body hash; only base URL differs.
3. Mutate Rust/Go candidate response status, a non-allowed header, JSON null/missing field, DB column, Redis key, and Redis TTL; each mutation emits the documented failure code.
4. Run a fixture with an expected `4xx`/`5xx`; verify rollback comparator catches a committed row, outbox event, audit event, or cache write not declared by fixture.
5. Run fixtures twice from clean state; normalized comparison reports remain equivalent except report metadata (`run_id`, capture timestamps, raw artifact hashes where generated values are allowed).
6. Verify a Rust-only run and Go-only run can be produced when the other server is unavailable; report must be `reference_unavailable` or `candidate_unavailable`, never a false parity pass.
7. Verify report JSON parses without a Go dependency and aggregate exit code follows Section 8.

The harness remains a contract until P4-12 supplies concrete reset, capture, and runner adapters. No executable command or Go implementation is implied by this specification.

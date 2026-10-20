# Canonical Migration Fixture Specification — P3-01

**Scope:** Canonical specification and JSON schema for Phase 3 parity-oracle fixtures in the Rust→Go `apiserver` migration (`leeattend`).  
**Ledger Task:** `P3-01`  
**Dependencies:** `P2-08` (`docs/migration/rust-go/comparison-normalization.md`), `P2-03` (`docs/migration/rust-go/error-contract.md`), `P2-04` (`docs/migration/rust-go/database-contract.md`), `P2-05` (`docs/migration/rust-go/redis-contract.md`), `P2-02` (`docs/migration/rust-go/endpoint-matrix.csv`).  
**Consumers:** Ledger tasks `P3-02` through `P3-10` (domain fixture suites), `P3-11` (state comparators), `P3-12` (differential harness), `P4-12` (differential runner implementation), and Phase 7 verification.

---

## 1. Executive Summary & Principles

Phase 3 establishes declarative, executable parity fixtures that capture the exact behavior of the Rust Axum `apiserver` reference to verify the Go `net/http` implementation. Every fixture file is a standalone JSON document capturing:
1. **Request definition:** HTTP method, URI path, query parameters, headers, and payload.
2. **Identity context:** Auth tokens, actor credentials, workspace tenant scope, roles, and granular permissions.
3. **Environment setup:** Pre-condition state including system/tenant PostgreSQL table seeds, Redis cache/counter seeds, and external mock service stubs (S3, SMTP, ClamAV).
4. **Expected response:** Exact HTTP status code, canonical headers, expected body JSON AST, and field-level normalization directives.
5. **Expected side effects:** Post-condition assertions on committed database row mutations, Redis key/TTL mutations, transactional outbox jobs, and structured audit log entries.
6. **Normalization rules:** Bounded time tolerances and explicit, closed-list field replacements per `P2-08`.

### Core Rules
- **One scenario per file:** Each `.json` fixture file represents exactly one deterministic HTTP request execution.
- **Traceable to source code:** Every fixture `metadata.source_ref` MUST cite the exact Rust handler, service, or integration test (`path:line`) from which it is derived.
- **Fail closed:** Any unlisted field difference, unexpected header, unseeded table mutation, or unmanaged Redis key constitutes parity failure. Wildcard ignores are strictly forbidden.

---

## 2. Directory Layout

Fixtures are organized by domain under `docs/migration/rust-go/fixtures/`:

```text
docs/migration/rust-go/fixtures/
├── README.md                      # This specification (P3-01)
├── example_fixture.json           # Canonical reference fixture (P3-01)
├── fixture.schema.json            # JSON Schema (Draft-07) definition
├── state-comparators.md           # DB/Redis/Rollback comparator specification (P3-11)
├── differential-harness.md        # Rust vs Go test runner specification (P3-12)
├── auth/                          # P3-02 Auth & RBAC fixtures
├── workspaces/                    # P3-03 Workspace, settings, feature flags
├── employees/                     # P3-04 Employees, employments, assignments
├── attendance/                    # P3-05 Attendance, punches, time tracking
├── roster/                        # P3-06 Shifts, schedules, roster patterns
├── leaves/                        # P3-07 Leaves, balances, approvals, ESS
├── payroll/                       # P3-08 Payroll, calculations, reports, dashboard
├── files/                         # P3-09 Files, documents, e-signatures, S3
└── system/                        # P3-10 Lifecycle, condition types, system admin
```

---

## 3. Canonical Schema Specification

Every fixture file MUST conform to the schema described below.

### 3.1 Top-Level Properties

| Field | Type | Required | Description |
|---|---|:---:|---|
| `schema_version` | `string` | **Yes** | Semantic version of fixture format. Must be `"1.0.0"`. |
| `metadata` | `object` | **Yes** | Identification, domain classification, and source traceability. |
| `request` | `object` | **Yes** | HTTP client request definition. |
| `identity` | `object` | **Yes** | Security principal, tenant context, and authentication tokens. |
| `setup` | `object` | **Yes** | Initial state setup before request execution (DB, Redis, Stubs). |
| `expected_response` | `object` | **Yes** | Expected HTTP response status, headers, body, and normalization tags. |
| `expected_side_effects` | `object` | **Yes** | Expected state mutations in DB, Redis, Outbox, and Audit log. |
| `normalization_rules` | `object` | **Yes** | Permitted tolerances and closed-list ignored/masked fields. |

---

### 3.2 `metadata` Schema

```json
{
  "id": "WS-LOC-001",
  "domain": "workspaces",
  "name": "create_work_location_success",
  "description": "Standard creation of non-headquarters work location by workspace owner",
  "source_ref": "apiserver/tests/integration/workspace/locations.rs:40-70; apiserver/src/handlers/workspaces/locations.rs:103-127",
  "tags": ["locations", "workspace", "crud", "p1"]
}
```

- `id` (`string`, required): Unique identifier formatted as `[DOMAIN]-[MODULE]-[000]`.
- `domain` (`string`, required): One of `auth`, `workspaces`, `employees`, `attendance`, `roster`, `leaves`, `payroll`, `files`, `system`.
- `name` (`string`, required): Concise snake_case name of scenario.
- `description` (`string`, required): Description of preconditions, intent, and expected behavior.
- `source_ref` (`string`, required): Concrete file and line references from Rust codebase grounding the fixture.
- `tags` (`array[string]`, optional): Indexing labels.

---

### 3.3 `request` Schema

```json
{
  "method": "POST",
  "path": "/api/v1/workspaces/{ws_id}/locations",
  "path_params": {
    "ws_id": "018d0000-0000-7000-8000-000000000001"
  },
  "query": {},
  "headers": {
    "content-type": "application/json",
    "x-request-id": "018d0000-0000-7000-8000-000000000099"
  },
  "body": {
    "name": "Branch Office Da Nang",
    "address": "123 Tran Phu, Da Nang",
    "timezone": "Asia/Ho_Chi_Minh",
    "is_headquarters": false,
    "status": "active"
  }
}
```

- `method` (`string`, required): HTTP verb in uppercase: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`.
- `path` (`string`, required): Canonical route path starting with `/api/v1/`. May contain `{param}` placeholders.
- `path_params` (`object[string, string]`, optional): Map of placeholder keys to values. Runner substitutes `{param}` in `path`.
- `query` (`object[string, string|number|boolean]`, optional): Key-value query parameters (e.g. `page`, `limit`).
- `headers` (`object[string, string]`, optional): Request headers. Header keys are lowercase.
- `body` (`any`, optional): Request body payload (JSON object, array, string, or null).

---

### 3.4 `identity` Schema

Models the principal extracted by Axum extractors (`UserId`, `MemberContext`, `SystemContext`, `AgentAuth`):

```json
{
  "auth_type": "bearer",
  "auth_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user_id": "018d0000-0000-7000-8000-000000000010",
  "workspace_id": "018d0000-0000-7000-8000-000000000001",
  "role": "owner",
  "permissions": [
    "workspaces:read",
    "workspaces:write",
    "locations:read",
    "locations:write"
  ],
  "system_roles": []
}
```

- `auth_type` (`string`, required): `"bearer"`, `"cookie"`, `"header_impersonation"` (debug mode `X-User-Id`), `"service_token"`, or `"anonymous"`.
- `auth_token` (`string`, nullable): JWT access token or API token if using Bearer/Cookie.
- `user_id` (`string(uuid)`, nullable): Target authenticated user UUID.
- `workspace_id` (`string(uuid)`, nullable): Target tenant workspace UUID.
- `role` (`string`, nullable): Workspace primary role (`owner`, `admin`, `hr_manager`, `shift_manager`, `team_lead`, `employee`).
- `permissions` (`array[string]`, required): Effective permissions list granted to this principal.
- `system_roles` (`array[string]`, optional): System-level roles (`platform_admin`).

---

### 3.5 `setup` Schema

State pre-conditions applied to isolated test databases, Redis instances, and mock service adapters before executing `request`:

```json
{
  "db_seeds": [
    {
      "database": "system",
      "table": "users",
      "rows": [
        {
          "id": "018d0000-0000-7000-8000-000000000010",
          "email": "owner@example.com",
          "password_hash": "$argon2id$v=19$m=19456,t=2,p=1$...",
          "full_name": "Workspace Owner",
          "created_at": "2026-01-01T00:00:00Z"
        }
      ]
    },
    {
      "database": "tenant",
      "table": "work_locations",
      "rows": [
        {
          "id": "018d0000-0000-7000-8000-000000000020",
          "workspace_id": "018d0000-0000-7000-8000-000000000001",
          "name": "Headquarters",
          "address": "1 Main St",
          "timezone": "Asia/Ho_Chi_Minh",
          "is_headquarters": true,
          "status": "active",
          "created_at": "2026-01-01T00:00:00Z",
          "updated_at": "2026-01-01T00:00:00Z"
        }
      ]
    }
  ],
  "redis_seeds": [
    {
      "key": "cache:auth:permissions:018d0000-0000-7000-8000-000000000001:018d0000-0000-7000-8000-000000000010",
      "type": "string",
      "value": "{\"workspace_id\":\"018d0000-0000-7000-8000-000000000001\",\"user_id\":\"018d0000-0000-7000-8000-000000000010\",\"role\":\"owner\",\"permissions\":[\"locations:write\"]}",
      "ttl_seconds": 3600
    }
  ],
  "external_stubs": [
    {
      "service": "s3",
      "endpoint": "https://s3.ap-southeast-1.amazonaws.com",
      "action": "PutObject",
      "response": {
        "status": 200,
        "headers": {"etag": "\"b10a8db164e0754105b7a99be72e3fe5\""}
      }
    }
  ]
}
```

- `db_seeds` (`array[object]`, optional):
  - `database` (`string`): `"system"` or `"tenant"`.
  - `table` (`string`): Target table name.
  - `rows` (`array[object]`): Raw column key-value objects to insert.
- `redis_seeds` (`array[object]`, optional):
  - `key` (`string`): Full Redis key.
  - `type` (`string`): `"string"`, `"hash"`, `"list"`, `"zset"`.
  - `value` (`any`): String, object (for JSON string or hash fields), or array (for list/zset).
  - `ttl_seconds` (`number`, nullable): Key TTL.
- `external_stubs` (`array[object]`, optional):
  - `service` (`string`): `"s3"`, `"smtp"`, `"clamav"`.
  - `endpoint` (`string`): Mock endpoint URI.
  - `action` (`string`): Service action / command name.
  - `response` (`object`): Stubbed response status, headers, and body.

---

### 3.6 `expected_response` Schema

```json
{
  "status": 201,
  "headers": {
    "content-type": "application/json",
    "x-request-id": "018d0000-0000-7000-8000-000000000099"
  },
  "body": {
    "id": "018d0000-0000-7000-8000-000000000090",
    "workspace_id": "018d0000-0000-7000-8000-000000000001",
    "name": "Branch Office Da Nang",
    "address": "123 Tran Phu, Da Nang",
    "timezone": "Asia/Ho_Chi_Minh",
    "is_headquarters": false,
    "status": "active",
    "created_at": "2026-10-19T10:00:00Z",
    "updated_at": "2026-10-19T10:00:00Z"
  },
  "body_normalizations": [
    {
      "path": "$.id",
      "kind": "generated_uuid",
      "reason": "Location ID is generated with UUIDv4 in Rust location repository",
      "source_citation": "apiserver/src/repositories/diesel_location_repository.rs:98-100"
    },
    {
      "path": "$.created_at",
      "kind": "generated_time",
      "tolerance_ms": 1000,
      "reason": "Timestamp generated via Utc::now() at row creation",
      "source_citation": "apiserver/src/repositories/diesel_location_repository.rs:106"
    },
    {
      "path": "$.updated_at",
      "kind": "generated_time",
      "tolerance_ms": 1000,
      "reason": "Timestamp generated via Utc::now() at row creation",
      "source_citation": "apiserver/src/repositories/diesel_location_repository.rs:107"
    }
  ]
}
```

- `status` (`integer`, required): Expected HTTP status code (`200`, `201`, `204`, `400`, `401`, `403`, `404`, `409`, `429`, `500`).
- `headers` (`object[string, string]`, required): Expected HTTP headers. Header keys are lowercase. Content-Type and custom headers MUST be explicitly defined.
- `body` (`any`, nullable): Expected response payload (JSON object, array, or null for 204). Error responses MUST use the envelope `{"error": {"code": "...", "message": "..."}}` per `P2-03`.
- `body_normalizations` (`array[object]`, optional): Field-level nondeterminism declarations conforming to `P2-08`.

---

### 3.7 `expected_side_effects` Schema

Defines the exact mutations that must occur as a consequence of the request:

```json
{
  "db_mutations": [
    {
      "database": "tenant",
      "table": "work_locations",
      "operation": "INSERT",
      "pk_filter": {
        "name": "Branch Office Da Nang",
        "workspace_id": "018d0000-0000-7000-8000-000000000001"
      },
      "columns": {
        "name": "Branch Office Da Nang",
        "address": "123 Tran Phu, Da Nang",
        "timezone": "Asia/Ho_Chi_Minh",
        "is_headquarters": false,
        "status": "active"
      }
    }
  ],
  "redis_mutations": [],
  "outbox_events": [],
  "audit_events": [
    {
      "action": "location.created",
      "workspace_id": "018d0000-0000-7000-8000-000000000001",
      "actor_id": "018d0000-0000-7000-8000-000000000010",
      "actor_type": "user",
      "target_resource_id": "$.body.id",
      "details": {
        "name": "Branch Office Da Nang",
        "timezone": "Asia/Ho_Chi_Minh"
      }
    }
  ]
}
```

- `db_mutations` (`array[object]`):
  - `database` (`string`): `"system"` or `"tenant"`.
  - `table` (`string`): Table name.
  - `operation` (`string`): `"INSERT"`, `"UPDATE"`, `"DELETE"`.
  - `pk_filter` (`object`): Column match criteria identifying the mutated row.
  - `columns` (`object`): Expected column values in the mutated row.
- `redis_mutations` (`array[object]`):
  - `command` (`string`): `"SET"`, `"DEL"`, `"INCR"`, `"EXPIRE"`, `"RPUSH"`, `"HSET"`.
  - `key` (`string`): Target Redis key or key pattern.
  - `value` (`any`): Expected value written.
  - `ttl_seconds` (`number`, optional): Expected TTL or TTL range.
- `outbox_events` (`array[object]`):
  - `job_type` (`string`): Job type identifier (e.g. `send_email`, `process_attendance`).
  - `status` (`string`): Initial status (must be `"pending"`).
  - `payload` (`object`): JSON payload of the outbox job.
- `audit_events` (`array[object]`):
  - `action` (`string`): Canonical audit event action string (e.g. `location.created`, `employee.updated`).
  - `workspace_id` (`string(uuid)`): Workspace scope.
  - `actor_id` (`string(uuid)`, nullable): User UUID of actor.
  - `actor_type` (`string`): `"user"`, `"agent"`, or `"system"`.
  - `target_resource_id` (`string`): Resource UUID or JSONPath reference (e.g. `"$.body.id"`).
  - `details` (`object`, nullable): Details JSON payload saved to `audit_logs.details`.

---

### 3.8 `normalization_rules` Schema

Encapsulates global and field-specific rules allowed by `P2-08` (`docs/migration/rust-go/comparison-normalization.md`):

```json
{
  "tolerances": {
    "timestamp_ms": 1000,
    "redis_ttl_seconds": 5
  },
  "ignored_fields": [
    {
      "path": "response.headers.x-request-id",
      "reason": "Axum request_id_middleware generates new UUIDv7 if incoming header absent",
      "source_citation": "apiserver/src/mw/request_id.rs:17-25",
      "verification": "uuid_v7"
    }
  ]
}
```

- `tolerances` (`object`):
  - `timestamp_ms` (`number`): Maximum millisecond divergence between clock times (`created_at`, `updated_at`). Default `1000`.
  - `redis_ttl_seconds` (`number`): Permitted jitter on TTL comparison. Default `5`.
- `ignored_fields` (`array[object]`):
  - `path` (`string`): Dot-notation path to field (e.g. `response.headers.x-request-id`, `response.body.token`).
  - `reason` (`string`): Technical reason justifying divergence.
  - `source_citation` (`string`): Exact Rust source citation.
  - `verification` (`string`): Validator rule (`"uuid_v7"`, `"uuid_v4"`, `"iso8601"`, `"non_empty_string"`).

---

## 4. Verification and Determinism Gate

A fixture is considered **valid and deterministic** when it satisfies:
1. **Schema compliance:** Passes JSON Schema validation against `docs/migration/rust-go/fixtures/fixture.schema.json`.
2. **Double execution:** Running the fixture twice against a freshly reset database produces byte-for-byte identical normalized diff reports.
3. **Parity completeness:** Contains all required sections; no placeholder comments or missing expected side effects.
4. **Source ground truth:** The cited `source_ref` exists in the repository and corresponds to the endpoint and behavior under test.

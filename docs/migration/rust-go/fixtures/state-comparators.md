# State Comparators Specification — P3-11

**Scope:** Specification for database row-level comparator, Redis key/TTL/value comparator, and transaction rollback verifier for differential testing of Rust and Go `apiserver`.  
**Ledger Task:** `P3-11`  
**Parent Dependencies:** `P2-08` (`comparison-normalization.md`), `P2-04` (`database-contract.md`), `P2-05` (`redis-contract.md`), `P3-01` (`fixtures/README.md`).  
**Verification Gate:** Differential runner can detect subtle row divergences, TTL mismatches, JSON payload drift, missing audit logs, and failed rollbacks.

---

## 1. Overview and Architecture

Differential API parity requires not only identical HTTP wire outputs (status, headers, body) but also **byte-for-byte and semantic parity in persistent side effects**. When a request modifies application state, both the reference Rust engine and the Go candidate must perform equivalent operations in PostgreSQL and Redis.

```text
               ┌────────────────────────────────────────────────────────┐
               │              Differential Test Runner                  │
               └──────────────────────────┬─────────────────────────────┘
                                          │
                  ┌───────────────────────┴───────────────────────┐
                  ▼                                               ▼
         [ Pre-Run Snapshot ]                            [ Post-Run Snapshot ]
      - System DB (table rows)                        - System DB (table rows)
      - Tenant DB (table rows)                        - Tenant DB (table rows)
      - Redis keys & TTLs                             - Redis keys & TTLs
                  │                                               │
                  └───────────────────────┬───────────────────────┘
                                          ▼
                         ┌─────────────────────────────────┐
                         │       Comparator Engine         │
                         │  1. DB Row-Level Comparator     │
                         │  2. Redis Key/TTL Comparator    │
                         │  3. Rollback Verifier           │
                         └─────────────────────────────────┘
```

The comparator engine operates non-invasively by snapshotting database and cache state immediately before and after fixture execution, computing mutation sets, and validating them against fixture `expected_side_effects` and against each other.

---

## 2. Database Row-Level Comparator

### 2.1 Database Topology & Scoping

The system employs a dual-database model (`docs/migration/rust-go/database-contract.md:17-22`):
1. **System Database:** Contains global entities (`workspaces`, `users`, `workspace_members`, `system_roles`, `user_system_roles`, `system_audit_logs`, `sessions`, `authorized_agents`).
2. **Tenant Databases:** Dedicated per-workspace databases or isolated schemas containing domain entities (`work_locations`, `employments`, `employees`, `shifts`, `time_entries`, `leaves`, `outbox_jobs`, `audit_logs`).

The comparator connects to both the system pool and the active workspace tenant pool using credentials supplied by test runner environment configuration (`TEST_SYSTEM_DATABASE_URL`, `TEST_TENANT_DATABASE_URL`).

### 2.2 Snapshot Protocol

For every table relevant to the domain or modified during the run:
1. **Pre-execution Capture:**
   ```sql
   SELECT * FROM <table_name> WHERE workspace_id = $1 (or unconstrained for system tables);
   ```
   Rows are indexed by their primary key in memory: `Map<PK_Hash, RowMap>`.
2. **Request Execution:** Execute HTTP request against candidate server.
3. **Post-execution Capture:**
   Re-query the same tables:
   ```sql
   SELECT * FROM <table_name> WHERE workspace_id = $1;
   ```
4. **Delta Calculation:**
   - **INSERT:** Present in post-execution, absent from pre-execution.
   - **DELETE:** Present in pre-execution, absent from post-execution.
   - **UPDATE:** Present in both, but one or more column values differ.
   - **UNCHANGED:** Present in both with identical values.

### 2.3 Row Comparison & Normalization Rules

Columns are compared strictly according to the rules established in `P2-08` and `database-contract.md`:

| Column Type | Comparison Algorithm | Allowed Nondeterminism | Failure Conditions |
|---|---|---|---|
| **Primary Key (UUID)** | Check version format (UUIDv7 or UUIDv4). Match cross-table foreign key references. | Generated UUIDv7 value may differ between Rust and Go runs. | Non-UUID format; FK mismatch with referencing child table or response body. |
| **Workspace ID** | Exact string/UUID match (`workspace_id`). | None. Tenant boundary is strict. | Any cross-tenant leak or mismatch. |
| **Timestamps (`Timestamptz`)** | Parse as UTC `chrono::DateTime<Utc>` / `time.Time`. Calculate absolute difference: $\Delta t = |t_{\text{observed}} - t_{\text{expected}}|$. | Permitted within `tolerances.timestamp_ms` (default 1000ms). | $\Delta t > \text{tolerance}$; missing timezone offset; non-UTC persistence. |
| **Numeric / Decimal** | Compare fixed-point arbitrary precision decimal (`shopspring/decimal` vs `rust_decimal`). | Zero tolerance. Scale and precision must match. | Floating point rounding error; representation mismatch. |
| **Text / Varchar** | Exact character and byte comparison after UTF-8 validation. | None. | Truncation, case divergence, or leading/trailing whitespace change. |
| **JSONB** | Recursively parse into JSON AST. Normalize object key ordering. Compare AST values. | Unordered object keys permitted. Field values must match. | Missing keys, `null` vs missing divergence, array order divergence. |
| **Boolean** | Exact boolean match (`true` vs `false`). | None. | Truthy/falsy conversion mismatch. |

### 2.4 Partitioned Table Comparator: `audit_logs`

Audit logs use PostgreSQL table partitioning by month (`docs/migration/rust-go/database-contract.md:10`):
- Parent table: `public.audit_logs`.
- Partitions: `audit_logs_y2026m01` ... `audit_logs_y2026m12`, `audit_logs_default`.

**Comparison Rule:**
The comparator queries the logical parent table `public.audit_logs`:
```sql
SELECT id, workspace_id, actor_id, actor_type, target_resource_id, action, details, created_at
FROM public.audit_logs
WHERE workspace_id = $1 AND created_at >= $2
ORDER BY created_at ASC;
```
- Validate `action` matches `audit_events[].action`.
- Validate `target_resource_id` matches the created or mutated entity ID.
- Validate `details` JSONB contains required domain metadata.

### 2.5 Outbox Jobs Comparator: `outbox_jobs`

The outbox table (`apiserver/src/schema.rs:1467-1478`) guarantees transactional job dispatch:
```sql
SELECT id, workspace_id, job_type, payload, status, error_message, created_at
FROM public.outbox_jobs
WHERE workspace_id = $1 AND status = 'pending'
ORDER BY created_at ASC;
```
- Assert `status == 'pending'`.
- Assert `job_type` matches expected job string (e.g. `roster_generate`, `leave_balance_update`).
- Assert `payload` matches `outbox_events[].payload`.

---

## 3. Redis Key, TTL, and Value Comparator

### 3.1 Redis Inspection Protocol

The Redis comparator inspects keys modified or touched during request execution against the 43 key families documented in `docs/migration/rust-go/redis-contract.md`.

```text
Pre-Scan (Redis KEYS/SCAN) ──► Execute ──► Post-Scan ──► Delta (New/Updated/Deleted Keys)
                                                                 │
               ┌─────────────────────────────────────────────────┴────────────────────────┐
               ▼                                                                          ▼
      [ Key Name & Type ]                                                        [ Value & TTL ]
  - Verify key pattern template                                              - Decode JSON / Int / Bytes
  - Verify Redis type (string, hash, list, zset)                             - Check TTL within bounds
```

### 3.2 Value Decoding Matrix

The comparator decodes Redis values based on the registered key family:

| Key Template | Redis Type | Encoding | Comparator Logic |
|---|---|---|---|
| `cache:workspace:{id}` | `string` | JSON `CachedWorkspaceConfig` | Parse JSON AST; match workspace settings and config entries. |
| `cache:shifts:{id}` | `string` | JSON `Vec<CachedShift>` | Parse JSON AST; compare shift list order and fields. |
| `cache:auth:permissions:{ws}:{user}` | `string` | JSON `CachedMemberContext` | Parse JSON AST; verify role and permissions array. |
| `roster:version:{ws}` | `string` | UTF-8 UUID string | Assert valid UUID; verify version increment on invalidation. |
| `login:attempts:{email}` | `string` | Decimal Integer string | Parse `int64`; verify atomic counter value. |
| `rate:{id}:{tier}:{path}` | `string` | Decimal Integer string | Parse `int64`; verify rate counter incremented. |
| `jwt:revoked:{sig}` | `string` | Literal `"1"` | Assert value is `"1"`; verify TTL equals remaining token lifetime. |
| `smtp:config` | `hash` | String fields | Compare `host`, `port`, `username`, `password`, `from`, `base_url`. |
| `worker:jobs:{tier}` | `list` | JSON `Job` entries | Parse queue items via `LRANGE`; verify job payload, priority, and serialization. |
| `report:job_list` | `zset` | Scored UUIDs | Inspect `ZREVRANGEBYSCORE`; verify epoch timestamp score. |

### 3.3 TTL Verification Rules

Every TTL is checked using `TTL <key>` or `PTTL <key>`:
1. **Configured TTL:** Match expected contract value (e.g., 300s for cache, 900s for auth, 86400s for jobs).
2. **Tolerance Window:** Due to elapsed test execution time, the observed TTL will be slightly less than the configured TTL:
   $$\text{TTL}_{\text{configured}} - \delta_{\text{test\_duration}} \le \text{TTL}_{\text{observed}} \le \text{TTL}_{\text{configured}}$$
   Maximum permitted jitter is defined in `normalization_rules.tolerances.redis_ttl_seconds` (default 5s).
3. **No-TTL Keys:** If the contract specifies no TTL (e.g. `roster:version:{ws}`, `smtp:config`), assert `TTL <key> == -1`.
4. **Expired/Non-Existent Keys:** Assert `TTL <key> == -2`.

### 3.4 Invalidation & Deletion Verification

When an update or delete operation occurs, the comparator asserts that target cache keys were invalidated:
- Query `EXISTS <key>`.
- Assert return value is `0` (deleted) or new cache version generation is bumped.
- If wildcard invalidation is used (e.g. `cache:auth:permissions:{ws}:*`), execute `SCAN` to verify no matching keys remain.

---

## 4. Rollback Verifier

### 4.1 Principle

When an API request results in an error status code (`400 Bad Request`, `401 Unauthorized`, `403 Forbidden`, `404 Not Found`, `409 Conflict`, `422 Unprocessable Entity`, `429 Too Many Requests`, `500 Internal Server Error`), **the transaction MUST roll back cleanly**.

In Rust `apiserver`, transactional handlers use `with_tenant_tx` (`apiserver/src/diesel_pool/core.rs:70-92`), which guarantees that if the handler closure returns `Err(AppError)`, Diesel issues `ROLLBACK` on the database connection.

### 4.2 Verifier Rules

Upon receiving an HTTP error response ($status \ge 400$):
1. **Database Delta is Zero:**
   $$\Delta_{\text{DB}} = \text{Rows}_{\text{post}} \setminus \text{Rows}_{\text{pre}} \equiv \emptyset$$
   - No rows may be inserted into tenant or system business tables.
   - No existing rows may be updated.
   - No existing rows may be deleted or soft-deleted.
2. **Audit Log Exception:**
   - Security-relevant rejection events (e.g. failed login attempts, unauthorized access warnings) MAY write to `system_audit_logs` or `audit_logs` if explicitly declared in `expected_side_effects.audit_events`.
   - If no audit event is declared, audit tables must also remain unchanged.
3. **Outbox Cleanliness:**
   - `outbox_jobs` delta must be strictly zero. No background job may be enqueued on a failed business transaction.
4. **Redis State:**
   - No cache entries may be dirtied or partially written.
   - Rate limit counters (`rate:*`) and failed login counters (`login:attempts:*`, `login:lockout:*`) may increment if the endpoint is rate-limited or auth-guarded, matching the contract.
5. **No Orphaned Storage Objects:**
   - If a file upload fails validation or malware scan, no object may remain in S3 mock storage.

---

## 5. Comparator Output Schema

The comparator produces structured JSON diff results:

```json
{
  "comparator": "state_comparator",
  "fixture_id": "WS-LOC-001",
  "verdict": "PASS",
  "database": {
    "status": "PASS",
    "mutations_found": 1,
    "mutations_expected": 1,
    "diffs": []
  },
  "redis": {
    "status": "PASS",
    "mutations_found": 0,
    "mutations_expected": 0,
    "diffs": []
  },
  "audit": {
    "status": "PASS",
    "events_found": 1,
    "events_expected": 1,
    "diffs": []
  },
  "rollback": {
    "verified": true,
    "leak_detected": false
  }
}
```

If a discrepancy occurs, `verdict` becomes `"FAIL"` and `diffs` lists exact paths, expected values, and observed values:

```json
{
  "table": "work_locations",
  "pk": "018d0000-0000-7000-8000-000000000090",
  "field": "timezone",
  "expected": "Asia/Ho_Chi_Minh",
  "observed": "UTC",
  "divergence": "value_mismatch"
}
```

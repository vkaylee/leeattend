# Task P2-01 — OpenAPI Baseline & Contract Freeze

**Date:** 2026-10-19  
**Git HEAD Commit SHA:** `456bdd4c00d6c050cbcfe8c43777234c4650aa8c`  
**Parent Task Ledger:** `rust-go-migration-task-ledger.md` (Task P2-01)  
**Status:** Baseline established; generated frontend types verified unchanged.

---

## 1. Source References & Line Citations

| Artifact Category | File Path | Line Range | Last Commit SHA | Verification Role |
|---|---|---|---|---|
| **OpenAPI Specification SSOT** | `apiserver/src/openapi.rs` | `1-1223` | `9346e5186d3bf0ebe1caf3510f526c4fa376e9e1` | Primary Rust `utoipa::OpenApi` contract declaration (`ApiDoc`). |
| ↳ Path Registrations | `apiserver/src/openapi.rs` | `9-480` | `9346e5186d3bf0ebe1caf3510f526c4fa376e9e1` | 397 handler path registrations in `paths(...)`. |
| ↳ Schema Registrations | `apiserver/src/openapi.rs` | `484-951` | `9346e5186d3bf0ebe1caf3510f526c4fa376e9e1` | 415 schema declarations in `components(schemas(...))`. |
| ↳ Generation & Parity Tests | `apiserver/src/openapi.rs` | `960-1223` | `9346e5186d3bf0ebe1caf3510f526c4fa376e9e1` | Unit tests for JSON emission, route coverage, and path matching. |
| **CLI Generation Entrypoint** | `apiserver/src/main.rs` | `50-56` | `456bdd4c00d6c050cbcfe8c43777234c4650aa8c` | `gen-openapi` CLI subcommand emitting pretty JSON to stdout. |
| **API Sync Pipeline Script** | `scripts/_sync-api.sh` | `1-28` | `456bdd4c00d6c050cbcfe8c43777234c4650aa8c` | Canonical generator runner: backend dump + `bunx openapi-typescript`. |
| **Frontend Codegen Script** | `webdashboard/package.json` | `26` | `456bdd4c00d6c050cbcfe8c43777234c4650aa8c` | `"gen-api": "bunx openapi-typescript openapi.json -o src/types/api-generated.ts"`. |
| **Frontend Generated Types** | `webdashboard/src/types/api-generated.ts` | `1-20032` | `9346e5186d3bf0ebe1caf3510f526c4fa376e9e1` | Generated TypeScript SDK interfaces: `paths`, `components`, `operations`. |
| **Frontend Re-exports Barrel** | `webdashboard/src/lib/api/api-generated-reexports.ts` | `1-76` | `456bdd4c00d6c050cbcfe8c43777234c4650aa8c` | 1:1 schema type aliases consumed by frontend components. |
| **Frontend Type Overrides** | `webdashboard/src/lib/api/types.ts` | `1-106` | `456bdd4c00d6c050cbcfe8c43777234c4650aa8c` | Local type augmentations, custom wrappers, and manual interfaces. |
| **Route Modules Reference** | `apiserver/src/routes/` (19 modules + `mod.rs`, 1,440 lines) | `1-1440` | `456bdd4c00d6c050cbcfe8c43777234c4650aa8c` | Axum HTTP router definitions mounted under `/api/v1` prefix (`apiserver/src/lib.rs:351-352`). |

---

## 2. OpenAPI Generation Pipeline

The LeeAttend API contract emission and synchronization follows a two-tier pipeline:

1. **Backend Emission (`utoipa` OpenAPI 3.1.0):**
   - Subcommand: `cargo run --bin apiserver -- gen-openapi` (`apiserver/src/main.rs:50-56`).
   - Implementation: Calls `ApiDoc::openapi().to_pretty_json()?` and streams to standard output.
   - Filtering: Container stdout logging lines preceding the opening `{` are stripped via `sed -n '/^{/,$p'` in `scripts/_sync-api.sh:20`.

2. **Frontend Ingestion (`openapi-typescript` v7.13.0):**
   - Ingestion command: `npm run gen-api` / `bunx openapi-typescript openapi.json -o src/types/api-generated.ts` (`webdashboard/package.json:26`).
   - Ephemeral artifact cleanup: `scripts/_sync-api.sh:25` executes `rm -f openapi.json` immediately after generation completes.
   - Ignored status: `.gitignore:47:openapi.json` permanently ignores the intermediate JSON file from git tracking.

---

## 3. Current OpenAPI Artifact Status & Hash

| Property | Value | Evidence / Rationale |
|---|---|---|
| **Current Static Artifact on Disk** | `NONE` (`UNKNOWN`) | `openapi.json` and `webdashboard/openapi.json` do not exist in the working directory. |
| **Git Tracking Policy** | Ignored (`.gitignore:47:openapi.json`) | Ignored intentionally to prevent intermediate build file churn. |
| **Deletion Commit** | `e2bb36f4a2556ba221d7e57de6f7e9b8b8add0f7` (2026-02-13) | Commit message: *"chore: delete openapi.json from repo (stopped generating in 9fb67d6)"*. |
| **Preceding Commit** | `9fb67d68e7d89cc620e8fa8b55a8511883a62a5a` (2026-02-13) | Stopped automatic disk writes during server startup. |
| **Historical Artifact SHA-256** | `f45de1593a2ab7067be9329038e535823f82ecfd368b309b83c67917adb7a35e` | Hash of `e2bb36f4a^:openapi.json` (8,640 bytes, 2 paths, 2 operations, 15 schemas); retained only as historical context, not current baseline. |
| **Current Baseline Artifact Hash** | `UNKNOWN` | No static `.json` artifact exists in the repository. As per P2-01 rule: "State whether generated types are unchanged and list exact diffs or UNKNOWN if no generated artifact exists. Do not generate or modify code." |

---

## 4. Schema and Operation Counts

### A. Rust Contract SSOT (`apiserver/src/openapi.rs`)

| Metric | Count | Source Citation |
|---|---|---|
| **Handler Path Registrations** | **397** | `apiserver/src/openapi.rs:9-480` |
| **Unique Declared Handlers** | **397** | Distinct handler identifiers registered in `paths(...)`. |
| **Axum Mounted Serving Routes** | **400** | 19 route modules (`apiserver/src/lib.rs:309-328`) mounted under `/api/v1` (`apiserver/src/lib.rs:351-352`). |
| **OpenAPI Registrations Unmounted in Axum** | **3** | `create_signature_request`, `get_pending_signatures`, `sign_document` (`apiserver/src/openapi.rs:345-347`, `decisions.md#D-08`). |
| **Axum Handlers Whitelisted from OpenAPI** | **5** | `ws_handler`, `report_stream_ws`, `monitor_stream_sse`, `health_check`, `readiness_check` (`apiserver/src/openapi.rs:1061-1068`). |
| **Schema Declarations in `schemas(...)`** | **415** | `apiserver/src/openapi.rs:484-952`. |
| **Unique Schema Short Names** | **410** | 410 distinct schema names after resolving duplicates. |
| **Duplicate Schema Declarations in `schemas(...)`** | **5** | `Department` (560, 727), `CreateDepartmentRequest` (561, 728), `UpdateDepartmentRequest` (562, 729), `ImportJobStatus` (515, 631), `JobStatusResponse` (791, 858). |

### B. Frontend Generated Types (`webdashboard/src/types/api-generated.ts`)

| Metric | Count | Source Citation |
|---|---|---|
| **Total Lines of Code** | **20,032** | `webdashboard/src/types/api-generated.ts:1-20032` |
| **SHA-256 Checksum** | `1f347eeaeb2d3500730605719671028297dc04c3c1dc86e2829c2149801e21b7` | Calculated on verbatim repository file. |
| **Working Tree Status** | **Unchanged for cited file** | No `git status --porcelain` entry for `webdashboard/src/types/api-generated.ts`; unrelated working-tree changes exist elsewhere. |
| **Path Keys (`export interface paths`)** | **279** | `webdashboard/src/types/api-generated.ts:6-4751`. |
| **Operation Method Bindings in `paths`** | **396** | 177 GET, 119 POST, 50 DELETE, 32 PUT, 18 PATCH. |
| **Operation Interfaces (`export interface operations`)** | **396** | `webdashboard/src/types/api-generated.ts:9606-20032`. |
| **Schema Definitions (`components["schemas"]`)** | **457** | `webdashboard/src/types/api-generated.ts:4754-9598`. |

---

## 5. Comparison against Frontend Generated Types

### A. Working Tree Invariant
The tracked file `webdashboard/src/types/api-generated.ts` is **UNCHANGED** from git commit `9346e5186d3bf0ebe1caf3510f526c4fa376e9e1` (2026-08-15 08:56:08 +0700). No status entry exists for this file. Regenerated-output parity against current Rust OpenAPI is **UNKNOWN** because no current JSON artifact exists and the local `cargo` executable is unavailable.

### B. Operation Reconciliation (397 vs 396)

Of the 397 handler registrations in `apiserver/src/openapi.rs:8-481`:
- **394 handlers** map 1:1 by identifier to operations in `webdashboard/src/types/api-generated.ts`.
- **2 handlers** map via explicit utoipa `operation_id` overrides:
  1. `crate::handlers::leaves::get_job_status` (`apiserver/src/openapi.rs:204`) overrides operation ID to `get_leave_job_status` (`apiserver/src/handlers/leaves.rs:651`), matching `webdashboard/src/types/api-generated.ts:18393`.
  2. `crate::handlers::report_export::get_job_status` (`apiserver/src/openapi.rs:272`) overrides operation ID to `get_report_export_job_status` (`apiserver/src/handlers/report_export.rs:207`), matching `webdashboard/src/types/api-generated.ts:19795`.
- **1 handler diff:** Exactly one registered handler is missing from generated operations:
  - Handler: `crate::handlers::recruitment_sources::get_recruitment_effectiveness` (`apiserver/src/openapi.rs:396`).
  - Path: `GET /v1/dashboard/recruitment-effectiveness` (`apiserver/src/handlers/recruitment_sources.rs:306`, mounted at `/api/v1/dashboard/recruitment-effectiveness` in `apiserver/src/routes/dashboard.rs:55`).
  - Cause: In commit `9346e5186d3bf0ebe1caf3510f526c4fa376e9e1`, its 6 response DTO schemas were manually pasted into `api-generated.ts:8044-8159`, but `scripts/_sync-api.sh` was not executed. The route was omitted from `paths` and `operations`.
  - Frontend consumption: The dashboard client invokes this route directly as a raw URL string in `webdashboard/src/hooks/use-dashboard.ts:317` (`/v1/dashboard/recruitment-effectiveness`), bypassing typed OpenAPI operation mapping. Recorded as decision `D-14`.

### C. Schema Reconciliation (410 unique declared vs 457 generated)

1. **Declared Schemas Present in Types (409):**
   - 409 unique short names declared in `apiserver/src/openapi.rs:484-951` exist verbatim under `components["schemas"]`.

2. **Declared Schema Alias Mismatch (1):**
   - `crate::handlers::worker_monitor::MonitorOverviewResponse` (`apiserver/src/openapi.rs:898`):
     - In `apiserver/src/handlers/worker_monitor.rs:608`, this is declared as `pub type MonitorOverviewResponse = MonitorOperationalSnapshot;`.
     - The generated OpenAPI spec emits the underlying struct name `MonitorOperationalSnapshot` (`webdashboard/src/types/api-generated.ts:7320`).

3. **Additional Schema Names in Generated Types (48):**
   - 48 names occur under `components["schemas"]` in `webdashboard/src/types/api-generated.ts:4754-9598` but do not occur among the 410 unique short names explicitly listed in `apiserver/src/openapi.rs:484-951`.
   - Provenance is **UNKNOWN**: no current OpenAPI JSON artifact exists, so this evidence cannot distinguish names emitted transitively by `openapi-typescript` from names added manually to the tracked generated file.
   - Enums: `AccrualFrequency`, `AssigneeType`, `AssignmentHistoryType`, `AssignmentStatusEnum`, `ContractStatus`, `ContractType`, `EntryType`, `FileCategory`, `FileOwnerType`, `LeaveDurationType`, `MonitorFreshness`, `OtAggregationMode`, `OtRequestStatus`, `OtStatus`, `PayrollCalculationMethod`, `PayrollRuleCategory`, `RecruitmentSourceType`, `RoleScopeType`, `RoundingPolicy`, `RuleAppliesTo`, `ScanStatus`, `SignatureStatus`, `SystemRoleType`, `TaskType`, `WorkerLiveness`.
   - Request / Response models: `BulkCreateWorkSegmentsRequest`, `CreateWorkSegmentRequest`, `EssCreateShiftSwapRequest`, `LeavePolicyConfig`, `MonitorOperationalSnapshot`, `OvertimeBreakdownResponse`, `OvertimeBreakdownRow`, `OvertimeBreakdownWorkspace`, `PaginatedApprovalWorkflowTasks`, `PaginatedBehalfAssignments`, `PaginatedOvertimeRequests`, `PaginatedShifts`, `RateQuantity`, `RespondShiftSwapRequest`, `SalaryBaseGroup`, `SalaryBaseGroupMember`, `SeniorityBonus`, `String`, `UpdateWorkSegmentRequest`, `WorkSegment`, `WorkSegmentListResponse`, `WorkflowStepRequest`, `WorkspaceFile`.

The 410-vs-457 totals reconcile as 409 exact declared names + 1 alias mismatch + 48 additional names; `MonitorOperationalSnapshot` is included in the 48-name set and also explains the alias mismatch, so categories are not disjoint. Counts are structural inventory, not proof of semantic equivalence.

### D. Unmounted Signature Routes Discrepancy

- OpenAPI operations:
  - `POST /v1/workspaces/{workspace_id}/documents/{document_id}/signature-requests` (`create_signature_request`, `webdashboard/src/types/api-generated.ts:1494`, `apiserver/src/handlers/signatures.rs:18`)
  - `GET /v1/ess/workspaces/{workspace_id}/signatures/pending` (`get_pending_signatures`, `webdashboard/src/types/api-generated.ts:1346`, `apiserver/src/handlers/signatures.rs:45`)
  - `POST /v1/ess/workspaces/{workspace_id}/signatures/{request_id}/sign` (`sign_document`, `webdashboard/src/types/api-generated.ts:1365`, `apiserver/src/handlers/signatures.rs:70`)
- Source: Registered in `apiserver/src/openapi.rs:345-347` and defined in `apiserver/src/handlers/signatures.rs:92-108`.
- Server status: Never mounted into Axum (`apiserver/src/lib.rs:308-328`).
- Contract Decision: `decisions.md#D-08` mandates that the Go implementation MUST NOT serve these routes (must return 404 `NOT_FOUND`), matching the observable runtime behavior of Rust.

---

## 6. Fixed Compatibility Rules for Go Implementation

1. **Operation ID Invariant:**
   - Every operation in Go OpenAPI specification must preserve the exact operation ID present in `webdashboard/src/types/api-generated.ts` (e.g. `get_leave_job_status`, `get_report_export_job_status`, `get_agents`).
   - The Go endpoint for recruitment effectiveness must export `operationId: get_recruitment_effectiveness` to restore contract parity.

2. **Path & Parameter Canonicalization:**
   - All paths must use standard OpenAPI 3.1 curly-brace parameter notation (`{workspace_id}`, `{ws_id}`, `{id}`).
   - The Go HTTP router (`net/http.ServeMux` + `oapi-codegen` strict server) must serve these paths under `/api/v1` exactly as inventoried in `docs/migration/rust-go/endpoint-matrix.csv`.

3. **Schema Field Nullability & Enums:**
   - Field names must preserve exact snake_case JSON serialization.
   - Enums must serialize to string literals matching `webdashboard/src/types/api-generated.ts` (e.g., `AdjustmentStatus: "pending" | "approved" | "rejected"`).
   - Date fields must remain RFC 3339 formatted strings (`YYYY-MM-DD` or ISO 8601 timestamps).

4. **Frontend Freeze:**
   - No modifications to `webdashboard/src/types/api-generated.ts` are permitted during Phase 2..Phase 7.
   - Go implementation must achieve 100% compatibility with existing generated frontend types before any code generation is re-run.

---

## 7. Unresolved Decisions & Escalations

| Decision ID | Subject | Escalation / Contract Rule | Status |
|---|---|---|---|
| **D-08** | Unmounted OpenAPI signature routes | Go MUST NOT mount `signatures` routes; parity target is observable 404 behavior, not unmounted OpenAPI declarations. | `Escalated` (Logged in `decisions.md`) |
| **D-14** | `get_recruitment_effectiveness` omission in frontend types | Go implements `/api/v1/dashboard/recruitment-effectiveness` (`operationId: get_recruitment_effectiveness`). Frontend types remain frozen until final cutover. | `Approved` (Logged in `decisions.md`) |

---

## 8. Verification Method

- **Static analysis & AST parsing:** Ran deterministic Python extraction on `apiserver/src/openapi.rs` and `webdashboard/src/types/api-generated.ts` to verify exact symbol counts.
- **Git status audit:** Checked `git status --porcelain` on all cited paths to guarantee a clean baseline.
- **Git history trace:** Traced deletion of `openapi.json` (`e2bb36f4a`, `9fb67d68e`) and addition of `recruitment_effectiveness` (`9346e5186`).
- **Endpoint matrix alignment:** Cross-referenced all 397 OpenAPI handlers against 403 rows in `docs/migration/rust-go/endpoint-matrix.csv`.

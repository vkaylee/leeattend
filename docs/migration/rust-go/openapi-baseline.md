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
| **Frontend Codegen Script** | `webdashboard/package.json` | `26` | `456bdd4c00d6c050cbcfe8c43777234c4650aa8c` | `bunx openapi-typescript openapi.json -o src/types/api-generated.ts`. |
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
| **Current Baseline Artifact Hash** | `UNKNOWN` | No static `.json` artifact exists in the repository. |

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
| **Duplicate Schema Declarations in `schemas(...)`** | **5** | `Department`, `CreateDepartmentRequest`, `UpdateDepartmentRequest`, `ImportJobStatus`, `JobStatusResponse`. |

### B. Frontend Generated Types (`webdashboard/src/types/api-generated.ts`)

| Metric | Count | Source Citation |
|---|---|---|
| **Total Lines of Code** | **20,032** | `webdashboard/src/types/api-generated.ts:1-20032` |
| **SHA-256 Checksum** | `1f347eeaeb2d3500730605719671028297dc04b7` | Calculated on verbatim repository file. |
| **Working Tree Status** | **Unchanged for cited file** | No status entry exists for the generated file at baseline. |
| **Path Keys (`export interface paths`)** | **279** | `webdashboard/src/types/api-generated.ts:6-4751`. |
| **Operation Method Bindings in `paths`** | **396** | 177 GET, 119 POST, 50 DELETE, 32 PUT, 18 PATCH. |
| **Operation Interfaces (`export interface operations`)** | **396** | `webdashboard/src/types/api-generated.ts:9606-20032`. |
| **Schema Definitions (`components["schemas"]`)** | **457** | `webdashboard/src/types/api-generated.ts:4754-9598`. |

---

## 5. Comparison against Frontend Generated Types

The generated contract remains frozen until a current OpenAPI artifact is available. Current static parity is UNKNOWN because the artifact is absent and the historical Rust source is not active runtime source.

---

## 6. Compatibility Rules

1. Generated frontend types MUST be regenerated from the active OpenAPI source; hand edits are prohibited.
2. Removed system tenant and platform-role operations MUST disappear from the generated contract after source restoration and regeneration.
3. Workspace RBAC and retained standalone SMTP, health, and monitor routes MUST remain in the regenerated contract.

---

## 7. Verification Method

- Confirm current OpenAPI source and artifact availability before generation.
- Run the repository API contract validation and frontend type generation through project wrappers.
- Verify removed paths are absent and retained paths remain present.

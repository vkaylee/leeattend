# Rust-to-Go Migration Recovery Checkpoint (Phase -1)

**Commit SHA:** `456bdd4c00d6c050cbcfe8c43777234c4650aa8c`  
**Date:** 2026-10-19  
**Ledger SHA256:** `d1dac8ab44fa054f0d7b911531dff29b85232d0bc385e2db250b45e6a653db02`  
**Plan SHA256:** `e5698841e100017201d08b85be5e6d709cfdb663923c20c66440ec892f93e259`  
**ADR SHA256:** `fa0749d3b32cef011899afbb6c9d5560110e6139c01455480d0eaf2e634d2d4c`  

---

## 1. Required Source Verification (R-01)

| Artifact Category | Path / Identifier | Status | Verification Note |
|---|---|---|---|
| **Rust Reference API** | `apiserver/` | Verified | 595 Rust source files, 205,855 LOC. Full reference behavior. |
| **Rust Entrypoints** | `apiserver/src/main.rs`, `apiserver/src/lib.rs` | Verified | Server bootstrap, middleware chain, CLI `gen-openapi`. |
| **OpenAPI Contract SSOT** | `apiserver/src/openapi.rs` | Verified | `utoipa::OpenApi` schema definition `ApiDoc`. |
| **Frontend Generated Types** | `webdashboard/src/types/api-generated.ts` | Verified | Generated from OpenAPI; immutable reference. |
| **Frontend Re-exports Barrel** | `webdashboard/src/lib/api/api-generated-reexports.ts` | Verified | 1:1 schema aliases for dashboard client. |
| **Deployment Definitions** | `docker-compose.yml`, `.compose/docker-compose.test.yml` | Verified | Multi-container definitions for api, db, redis, web. |
| **Integration Test Fixtures** | `apiserver/tests/integration/` | Verified | 53 test files spanning attendance, auth, employees, leave, payroll, shifts, workspaces. |
| **Database Migrations** | `apiserver/migrations/system/`, `apiserver/migrations/tenant/` | Verified | 9 system migrations, 31 tenant migrations with forward (`up.sql`) and backward (`down.sql`) scripts. |

---

## 2. In-Process Domain Calculation Verification (R-02)

| Component | Path | RPC / External Check | Result |
|---|---|---|---|
| **Attendance Pairing** | `apiserver/src/services/pairing/` | Grep `tonic`, `prost`, `grpc`, `attendance-compute` | Zero external RPC. In-process pure domain logic. |
| **Attendance Matcher** | `apiserver/src/services/attendance_matcher/` | Grep `tonic`, `prost`, `grpc`, `attendance-compute` | Zero external RPC. In-process pure domain logic. |
| **Payroll Evaluator** | `apiserver/src/services/payroll_evaluator.rs` | Grep `tonic`, `prost`, `grpc` | In-process calculation. |
| **Financial Calculator** | `apiserver/src/services/financial_calculator.rs` | Grep `tonic`, `prost`, `grpc` | In-process arbitrary-precision math. |
| **Cargo Workspace Dependencies** | `Cargo.toml`, `Cargo.lock` | Grep `tonic`, `prost` | Zero gRPC or external compute dependencies. |

**Conclusion:** Migration requires exclusively pure, in-process Go packages (`internal/domain/pairing`, `internal/domain/matcher`, etc.). No external compute service or RPC boundary is needed.

---

## 3. Context-Loss Recovery Protocol (R-03)

If context compaction or restart occurs:
1. Re-compute SHA256 of `rust-go-migration-task-ledger.md`.
2. Inspect `docs/migration/rust-go/task-status.csv`.
3. Resume execution from the first task marked `not-started` whose dependencies are in `verified` or `implemented` status.
4. Never infer task completion from filesystem presence alone; evidence must exist in the designated location.

---

## 4. Blocker Evaluation (R-04)

- **Sources Missing:** None.
- **Contract Ambiguities:** None.
- **RPC Dependencies:** None.
- **Rollback Artifacts Missing:** None (Rust API remains untouched and operational during parallel development).
- **Status:** Unblocked. Ready for Phase 0 execution.

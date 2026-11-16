# Condition Type Catalog — Phase 1: Data-Driven Catalog Foundation

**Date**: 2026-07-18
**Phase**: 1 of 2 (see Known Gap below for what Phase 2 must close)
**Status**: ✅ CRUD + Resolver primitives complete — ⚠️ NOT wired into production calculation paths
**Test Coverage**: 24 new tests (11 service unit + 8 resolver unit + 5 integration), all passing. 1585/1585 total unit tests pass.

---

## Overview

Phase 1 replaced the hardcoded working-condition types (`REGULAR`, `MATERNITY`, `SICK`, `UNPAID`, and the resolver's hardcoded `"EARLY_LEAVE"` / `"BREASTFEEDING"` string matches) with a organization catalog (`condition_type_definitions`) that lets the organization define its own condition types with an `affects_rules` JSONB describing which `PolicyRules` fields a type touches.

This built:
- Migration: `condition_type_definitions` table + `condition_type_id` FK on `working_conditions` (nullable, `ON DELETE SET NULL`) + trigger to auto-seed 5 system types for the organization baseline
- Full CRUD: repository, service (with validation + system-type delete protection), handlers, routes
- Settings UI: catalog management page, wired into Settings Hub nav
- `WorkingConditionConfig` component refactored off the hardcoded enum, now fetches types from the catalog
- OpenAPI types regenerated (`webdashboard/src/types/api-generated.ts`); `condition-types.ts` API client now uses generated types instead of hand-written ones
- Resolver primitives: `PolicyResolver::resolve_attendance_rules_pure_with_catalog` and `build_rules_from_conditions_with_catalog`, which read `affects_rules` from catalog entries instead of hardcoded string matches. Falls back to the old hardcoded match when no catalog is passed (backward compatible).

## Known Gap — Catalog Not Wired Into Calculation Paths

**The `_with_catalog` resolver variants are exercised only by unit tests. No production caller uses them.**

All real calculation call sites still call the non-catalog resolver functions, meaning custom condition types created via the Settings UI have **zero effect on computed attendance/payroll rules** today. Confirmed call sites still on the old path:

| Area | Current Go location | Required follow-up |
|---|---|---|
| Attendance and payroll policy resolution | `apiserver/internal/domain/workpolicy/` and callers under `apiserver/internal/api/` and `apiserver/internal/repository/` | Trace production callers and pass active catalog into calculation paths |
| Attendance processing | `apiserver/internal/repository/attendance.go` and `apiserver/internal/repository/attendance_logs.go` | Add end-to-end custom-condition coverage before enabling catalog-driven behavior |

Only the 5 **system-seeded** types (`EARLY_LEAVE`, `BREASTFEEDING`, `MATERNITY`, `SICK`, `UNPAID`) behave correctly today, because the hardcoded fallback match still handles those two codes exactly as before. Any **custom** type an administrator creates through the new catalog UI will save fine, display fine in the employee drawer, but **silently no-op** in daily reports, summary reports, the attendance worker, and payroll-adjacent policy resolution.

### What Phase 2 must do

1. At each of the 7 call sites above, fetch the organization's active catalog (`ConditionTypeRepository::list_condition_types(auth_token, active_only=true)`) once per batch/request and pass it into the `_with_catalog` variant instead of the current call.
2. These are payroll-adjacent hot paths (daily/summary reports, attendance worker batch processing, policy resolver service) — changes need integration tests that assert a **custom** condition type (not just the 5 system ones) actually changes a computed `PolicyRules` output end-to-end, not just at the resolver unit-test level.
3. Decide on caching strategy for the catalog fetch (per-organization, likely short-TTL cache) since these paths run in batch loops over many employees/dates — re-fetching the catalog per employee would be an N+1.

This was deliberately deferred rather than done opportunistically in Phase 1, since wiring it in touches live payroll/attendance calculation logic and deserves its own reviewed change with dedicated integration coverage, rather than being bundled into the catalog CRUD work.

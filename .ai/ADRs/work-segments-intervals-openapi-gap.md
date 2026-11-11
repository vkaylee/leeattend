# ADR: Work Segments & Shift Intervals — OpenAPI Coverage Gap

## Status
**Accepted** (Date: 2026-08-03)

## Context

`webdashboard/src/lib/api/work-segments.ts` contains 12 API functions. The task was to migrate
all of them from the legacy `fetch<T>()` / `fire()` escape hatch to the typed
`client.GET()` / `client.POST()` / `client.PUT()` / `client.DELETE()` pattern.

After auditing the Rust backend (`apiserver/src/openapi.rs`, `apiserver/src/routes/shifts.rs`,
`apiserver/src/handlers/work_segments.rs`), we found:

| # | Function | Path | Backend Status |
|---|----------|------|----------------|
| 1 | `fetchWorkSegments` | `GET /api/v1/work-segments` | ✅ Handler + OpenAPI |
| 2 | `createWorkSegment` | `POST /api/v1/work-segments` | ✅ Handler + OpenAPI |
| 3 | `getWorkSegment` | `GET /api/v1/work-segments/{id}` | ✅ Handler + OpenAPI |
| 4 | `updateWorkSegment` | `PUT /api/v1/work-segments/{id}` | ✅ Handler + OpenAPI |
| 5 | `deleteWorkSegment` | `DELETE /api/v1/work-segments/{id}` | ✅ Handler + OpenAPI |
| 6 | `bulkCreateWorkSegments` | `POST /api/v1/work-segments/bulk` | ✅ Handler + OpenAPI |
| 7 | `scheduleWorkSegments` | `GET /api/v1/work-segments/schedule` | ❌ No handler exists |
| 8 | `fetchShiftIntervals` | `GET /api/v1/shifts/{id}/intervals` | ❌ No route exists |
| 9 | `createShiftInterval` | `POST /api/v1/shifts/{id}/intervals` | ❌ No route exists |
| 10 | `updateShiftInterval` | `PUT /api/v1/shifts/{id}/intervals/{iid}` | ❌ No route exists |
| 11 | `deleteShiftInterval` | `DELETE /api/v1/shifts/{id}/intervals/{iid}` | ❌ No route exists |
| 12 | `reorderShiftIntervals` | `POST /api/v1/shifts/{id}/intervals/reorder` | ❌ No route exists |

**6 of 12 functions (items 1–6)** have complete backend support: Axum routes defined in
`apiserver/src/routes/shifts.rs`, handler functions in `apiserver/src/handlers/work_segments.rs`,
and `utoipa::path` attributes registered in `apiserver/src/openapi.rs`. These were migrated
to `client.METHOD(...)` in this change.

**6 of 12 functions (items 7–12)** cannot be migrated:

- `scheduleWorkSegments` (item 7): No Rust handler function exists anywhere in the codebase.
  The frontend calls this path, but the backend routing table has no entry for
  `/work-segments/schedule`. This appears to be either a planned feature not yet implemented
  or a dead code path.

- `fetchShiftIntervals`, `createShiftInterval`, `updateShiftInterval`, `deleteShiftInterval`,
  `reorderShiftIntervals` (items 8–12): No Axum routes exist for `/shifts/:shift_id/intervals*`.
  The `shifts.rs` route file only defines CRUD for shifts themselves plus working-pattern,
  assignment-rule, shift-swap, roster, and work-segment routes. The shift-intervals sub-resource
  has never been wired up.

  The data model (`ShiftDefaultIntervalEntity`) and request/response types exist in the Rust
  codebase, suggesting intervals were designed as part of the shift model but the REST endpoints
  were never implemented. Intervals are currently managed inline through the shift create/update
  payload's `intervals` field rather than through a separate sub-resource API.

## Decision

1. **Functions 1–6 migrated immediately** to typed `client.GET/POST/PUT/DELETE` with `as`
   casts where the hand-written types (union literals) are stricter than the generated
   OpenAPI types (bare `string`).

2. **Functions 7–12 kept on legacy `fetch<T>()` / `fire()`** with an updated comment
   pointing to this ADR.

3. **No OpenAPI schema changes needed** — the 6 migrated paths are already present in
   `api-generated.ts` via the existing `utoipa::path` attributes.

## Consequences

- **Positive**: 6 of 12 functions now enjoy compile-time path validation, parameter type
  checking, and response shape inference through `SafeClient`.
- **Positive**: The legacy comment claiming "paths are not yet registered in the generated
  OpenAPI schema" has been corrected — the work-segment paths WERE registered; the shift-interval
  and schedule paths were not.
- **Negative**: 6 functions remain on the untyped escape hatch, carrying forward the risk
  of path typos and response shape mismatches at runtime.
- **Negative**: `as` casts on response data (`as WorkSegment`) and query params (`as
  Record<string, unknown>`) weaken type safety at the boundary between OpenAPI-generated
  types and hand-written domain types.

## Migration Timeline

| Milestone | What | Owner | Target |
|-----------|------|-------|--------|
| Backend: shift-interval CRUD | Add Axum routes + handlers + utoipa attrs for `/shifts/{shift_id}/intervals*` | Backend | TBD |
| Backend: schedule work-segments | Implement or deprecate `/work-segments/schedule` endpoint | Backend | TBD |
| Frontend: regenerate types | Run `scripts/_sync-api.sh` after backend paths land | Frontend | After backend |
| Frontend: migrate remaining 6 | Replace `fetch`/`fire` with `client.METHOD()` | Frontend | After regenerate |
| Frontend: remove `as` casts | Replace hand-written `WorkSegment` return type with OpenAPI-generated type, or add runtime validation | Frontend | TBD |

## References

- `apiserver/src/routes/shifts.rs:96-109` — existing work-segment routes
- `apiserver/src/handlers/work_segments.rs` — handler implementations
- `apiserver/src/openapi.rs:158-164` — utoipa path registration
- `webdashboard/src/lib/api/work-segments.ts` — migrated frontend API module
- `webdashboard/src/lib/api/client.ts` — SafeClient wrapper and legacy escape hatch
- [[safe-client-wrapper]] — ADR explaining the typed client architecture

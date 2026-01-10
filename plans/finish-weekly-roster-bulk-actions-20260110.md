# Implementation Plan: Finish Weekly Roster with Bulk Actions

## Overview
Currently, the Shift Rostering system only supports single-day updates. This plan adds bulk actions to allow managers to assign shifts for an entire week, copy rosters between weeks, and manage schedules more efficiently.

## Phase 1: Backend (Rust API)

### 1.1 Data Models (`apiserver/src/models.rs`)
Add `BulkRosterRequest` and `RosterItem` to support multiple updates in one request.
```rust
#[derive(Debug, Deserialize)]
pub struct RosterItem {
    pub date: NaiveDate,
    pub shift_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkRosterRequest {
    pub items: Vec<RosterItem>,
}
```

### 1.2 Repository (`apiserver/src/repositories/roster_repository.rs`)
Add a transactional bulk upsert function.
```rust
pub async fn bulk_upsert_roster(
    pool: &PgPool,
    employee_id: Uuid,
    items: Vec<RosterItem>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for item in items {
        // Upsert logic here
    }
    tx.commit().await?;
    Ok(())
}
```

### 1.3 Handlers (`apiserver/src/handlers.rs`)
Add `bulk_upsert_roster` handler.
```rust
pub async fn bulk_upsert_roster(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<crate::models::BulkRosterRequest>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, StatusCode> {
    crate::repositories::roster_repository::bulk_upsert_roster(&state.db, employee_id, req.items)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}
```

### 1.4 Routes (`apiserver/src/lib.rs`)
Register the new route:
`POST /workspaces/:id/employees/:employee_id/rosters/bulk`

---

## Phase 2: Frontend (React)

### 2.1 API Client (`webdashboard/src/lib/api.ts`)
Add `bulkUpsertRoster` helper.

### 2.2 UI Enhancements (`webdashboard/src/pages/ShiftsPage.tsx`)
1. **Bulk Action Toolbar**:
   - Add a dropdown or button group: "Gán toàn tuần", "Xóa tuần này", "Sao chép tuần trước".
2. **"Apply to Week" Logic**:
   - A modal or dropdown to select a shift, which then calls `bulkUpsertRoster` for all 7 days of the current week.
3. **"Copy Previous Week" Logic**:
   - Fetch schedules for `startOfWeek - 7 days`.
   - Map them to the current week's dates.
   - Send `bulkUpsertRoster` request.
4. **Multi-select Mode**:
   - Add checkboxes to each day card.
   - Show a floating action bar when 1+ days are selected to "Assign Shift" or "Clear".

---

## Phase 3: Verification
1. Run `cargo check` to ensure backend compiles.
2. Run `npm run build` for frontend.
3. Manual testing of:
   - Assigning a shift to the whole week.
   - Copying a complex week's roster to the next week.
   - Clearing all overrides for a week.

## Rollback Plan
- Revert changes to `ShiftsPage.tsx`.
- Delete the bulk API route if it causes instability.

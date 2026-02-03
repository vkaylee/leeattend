# Scout Report: Unknown Logs System Implementation

**Date:** 2026-01-29 23:16  
**Task:** Locate files for Unknown Logs system implementation  
**Environment:** /home/elt1541/leeattend

## Summary

Located all required backend and frontend files for implementing Unknown Logs system (staging layer for unrecognized employee IDs).

## Backend Files

### Core Service Layer
- `/home/elt1541/leeattend/apiserver/src/services/attendance_log_service.rs` (259 lines)
  - Already implements staging logic (lines 126-151)
  - `process_log()` routes unknown users to staging via `staging_log_repository::create_staging_log()`
  - Uses fingerprint-based deduplication (SHA256 hash)
  - Returns `Ok(false)` for staged logs (no agent retry)

### Repository Layer
- `/home/elt1541/leeattend/apiserver/src/repositories/staging_log_repository.rs` (98 lines)
  - COMPLETE implementation with:
    - `create_staging_log()` - inserts with conflict handling
    - `get_pending_staging_logs()` - fetch by workspace
    - `update_staging_status()` - mark processed/dismissed
    - `count_pending_staging_logs()` - for UI badge counts

### Handler Layer
- `/home/elt1541/leeattend/apiserver/src/handlers/attendance.rs` (1135 lines)
  - Existing patterns for attendance endpoints (lines 366-456)
  - `get_raw_attendance_logs()` - template for staging endpoint
  - Uses `Query<AttendanceLogQuery>` for pagination

- `/home/elt1541/leeattend/apiserver/src/handlers/employees.rs` (185 lines)
  - `create_employee()` - line 89-102
  - `update_employee()` - line 104-117
  - Admin-only RBAC pattern (line 95-96)

### Route Registration
- `/home/elt1541/leeattend/apiserver/src/lib.rs` (200+ lines read)
  - Routes registered in `create_app()` starting line 144
  - Pattern: `.route("/workspaces/:ws_id/attendance/...", get(handler))`
  - Need to add staging endpoints here

### Database Migration
- `/home/elt1541/leeattend/apiserver/migrations/20260129220000_add_staging_and_fingerprint.sql`
  - Migration exists for `attendance_staging_logs` table
  - Includes workspace_id, raw_user_id, fingerprint, status columns

## Frontend Files

### Main Page
- `/home/elt1541/leeattend/webdashboard/src/pages/AttendanceLogsPage.tsx` (244 lines)
  - Current implementation: single view with filters
  - Uses tab pattern would be ideal (see Tabs.tsx)
  - Line 100-108: Query pattern for fetching logs
  - Line 173-238: Table rendering structure
  - Line 127-137: Header with import button

### Tab Component
- `/home/elt1541/leeattend/webdashboard/src/components/ui/Tabs.tsx` (54 lines)
  - Complete implementation:
    - `<Tabs>` - context provider
    - `<TabsList>` - container
    - `<TabsTrigger>` - tab buttons
    - `<TabsContent>` - conditional content
  - Styling: primary-500 border for active tab

### API Client
- `/home/elt1541/leeattend/webdashboard/src/lib/api/attendance.ts` (366 lines)
  - `fetchRawAttendanceLogs()` - line 146-168 (template for staging)
  - Pattern: SearchParams + apiFetch + pagination response
  - Returns: `{ data, total, page, limit }`

### Employee API
- `/home/elt1541/leeattend/webdashboard/src/lib/api/employees.ts` (140 lines)
  - `createEmployee()` - line 91-101
  - `updateEmployee()` - line 103-114
  - Both use workspace-scoped endpoints

### Employee Management Page
- `/home/elt1541/leeattend/webdashboard/src/pages/EmployeesPage.tsx` (100+ lines read)
  - Employee creation modal pattern (line 60-62)
  - Search/filter UI patterns (line 64, 74)
  - Mutation hooks for create/update (line 90-98)

## Key Patterns Identified

### Backend Routing Pattern
```rust
.route("/workspaces/:ws_id/attendance/staging-logs", 
    get(handlers::get_staging_logs))
.route("/workspaces/:ws_id/attendance/staging-logs/:id/resolve", 
    post(handlers::resolve_staging_log))
```

### Frontend Tab Pattern
```tsx
<Tabs value={activeTab} onValueChange={setActiveTab}>
  <TabsList>
    <TabsTrigger value="recorded">Recorded</TabsTrigger>
    <TabsTrigger value="unknown">Unknown</TabsTrigger>
  </TabsList>
  <TabsContent value="recorded">...</TabsContent>
  <TabsContent value="unknown">...</TabsContent>
</Tabs>
```

### Pagination Query Pattern
```typescript
const { data } = useQuery({
  queryKey: ["staging-logs", wsId, page],
  queryFn: () => fetchStagingLogs({ page, limit }),
  enabled: !!wsId,
});
```

## Implementation Checklist

### Backend (Rust)
- [ ] Add handler functions in `handlers/attendance.rs`:
  - `get_staging_logs()` - paginated list
  - `resolve_staging_log()` - map to employee + reprocess
  - `dismiss_staging_log()` - mark as dismissed
- [ ] Register routes in `lib.rs` 
- [ ] Add `StagingLog` model to `models/mod.rs` (if not exists)
- [ ] Update `repositories/mod.rs` to export staging repo (likely done)

### Frontend (TypeScript/React)
- [ ] Add API functions in `lib/api/attendance.ts`:
  - `fetchStagingLogs()`
  - `resolveStagingLog()`
  - `dismissStagingLog()`
- [ ] Refactor `AttendanceLogsPage.tsx`:
  - Import and use `<Tabs>` component
  - Split into two tab contents
  - Add "Unknown" tab with staging table
  - Add resolve/dismiss actions
- [ ] Create resolution modal/dialog for employee mapping
- [ ] Add badge counter for pending staging logs

## Unresolved Questions

1. Should resolved staging logs trigger automatic pairing/reprocessing?
2. UI for "create new employee" from staging log (quick-create vs full form)?
3. Batch resolution for multiple staging logs from same raw_user_id?
4. Retention policy for dismissed staging logs?
5. Notification/alert system for new unknown logs?

## File Count Summary

**Backend:** 5 core files (service, repository, handler, routes, migration)  
**Frontend:** 5 core files (page, tabs component, 2x API clients, employee page)  

**Total:** 10 files identified for Unknown Logs system implementation.

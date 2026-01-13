# Implementation Plan: Attendance Logs Page

Implement a detailed transaction list for attendance records with advanced filtering and pagination.

## Approach
- **Transaction List**: Every punch in/out is a record.
- **Server-side Pagination**: Essential for handling thousands of records.
- **Filtering**: By Date range and Employee.

## Steps

### 1. Backend: Repository Enhancement
Modify `apiserver/src/repositories/time_entry_repository.rs`:
- Add `get_paginated_logs(pool, workspace_id, filters, limit, offset)`
- Add `count_logs(pool, workspace_id, filters)`

### 2. Backend: API Handler
Modify `apiserver/src/handlers.rs`:
- Create `get_attendance_logs` handler.
- Map query parameters to repository filters.
- Return `{ data: Vec<TimeEntry>, total: i32, page: i32, limit: i32 }`.

### 3. Frontend: API Client
Modify `webdashboard/src/lib/api.ts`:
- Add `getAttendanceLogs` function.

### 4. Frontend: UI Component
Create `webdashboard/src/pages/AttendanceLogsPage.tsx`:
- Use `shadcn/ui` Table.
- Implement `useQuery` with polling or manual refresh.
- Add Date Range Picker and Employee search.

## Testing Plan
- Test pagination with 50+ records.
- Test date filtering (UTC vs Local issues).
- Test cross-workspace security (ensure no data leak).

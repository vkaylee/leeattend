# Implementation Plan: Add Device Names to Attendance Logs

## Approach
- **Frontend Join (Approach B):** Fetch the list of devices via `useDevices` and map `device_id` to `device_name` in-memory.
- **Why:** Avoids backend refactoring, leverages existing caching, and keeps the raw log API performant.

## Steps
1. **Data Integration**: Hook `useDevices` into `AttendanceLogsPage.tsx`.
2. **Mapping Logic**: Create a dictionary of device names keyed by ID.
3. **UI Polish**: Update the table to show the Name and ID together.

## Timeline
- **Implementation**: 20 min
- **Verification**: 10 min
- **Total**: 30 min

## Rollback
- `git checkout src/pages/AttendanceLogsPage.tsx`

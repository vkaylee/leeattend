# Implementation Plan: Add time_format column to workspaces table and update UI

Add a `time_format` setting to the `workspaces` table to allow users to choose between 24-hour (default) and 12-hour (AM/PM) time display.

## Approach
- **Database:** Add a `time_format` column to the `workspaces` table.
- **Backend:** Update the `Workspace` model, repository, and service to handle the new field.
- **Frontend:** Update the workspace settings page to allow users to change the format, and update time display components to respect the selected format.

## Steps

### 1. Database Migration (5 min)
- Create migration file `apiserver/migrations/20260113000001_add_time_format_to_workspaces.sql`.
- Content:
  ```sql
  ALTER TABLE workspaces ADD COLUMN time_format VARCHAR(10) NOT NULL DEFAULT '24h';
  ```

### 2. Backend Implementation (15 min)
- **apiserver/src/models.rs**: Add `time_format: String` to `Workspace` struct.
- **apiserver/src/handlers.rs**: Add `time_format: Option<String>` to `UpdateWorkspaceRequest`.
- **apiserver/src/repositories/workspace_repository.rs**: Update `create_workspace` and `update_workspace` SQL queries to include `time_format`.
- **apiserver/src/services/workspace_service.rs**: Update the dynamic `update_workspace` logic to handle the `time_format` field.

### 3. Frontend Implementation (25 min)
- **webdashboard/src/lib/api.ts**: Add `time_format: string` to `Workspace` interface.
- **webdashboard/src/pages/WorkspaceSettingsPage.tsx**:
  - Add `timeFormat` state.
  - Add a select dropdown for "Định dạng giờ ưu tiên" (24h vs 12h).
  - Update `handleSaveConfig` to send `time_format`.
- **Time Display Integration**:
  - Update `AttendanceLogsPage.tsx` and other relevant pages to fetch the workspace config and use it for formatting.
  - Define constants for formats:
    - `24h` -> `HH:mm:ss` (or `HH:mm`)
    - `12h` -> `hh:mm:ss a` (or `hh:mm a`)

### 4. Verification (15 min)
- Run migrations: `sqlx migrate run` (if applicable) or manual SQL execution.
- Test updating the setting in the UI.
- Verify time display changes in `AttendanceLogsPage.tsx`.

## Timeline
| Phase | Duration |
|-------|----------|
| Database | 5 min |
| Backend | 15 min |
| Frontend | 25 min |
| Testing | 15 min |
| **Total** | **1 hour** |

## Rollback Plan
- Run migration down (if created): `ALTER TABLE workspaces DROP COLUMN time_format;`.
- Revert code changes in backend and frontend.

## Security Checklist
- [x] Input validation for `time_format` (should only be "24h" or "12h").
- [x] Auth checks (only admins/owners can update workspace settings).

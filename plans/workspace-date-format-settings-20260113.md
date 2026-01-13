# Implementation Plan: Workspace Date Format Settings

## Approach
We will implement workspace-level date format configuration to resolve ambiguity during Excel/CSV imports (e.g., `01/02/2025` being Jan 2nd or Feb 1st). 
- **Solution:** Add a `date_format` column to the `workspaces` table.
- **Alternative considered:** Detecting based on user locale, but workspace-level is more consistent for shared business data.

## Steps

### 1. Database Migration (5 min)
Create a new migration file in `apiserver/migrations/`:
- `ALTER TABLE workspaces ADD COLUMN date_format VARCHAR(20) NOT NULL DEFAULT 'DD/MM/YYYY';`

### 2. Backend Implementation (20 min)
- **Models (`apiserver/src/models.rs`):**
  - Add `date_format: String` to `Workspace` struct.
  - Add `date_format: Option<String>` to `UpdateWorkspaceRequest`.
- **Repository (`apiserver/src/repositories/workspace_repository.rs`):**
  - Update SQL queries in `find_by_id`, `create`, and `update` methods.
- **Service (`apiserver/src/services/attendance_adjustment_service.rs`):**
  - In `process_job`, fetch the workspace to get its `date_format`.
  - Update `parse_datetime` to prioritize formats matching the workspace setting.

### 3. API & Frontend (25 min)
- **Handlers (`apiserver/src/handlers.rs`):**
  - Ensure `update_workspace` properly maps the new field.
- **API Client (`webdashboard/src/lib/api.ts`):**
  - Update `Workspace` interface.
- **UI (`webdashboard/src/pages/WorkspaceSettingsPage.tsx`):**
  - Add a "Date Format" dropdown in the settings form.
  - Options: `DD/MM/YYYY` (Standard), `MM/DD/YYYY` (US), `YYYY-MM-DD` (ISO).

### 4. Testing (15 min)
- Create an integration test where:
  1. Workspace is set to `MM/DD/YYYY`.
  2. Upload a file with `11/12/2025` (Nov 12th).
  3. Verify the resulting `time_entry` month is 11.

## Timeline
| Phase | Duration |
|-------|----------|
| Database & Models | 10 min |
| Service Logic | 20 min |
| UI & API | 20 min |
| Testing | 15 min |
| **Total** | **1 hour 5 min** |

## Rollback Plan
1. Revert commits.
2. Run SQL: `ALTER TABLE workspaces DROP COLUMN date_format;`

## Security Checklist
- [x] Input validation: Only allow specific format strings (`DD/MM/YYYY`, `MM/DD/YYYY`, `YYYY-MM-DD`).
- [x] RBAC: Verify `ctx.is_admin_or_owner()` in the update handler.

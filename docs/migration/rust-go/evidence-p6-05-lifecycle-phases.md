# P6-05 Employment Phases and Lifecycle Evidence

Status: implemented; verification deferred to parent gate run.

## Implemented routes

- Employment phase definitions: `POST`/`GET /api/v1/workspaces/{ws_id}/employment-phases`.
- Employee phase assignment/list/status routes under `/employees/{user_id}/phases`.
- Employment phase assignment/list/status routes under `/employments/{employment_id}/phases`.
- Lifecycle template CRUD under `/lifecycle-templates`.
- Lifecycle template item CRUD under `/lifecycle-templates/{id}/items`.
- Active checklist create/list/get and item status update routes.
- Current-user pending tasks at `/lifecycle/my-tasks`.

## Contract behavior

- Tenant database and `workspace_id` filters cover definitions, assignments, templates, template items, checklists, checklist items, and pending tasks.
- Phase assignment transaction closes active prior phases starting on or before new phase start and sets end date to `new_start - 1 day` without extending earlier end dates.
- Phase status updates preserve Rust text status semantics and always set evaluator/completion metadata.
- Template defaults are replaced atomically per workspace and checklist type. Template deletion is soft delete; lists filter active, non-deleted templates and sort by name.
- Template item update uses nested pointer state for explicit `assignee_user_id: null` clearing.
- Checklist creation copies template items transactionally, computes base/due dates, resolves assignees, and immediately completes empty templates.
- Item status updates recompute rounded progress and checklist completion state; authorization accepts item assignee or `employees:update`.
- My tasks returns only `pending` items for current user, sorted `due_date ASC NULLS LAST`.

## Files

- `apiserver/internal/api/employment_phases.go`
- `apiserver/internal/api/lifecycle.go`
- `apiserver/internal/repository/employment_phases.go`
- `apiserver/internal/repository/lifecycle.go`
- `apiserver/internal/repository/workspace_authz.go`
- `apiserver/main.go`

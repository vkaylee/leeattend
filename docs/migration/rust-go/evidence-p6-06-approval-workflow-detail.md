# P6-06 Approval Workflow Detail Evidence

Status: implemented; full approval engine differential parity remains pending.

## Implemented

- `GET /api/v1/workspaces/{ws_id}/approval-workflows/{id}`
  - Requires `approval_workflows:read`.
  - Returns the workflow with steps ordered by `step_order`.
  - Missing workflow returns `404`.
- `PUT /api/v1/workspaces/{ws_id}/approval-workflows/{id}`
  - Requires `approval_workflows:manage`.
  - Partially updates `name`, `description`, and `is_active`.
  - Preserves existing steps; returns the workflow with embedded steps.
  - Missing workflow returns `404`.
- `DELETE /api/v1/workspaces/{ws_id}/approval-workflows/{id}`
  - Requires `approval_workflows:manage`.
  - Deletes by `id` plus `workspace_id`; returns `200` with empty body.
  - Missing workflow returns `404`.

Every query constrains `workspace_id`.

## Files

- `apiserver/internal/api/approval_workflow.go`
- `apiserver/internal/api/approval_workflow_test.go`
- `apiserver/internal/repository/approval_workflow.go`
- `apiserver/main.go`

## Verification

- `./scripts/_go-gates.sh`: ALL GATES PASSED.
- `./leedevkit test api-go`: ALL GATES PASSED.
- Endpoint metrics: contract routes **403**, Go registrations **152**, covered **129**, missing **274**.

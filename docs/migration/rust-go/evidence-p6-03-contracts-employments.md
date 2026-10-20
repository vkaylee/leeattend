# P6-03 Contracts and Employments

## Scope

Batch 2 Task 2.2 migrates 11 Rust contract/employment routes to Go:

- workspace contract list
- employee contract list/create/get/update/delete
- employee employment list/create/get/update/delete

Routes are registered in `apiserver/main.go` under `/api/v1/workspaces/{ws_id}`.

## DTO parity

`apiserver/internal/api/contracts_employments.go` defines `EmployeeContract`, `EmployeeContractWithEmployee`, `Employment`, create requests, and patch requests. Contract enums validate the Rust values (`probation`, `training`, `fixed_term`, `indefinite`; `draft`, `active`, `expired`, `terminated`). Employment status and type remain free strings.

Employment nullable patch fields use `**uuid.UUID` / `**string` plus presence bits. Omitted values remain unchanged; explicit JSON `null` clears the database column. Contract nullable patch fields use the same presence behavior for `end_date`, `document_id`, and `notes`.

## Tenant isolation and mutation behavior

`apiserver/internal/repository/contracts_employments.go` resolves every query through workspace tenant DB and includes `workspace_id` and `deleted_at IS NULL` filters on reads. Contract create checks active date overlap and performs auto-close plus insert in one tenant transaction when requested. Contract and employment delete operations set `deleted_at` rather than hard deleting.

Employment create/update handlers retain Rust overlap semantics: same employee and employer cannot have overlapping ranges; different employers may overlap; open-ended ranges overlap all later ranges.

Employment responses load custom-field values through existing tenant custom-field repository. Workspace contract list joins non-deleted contracts, employments, and employees and returns flattened employee identity fields without pagination.

All handlers require authenticated workspace membership through `WorkspaceMemberAuthorizer.IsMember`; no resource permission is added because the Rust endpoint matrix evidences no explicit permission guard.

## Tests

Focused handler tests cover:

- workspace contract list and membership rejection
- contract create active status
- overlap conflict and auto-close path
- contract get, patch, and 204 soft delete
- employment create/list-related state, same-employer overlap conflict, different-employer overlap success
- employment patch explicit-null clearing and 204 soft delete

Build, formatter, and test gates are intentionally deferred to parent task owner per migration workflow.

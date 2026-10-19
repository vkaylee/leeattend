# P6-04 assignments evidence

Rust contract sources:

- `apiserver/src/handlers/employee_assignments.rs`
- `apiserver/src/repositories/diesel_employee_repository/assignments.rs`
- `apiserver/src/handlers/assignment_rules.rs`
- `apiserver/src/repositories/diesel_assignment_rule_repository.rs`
- `apiserver/src/handlers/shifts.rs` (`get_assignment_history`)
- `apiserver/src/repositories/diesel_roster_repository/roster.rs` (`get_assignment_history`)

Go implementation:

- DTOs and handlers: `apiserver/internal/api/assignments.go`
- Employee assignment and rule repositories: `apiserver/internal/repository/assignments.go`
- Assignment history repository: `apiserver/internal/repository/assignment_history.go`
- Route registration: `apiserver/main.go`
- Focused contract tests: `apiserver/internal/api/assignments_test.go`

Contract parity:

- Employee assignment POST returns `200`; DELETE returns `200` with JSON `null`.
- Assignment POST requires employee active employment on workspace-local current date, primary location, and distinct home/host locations. No overlap validation added.
- Assignment PATCH supports omitted-vs-explicit-null for `end_date` and `reason`; status accepts only `active`, `completed`, `cancelled`.
- All employee assignment queries scope both workspace and employee/employment.
- Assignment rules require `assignment_rules:manage`, list priority descending then `created_at` ascending, create returns `201` with `is_active=true`, and delete returns `204` regardless of row existence.
- Assignment history reads only `employee_shifts`, `employee_working_patterns`, and `shift_rosters`; secondment and rule rows excluded. Workspace scope joins through `employments`; pagination defaults to page 1 / limit 50 and clamps limit to 3000.

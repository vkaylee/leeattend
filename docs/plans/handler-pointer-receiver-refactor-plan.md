# Handler Pointer Receiver Refactor

Status: Awaiting approval

## Goal
Standardize HTTP API handlers on pointer receivers and pointer-owned route wiring without changing endpoint behavior, authorization, response contracts, or shared dependency ownership.

## Evidence
- Most `apiserver/internal/api/*Handler` methods use value receivers.
- `buildRouter` constructs handlers as values and route registration functions accept handler values.
- Handler fields are mostly interfaces, strings, and pointers; current value copies are shallow and usually work, but method-set and future mutable-state behavior are inconsistent.
- `health.Handler` already uses pointer receivers, establishing compatible project precedent.

## Constraints
- No API route, JSON schema, status code, auth behavior, or database query changes.
- Preserve handler construction order and shared repository/client instances.
- Do not add interfaces or factories solely for this refactor.
- Do not convert repository receivers in same change.
- Preserve unrelated working-tree changes.

## Tasks
- [ ] Inventory every API handler type, receiver, route registration function, interface assertion, method expression, and test construction. Classify handlers as stateless, pointer-heavy, or stateful. Verify: inventory covers all `*Handler` types under `apiserver/internal/api`.
- [ ] Convert one low-risk handler group first (`AttendanceHandler`, `ConditionTypeHandler`, `DepartmentHandler`, `LocationHandler`) to pointer receivers. Change route registration parameters and router construction to pointers; update empty test handlers to `&api.Handler{}`. Verify: focused API tests and `go vet` pass.
- [ ] Convert remaining stateless handlers in bounded groups, keeping receiver declarations and route parameters aligned. Verify: each group passes focused package tests and route registration tests.
- [ ] Convert pointer-heavy/stateful handlers (`MonitorHandler`, `FileHandler`, `EmployeeImportHandler`, `ReportHandler`, `PayrollHandler`, `ScheduleHandler`, `ESSHandler`) last. Inspect mutex, cache, stream, Redis, storage, and lifecycle fields before conversion. Verify: no copied synchronization/state fields and race-enabled handler tests pass.
- [ ] Add one compile-time method-set check or route wiring test proving pointer handler instances satisfy all required registration contracts. Verify: value handler cannot silently re-enter route wiring through accidental copies.
- [ ] Run formatting, vet, static analysis, focused API tests, then full `leedevkit test api`; inspect phase logs. Verify: no behavior or contract regression.

## Done When
- [ ] All HTTP API handler methods use pointer receivers unless an explicit documented exception exists.
- [ ] Route registration accepts handler pointers consistently.
- [ ] Handler instances are constructed once and passed by pointer from router wiring.
- [ ] No API contract, authorization, persistence, or observability behavior changes.
- [ ] Full API verification passes, or failures are documented with exact phase evidence.

## Review Focus
- Method-set changes can break interfaces or method expressions outside route files.
- Pointer conversion must not introduce shared mutable state or data races.
- Test-only zero-value handlers must remain valid where route registration tests intentionally exercise authorization failures.
- Large handler structs may reveal accidental state copying currently hidden by value receivers.

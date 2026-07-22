# Attendance No-show Improvement Compliance

## Request classification

- Type: COMPLEX CODE / Bug Fix
- Domain: Attendance reporting, Rust/Axum backend, React/TypeScript frontend

## Applicable rules

- CS-03: Use structured `AppError` handling and strict input validation.
- CS-07: Prevent N+1 queries and keep collection APIs correctly paginated.
- DB-02: Batch-fetch tenant data; never execute database queries inside loops.
- AC-01: Enforce authorization before attendance report business logic.
- DG-01: Attendance and employee identity data are workspace-scoped T3 data.
- TS-02: Add regression coverage for corrected business logic and pagination.
- UX-04: Clearly distinguish loading, pending-grace, and action-required states.

## Intended outcome

The no-show workflow must provide a complete, current, and operationally clear
queue of employees whose scheduled shift has started but whose attendance data
requires HR attention. Pending grace-period rows remain informational, while
overdue rows form the default action queue.

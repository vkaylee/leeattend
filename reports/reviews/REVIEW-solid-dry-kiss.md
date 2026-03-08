# Code Review: SOLID / DRY / KISS Compliance — `apiserver`

## Summary

| Metric | Value |
|--------|-------|
| Files reviewed | ~30 core files across all layers |
| Layers analyzed | Repository → Service → Handler → Domain → State |
| Overall verdict | **MOSTLY COMPLIANT** — architecture is well-structured with isolated issues |

---

## ✅ SOLID Compliance

### S — Single Responsibility ✅ (with minor concerns)

**Positives:**
- Clean layer separation: `repositories/` (data access) → `services/` (business logic) → `handlers/` (HTTP transport) → `routes/` (route definition)
- `state.rs` groups 44+ fields into 9 domain sub-structs (`EmployeeState`, `AttendanceState`, `ShiftState`, etc.) — each with a clear bounded context
- `errors.rs` is a single focused `AppError` enum with clean `IntoResponse` conversion
- `common.rs` provides dedicated Axum extractors (`MemberContext`, `WorkspaceId`, `EmployeeContext`)
- `permissions.rs` is a standalone RBAC module with its own tests

**⚠️ Concerns:**
- `EmployeeService` (490 lines) also handles **Recruitment Sources** and **Department Transfers** — these are separate concerns that should ideally have their own services
- `EmployeeRepository` (1411 lines, 77 outline items) is a god repository — consider splitting `RecruitmentSourceRepository` and `DepartmentHistoryRepository`
- `main.rs` (481 lines) is the DI composition root AND server startup AND port negotiation AND graceful shutdown — could extract a `startup.rs` or a builder pattern

### O — Open/Closed ✅

- Repository traits (`EmployeeRepository`, `RosterRepository`, etc.) allow new implementations without modifying existing code
- `PoolBacked` supertrait enforces a consistent construction contract for all future repos
- The `Permission` enum is easily extensible with new variants
- `AppError` uses `#[from]` for automatic conversion from new error types

### L — Liskov Substitution ✅

- All repository traits use `async_trait` with `dyn` dispatch — any concrete implementation can substitute the trait without breaking callers
- `SqlxEmployeeRepository` and other `Sqlx*` implementations conform exactly to their trait contracts
- No trait method overrides that would violate behavioral contracts

### I — Interface Segregation ✅ (minor concern)

- Repository traits define focused sets of methods per domain
- Service constructors only accept the specific trait objects they need (e.g., `LeaveService::new(leave_balance_repo, leave_request_repo, employee_repo, roster_repo, pool)`)

**⚠️ Concern:** `EmployeeRepository` trait has ~30+ methods spanning employees, working conditions, recruitment sources, department history, and payroll config — violates ISP. Should be split into smaller focused traits.

### D — Dependency Inversion ✅✅ (strongest point)

- Services depend on trait objects (`Arc<dyn EmployeeRepository>`) not concrete `SqlxEmployeeRepository`
- All DI wiring is centralized in `main.rs` composition root
- `PoolBacked` trait enforces construction discipline
- `MemberContext` extracts auth info through Axum's `FromRequestParts`, decoupling auth from handler logic

---

## 🔁 DRY Compliance — ⚠️ Issues Found

### 🔴 Critical: Duplicated Department Lookup Query

The same inline SQL appears in **6+ handler functions** across `employees.rs` and `attendance/overtime.rs`:

```rust
let target_dept_id: Option<Uuid> = sqlx::query_scalar(
    "SELECT department_id FROM employee_department_history WHERE employee_id = $1 ORDER BY effective_date DESC LIMIT 1"
).bind(id).fetch_optional(state.infra.db.api()).await.unwrap_or(None);
```

**Impact:** If the query logic changes (e.g., add `workspace_id` filter, change join), you must update every occurrence. Also leaks SQL into the handler layer, breaking the repository abstraction.

**Fix:** Extract to a repository method like `employee_repo.get_current_department_id(employee_id)` or a helper on `MemberContext`.

### 🟠 Major: `pool.clone()` repeated 37 times in `main.rs`

The DI wiring in `main.rs` calls `pool.clone()` 37 times and `worker_pool.clone()` 4 times. While functionally harmless (Arc clones are cheap), it makes the composition root verbose and harder to maintain.

**Fix:** Consider a builder/factory pattern or a `ServiceFactory` struct that holds the pool references and creates repos/services.

### 🟡 Minor: Inconsistent Repo Construction

Some repos use `::new(pool)` while others use struct literal `{ pool: pool.clone() }`. Notably:
- `SqlxAgentRepository { pool: pool.clone() }` — struct literal
- `SqlxEmployeeRepository::new(pool.clone())` — constructor

All should use `::new()` or `PoolBacked::from_pool()` consistently.

---

## 💋 KISS Compliance — ✅ Good

### Positives:
- Handlers are **thin controllers** — typically 5–15 lines, delegating immediately to services
- `AppResult<T>` type alias eliminates boilerplate error handling
- `determine_daily_status()` in `attendance_utils.rs` is documented as the **"single source of truth"** and uses a clean priority chain
- Routes are cleanly separated into domain modules (`routes::employees`, `routes::leaves`, etc.)
- `AttendanceStatus` enum uses `Display` + `from_str_legacy()` for clean serialization

### ⚠️ Minor Complexity:
- Redis initialization in `main.rs` (lines 79–124) has 5 levels of nesting (env check → timeout → result → health check → timeout). Could flatten with early returns or an `init_redis()` helper.
- `handlers/mod.rs` uses glob re-exports (`pub use agents::*`) which can pollute the namespace

---

## 📊 Scorecard

| Principle | Score | Notes |
|-----------|-------|-------|
| **S** — Single Responsibility | 8/10 | Clean overall; `EmployeeService/Repository` too broad |
| **O** — Open/Closed | 9/10 | Trait-based design, easy extensibility |
| **L** — Liskov Substitution | 10/10 | Perfect trait implementations |
| **I** — Interface Segregation | 7/10 | `EmployeeRepository` trait is a fat interface |
| **D** — Dependency Inversion | 10/10 | Exemplary use of `Arc<dyn Trait>` + composition root |
| **DRY** | 6/10 | Department lookup duplication is significant |
| **KISS** | 8/10 | Clean overall; Redis init and `main.rs` DI are complex |
| **Overall** | **8.3/10** | Well-architected with actionable improvements |

---

## 🎯 Recommended Actions (Priority Order)

1. **Extract `get_current_department_id()`** into repository → eliminate 6+ duplicated SQL queries
2. **Split `EmployeeRepository`** trait into `EmployeeRepository` + `RecruitmentSourceRepository` + `DepartmentHistoryRepository`
3. **Unify repo construction** — standardize on `::new()` or `from_pool()`
4. **Extract `init_redis()`** helper from `main.rs`
5. **Consider `ServiceFactory`** pattern to simplify DI composition in `main.rs`

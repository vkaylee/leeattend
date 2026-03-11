# Architectural Review: `apiserver`

## 1. Architectural Context & Current State
The `apiserver` crate serves as the core backend component for the LeeAttend project. It exposes an HTTP API using `axum`/`tokio` and interfaces with a PostgreSQL database via `sqlx`. The system relies on Redis for queuing and caching, and integrates heavily with background workers (e.g., `attendance_log_service.rs`, `job_worker`). 

The current state incorporates **Layered/Clean Architecture** principles:
- **Presentation / Transport Layer:** `handlers/`, `routes/`, `mw/` (Middleware).
- **Domain / Service Layer:** `services/` containing the core business logic (e.g., `attendance_summary_service.rs`, `payroll_service.rs`).
- **Data Access Layer:** `repositories/` abstracting the DB logic behind Rust Traits (e.g., `dyn EmployeeRepository`).
- **Inversion of Control (IoC):** Dependency Injection is managed effectively within `src/state.rs` and `src/app_builder.rs` by utilizing `Arc` and strongly typing the application state payload.

## 2. Assessment of Pattern Compliance
**Strengths:**
- **Dependency Inversion Principle:** Achieved perfectly. `app_builder.rs` constructs concrete Repository implementations (`SqlxEmployeeRepository`) and injects them into Services as trait objects (`Arc<dyn EmployeeRepository>`). This ensures that unit tests can mock the repositories seamlessly.
- **Single Responsibility Principle (Partial):** Many concepts are separated into their own services (`ShiftSwapService`, `AttendanceAdjustmentService`), enhancing module cohesion.
- **Robust Background Processing:** High-concurrency operations like attendance syncing are separated intelligently into worker pools ensuring the main API thread does not get starved.
- **Configurable Environments:** Separation of API and Worker db pools (`db_pools.api()` vs `db_pools.worker()`) guarantees API availability during heavy background jobs.

## 3. Violations & Anti-Patterns Identified

### 3.1. God-Files and Large Repositories
Despite the excellent modularity at the structural level, some individual files have ballooned in size, becoming **"God-files"** that handle too many responsibilities:
- `employee_repository.rs` (~45 KB)
- `roster_repository.rs` (~41 KB)
- `leave_accrual_service.rs` (~35 KB)
- `excel_writer.rs` (~41 KB)

*Impact:* High cognitive load for developers, frequent merge conflicts, and violations of the Single Responsibility Principle.

### 3.2. Centralized State Bottleneck
The `AppState` struct inside `src/state.rs` aggregates literally every single service and repository. While convenient, the construct function `AppState::build` in `app_builder.rs` is massive (~300 lines). Every request handler inherently holds an `Arc` pointer to the entire universe of services, even if the handler only needs one domain context.

## 4. Scalability & Resilience Implications
- **Connection Issues & Throttling:** The current approach of explicitly passing `pool` along with the `repository` to services limits some architectural flexibility (repositories should fully encapsulate the DB pool). However, this choice seems deliberate for transaction handling across multiple repositories.
- **Caching Mechanism:** Caching via `moka` and `redis` is present, acting as an excellent resilience layer.

## 5. Architectural Recommendations & Next Steps

1. **Decompose God-Files (High Priority):**
   - Split `employee_repository.rs` into logical sub-domains or traits (e.g., `EmployeeCoreDataRepo`, `EmployeeOnboardingRepo`, `EmployeeStatusRepo`).
   - Break down `leave_accrual_service.rs` into smaller calculation strategy modules (Strategy Pattern).
   
2. **Refactor AppState into Module-Specific States (Medium Priority):**
   - Use Axum's `FromRef` extractors to pull out *only* `EmployeeState` from `AppState` within the handlers rather than passing the global `Arc<AppState>`. This cleans up handler signatures and scopes dependencies.

3. **Database Transaction Management (Medium Priority):**
   - Avoid passing the raw `DbPool` into `Service` instances directly. Instead, implement a **Unit of Work** pattern. This enables services to coordinate multi-repository modifications inside a unified transaction boundary without leaking the SQLx pool implementation to the service layer.

## Conclusion
The `apiserver` boasts a highly mature, production-ready Rust architecture, demonstrating a solid grasp of Clean Architecture and safe concurrent programming. To maintain velocity as the system grows, the team must address the bloated repository files immediately and consider standardizing the Unit of Work pattern for transaction safety.

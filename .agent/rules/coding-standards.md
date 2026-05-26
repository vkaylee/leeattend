# 💻 Coding Standards & Best Practices

## 1. Language Handling
- **Communication:** Respond in the language the user initiated the prompt with.
- **Code:** ALL variables, function names, and code comments MUST be in English.

## 2. Clean Code Standards
- **SOLID & DRY:** Enforce Single Responsibility. Extract reusable logic.
- **Self-Documenting:** Write clear names. Minimal comments. No over-engineering.
- **Strict Encapsulation (OOP Style):** ALL struct fields MUST be `private` by default to ensure strict data protection and integrity.
  - Do NOT use `pub` for struct fields.
  - State mutation and data access MUST be controlled via explicitly defined `new()` constructors and getter methods (e.g., `inner()`, `into_inner()`, `name()`).

## 3. Error Handling & Observability
- **Structured Errors:** Use domain-specific error types (e.g., `AppError`). Never throw generic `Error`.
- **Error Boundaries:** Wrap all independent UI components in React Error Boundaries.
- **Telemetry Baseline:** All backend APIs MUST include request tracing/metrics (e.g., OpenTelemetry span/context).

## 4. Security & Compliance
- **Data Privacy:** NEVER log Personally Identifiable Information (PII) like emails, passwords, phone numbers in raw text. Mask them (e.g., `user_***@domain.com`).
- **Secrets:** NEVER hardcode secrets. Use environment variables.

## 5. Boy Scout Rule
If you see a bug, typo, or poor pattern nearby while working on a file, FIX IT IMMEDIATELY.
> 🔴 **NEVER say "this is out of scope for the current task" when a clear bug is visible.**

## 6. Enterprise-Grade Premium Standard (MANDATORY)
> 🔴 **MANDATORY:** Every single feature, component, and API developed in this workspace MUST default to an **Enterprise-Grade Premium Ready** standard. Do NOT deliver MVP-style or basic implementations.
- **Frontend:** Premium design patterns, i18n support, skeleton loaders, empty states, micro-interactions.
- **Backend:** Strict validation (Type-safety, UUIDs), explicit error handling, transaction boundaries, audit logging.

## 7. Performance & Resource Optimization
- **Performance Budget:** JS bundle `< 200KB` gzipped. Core Web Vitals: LCP < 2.5s. Zero memory leaks for SPAs.
- **Scalability:** ZERO N+1 Queries allowed. ALL public APIs MUST implement Rate Limiting.
- **Pure Domain Logic (N+1 Prevention):** To strictly prevent N+1 queries at the architectural level, do NOT pass Database Connections (`PgConnection`, `TenantAuthToken`) or Repositories into pure Business/Domain logic functions (e.g., calculation engines). Force data to be batch-fetched beforehand and passed in as memory structures.

## 8. Escalation Protocol
- **The Rule:** If an error (e.g., Infra, DevOps) persists after **2 failed self-correction attempts**, STOP IMMEDIATELY. Generate an Incident Report and ask for human intervention. Do not endlessly retry.

## 9. Code-Level Execution Rules (Enterprise Pro Max)
- **Dependency Injection & Decoupling:** NEVER tightly couple the Service layer to Infrastructure. Use Traits/Interfaces for external dependencies (DB, Redis, external APIs) to ensure 100% unit-testability without spinning up infrastructure.
- **Concurrency Safety (Rust):** NEVER hold an OS-level lock (`MutexGuard`, `RwLockReadGuard`) across an `await` point to avoid deadlocks. NEVER use blocking I/O calls (e.g., `std::fs`, `std::thread::sleep`) inside an async runtime; use `tokio` equivalents.
- **API Contract Design:** ALL APIs returning a collection/list MUST implement Pagination. Never assume small datasets. Do NOT introduce breaking schema changes without bumping the API version.
- **Frontend Architecture:** Strictly separate Business Logic (Custom Hooks/Services) from UI Components. UI Components must remain "dumb" (Props in, JSX out). Prevent waterfall requests by using parallel fetching (`Promise.all` or Suspense data fetching).

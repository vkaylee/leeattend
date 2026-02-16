# GEMINI.md - Redirect to PROJECT_OS.md

**[IMPORTANT]** Before doing anything, you MUST read and follow the project rules in:
- **[PROJECT_OS.md](../PROJECT_OS.md)** — Infrastructure, testing, debugging
- **[docs/code-standards.md](../docs/code-standards.md)** — Full code standards

## Rust Security Rules (MANDATORY)

Every Rust handler/service/repository change MUST follow these rules:

### Compile-Time Guards
- `clippy::unwrap_used = "deny"` — Never use `.unwrap()`. Use `?`, `.ok_or()`, or `.expect("reason")`. Allowed in test code.
- Never silently discard `Result` with `let _ =` on critical DB operations. Use `if let Err(e) = ... { tracing::error!(...) }` instead. (Enforced by code review, not compile-time.)

### Multi-Tenancy (CRITICAL)
- Every SELECT/UPDATE/DELETE MUST include `WHERE workspace_id = $N`
- Never expose data across workspaces
- Use `MemberContext` or `WorkspaceId` extractor for workspace scoping

### Input Validation
- Pagination: `limit.clamp(1, 100)` and `page.max(1)` on ALL list endpoints
- String lengths: Validate min/max (especially passwords: 8-128 chars)
- UUIDs: Parse from path params with proper error handling

### Error Handling
- All handlers return `AppResult<T>` — never `(StatusCode, String)`
- Propagate errors with `?` — never `.unwrap()` in production code
- Critical DB ops inside transactions: use `?` not `let _ =`
- Non-critical side effects: `if let Err(e) = ... { tracing::error!(...) }`

### Race Conditions
- Multi-step DB ops wrapped in transactions
- State-check-then-act: use `FOR UPDATE NOWAIT`
- On lock conflict: return `AppError::Conflict`

### SQL Safety
- Never interpolate user input via `format!()`
- If `format!()` used for ORDER BY: add `// SAFETY:` comment
- Prefer `sqlx::query!` or `query_as` with bind parameters

### Anti-Patterns (REJECT IMMEDIATELY)
```rust
let _ = repository::update(&pool, id).await;        // Silent error discard
let user = get_user(id).await.unwrap();              // Unwrap in production
SELECT * FROM employees WHERE id = $1;               // Missing workspace_id
let limit = query.limit.unwrap_or(20);               // Unbounded pagination
format!("SELECT * FROM {} WHERE name = '{}'", t, i); // SQL injection
```

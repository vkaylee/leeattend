# Rust Security Checklist

Every handler, service, or repository change MUST pass this checklist before completion.

## Compile-Time Guards (Enforced by Cargo.toml)
- `clippy::unwrap_used = "deny"` — Use `?`, `.ok_or()`, or `.expect("reason")` instead of `.unwrap()`. Allowed in `#[cfg(test)]` modules.

## AI-Enforced Rules (Not compile-time, but mandatory in code review)
- Never silently discard `Result` with `let _ =` on critical DB operations. Use `if let Err(e) = ... { tracing::error!(...) }` instead.

## Multi-Tenancy (CRITICAL)
- [ ] Every SELECT/UPDATE/DELETE query includes `WHERE workspace_id = $N`
- [ ] Never expose data across workspaces — JOIN through `workspace_members` when listing users
- [ ] Use `MemberContext` or `WorkspaceId` extractor, never trust client-only workspace claims

## Input Validation
- [ ] Pagination: `limit.clamp(1, 100)` and `page.max(1)` on ALL list endpoints
- [ ] String lengths: Validate min/max on passwords, names, descriptions (prevent bcrypt DoS with >128 char passwords)
- [ ] UUIDs: Parse from path params, never trust raw strings in SQL

## Error Handling
- [ ] All handlers return `AppResult<T>` — never `(StatusCode, String)`
- [ ] Propagate errors with `?` operator — never `.unwrap()` in production code
- [ ] Critical DB operations: use `?` not `let _ =` (especially inside transactions)
- [ ] Non-critical side effects (post-commit notifications): use `if let Err(e) = ... { tracing::error!(...) }`

## Race Conditions & Atomicity
- [ ] Multi-step DB operations wrapped in transactions (`pool.begin()` ... `tx.commit()`)
- [ ] State-check-then-act patterns use `FOR UPDATE NOWAIT` to prevent double-processing
- [ ] On lock conflict, return `AppError::Conflict` with user-friendly message

## SQL Safety
- [ ] Never interpolate user input into SQL strings via `format!()`
- [ ] If `format!()` is used for ORDER BY direction, add `// SAFETY:` comment proving the value is hardcoded
- [ ] Prefer `sqlx::query!` macro or `query_as` with bind parameters

## Anti-Patterns to Reject
```rust
// NEVER: Silent error discard on DB operations
let _ = repository::update(&pool, id).await;

// NEVER: Unwrap in handler/service code
let user = get_user(id).await.unwrap();

// NEVER: Query without workspace scope
SELECT * FROM employees WHERE id = $1  // Missing workspace_id!

// NEVER: Unbounded pagination
let limit = query.limit.unwrap_or(20);  // Missing .clamp()!

// NEVER: format!() with user input in SQL
format!("SELECT * FROM {} WHERE name = '{}'", table, user_input)
```

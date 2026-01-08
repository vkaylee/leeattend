# Implementation Plan: Test Factory Utilities

### ## Approach
Implement a robust testing infrastructure using the **Fluent Factory Pattern** for the Rust backend and **Functional Factories** for the TypeScript frontend. This reduces boilerplate, ensures consistent test data, and makes tests more resilient to schema changes.

### ## Steps

1. **Backend Infrastructure (Rust)** (45 min)
   - Create `apiserver/tests/common/mod.rs` and `apiserver/tests/common/factory.rs`.
   - Implement `TestFactory` to hold the database pool.
   - Implement `UserBuilder` and `WorkspaceBuilder` with fluent APIs.
   ```rust
   // Example Backend Usage
   let user = factory.user().with_email("test@example.com").create().await?;
   let ws = factory.workspace().with_name("Test Org").create().await?;
   ```

2. **Frontend Infrastructure (TypeScript)** (30 min)
   - Install `faker` for generating random test data.
   - Create `webdashboard/src/test/factories.ts`.
   - Implement factory functions for `User`, `Workspace`, and `Agent`.
   ```typescript
   // Example Frontend Usage
   const user = userFactory({ full_name: 'John Doe' });
   ```

3. **Refactoring Backend Tests** (30 min)
   - Update `apiserver/tests/agent_crud.rs` to use the new factory.
   - Update `apiserver/tests/device_management.rs`.

4. **Refactoring Frontend Tests** (15 min)
   - Update `webdashboard/src/components/AgentEditModal.test.tsx` to use the factory.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Backend Setup | 45 min |
| Frontend Setup | 30 min |
| Backend Refactor| 30 min |
| Frontend Refactor| 15 min |
| **Total** | **2 hours** |

### ## Rollback Plan
- Revert changes in `apiserver/tests/` and delete the `common/` directory.
- Delete `webdashboard/src/test/factories.ts`.

### ## Security Checklist
- [ ] Ensure test factories are **only** compiled in test mode (`#[cfg(test)]`).
- [ ] Database cleanup logic: Factories should ideally use transactions or a dedicated test database to avoid side effects.
- [ ] No real user data in factories (use Faker).

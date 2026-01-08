# Implementation Plan: Refactor AgentService to use AgentRepository (Dependency Injection)

### ## Approach
- **Instance-based Service:** Transition `AgentService` from static methods to an instance-based struct that holds its dependencies (`AgentRepository` and `AgentManager`).
- **Better Testability:** By injecting dependencies into the service instance, we can easily provide `MockAgentRepository` during unit testing without needing a full `AppState` or database.
- **Consistency:** Align with the Hexagonal Architecture pattern where services are components with clear dependencies.

### ## Steps

1. **Modify AgentService Definition** (10 min)
   - Update `apiserver/src/services/agent_service.rs`.
   - Add fields for `agent_repo` and `agent_manager`.
   ```rust
   pub struct AgentService {
       repo: Arc<dyn AgentRepository>,
       manager: AgentManager,
   }
   ```

2. **Refactor Service Methods** (15 min)
   - Change methods from `pub async fn name(repo: &dyn AgentRepository, ...)` to `pub async fn name(&self, ...)`.
   - Update internal logic to use `self.repo` and `self.manager`.

3. **Update AppState and main.rs** (10 min)
   - Add `agent_service: Arc<AgentService>` to `AppState` in `apiserver/src/lib.rs`.
   - Initialize `AgentService` in `main.rs` and inject it into `AppState`.

4. **Update Handlers** (15 min)
   - Refactor `handlers.rs` to call `state.agent_service.method()` instead of `AgentService::method()`.

5. **Unit Testing** (20 min)
   - Create a `MockAgentRepository` in `apiserver/src/services/agent_service.rs` (inside a `#[cfg(test)]` block).
   - Add unit tests for `authorize_agent` logic using the mock.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Struct Redefinition | 10 min |
| Method Refactor | 15 min |
| AppState Integration | 10 min |
| Handler Update | 15 min |
| Unit Testing | 20 min |
| **Total** | **~1 hour** |

### ## Rollback Plan
1. Revert `agent_service.rs` to the previous version (static methods).
2. Remove `agent_service` from `AppState`.
3. Restore `handlers.rs` calls to static style.
4. Git rollback: `git checkout src/services/agent_service.rs src/lib.rs src/main.rs src/handlers.rs`.

### ## Security Checklist
- [x] Error handling (Ensure repo errors are properly mapped)
- [x] No sensitive data logging
- [x] Thread safety (Use `Arc` for shared service/repo)

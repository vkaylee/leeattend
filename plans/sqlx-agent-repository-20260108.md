# Implementation Plan: SqlxAgentRepository Integration

## Approach
- Implement the `AgentRepository` trait using `sqlx` and `PostgreSQL`.
- Use the existing `PgPool` already managed in `main.rs`.
- Decouple database logic from `AgentService` by injecting this concrete implementation at startup.
- **Alternatives considered**: Continuing with inline SQL in services (rejected to maintain Hexagonal Architecture/SRP).

## Steps

1. **Define SqlxAgentRepository** (15 min)
   - File: `apiserver/src/repositories/agent_repository.rs`
   ```rust
   pub struct SqlxAgentRepository {
       pool: sqlx::PgPool,
   }

   impl SqlxAgentRepository {
       pub fn new(pool: sqlx::PgPool) -> Self {
           Self { pool }
       }
   }

   #[async_trait]
   impl AgentRepository for SqlxAgentRepository {
       // Implementation of find_by_id, update_last_ip, etc.
   }
   ```

2. **Core Implementation** (30 min)
   - Implement `find_by_id`: Select from `authorized_agents`.
   - Implement `update_last_ip`: Update `authorized_agents` set `last_ip`.
   - Implement `find_devices_by_agent_id`: Select from `devices` where `agent_id`.

3. **Integration into main.rs** (10 min)
   - Modify `apiserver/src/main.rs` to instantiate `SqlxAgentRepository`.
   - Pass it to `AgentService::new()`.

4. **Testing** (20 min)
   - Use existing integration tests (`tests/agent_trust.rs`) to verify real DB interaction.
   - Ensure `cargo test` passes with the new repository.

## Timeline
| Phase | Duration |
|-------|----------|
| Implementation | 45 min |
| Integration | 10 min |
| Testing | 20 min |
| **Total** | **1 hour 15 min** |

## Rollback Plan
- Revert `main.rs` changes to use the previous (if any) or a mock implementation.
- Since we are adding a new struct, no existing production code is deleted, only modified at the entry point.

## Security Checklist
- [x] SQL Injection prevention (using `sqlx` bind parameters).
- [x] Error handling (map DB errors to human-readable strings).
- [x] Connection pooling (utilize existing pool).

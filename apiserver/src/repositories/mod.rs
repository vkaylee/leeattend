pub mod agent_repository;
pub mod workspace_repository;
pub mod user_repository;
pub mod employee_repository;
pub mod time_entry_repository;
pub mod leave_request_repository;
pub mod payroll_repository;
pub mod shift_repository;

pub use agent_repository::{AgentRepository, SqlxAgentRepository};
pub use workspace_repository::{SqlxWorkspaceRepository, WorkspaceRepository};
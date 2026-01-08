use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenericResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceStatus {
    pub device_id: String,
    pub status: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentStatus {
    pub agent_id: String,
    pub last_seen: DateTime<Utc>,
    pub is_online: bool,
    pub devices: Vec<DeviceStatus>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AuthorizedAgent {
    pub id: String,
    pub name: String,
    pub token: String,
    pub is_trusted: bool,
    pub last_ip: Option<String>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
    pub workspace_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAuthorizedAgentRequest {
    pub name: Option<String>,
    pub is_trusted: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuthorizedAgentRequest {
    pub id: String,
    pub name: String,
    pub token: String,
    pub workspace_id: Option<Uuid>, // Optional during creation if we use a default or session-based ws
}

#[derive(Debug, Deserialize)]
pub struct TrustAgentRequest {
    pub is_trusted: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Device {
    pub id: Uuid,
    pub agent_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub is_enabled: bool,
    pub last_status: String,
    pub created_at: DateTime<Utc>,
    pub workspace_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeviceRequest {
    pub agent_id: String,
    pub name: String,
    pub ip: String,
    pub port: Option<i32>,
    pub workspace_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub name: Option<String>,
    pub ip: Option<String>,
    pub port: Option<i32>,
    pub is_enabled: Option<bool>,
}

// --- Workspace Management Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct WorkspaceMember {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct MemberExtraRole {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AssignRoleRequest {
    pub role: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct WorkspaceMemberDetail {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
}

use chrono::{DateTime, Utc, NaiveDate};
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
    pub last_seen_at: DateTime<Utc>,
    pub is_active: bool,
    pub workspace_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAuthorizedAgentRequest {
    pub name: Option<String>,
    pub token: Option<String>,
    pub is_trusted: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuthorizedAgentRequest {
    pub id: String,
    pub name: String,
    pub token: String,
    pub workspace_id: Option<Uuid>,
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
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
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

// --- Employee Management Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Employee {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub employee_id_number: String,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployeeRequest {
    pub user_id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub employee_id_number: String,
    pub hire_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeRequest {
    pub user_id: Option<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub employee_id_number: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<NaiveDate>,
    pub is_active: Option<bool>,
}

// --- Time Tracking Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct TimeEntry {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub punch_in_time: DateTime<Utc>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub entry_type: String,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PunchInRequest {
    pub employee_id: Uuid,
    pub entry_type: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PunchOutRequest {
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTimeEntryRequest {
    pub punch_in_time: Option<DateTime<Utc>>,
    pub punch_out_time: Option<DateTime<Utc>>,
    pub entry_type: Option<String>,
    pub note: Option<String>,
}

// --- Leave Management Models ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveRequest {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    pub status: String,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitLeaveRequest {
    pub employee_id: Uuid,
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveLeaveRequest {
    pub status: String,
}

// --- Basic Reporting Models ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttendanceSummary {
    pub employee_id: Uuid,
    pub employee_name: String,
    pub total_hours: f64,
    pub total_leave_days: i64,
}
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Status enum
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ShiftSwapStatus {
    PendingTarget,
    PendingAdmin,
    Approved,
    Rejected,
    Cancelled,
}

// ---------------------------------------------------------------------------
// Database row model
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct ShiftSwapRequest {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub requester_id: Uuid,
    pub target_id: Uuid,
    pub swap_date: NaiveDate,
    pub requester_shift_id: Option<Uuid>,
    pub target_shift_id: Option<Uuid>,
    pub status: ShiftSwapStatus,
    pub reason: Option<String>,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Enriched response with joined employee/shift names
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct ShiftSwapRequestWithNames {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub requester_id: Uuid,
    pub target_id: Uuid,
    pub swap_date: NaiveDate,
    pub requester_shift_id: Option<Uuid>,
    pub target_shift_id: Option<Uuid>,
    pub status: ShiftSwapStatus,
    pub reason: Option<String>,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined fields
    pub requester_name: Option<String>,
    pub target_name: Option<String>,
    pub requester_shift_name: Option<String>,
    pub target_shift_name: Option<String>,
}

// ---------------------------------------------------------------------------
// API request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateShiftSwapRequest {
    pub requester_id: Uuid,
    pub target_id: Uuid,
    pub swap_date: NaiveDate,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ApproveShiftSwapRequest {
    pub status: ShiftSwapStatus, // approved or rejected
}

// ---------------------------------------------------------------------------
// ESS (Employee Self-Service) request types
// ---------------------------------------------------------------------------

/// Employee creates a swap request (requester_id comes from EmployeeContext)
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EssCreateShiftSwapRequest {
    pub target_id: Uuid,
    pub swap_date: NaiveDate,
    pub reason: Option<String>,
}

/// Target employee responds to a swap request
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RespondShiftSwapRequest {
    pub accept: bool,
}

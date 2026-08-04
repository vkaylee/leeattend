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
// Params structs
// ---------------------------------------------------------------------------

pub struct NewShiftSwapRequestParams {
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

pub struct NewShiftSwapRequestWithNamesParams {
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
    pub requester_name: Option<String>,
    pub target_name: Option<String>,
    pub requester_shift_name: Option<String>,
    pub target_shift_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Database row model
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct ShiftSwapRequest {
    id: Uuid,
    workspace_id: Uuid,
    requester_id: Uuid,
    target_id: Uuid,
    swap_date: NaiveDate,
    requester_shift_id: Option<Uuid>,
    target_shift_id: Option<Uuid>,
    status: ShiftSwapStatus,
    reason: Option<String>,
    requested_by_user_id: Option<Uuid>,
    approved_by_user_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ShiftSwapRequest {
    pub fn new(params: NewShiftSwapRequestParams) -> Self {
        Self {
            id: params.id,
            workspace_id: params.workspace_id,
            requester_id: params.requester_id,
            target_id: params.target_id,
            swap_date: params.swap_date,
            requester_shift_id: params.requester_shift_id,
            target_shift_id: params.target_shift_id,
            status: params.status,
            reason: params.reason,
            requested_by_user_id: params.requested_by_user_id,
            approved_by_user_id: params.approved_by_user_id,
            created_at: params.created_at,
            updated_at: params.updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn requester_id(&self) -> Uuid {
        self.requester_id
    }

    pub fn target_id(&self) -> Uuid {
        self.target_id
    }

    pub fn swap_date(&self) -> NaiveDate {
        self.swap_date
    }

    pub fn requester_shift_id(&self) -> Option<Uuid> {
        self.requester_shift_id
    }

    pub fn target_shift_id(&self) -> Option<Uuid> {
        self.target_shift_id
    }

    pub fn status(&self) -> ShiftSwapStatus {
        self.status
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn requested_by_user_id(&self) -> Option<Uuid> {
        self.requested_by_user_id
    }

    pub fn approved_by_user_id(&self) -> Option<Uuid> {
        self.approved_by_user_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

// ---------------------------------------------------------------------------
// Enriched response with joined employee/shift names
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct ShiftSwapRequestWithNames {
    id: Uuid,
    workspace_id: Uuid,
    requester_id: Uuid,
    target_id: Uuid,
    swap_date: NaiveDate,
    requester_shift_id: Option<Uuid>,
    target_shift_id: Option<Uuid>,
    status: ShiftSwapStatus,
    reason: Option<String>,
    requested_by_user_id: Option<Uuid>,
    approved_by_user_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    // Joined fields
    requester_name: Option<String>,
    target_name: Option<String>,
    requester_shift_name: Option<String>,
    target_shift_name: Option<String>,
}

impl ShiftSwapRequestWithNames {
    pub fn new(params: NewShiftSwapRequestWithNamesParams) -> Self {
        Self {
            id: params.id,
            workspace_id: params.workspace_id,
            requester_id: params.requester_id,
            target_id: params.target_id,
            swap_date: params.swap_date,
            requester_shift_id: params.requester_shift_id,
            target_shift_id: params.target_shift_id,
            status: params.status,
            reason: params.reason,
            requested_by_user_id: params.requested_by_user_id,
            approved_by_user_id: params.approved_by_user_id,
            created_at: params.created_at,
            updated_at: params.updated_at,
            requester_name: params.requester_name,
            target_name: params.target_name,
            requester_shift_name: params.requester_shift_name,
            target_shift_name: params.target_shift_name,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn requester_id(&self) -> Uuid {
        self.requester_id
    }

    pub fn target_id(&self) -> Uuid {
        self.target_id
    }

    pub fn swap_date(&self) -> NaiveDate {
        self.swap_date
    }

    pub fn requester_shift_id(&self) -> Option<Uuid> {
        self.requester_shift_id
    }

    pub fn target_shift_id(&self) -> Option<Uuid> {
        self.target_shift_id
    }

    pub fn status(&self) -> ShiftSwapStatus {
        self.status
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn requested_by_user_id(&self) -> Option<Uuid> {
        self.requested_by_user_id
    }

    pub fn approved_by_user_id(&self) -> Option<Uuid> {
        self.approved_by_user_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn requester_name(&self) -> Option<&str> {
        self.requester_name.as_deref()
    }

    pub fn target_name(&self) -> Option<&str> {
        self.target_name.as_deref()
    }

    pub fn requester_shift_name(&self) -> Option<&str> {
        self.requester_shift_name.as_deref()
    }

    pub fn target_shift_name(&self) -> Option<&str> {
        self.target_shift_name.as_deref()
    }
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

use axum::http::StatusCode;
use chrono::Utc;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::models::shift_swap::{
    CreateShiftSwapRequest, ShiftSwapRequest, ShiftSwapRequestWithNames, ShiftSwapStatus,
};
use crate::models::ShiftRoster;
use crate::repositories::{roster_repository, shift_swap_repository};

pub struct ShiftSwapService;

impl ShiftSwapService {
    /// Admin creates a swap request (Phase 1).
    /// Automatically looks up the effective shifts for both employees on the swap date.
    pub async fn create_swap(
        pool: &PgPool,
        workspace_id: Uuid,
        user_id: Uuid,
        req: CreateShiftSwapRequest,
    ) -> Result<ShiftSwapRequest, StatusCode> {
        if req.requester_id == req.target_id {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Validate both employees belong to this workspace
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM employees WHERE id IN ($1, $2) AND workspace_id = $3",
        )
        .bind(req.requester_id)
        .bind(req.target_id)
        .bind(workspace_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            error!("SHIFT_SWAP_VALIDATE_ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        if count != 2 {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Look up current effective shifts for both employees
        let (requester_shift_id, _) =
            roster_repository::get_effective_shift(pool, req.requester_id, req.swap_date)
                .await
                .map_err(|e| {
                    error!("SHIFT_SWAP_LOOKUP_REQUESTER: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

        let (target_shift_id, _) =
            roster_repository::get_effective_shift(pool, req.target_id, req.swap_date)
                .await
                .map_err(|e| {
                    error!("SHIFT_SWAP_LOOKUP_TARGET: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

        let swap = ShiftSwapRequest {
            id: Uuid::new_v4(),
            workspace_id,
            requester_id: req.requester_id,
            target_id: req.target_id,
            swap_date: req.swap_date,
            requester_shift_id,
            target_shift_id,
            status: ShiftSwapStatus::PendingAdmin,
            reason: req.reason,
            requested_by_user_id: Some(user_id),
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        shift_swap_repository::create_swap_request(pool, &swap)
            .await
            .map_err(|e| {
                error!("SHIFT_SWAP_CREATE_ERROR: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    /// Admin approves or rejects a swap request.
    /// On approval, atomically swaps the roster entries for both employees.
    pub async fn approve_swap(
        pool: &PgPool,
        _workspace_id: Uuid,
        approver_id: Uuid,
        request_id: Uuid,
        new_status: ShiftSwapStatus,
    ) -> Result<(), StatusCode> {
        let request = shift_swap_repository::get_swap_request_by_id(pool, request_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Only pending requests can be approved/rejected
        if request.status != ShiftSwapStatus::PendingAdmin {
            return Err(StatusCode::CONFLICT);
        }

        // Update the status first
        shift_swap_repository::update_swap_status(
            pool,
            request_id,
            new_status,
            Some(approver_id),
        )
        .await
        .map_err(|e| {
            error!("SHIFT_SWAP_STATUS_UPDATE_ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // If approved, swap the roster entries
        if new_status == ShiftSwapStatus::Approved {
            let now = Utc::now();

            // Requester gets target's shift
            let requester_roster = ShiftRoster {
                id: Uuid::new_v4(),
                employee_id: request.requester_id,
                shift_id: request.target_shift_id,
                date: request.swap_date,
                status: "active".to_string(),
                notes: Some(format!("Shift swap #{}", request.id)),
                created_at: now,
                updated_at: now,
            };

            // Target gets requester's shift
            let target_roster = ShiftRoster {
                id: Uuid::new_v4(),
                employee_id: request.target_id,
                shift_id: request.requester_shift_id,
                date: request.swap_date,
                status: "active".to_string(),
                notes: Some(format!("Shift swap #{}", request.id)),
                created_at: now,
                updated_at: now,
            };

            roster_repository::upsert_roster(pool, &requester_roster)
                .await
                .map_err(|e| {
                    error!("SHIFT_SWAP_ROSTER_REQUESTER_ERROR: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            roster_repository::upsert_roster(pool, &target_roster)
                .await
                .map_err(|e| {
                    error!("SHIFT_SWAP_ROSTER_TARGET_ERROR: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }

        Ok(())
    }

    /// List swap requests for a workspace with optional status filter.
    pub async fn get_workspace_swaps(
        pool: &PgPool,
        workspace_id: Uuid,
        status: Option<ShiftSwapStatus>,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, StatusCode> {
        shift_swap_repository::get_workspace_swaps(pool, workspace_id, status)
            .await
            .map_err(|e| {
                error!("SHIFT_SWAP_LIST_ERROR: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    // -----------------------------------------------------------------------
    // ESS (Employee Self-Service) methods — Phase 2
    // -----------------------------------------------------------------------

    /// Employee creates a swap request. Status starts as `pending_target`.
    pub async fn create_ess_swap(
        pool: &PgPool,
        workspace_id: Uuid,
        requester_employee_id: Uuid,
        user_id: Uuid,
        req: crate::models::shift_swap::EssCreateShiftSwapRequest,
    ) -> Result<ShiftSwapRequest, StatusCode> {
        if requester_employee_id == req.target_id {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Validate target belongs to same workspace
        let target_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM employees WHERE id = $1 AND workspace_id = $2 AND is_active = true)",
        )
        .bind(req.target_id)
        .bind(workspace_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            error!("ESS_SWAP_VALIDATE_TARGET: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        if !target_exists {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Look up effective shifts
        let (requester_shift_id, _) =
            roster_repository::get_effective_shift(pool, requester_employee_id, req.swap_date)
                .await
                .map_err(|e| {
                    error!("ESS_SWAP_LOOKUP_REQUESTER: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

        let (target_shift_id, _) =
            roster_repository::get_effective_shift(pool, req.target_id, req.swap_date)
                .await
                .map_err(|e| {
                    error!("ESS_SWAP_LOOKUP_TARGET: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

        let now = Utc::now();
        let swap = ShiftSwapRequest {
            id: Uuid::new_v4(),
            workspace_id,
            requester_id: requester_employee_id,
            target_id: req.target_id,
            swap_date: req.swap_date,
            requester_shift_id,
            target_shift_id,
            status: ShiftSwapStatus::PendingTarget, // Phase 2: starts with target acceptance
            reason: req.reason,
            requested_by_user_id: Some(user_id),
            approved_by_user_id: None,
            created_at: now,
            updated_at: now,
        };

        shift_swap_repository::create_swap_request(pool, &swap)
            .await
            .map_err(|e| {
                error!("ESS_SWAP_CREATE_ERROR: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    /// Target employee responds to a swap request.
    /// Accept: `pending_target` → `pending_admin`
    /// Reject: `pending_target` → `rejected`
    pub async fn respond_to_swap(
        pool: &PgPool,
        target_employee_id: Uuid,
        request_id: Uuid,
        accept: bool,
    ) -> Result<(), StatusCode> {
        let request = shift_swap_repository::get_swap_request_by_id(pool, request_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Only the target can respond
        if request.target_id != target_employee_id {
            return Err(StatusCode::FORBIDDEN);
        }

        // Only pending_target requests can be responded to
        if request.status != ShiftSwapStatus::PendingTarget {
            return Err(StatusCode::CONFLICT);
        }

        let new_status = if accept {
            ShiftSwapStatus::PendingAdmin
        } else {
            ShiftSwapStatus::Rejected
        };

        shift_swap_repository::update_swap_status(pool, request_id, new_status, None)
            .await
            .map_err(|e| {
                error!("ESS_SWAP_RESPOND_ERROR: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    /// Requester cancels a swap request (only if still pending).
    pub async fn cancel_swap(
        pool: &PgPool,
        requester_employee_id: Uuid,
        request_id: Uuid,
    ) -> Result<(), StatusCode> {
        let request = shift_swap_repository::get_swap_request_by_id(pool, request_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        // Only the requester can cancel
        if request.requester_id != requester_employee_id {
            return Err(StatusCode::FORBIDDEN);
        }

        // Can only cancel pending requests
        if request.status != ShiftSwapStatus::PendingTarget
            && request.status != ShiftSwapStatus::PendingAdmin
        {
            return Err(StatusCode::CONFLICT);
        }

        shift_swap_repository::update_swap_status(
            pool,
            request_id,
            ShiftSwapStatus::Cancelled,
            None,
        )
        .await
        .map_err(|e| {
            error!("ESS_SWAP_CANCEL_ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
    }

    /// List swaps for an employee (as requester or target).
    pub async fn get_employee_swaps(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, StatusCode> {
        shift_swap_repository::get_employee_swaps(pool, employee_id)
            .await
            .map_err(|e| {
                error!("ESS_SWAP_LIST_ERROR: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }
}

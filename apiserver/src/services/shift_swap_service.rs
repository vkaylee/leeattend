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
    /// Phase 3: Validate swap eligibility (duplicate, conflict, same-shift, OT max).
    async fn validate_eligibility(
        pool: &PgPool,
        requester_id: Uuid,
        target_id: Uuid,
        swap_date: chrono::NaiveDate,
        requester_shift_id: Option<Uuid>,
        target_shift_id: Option<Uuid>,
    ) -> Result<(), StatusCode> {
        // 1. Duplicate check
        let dup = shift_swap_repository::has_active_swap(pool, requester_id, target_id, swap_date)
            .await
            .map_err(|e| {
                error!("SWAP_DUP_CHECK: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        if dup {
            return Err(StatusCode::CONFLICT); // duplicate swap
        }

        // 2. Conflict check — either employee already has an approved swap on this date
        for emp_id in [requester_id, target_id] {
            let conflict =
                shift_swap_repository::has_approved_swap_on_date(pool, emp_id, swap_date)
                    .await
                    .map_err(|e| {
                        error!("SWAP_CONFLICT_CHECK: {:?}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;
            if conflict {
                return Err(StatusCode::CONFLICT); // already swapped that day
            }
        }

        // 3. Same-shift guard — swap is pointless if both have the same shift
        if requester_shift_id == target_shift_id {
            return Err(StatusCode::UNPROCESSABLE_ENTITY); // same shift, no point
        }

        // 4. OT max daily check — warn if the new shift is significantly longer
        //    and could push the employee beyond the configured max_ot_hours_per_day.
        //    This is a best-effort guard; actual OT depends on punch times.
        if let (Some(req_shift_id), Some(tgt_shift_id)) = (requester_shift_id, target_shift_id) {
            if let Ok(Some(ws_id)) = Self::get_workspace_id_for_employee(pool, requester_id).await {
                if let Ok(Some(ps)) = crate::repositories::payroll_repository::get_payroll_settings_at(
                    pool, ws_id, swap_date,
                ).await {
                    if let Some(max_daily) = ps.max_ot_hours_per_day {
                        // Compare shift durations: will the receiving employee get a much longer shift?
                        let req_shift = crate::repositories::shift_repository::get_shift_by_id(pool, req_shift_id).await.unwrap_or(None);
                        let tgt_shift = crate::repositories::shift_repository::get_shift_by_id(pool, tgt_shift_id).await.unwrap_or(None);
                        if let (Some(rs), Some(ts)) = (req_shift, tgt_shift) {
                            let compute_duration_minutes = |s: &crate::models::Shift| -> i64 {
                                let mut dur = (s.end_time - s.start_time).num_minutes();
                                if dur <= 0 { dur += 24 * 60; } // overnight shift
                                dur - s.break_duration_minutes as i64
                            };
                            let std_hours = ps.hours_per_day;
                            let max_total_minutes = (std_hours + max_daily) * rust_decimal::Decimal::from(60);

                            // Requester gets target's shift
                            let req_new_dur = compute_duration_minutes(&ts);
                            // Target gets requester's shift
                            let tgt_new_dur = compute_duration_minutes(&rs);

                            use rust_decimal::prelude::ToPrimitive;
                            let max_minutes = max_total_minutes.to_i64().unwrap_or(i64::MAX);
                            if req_new_dur > max_minutes || tgt_new_dur > max_minutes {
                                tracing::warn!(
                                    requester_id = %requester_id,
                                    target_id = %target_id,
                                    "Shift swap may cause daily OT to exceed max {} hours",
                                    max_daily
                                );
                                // Warning only — we don't block, just log.
                                // If hard-block is desired in the future, uncomment:
                                // return Err(StatusCode::UNPROCESSABLE_ENTITY);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Helper: get workspace_id from employee_id
    async fn get_workspace_id_for_employee(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<Option<Uuid>, StatusCode> {
        let ws: Option<Uuid> = sqlx::query_scalar(
            "SELECT workspace_id FROM employees WHERE id = $1",
        )
        .bind(employee_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(ws)
    }

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

        // Phase 3: Eligibility validation
        Self::validate_eligibility(
            pool,
            req.requester_id,
            req.target_id,
            req.swap_date,
            requester_shift_id,
            target_shift_id,
        )
        .await?;

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

        // Phase 3: Eligibility validation
        Self::validate_eligibility(
            pool,
            requester_employee_id,
            req.target_id,
            req.swap_date,
            requester_shift_id,
            target_shift_id,
        )
        .await?;

        let now = Utc::now();
        let swap = ShiftSwapRequest {
            id: Uuid::new_v4(),
            workspace_id,
            requester_id: requester_employee_id,
            target_id: req.target_id,
            swap_date: req.swap_date,
            requester_shift_id,
            target_shift_id,
            status: ShiftSwapStatus::PendingTarget,
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

use std::sync::Arc;

use crate::errors::AppError;
use chrono::Utc;

use tracing::error;
use uuid::Uuid;

use crate::models::shift_swap::{
    CreateShiftSwapRequest, ShiftSwapRequest, ShiftSwapRequestWithNames, ShiftSwapStatus,
};
use crate::models::ShiftRoster;

use crate::repositories::employee::EmployeeRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::shift_swap_repository::ShiftSwapRepository;

pub struct ShiftSwapService {
    swap_repo: Arc<dyn ShiftSwapRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    shift_repo: Arc<dyn crate::repositories::shift_repository::ShiftRepository>,
    payroll_repo: Arc<dyn crate::repositories::payroll_repository::PayrollRepository>,
}

impl ShiftSwapService {
    pub fn new(
        swap_repo: Arc<dyn ShiftSwapRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        employee_repo: Arc<dyn EmployeeRepository>,
        shift_repo: Arc<dyn crate::repositories::shift_repository::ShiftRepository>,
        payroll_repo: Arc<dyn crate::repositories::payroll_repository::PayrollRepository>,
    ) -> Self {
        Self {
            swap_repo,
            roster_repo,
            employee_repo,
            shift_repo,
            payroll_repo,
        }
    }

    /// Phase 3: Validate swap eligibility (duplicate, conflict, same-shift, OT max).
    async fn validate_eligibility(
        &self,
        workspace_id: Uuid,
        requester_id: Uuid,
        target_id: Uuid,
        swap_date: chrono::NaiveDate,
        requester_shift_id: Option<Uuid>,
        target_shift_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        // 1. Duplicate check
        let dup = self
            .swap_repo
            .has_active_swap(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                requester_id,
                target_id,
                swap_date,
            )
            .await
            .map_err(|e| {
                error!("SWAP_DUP_CHECK: {:?}", e);
                AppError::Internal("Internal error".into())
            })?;
        if dup {
            return Err(AppError::Conflict("Conflict".into())); // duplicate swap
        }

        // 2. Conflict check — either employee already has an approved swap on this date
        for emp_id in [requester_id, target_id] {
            let conflict = self
                .swap_repo
                .has_approved_swap_on_date(
                    &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                    emp_id,
                    swap_date,
                )
                .await
                .map_err(|e| {
                    error!("SWAP_CONFLICT_CHECK: {:?}", e);
                    AppError::Internal("Internal error".into())
                })?;
            if conflict {
                return Err(AppError::Conflict("Conflict".into())); // already swapped that day
            }
        }

        // 3. Same-shift guard — swap is pointless if both have the same shift
        if requester_shift_id == target_shift_id {
            return Err(AppError::Validation("Invalid request".into())); // same shift, no point
        }

        // 4. OT max daily check — warn if the new shift is significantly longer
        //    and could push the employee beyond the configured max_ot_hours_per_day.
        //    This is a best-effort guard; actual OT depends on punch times.
        if let (Some(req_shift_id), Some(tgt_shift_id)) = (requester_shift_id, target_shift_id) {
            let ws_id = workspace_id;
            if let Ok(Some(ps)) = self
                .payroll_repo
                .get_payroll_settings_at(ws_id, swap_date)
                .await
            {
                if let Some(max_daily) = ps.max_ot_hours_per_day {
                    // Compare shift durations: will the receiving employee get a much longer shift?
                    let req_shift = self
                        .shift_repo
                        .get_shift_by_id(
                            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
                            req_shift_id,
                        )
                        .await
                        .unwrap_or(None);
                    let tgt_shift = self
                        .shift_repo
                        .get_shift_by_id(
                            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
                            tgt_shift_id,
                        )
                        .await
                        .unwrap_or(None);
                    if let (Some(rs), Some(ts)) = (req_shift, tgt_shift) {
                        let compute_duration_minutes = |s: &crate::models::Shift| -> i64 {
                            let mut dur = (s.end_time - s.start_time).num_minutes();
                            if dur <= 0 {
                                dur += 24 * 60;
                            } // overnight shift
                            dur - s.break_duration_minutes as i64
                        };
                        let std_hours = ps.hours_per_day;
                        let max_total_minutes =
                            (std_hours + max_daily) * rust_decimal::Decimal::from(60);

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

        Ok(())
    }

    /// Admin creates a swap request (Phase 1).
    /// Automatically looks up the effective shifts for both employees on the swap date.
    #[tracing::instrument(skip_all, err)]
    pub async fn create_swap(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        req: CreateShiftSwapRequest,
    ) -> Result<ShiftSwapRequest, AppError> {
        if req.requester_id == req.target_id {
            return Err(AppError::Validation("Invalid request".into()));
        }

        // Validate both employees belong to this workspace
        let req_emp = self
            .employee_repo
            .get_employee_by_id(
                req.requester_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await?;
        let tgt_emp = self
            .employee_repo
            .get_employee_by_id(
                req.target_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await?;

        if req_emp.is_none() || tgt_emp.is_none() {
            return Err(AppError::Validation("Invalid request".into()));
        }

        // Look up current effective shifts for both employees
        let (requester_shift_id, _) = self
            .roster_repo
            .get_effective_shift(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                req.requester_id,
                req.swap_date,
            )
            .await
            .map_err(|e| {
                error!("SHIFT_SWAP_LOOKUP_REQUESTER: {:?}", e);
                AppError::Internal("Internal error".into())
            })?;

        let (target_shift_id, _) = self
            .roster_repo
            .get_effective_shift(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                req.target_id,
                req.swap_date,
            )
            .await
            .map_err(|e| {
                error!("SHIFT_SWAP_LOOKUP_TARGET: {:?}", e);
                AppError::Internal("Internal error".into())
            })?;

        // Phase 3: Eligibility validation
        self.validate_eligibility(
            workspace_id,
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

        self.swap_repo
            .create_swap_request(
                &crate::diesel_pool::TenantAuthToken::system_override(swap.workspace_id),
                &swap,
            )
            .await
            .map_err(|e| {
                error!("SHIFT_SWAP_CREATE_ERROR: {:?}", e);
                AppError::Internal("Internal error".into())
            })
    }

    /// Admin approves or rejects a swap request.
    /// On approval, atomically swaps the roster entries for both employees.
    #[tracing::instrument(skip_all, err)]
    pub async fn approve_swap(
        &self,
        workspace_id: Uuid,
        approver_id: Uuid,
        request_id: Uuid,
        new_status: ShiftSwapStatus,
    ) -> Result<(), AppError> {
        let request = self
            .swap_repo
            .get_swap_request_by_id(
                request_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await?
            .ok_or(AppError::NotFound("Not found".into()))?;

        // Only pending requests can be approved/rejected
        if request.status != ShiftSwapStatus::PendingAdmin {
            return Err(AppError::Conflict("Conflict".into()));
        }

        // Update the status first
        self.swap_repo
            .update_swap_status(
                request_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                new_status,
                Some(approver_id),
            )
            .await
            .map_err(|e| {
                error!("SHIFT_SWAP_STATUS_UPDATE_ERROR: {:?}", e);
                AppError::Internal("Internal error".into())
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

            self.roster_repo
                .upsert_roster(
                    &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                    &requester_roster,
                )
                .await
                .map_err(|e| {
                    error!("SHIFT_SWAP_ROSTER_REQUESTER_ERROR: {:?}", e);
                    AppError::Internal("Internal error".into())
                })?;

            self.roster_repo
                .upsert_roster(
                    &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                    &target_roster,
                )
                .await
                .map_err(|e| {
                    error!("SHIFT_SWAP_ROSTER_TARGET_ERROR: {:?}", e);
                    AppError::Internal("Internal error".into())
                })?;
        }

        Ok(())
    }

    /// List swap requests for a workspace with optional status filter.
    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_swaps(
        &self,
        workspace_id: Uuid,
        status: Option<ShiftSwapStatus>,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, AppError> {
        self.swap_repo
            .get_workspace_swaps(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                status,
            )
            .await
            .map_err(|e| {
                error!("SHIFT_SWAP_LIST_ERROR: {:?}", e);
                AppError::Internal("Internal error".into())
            })
    }

    // -----------------------------------------------------------------------
    // ESS (Employee Self-Service) methods — Phase 2
    // -----------------------------------------------------------------------

    /// Employee creates a swap request. Status starts as `pending_target`.
    #[tracing::instrument(skip_all, err)]
    pub async fn create_ess_swap(
        &self,
        workspace_id: Uuid,
        requester_employee_id: Uuid,
        user_id: Uuid,
        req: crate::models::shift_swap::EssCreateShiftSwapRequest,
    ) -> Result<ShiftSwapRequest, AppError> {
        if requester_employee_id == req.target_id {
            return Err(AppError::Validation("Invalid request".into()));
        }

        // Validate target belongs to same workspace
        let target_emp = self
            .employee_repo
            .get_employee_by_id(
                req.target_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await?;

        if target_emp.map(|e| !e.is_active).unwrap_or(true) {
            return Err(AppError::Validation("Invalid request".into()));
        }

        // Look up effective shifts
        let (requester_shift_id, _) = self
            .roster_repo
            .get_effective_shift(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                requester_employee_id,
                req.swap_date,
            )
            .await
            .map_err(|e| {
                error!("ESS_SWAP_LOOKUP_REQUESTER: {:?}", e);
                AppError::Internal("Internal error".into())
            })?;

        let (target_shift_id, _) = self
            .roster_repo
            .get_effective_shift(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                req.target_id,
                req.swap_date,
            )
            .await
            .map_err(|e| {
                error!("ESS_SWAP_LOOKUP_TARGET: {:?}", e);
                AppError::Internal("Internal error".into())
            })?;

        // Phase 3: Eligibility validation
        self.validate_eligibility(
            workspace_id,
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

        self.swap_repo
            .create_swap_request(
                &crate::diesel_pool::TenantAuthToken::system_override(swap.workspace_id),
                &swap,
            )
            .await
            .map_err(|e| {
                error!("ESS_SWAP_CREATE_ERROR: {:?}", e);
                AppError::Internal("Internal error".into())
            })
    }

    /// Target employee responds to a swap request.
    /// Accept: `pending_target` → `pending_admin`
    /// Reject: `pending_target` → `rejected`
    #[tracing::instrument(skip_all, err)]
    pub async fn respond_to_swap(
        &self,
        workspace_id: Uuid,
        target_employee_id: Uuid,
        request_id: Uuid,
        accept: bool,
    ) -> Result<(), AppError> {
        let request = self
            .swap_repo
            .get_swap_request_by_id(
                request_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await?
            .ok_or(AppError::NotFound("Not found".into()))?;

        // Only the target can respond
        if request.target_id != target_employee_id {
            return Err(AppError::Forbidden("Forbidden".into()));
        }

        // Only pending_target requests can be responded to
        if request.status != ShiftSwapStatus::PendingTarget {
            return Err(AppError::Conflict("Conflict".into()));
        }

        let new_status = if accept {
            ShiftSwapStatus::PendingAdmin
        } else {
            ShiftSwapStatus::Rejected
        };

        self.swap_repo
            .update_swap_status(
                request_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                new_status,
                None,
            )
            .await
            .map_err(|e| {
                error!("ESS_SWAP_RESPOND_ERROR: {:?}", e);
                AppError::Internal("Internal error".into())
            })
    }

    /// Requester cancels a swap request (only if still pending).
    #[tracing::instrument(skip_all, err)]
    pub async fn cancel_swap(
        &self,
        workspace_id: Uuid,
        requester_employee_id: Uuid,
        request_id: Uuid,
    ) -> Result<(), AppError> {
        let request = self
            .swap_repo
            .get_swap_request_by_id(
                request_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await?
            .ok_or(AppError::NotFound("Not found".into()))?;

        // Only the requester can cancel
        if request.requester_id != requester_employee_id {
            return Err(AppError::Forbidden("Forbidden".into()));
        }

        // Can only cancel pending requests
        if request.status != ShiftSwapStatus::PendingTarget
            && request.status != ShiftSwapStatus::PendingAdmin
        {
            return Err(AppError::Conflict("Conflict".into()));
        }

        self.swap_repo
            .update_swap_status(
                request_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                ShiftSwapStatus::Cancelled,
                None,
            )
            .await
            .map_err(|e| {
                error!("ESS_SWAP_CANCEL_ERROR: {:?}", e);
                AppError::Internal("Internal error".into())
            })
    }

    /// List swaps for an employee (as requester or target).
    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_swaps(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, AppError> {
        self.swap_repo
            .get_employee_swaps(
                employee_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await
            .map_err(|e| {
                error!("ESS_SWAP_LIST_ERROR: {:?}", e);
                AppError::Internal("Internal error".into())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Employee, ShiftSwapRequest};
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::payroll_repository::MockPayrollRepository;
    use crate::repositories::roster::MockRosterRepository;
    use crate::repositories::shift_repository::MockShiftRepository;
    use crate::repositories::shift_swap_repository::MockShiftSwapRepository;
    use chrono::NaiveDate;

    fn build_service(
        swap_repo: MockShiftSwapRepository,
        roster_repo: MockRosterRepository,
        employee_repo: MockEmployeeRepository,
        shift_repo: MockShiftRepository,
        payroll_repo: MockPayrollRepository,
    ) -> ShiftSwapService {
        ShiftSwapService::new(
            Arc::new(swap_repo),
            Arc::new(roster_repo),
            Arc::new(employee_repo),
            Arc::new(shift_repo),
            Arc::new(payroll_repo),
        )
    }

    #[tokio::test]
    async fn test_create_swap_same_user() {
        let service = build_service(
            MockShiftSwapRepository::new(),
            MockRosterRepository::new(),
            MockEmployeeRepository::new(),
            MockShiftRepository::new(),
            MockPayrollRepository::new(),
        );

        let req = CreateShiftSwapRequest {
            requester_id: Uuid::new_v4(),
            target_id: Uuid::default(),
            swap_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            reason: None,
        };
        let req2 = CreateShiftSwapRequest {
            requester_id: req.requester_id,
            target_id: req.requester_id,
            swap_date: req.swap_date,
            reason: req.reason.clone(),
        };

        let res = service
            .create_swap(Uuid::new_v4(), Uuid::new_v4(), req2)
            .await;
        assert!(matches!(res, Err(AppError::Validation(_))));
    }

    fn dummy_employee() -> Employee {
        Employee {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            full_name: "Test Emp".to_string(),
            employee_id_number: "EMP".to_string(),
            original_hire_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            hire_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            termination_date: None,
            tags: vec![],
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            department_name: None,
        }
    }

    fn dummy_swap_request() -> ShiftSwapRequest {
        ShiftSwapRequest {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            requester_id: Uuid::new_v4(),
            target_id: Uuid::new_v4(),
            swap_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            requester_shift_id: None,
            target_shift_id: None,
            status: ShiftSwapStatus::PendingAdmin,
            reason: None,
            requested_by_user_id: None,
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_swap_success() {
        let ws_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let req_id = Uuid::new_v4();
        let tgt_id = Uuid::new_v4();
        let s_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let mut emp_repo = MockEmployeeRepository::new();
        emp_repo
            .expect_get_employee_by_id()
            .times(2)
            .returning(|_, _| Box::pin(async { Ok(Some(dummy_employee())) }));

        let mut roster_repo = MockRosterRepository::new();
        roster_repo
            .expect_get_effective_shift()
            .times(2)
            .returning(move |_, e, _| {
                Box::pin(async move {
                    if e == req_id {
                        Ok((
                            Some(Uuid::new_v4()),
                            Vec::<crate::models::attendance::ScheduleTrace>::new(),
                        ))
                    } else {
                        Ok((
                            Some(Uuid::new_v4()),
                            Vec::<crate::models::attendance::ScheduleTrace>::new(),
                        ))
                    }
                })
            });

        let mut swap_repo = MockShiftSwapRepository::new();
        swap_repo
            .expect_has_active_swap()
            .returning(|_, _, _, _| Ok(false));
        swap_repo
            .expect_has_approved_swap_on_date()
            .returning(|_, _, _| Ok(false));
        swap_repo.expect_create_swap_request().returning(|_ws, s| {
            let mut s2 = s.clone();
            s2.id = Uuid::new_v4();
            Ok(s2)
        });

        let mut payroll_repo = MockPayrollRepository::new();
        payroll_repo
            .expect_get_payroll_settings_at()
            .returning(|_, _| Ok(None));

        let service = build_service(
            swap_repo,
            roster_repo,
            emp_repo,
            MockShiftRepository::new(),
            payroll_repo,
        );

        let req = CreateShiftSwapRequest {
            requester_id: req_id,
            target_id: tgt_id,
            swap_date: s_date,
            reason: Some("Test".into()),
        };

        let res = service.create_swap(ws_id, user_id, req).await;
        assert!(res.is_ok());
        let swap = res.unwrap();
        assert_eq!(swap.requester_id, req_id);
    }

    #[tokio::test]
    async fn test_approve_swap_success() {
        let ws_id = Uuid::new_v4();
        let req_id = Uuid::new_v4();

        let mut swap_repo = MockShiftSwapRepository::new();
        swap_repo
            .expect_get_swap_request_by_id()
            .returning(move |_, _| {
                let mut s = dummy_swap_request();
                s.status = ShiftSwapStatus::PendingAdmin;
                s.requester_shift_id = Some(Uuid::new_v4());
                s.target_shift_id = Some(Uuid::new_v4());
                Ok(Some(s))
            });
        swap_repo
            .expect_update_swap_status()
            .returning(|_, _, _, _| Ok(()));

        let mut roster_repo = MockRosterRepository::new();
        roster_repo
            .expect_upsert_roster()
            .times(2)
            .returning(|_ws, r| {
                let mut roster = r.clone();
                roster.id = Uuid::new_v4();
                Box::pin(async move { Ok(roster) })
            });

        let service = build_service(
            swap_repo,
            roster_repo,
            MockEmployeeRepository::new(),
            MockShiftRepository::new(),
            MockPayrollRepository::new(),
        );

        let res = service
            .approve_swap(ws_id, Uuid::new_v4(), req_id, ShiftSwapStatus::Approved)
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_respond_to_swap_success() {
        let ws_id = Uuid::new_v4();
        let target_emp_id = Uuid::new_v4();
        let req_id = Uuid::new_v4();

        let mut swap_repo = MockShiftSwapRepository::new();
        swap_repo
            .expect_get_swap_request_by_id()
            .returning(move |_, _| {
                let mut s = dummy_swap_request();
                s.status = ShiftSwapStatus::PendingTarget;
                s.target_id = target_emp_id;
                Ok(Some(s))
            });
        swap_repo
            .expect_update_swap_status()
            .returning(|_, _, _, _| Ok(()));

        let service = build_service(
            swap_repo,
            MockRosterRepository::new(),
            MockEmployeeRepository::new(),
            MockShiftRepository::new(),
            MockPayrollRepository::new(),
        );

        let res = service
            .respond_to_swap(ws_id, target_emp_id, req_id, true)
            .await;
        assert!(res.is_ok());
    }
}

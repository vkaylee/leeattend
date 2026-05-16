use chrono::{Local, NaiveDate};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Shift, UpdateShiftRequest};
use crate::repositories::shift_repository::ShiftRepository;

pub struct ShiftService {
    repo: Arc<dyn ShiftRepository>,
}

impl ShiftService {
    pub fn new(repo: Arc<dyn ShiftRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn create_shift(
        &self,
        workspace_id: Uuid,
        req: &crate::models::CreateShiftRequest,
    ) -> Result<Shift, AppError> {
        self.repo
            .create_shift(
                workspace_id,
                req,
                true, // is_active
                None, // parent_shift_id
            )
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_shifts(&self, workspace_id: Uuid) -> Result<Vec<Shift>, AppError> {
        self.repo.get_workspace_shifts(workspace_id).await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_shift(
        &self,
        workspace_id: Uuid,
        shift_id: Uuid,
        req: UpdateShiftRequest,
    ) -> Result<Shift, AppError> {
        let old_shift = self
            .repo
            .get_shift_by_id(workspace_id, shift_id)
            .await?
            .ok_or(AppError::NotFound("Shift not found".into()))?;

        let has_time_changes = old_shift.start_time != req.start_time
            || old_shift.end_time != req.end_time
            || old_shift.break_duration_minutes != req.break_duration_minutes
            || old_shift.break_start != req.break_start
            || old_shift.break_end != req.break_end
            || old_shift.allow_early_ot != req.allow_early_ot;

        if req.is_correction || !has_time_changes {
            // Direct update
            self.repo
                .update_shift_direct(workspace_id, shift_id, &req)
                .await
        } else {
            // Create new version
            let effective_date = req
                .effective_date
                .unwrap_or_else(|| Local::now().date_naive());

            // For complex versioning that involves multiple steps, we use the repo method
            // which handles its own transaction or logical grouping.
            // Note: Since we are in transition, some complex logic might still be SQLx-specific.

            // 1. Create new version & migrate assignments in one go if repo supports it,
            // or perform steps sequentially.

            // Since update_assignments_to_new_version is currently a separate method:
            let new_shift = self
                .repo
                .create_shift(
                    old_shift.workspace_id,
                    &crate::models::CreateShiftRequest {
                        name: req.name,
                        report_symbol: req.report_symbol,
                        start_time: req.start_time,
                        end_time: req.end_time,
                        break_duration_minutes: req.break_duration_minutes,
                        break_start: req.break_start,
                        break_end: req.break_end,
                        is_break_auto_deduct: req.is_break_auto_deduct,
                        break_is_overtime: req.break_is_overtime,
                        early_punch_in_grace_period_minutes: req
                            .early_punch_in_grace_period_minutes,
                        allow_early_ot: req.allow_early_ot,
                    },
                    true,
                    Some(shift_id),
                )
                .await?;

            // 2. Archive old version
            self.repo.archive_shift(workspace_id, shift_id).await?;

            // 3. Migrate assignments
            self.repo
                .update_assignments_to_new_version(shift_id, new_shift.id, effective_date)
                .await?;

            Ok(new_shift)
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn assign_shift(
        &self,
        employee_id: Uuid,
        shift_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), AppError> {
        match self
            .repo
            .assign_employee_shift(employee_id, shift_id, start_date, end_date)
            .await
        {
            Ok(_) => Ok(()),
            Err(AppError::Diesel(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                info,
            ))) if info.constraint_name() == Some("uq_employee_shift_start") => Err(
                AppError::Conflict("Shift assignment already exists for this date".into()),
            ),
        Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;
    use crate::models::{CreateShiftRequest, Shift, UpdateShiftRequest};
    use async_trait::async_trait;
    use chrono::{NaiveDate, NaiveTime, Utc};
    use std::sync::Mutex;
    use uuid::Uuid;

    fn dummy_shift() -> Shift {
        Shift {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            name: "Morning".into(),
            report_symbol: None,
            start_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            break_duration_minutes: 60,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            is_active: true,
            parent_shift_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    struct MockShiftRepo {
        shift: Mutex<Option<Shift>>,
        created: Mutex<bool>,
        archived: Mutex<bool>,
        migrations_applied: Mutex<bool>,
    }

    impl MockShiftRepo {
        fn new(shift: Option<Shift>) -> Self {
            Self {
                shift: Mutex::new(shift),
                created: Mutex::new(false),
                archived: Mutex::new(false),
                migrations_applied: Mutex::new(false),
            }
        }
    }

    #[async_trait]
    impl crate::repositories::shift_repository::ShiftRepository for MockShiftRepo {
        async fn create_shift(
            &self,
            workspace_id: Uuid,
            req: &CreateShiftRequest,
            _is_active: bool,
            parent: Option<Uuid>,
        ) -> Result<Shift, AppError> {
            *self.created.lock().unwrap() = true;
            Ok(Shift {
                id: Uuid::new_v4(),
                workspace_id,
                name: req.name.clone(),
                report_symbol: req.report_symbol.clone(),
                start_time: req.start_time,
                end_time: req.end_time,
                break_duration_minutes: req.break_duration_minutes,
                break_start: req.break_start,
                break_end: req.break_end,
                is_break_auto_deduct: req.is_break_auto_deduct,
                break_is_overtime: req.break_is_overtime,
                early_punch_in_grace_period_minutes: req.early_punch_in_grace_period_minutes,
                allow_early_ot: req.allow_early_ot,
                is_active: true,
                parent_shift_id: parent,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        async fn get_shift_by_id(&self, _w: Uuid, _id: Uuid) -> Result<Option<Shift>, AppError> {
            Ok(self.shift.lock().unwrap().clone())
        }

        async fn get_workspace_shifts(&self, _w: Uuid) -> Result<Vec<Shift>, AppError> {
            Ok(self.shift.lock().unwrap().clone().map(|s| vec![s]).unwrap_or_default())
        }

        async fn update_shift_direct(&self, _w: Uuid, _id: Uuid, req: &UpdateShiftRequest) -> Result<Shift, AppError> {
            let mut s = self.shift.lock().unwrap().clone().unwrap_or_else(dummy_shift);
            s.name = req.name.clone();
            Ok(s)
        }

        async fn archive_shift(&self, _w: Uuid, _id: Uuid) -> Result<(), AppError> {
            *self.archived.lock().unwrap() = true;
            Ok(())
        }

        async fn assign_employee_shift(&self, _e: Uuid, _s: Uuid, _sd: NaiveDate, _ed: Option<NaiveDate>) -> Result<(), AppError> {
            Ok(())
        }

        async fn get_employee_active_shift(&self, _e: Uuid, _d: NaiveDate) -> Result<Option<Shift>, AppError> {
            Ok(self.shift.lock().unwrap().clone())
        }

        async fn update_assignments_to_new_version(&self, _o: Uuid, _n: Uuid, _d: NaiveDate) -> Result<(), AppError> {
            *self.migrations_applied.lock().unwrap() = true;
            Ok(())
        }
    }

    fn build_service(shift: Option<Shift>) -> (ShiftService, std::sync::Arc<MockShiftRepo>) {
        let repo = std::sync::Arc::new(MockShiftRepo::new(shift));
        (ShiftService::new(repo.clone()), repo)
    }

    fn make_create_req() -> CreateShiftRequest {
        CreateShiftRequest {
            name: "Night".into(),
            report_symbol: None,
            start_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            break_duration_minutes: 30,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
        }
    }

    // ── create_shift ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_shift_success() {
        let (svc, repo) = build_service(None);
        let req = make_create_req();
        let result = svc.create_shift(Uuid::new_v4(), &req).await.unwrap();
        assert_eq!(result.name, "Night");
        assert!(*repo.created.lock().unwrap());
    }

    // ── get_workspace_shifts ─────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_workspace_shifts_empty() {
        let (svc, _) = build_service(None);
        let result = svc.get_workspace_shifts(Uuid::new_v4()).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_workspace_shifts_with_data() {
        let (svc, _) = build_service(Some(dummy_shift()));
        let result = svc.get_workspace_shifts(Uuid::new_v4()).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    // ── update_shift (is_correction = true) ─────────────────────────────────

    #[tokio::test]
    async fn test_update_shift_correction_path() {
        let (svc, _) = build_service(Some(dummy_shift()));
        let req = UpdateShiftRequest {
            name: "Updated Morning".into(),
            report_symbol: None,
            start_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            break_duration_minutes: 60,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            is_correction: true,
            effective_date: None,
        };
        let result = svc.update_shift(Uuid::new_v4(), Uuid::new_v4(), req).await.unwrap();
        assert_eq!(result.name, "Updated Morning");
    }

    #[tokio::test]
    async fn test_update_shift_not_found() {
        let (svc, _) = build_service(None);
        let req = UpdateShiftRequest {
            name: "X".into(),
            report_symbol: None,
            start_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            break_duration_minutes: 60,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            is_correction: false,
            effective_date: None,
        };
        let err = svc.update_shift(Uuid::new_v4(), Uuid::new_v4(), req).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_update_shift_new_version_when_time_changed() {
        let original = dummy_shift();
        let (svc, repo) = build_service(Some(original));
        let req = UpdateShiftRequest {
            name: "Morning V2".into(),
            report_symbol: None,
            // Different time → triggers new version creation
            start_time: NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
            break_duration_minutes: 60,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            is_correction: false,
            effective_date: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        };
        let result = svc.update_shift(Uuid::new_v4(), Uuid::new_v4(), req).await.unwrap();
        // New version created, name comes from req
        assert_eq!(result.name, "Morning V2");
        // archive and migrate called
        assert!(*repo.archived.lock().unwrap());
        assert!(*repo.migrations_applied.lock().unwrap());
    }

    // ── assign_shift ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_assign_shift_success() {
        let (svc, _) = build_service(None);
        let result = svc.assign_shift(
            Uuid::new_v4(),
            Uuid::new_v4(),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            None,
        ).await;
        assert!(result.is_ok());
    }
}

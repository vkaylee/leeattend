use chrono::{Local, NaiveDate};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Shift, ShiftVersionHistoryItem, UpdateShiftRequest};
use crate::repositories::shift_repository::ShiftRepository;
use crate::utils::i18n::I18nKey;

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
        actor_id: Uuid,
        req: &crate::models::CreateShiftRequest,
    ) -> Result<Shift, AppError> {
        self.repo
            .create_shift(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                actor_id,
                req,
            )
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_shifts(
        &self,
        workspace_id: Uuid,
        location_id: Option<Uuid>,
        employer_id: Option<Uuid>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<Shift>, i64), AppError> {
        self.repo
            .get_workspace_shifts(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                location_id,
                employer_id,
                page,
                limit,
            )
            .await
    }

    /// Update a shift template by creating a new version.
    ///
    /// Two modes:
    /// - **Correction** (`is_correction=true` or no effective_date):
    ///   Fixes data errors. Keeps same `effective_from` as the current version.
    ///   The old version is superseded but its effective range stays intact.
    /// - **Revision** (`is_correction=false` with effective_date):
    ///   Scheduled change. Uses `effective_date` as new `effective_from`.
    ///   The old version's `effective_to` is set to `effective_date - 1`.
    #[tracing::instrument(skip_all, err)]
    pub async fn update_shift(
        &self,
        workspace_id: Uuid,
        template_id: Uuid,
        actor_id: Uuid,
        req: UpdateShiftRequest,
    ) -> Result<Shift, AppError> {
        // Verify the template exists
        let old_shift = self
            .repo
            .get_shift_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                template_id,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::ShiftNotFound))?;

        // Determine change type and effective date
        let has_envelope_changes = old_shift.name() != req.name
            || old_shift.report_symbol().map(String::from) != req.report_symbol
            || old_shift.location_id() != req.location_id
            || old_shift.employer_id() != req.employer_id;
        let has_interval_changes = !req.intervals.is_empty();

        if !has_envelope_changes && !has_interval_changes {
            // Nothing changed — return current shift as-is
            return Ok(old_shift);
        }

        if req.is_correction {
            // Correction: keep same effective_from, supersede current version
            let effective_from = old_shift.created_at().date_naive(); // fallback to original
            self.repo
                .create_new_version(
                    &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                    template_id,
                    &req,
                    "correct",
                    effective_from,
                    actor_id,
                    req.change_reason.clone(),
                )
                .await
        } else {
            // Revision: new effective_from = effective_date or today
            let effective_date = req
                .effective_date
                .unwrap_or_else(|| Local::now().date_naive());
            self.repo
                .create_new_version(
                    &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                    template_id,
                    &req,
                    "revise",
                    effective_date,
                    actor_id,
                    req.change_reason.clone(),
                )
                .await
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn assign_shift(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        template_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), AppError> {
        match self
            .repo
            .assign_employment_shift(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employment_id,
                template_id,
                start_date,
                end_date,
            )
            .await
        {
            Ok(()) => Ok(()),
            Err(AppError::Diesel(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                info,
            ))) if info.constraint_name() == Some("uq_employee_shift_start") => {
                Err(AppError::conflict(I18nKey::ShiftAssignmentAlreadyExists))
            }
            Err(e) => Err(e),
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_shift_versions(
        &self,
        workspace_id: Uuid,
        template_id: Uuid,
    ) -> Result<Vec<ShiftVersionHistoryItem>, AppError> {
        self.repo
            .get_shift_versions(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                template_id,
            )
            .await
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

    fn dummy_shift(template_id: Uuid) -> Shift {
        Shift::new(
            template_id,
            Uuid::new_v4(),
            "Morning".into(),
            None,
            true,
            None,
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    struct MockShiftRepo {
        shift: Mutex<Option<(Uuid, Shift)>>, // (template_id, shift)
        created: Mutex<bool>,
        archived: Mutex<bool>,
        new_version_created: Mutex<bool>,
        /// Records the change_type passed to create_new_version
        last_change_type: Mutex<Option<String>>,
        /// Records the effective_date passed to create_new_version
        last_effective_date: Mutex<Option<NaiveDate>>,
        /// Records the change_reason passed to create_new_version
        last_change_reason: Mutex<Option<Option<String>>>,
        /// Pre-populated version history for get_shift_versions
        versions: Mutex<Vec<ShiftVersionHistoryItem>>,
    }

    impl MockShiftRepo {
        fn new(template_id: Option<Uuid>, shift: Option<Shift>) -> Self {
            Self {
                shift: Mutex::new(template_id.zip(shift)),
                created: Mutex::new(false),
                archived: Mutex::new(false),
                new_version_created: Mutex::new(false),
                last_change_type: Mutex::new(None),
                last_effective_date: Mutex::new(None),
                last_change_reason: Mutex::new(None),
                versions: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl crate::repositories::shift_repository::ShiftRepository for MockShiftRepo {
        async fn create_shift(
            &self,
            auth_token: &crate::diesel_pool::TenantAuthToken,
            _actor_id: Uuid,
            req: &CreateShiftRequest,
        ) -> Result<Shift, AppError> {
            *self.created.lock().unwrap() = true;
            Ok(Shift::new(
                Uuid::new_v4(),
                auth_token.0,
                req.name.clone(),
                req.report_symbol.clone(),
                true,
                None,
                req.location_id,
                req.employer_id,
                Utc::now(),
                Utc::now(),
            ))
        }

        async fn get_shift_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _template_id: Uuid,
        ) -> Result<Option<Shift>, AppError> {
            Ok(self.shift.lock().unwrap().as_ref().map(|(_, s)| s.clone()))
        }

        async fn get_workspace_shifts(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _location_id: Option<Uuid>,
            _employer_id: Option<Uuid>,
            _page: Option<i64>,
            _limit: Option<i64>,
        ) -> Result<(Vec<Shift>, i64), AppError> {
            let shifts = self
                .shift
                .lock()
                .unwrap()
                .as_ref()
                .map(|(_, s)| vec![s.clone()])
                .unwrap_or_default();
            let total = shifts.len() as i64;
            Ok((shifts, total))
        }

        async fn create_new_version(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _template_id: Uuid,
            req: &UpdateShiftRequest,
            change_type: &str,
            effective_date: NaiveDate,
            _created_by: Uuid,
            change_reason: Option<String>,
        ) -> Result<Shift, AppError> {
            *self.new_version_created.lock().unwrap() = true;
            *self.last_change_type.lock().unwrap() = Some(change_type.to_string());
            *self.last_effective_date.lock().unwrap() = Some(effective_date);
            *self.last_change_reason.lock().unwrap() = Some(change_reason);
            let (tid, _) = self
                .shift
                .lock()
                .unwrap()
                .clone()
                .unwrap_or((Uuid::new_v4(), dummy_shift(Uuid::new_v4())));
            Ok(Shift::new(
                tid,
                Uuid::new_v4(),
                req.name.clone(),
                req.report_symbol.clone(),
                true,
                None,
                req.location_id,
                req.employer_id,
                Utc::now(),
                Utc::now(),
            ))
        }

        async fn archive_shift(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _template_id: Uuid,
        ) -> Result<(), AppError> {
            *self.archived.lock().unwrap() = true;
            Ok(())
        }

        async fn get_shift_versions(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _template_id: Uuid,
        ) -> Result<Vec<ShiftVersionHistoryItem>, AppError> {
            Ok(self.versions.lock().unwrap().clone())
        }

        async fn assign_employment_shift(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _t: Uuid,
            _sd: NaiveDate,
            _ed: Option<NaiveDate>,
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn get_employment_active_shift(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<Shift>, AppError> {
            Ok(self.shift.lock().unwrap().as_ref().map(|(_, s)| s.clone()))
        }
    }

    fn build_service(
        shift: Option<(Uuid, Shift)>,
    ) -> (ShiftService, std::sync::Arc<MockShiftRepo>) {
        let tid = shift.as_ref().map(|s| s.0);
        let s = shift.map(|s| s.1);
        let repo = std::sync::Arc::new(MockShiftRepo::new(tid, s));
        (ShiftService::new(repo.clone()), repo)
    }

    fn make_create_req() -> CreateShiftRequest {
        CreateShiftRequest {
            name: "Night".into(),
            report_symbol: None,
            location_id: None,
            employer_id: None,
            intervals: vec![crate::models::CreateIntervalRequest {
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
                break_duration_minutes: 30,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: "aggregate".to_string(),
                is_night_shift: false,
            }],
        }
    }

    // ── create_shift ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_shift_success() {
        let (svc, repo) = build_service(None);
        let req = make_create_req();
        let result = svc
            .create_shift(Uuid::new_v4(), Uuid::new_v4(), &req)
            .await
            .unwrap();
        assert_eq!(result.name(), "Night");
        assert!(*repo.created.lock().unwrap());
    }

    // ── get_workspace_shifts ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_workspace_shifts_empty() {
        let (svc, _) = build_service(None);
        let (result, total) = svc
            .get_workspace_shifts(Uuid::new_v4(), None, None, None, None)
            .await
            .unwrap();
        assert!(result.is_empty());
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_get_workspace_shifts_with_data() {
        let tid = Uuid::new_v4();
        let (svc, _) = build_service(Some((tid, dummy_shift(tid))));
        let (result, total) = svc
            .get_workspace_shifts(Uuid::new_v4(), None, None, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(total, 1);
    }

    // ── update_shift (correction) ──────────────────────────────────────────────

    #[tokio::test]
    async fn test_update_shift_correction_path() {
        let tid = Uuid::new_v4();
        let (svc, repo) = build_service(Some((tid, dummy_shift(tid))));
        let req = UpdateShiftRequest {
            name: "Updated Morning".into(),
            report_symbol: None,
            location_id: None,
            employer_id: None,
            intervals: vec![crate::models::CreateIntervalRequest {
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                break_duration_minutes: 60,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: "aggregate".to_string(),
                is_night_shift: false,
            }],
            change_reason: None,
            is_correction: true,
            effective_date: None,
        };
        let result = svc
            .update_shift(Uuid::new_v4(), tid, Uuid::new_v4(), req)
            .await
            .unwrap();
        assert_eq!(result.name(), "Updated Morning");
        assert!(*repo.new_version_created.lock().unwrap());
    }

    #[tokio::test]
    async fn test_update_shift_not_found() {
        let (svc, _) = build_service(None);
        let req = UpdateShiftRequest {
            name: "X".into(),
            report_symbol: None,
            location_id: None,
            employer_id: None,
            intervals: vec![crate::models::CreateIntervalRequest {
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                break_duration_minutes: 60,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: "aggregate".to_string(),
                is_night_shift: false,
            }],
            change_reason: None,
            is_correction: false,
            effective_date: None,
        };
        let err = svc
            .update_shift(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_update_shift_new_version_when_time_changed() {
        let tid = Uuid::new_v4();
        let (svc, repo) = build_service(Some((tid, dummy_shift(tid))));
        let req = UpdateShiftRequest {
            name: "Morning V2".into(),
            report_symbol: None,
            location_id: None,
            employer_id: None,
            intervals: vec![crate::models::CreateIntervalRequest {
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
                break_duration_minutes: 60,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: "aggregate".to_string(),
                is_night_shift: false,
            }],
            change_reason: None,
            is_correction: false,
            effective_date: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        };
        let result = svc
            .update_shift(Uuid::new_v4(), tid, Uuid::new_v4(), req)
            .await
            .unwrap();
        assert_eq!(result.name(), "Morning V2");
        assert!(*repo.new_version_created.lock().unwrap());
    }

    // ── assign_shift ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_assign_shift_success() {
        let (svc, _) = build_service(None);
        let result = svc
            .assign_shift(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                None,
            )
            .await;
        assert!(result.is_ok());
    }

    // ── New: Versioning semantics tests ───────────────────────────────────────────

    #[tokio::test]
    async fn test_correction_sends_change_type_correct() {
        let tid = Uuid::new_v4();
        let (svc, repo) = build_service(Some((tid, dummy_shift(tid))));
        let req = UpdateShiftRequest {
            name: "Morning (fixed typo)".into(),
            report_symbol: None,
            location_id: None,
            employer_id: None,
            intervals: vec![crate::models::CreateIntervalRequest {
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                break_duration_minutes: 60,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: "aggregate".to_string(),
                is_night_shift: false,
            }],
            change_reason: None,
            is_correction: true,
            effective_date: None,
        };
        let result = svc
            .update_shift(Uuid::new_v4(), tid, Uuid::new_v4(), req)
            .await
            .unwrap();
        assert_eq!(result.name(), "Morning (fixed typo)");
        assert_eq!(
            repo.last_change_type.lock().unwrap().as_deref(),
            Some("correct"),
            "Correction must use change_type='correct'"
        );
    }

    #[tokio::test]
    async fn test_revision_sends_change_type_revise_with_effective_date() {
        let tid = Uuid::new_v4();
        let (svc, repo) = build_service(Some((tid, dummy_shift(tid))));
        let effective = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
        let req = UpdateShiftRequest {
            name: "Morning V2".into(),
            report_symbol: None,
            location_id: None,
            employer_id: None,
            intervals: vec![crate::models::CreateIntervalRequest {
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
                break_duration_minutes: 45,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: "aggregate".to_string(),
                is_night_shift: false,
            }],
            change_reason: None,
            is_correction: false,
            effective_date: Some(effective),
        };
        let result = svc
            .update_shift(Uuid::new_v4(), tid, Uuid::new_v4(), req)
            .await
            .unwrap();
        assert_eq!(result.name(), "Morning V2");
        assert_eq!(
            repo.last_change_type.lock().unwrap().as_deref(),
            Some("revise"),
            "Revision must use change_type='revise'"
        );
        assert_eq!(
            *repo.last_effective_date.lock().unwrap(),
            Some(effective),
            "Revision must use the user-provided effective_date"
        );
    }

    /// No changes at all (same name, no intervals) → return current shift, no DB write.
    #[tokio::test]
    async fn test_update_shift_no_changes_returns_current() {
        let tid = Uuid::new_v4();
        let original = dummy_shift(tid);
        let (svc, repo) = build_service(Some((tid, original.clone())));
        // Same name, empty intervals → no changes
        let req = UpdateShiftRequest {
            name: original.name().to_string(),
            report_symbol: original.report_symbol().map(String::from),
            location_id: original.location_id(),
            employer_id: original.employer_id(),
            intervals: vec![], // empty = no interval changes
            change_reason: None,
            is_correction: false,
            effective_date: None,
        };
        let result = svc
            .update_shift(Uuid::new_v4(), tid, Uuid::new_v4(), req)
            .await
            .unwrap();
        assert_eq!(result.name(), original.name());
        assert!(
            !*repo.new_version_created.lock().unwrap(),
            "No-change update must NOT create a new version"
        );
    }

    /// Template ID is preserved across version changes (stable identity).
    #[tokio::test]
    async fn test_template_id_stable_across_versions() {
        let tid = Uuid::new_v4();
        let (svc, _repo) = build_service(Some((tid, dummy_shift(tid))));
        let req = UpdateShiftRequest {
            name: "Morning V3".into(),
            report_symbol: None,
            location_id: None,
            employer_id: None,
            intervals: vec![crate::models::CreateIntervalRequest {
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                break_duration_minutes: 60,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: "aggregate".to_string(),
                is_night_shift: false,
            }],
            change_reason: None,
            is_correction: false,
            effective_date: Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        };
        let result = svc
            .update_shift(Uuid::new_v4(), tid, Uuid::new_v4(), req)
            .await
            .unwrap();
        // The template_id (shift.id()) must remain `tid` — stable identity
        assert_eq!(
            result.id(),
            tid,
            "template_id must be stable across version changes"
        );
        assert_eq!(
            result.name(),
            "Morning V3",
            "name must reflect the new version"
        );
    }

    // ── get_shift_versions ──────────────────────────────────────────────────────

    fn build_service_with_versions(
        shift: Option<(Uuid, Shift)>,
        versions: Vec<ShiftVersionHistoryItem>,
    ) -> (ShiftService, std::sync::Arc<MockShiftRepo>) {
        let tid = shift.as_ref().map(|s| s.0);
        let s = shift.map(|s| s.1);
        let repo = std::sync::Arc::new(MockShiftRepo {
            shift: Mutex::new(tid.zip(s)),
            created: Mutex::new(false),
            archived: Mutex::new(false),
            new_version_created: Mutex::new(false),
            last_change_type: Mutex::new(None),
            last_effective_date: Mutex::new(None),
            last_change_reason: Mutex::new(None),
            versions: Mutex::new(versions),
        });
        (ShiftService::new(repo.clone()), repo)
    }

    #[tokio::test]
    async fn test_get_shift_versions_empty() {
        let tid = Uuid::new_v4();
        let (svc, _repo) = build_service_with_versions(Some((tid, dummy_shift(tid))), vec![]);
        let result = svc.get_shift_versions(Uuid::new_v4(), tid).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_shift_versions_returns_multiple() {
        let tid = Uuid::new_v4();
        let v1 = ShiftVersionHistoryItem {
            id: Uuid::new_v4(),
            template_id: tid,
            version_number: 2,
            name: "Morning V2".into(),
            report_symbol: Some("M2".into()),
            location_id: None,
            employer_id: None,
            effective_from: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            effective_to: None,
            change_type: "revise".into(),
            change_reason: Some("Updated start time".into()),
            superseded_by: None,
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
            intervals: vec![],
        };
        let v0 = ShiftVersionHistoryItem {
            id: Uuid::new_v4(),
            template_id: tid,
            version_number: 1,
            name: "Morning V1".into(),
            report_symbol: None,
            location_id: None,
            employer_id: None,
            effective_from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            effective_to: Some(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()),
            change_type: "create".into(),
            change_reason: None,
            superseded_by: Some(v1.id),
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
            intervals: vec![],
        };
        let versions = vec![v1, v0];
        let (svc, _repo) =
            build_service_with_versions(Some((tid, dummy_shift(tid))), versions.clone());
        let result = svc.get_shift_versions(Uuid::new_v4(), tid).await.unwrap();
        assert_eq!(result.len(), 2);
        // Should be ordered newest first (version_number DESC)
        assert_eq!(result[0].version_number, 2);
        assert_eq!(result[0].name, "Morning V2");
        assert_eq!(
            result[0].change_reason.as_deref(),
            Some("Updated start time")
        );
        assert_eq!(result[1].version_number, 1);
        assert_eq!(result[1].name, "Morning V1");
        assert!(result[1].superseded_by.is_some());
    }

    // ── change_reason propagation ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_change_reason_propagated_to_repo() {
        let tid = Uuid::new_v4();
        let (svc, repo) = build_service(Some((tid, dummy_shift(tid))));
        let req = UpdateShiftRequest {
            name: "Morning V2".into(),
            report_symbol: None,
            location_id: None,
            employer_id: None,
            intervals: vec![crate::models::CreateIntervalRequest {
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
                break_duration_minutes: 45,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: "aggregate".to_string(),
                is_night_shift: false,
            }],
            change_reason: Some("HR requested time adjustment".into()),
            is_correction: false,
            effective_date: Some(NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()),
        };
        svc.update_shift(Uuid::new_v4(), tid, Uuid::new_v4(), req)
            .await
            .unwrap();
        let reason = repo.last_change_reason.lock().unwrap().clone();
        assert_eq!(
            reason,
            Some(Some("HR requested time adjustment".to_string())),
            "change_reason must be propagated to the repository"
        );
    }

    #[tokio::test]
    async fn test_change_reason_none_when_not_provided() {
        let tid = Uuid::new_v4();
        let (svc, repo) = build_service(Some((tid, dummy_shift(tid))));
        let req = UpdateShiftRequest {
            name: "Morning V2".into(),
            report_symbol: None,
            location_id: None,
            employer_id: None,
            intervals: vec![crate::models::CreateIntervalRequest {
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
                break_duration_minutes: 45,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: "aggregate".to_string(),
                is_night_shift: false,
            }],
            change_reason: None,
            is_correction: true,
            effective_date: None,
        };
        svc.update_shift(Uuid::new_v4(), tid, Uuid::new_v4(), req)
            .await
            .unwrap();
        let reason = repo.last_change_reason.lock().unwrap().clone();
        assert_eq!(
            reason,
            Some(None),
            "change_reason should be None when not provided by the caller"
        );
    }
}

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use crate::models::{
    CreateRecruitmentSourceRequest, RecruitmentSource, RecruitmentSourceBreakdown,
    TurnoverAnalytics, UpdateRecruitmentSourceRequest,
};
use crate::repositories::employee::EmployeeRecruitmentRepository;
use crate::utils::id::new_id;

/// Manages recruitment sources, breakdown analytics, and turnover reporting.
///
/// Extracted from `EmployeeService` (SRP refactor) — owns the full
/// Recruitment bounded-context with its own repository dependency.
pub struct RecruitmentService {
    repo: Arc<dyn EmployeeRecruitmentRepository>,
}

impl RecruitmentService {
    pub fn new(repo: Arc<dyn EmployeeRecruitmentRepository>) -> Self {
        Self { repo }
    }

    // ── Recruitment Source CRUD ──────────────────────────────────────

    #[tracing::instrument(skip_all, err)]
    pub async fn create_recruitment_source(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: CreateRecruitmentSourceRequest,
    ) -> Result<RecruitmentSource, AppError> {
        let workspace_id = auth_token.workspace_id();
        let source = RecruitmentSource::new(
            new_id(),
            workspace_id,
            req.source_type,
            req.name,
            req.referrer_employee_id,
            req.referral_bonus_amount,
            false,
            req.agency_contact_name,
            req.agency_contact_email,
            req.agency_fee_amount,
            false,
            req.notes,
            Utc::now(),
            Utc::now(),
        );
        self.repo.create_recruitment_source(&source).await?;
        Ok(source)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_recruitment_sources(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<RecruitmentSource>, i64), AppError> {
        let _workspace_id = auth_token.workspace_id();
        self.repo
            .get_recruitment_sources(auth_token, page, limit)
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: UpdateRecruitmentSourceRequest,
    ) -> Result<RecruitmentSource, AppError> {
        let _workspace_id = auth_token.workspace_id();
        self.repo
            .update_recruitment_source(id, auth_token, &req)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::RecruitmentSourceNotFound))
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn delete_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<(), AppError> {
        let _workspace_id = auth_token.workspace_id();
        let rows = self.repo.delete_recruitment_source(id, auth_token).await?;
        if rows == 0 {
            return Err(AppError::not_found(I18nKey::RecruitmentSourceNotFound));
        }
        Ok(())
    }

    // ── Analytics ────────────────────────────────────────────────────

    #[tracing::instrument(skip_all, err)]
    pub async fn get_recruitment_source_breakdown(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
        let _workspace_id = auth_token.workspace_id();
        self.repo.get_recruitment_source_breakdown(auth_token).await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_turnover_analytics(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<TurnoverAnalytics, AppError> {
        let _workspace_id = auth_token.workspace_id();
        self.repo
            .get_turnover_analytics(auth_token, start_date, end_date)
            .await
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        RecruitmentSourceBreakdown, TurnoverAnalytics, TurnoverByDepartment, TurnoverBySource,
        TurnoverTenureBucket,
    };
    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};
    use std::sync::Mutex;

    // ── Mock Repository ──────────────────────────────────────────────

    struct MockRecruitmentRepo {
        recruitment_sources: Mutex<Vec<RecruitmentSource>>,
        delete_rs_rows: Mutex<u64>,
        turnover: Mutex<Option<TurnoverAnalytics>>,
        updated_recruitment_source: Mutex<Option<RecruitmentSource>>,
        recruitment_source_breakdown: Mutex<Vec<RecruitmentSourceBreakdown>>,
    }

    impl MockRecruitmentRepo {
        fn empty() -> Self {
            Self {
                recruitment_sources: Mutex::new(vec![]),
                delete_rs_rows: Mutex::new(0),
                turnover: Mutex::new(None),
                updated_recruitment_source: Mutex::new(None),
                recruitment_source_breakdown: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl EmployeeRecruitmentRepository for MockRecruitmentRepo {
        async fn set_employee_recruitment_source(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _recruitment_source_id: Option<Uuid>,
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn create_recruitment_source(
            &self,
            _source: &RecruitmentSource,
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn get_recruitment_sources(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            page: i64,
            limit: i64,
        ) -> Result<(Vec<RecruitmentSource>, i64), AppError> {
            let list = self.recruitment_sources.lock().expect("lock").clone();
            let total = list.len() as i64;
            let start = ((page - 1) * limit) as usize;
            let end = (start + limit as usize).min(list.len());
            let paginated = if start < list.len() {
                list[start..end].to_vec()
            } else {
                vec![]
            };
            Ok((paginated, total))
        }

        async fn get_recruitment_source_by_id(
            &self,
            _id: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            Ok(None)
        }

        async fn update_recruitment_source(
            &self,
            _id: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _req: &UpdateRecruitmentSourceRequest,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            Ok(self
                .updated_recruitment_source
                .lock()
                .expect("lock")
                .clone())
        }

        async fn delete_recruitment_source(
            &self,
            _id: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<u64, AppError> {
            Ok(*self.delete_rs_rows.lock().expect("lock"))
        }

        async fn get_recruitment_source_breakdown(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
            Ok(self
                .recruitment_source_breakdown
                .lock()
                .expect("lock")
                .clone())
        }

        async fn get_turnover_analytics(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            start_date: NaiveDate,
            end_date: NaiveDate,
        ) -> Result<TurnoverAnalytics, AppError> {
            Ok(self
                .turnover
                .lock()
                .expect("lock")
                .clone()
                .unwrap_or(TurnoverAnalytics {
                    tenure_distribution: vec![],
                    by_source: vec![],
                    by_department: vec![],
                    total_terminated: 0,
                    period_start: start_date.to_string(),
                    period_end: end_date.to_string(),
                }))
        }
    }

    fn make_service(mock: MockRecruitmentRepo) -> RecruitmentService {
        RecruitmentService::new(Arc::new(mock))
    }

    // ── Tests: get_recruitment_sources ───────────────────────────────

    #[tokio::test]
    async fn get_recruitment_sources_returns_list() {
        let ws = Uuid::new_v4();
        let source = RecruitmentSource::new(
            Uuid::new_v4(),
            ws,
            crate::models::enums::RecruitmentSourceType::Direct,
            "LinkedIn".into(),
            None,
            None,
            false,
            None,
            None,
            None,
            false,
            None,
            Utc::now(),
            Utc::now(),
        );

        let mock = MockRecruitmentRepo::empty();
        *mock.recruitment_sources.lock().expect("lock") = vec![source];

        let svc = make_service(mock);
        let result = svc
            .get_recruitment_sources(
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                1,
                50,
            )
            .await
            .expect("ok");
        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].name(), "LinkedIn");
    }

    // ── Tests: create_recruitment_source ─────────────────────────────

    #[tokio::test]
    async fn create_recruitment_source_success() {
        let ws = Uuid::new_v4();
        let svc = make_service(MockRecruitmentRepo::empty());
        let req = CreateRecruitmentSourceRequest {
            source_type: crate::models::enums::RecruitmentSourceType::Direct,
            name: "Indeed".into(),
            referrer_employee_id: None,
            referral_bonus_amount: None,
            agency_contact_name: None,
            agency_contact_email: None,
            agency_fee_amount: None,
            notes: None,
        };
        let result = svc
            .create_recruitment_source(
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                req,
            )
            .await;
        assert!(result.is_ok());
        let src = result.unwrap();
        assert_eq!(src.name(), "Indeed");
        assert_eq!(src.workspace_id(), ws);
    }

    // ── Tests: update_recruitment_source ─────────────────────────────

    #[tokio::test]
    async fn update_recruitment_source_not_found() {
        let svc = make_service(MockRecruitmentRepo::empty());
        let req = UpdateRecruitmentSourceRequest {
            name: Some("Updated".into()),
            referrer_employee_id: None,
            referral_bonus_amount: None,
            referral_bonus_paid: None,
            agency_contact_name: None,
            agency_contact_email: None,
            agency_fee_amount: None,
            agency_fee_paid: None,
            notes: None,
        };
        let result = svc
            .update_recruitment_source(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
                req,
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn update_recruitment_source_success() {
        let ws = Uuid::new_v4();
        let id = Uuid::new_v4();
        let mock = MockRecruitmentRepo::empty();
        let updated = RecruitmentSource::new(
            id,
            ws,
            crate::models::enums::RecruitmentSourceType::Direct,
            "Updated Name".into(),
            None,
            None,
            false,
            None,
            None,
            None,
            false,
            None,
            Utc::now(),
            Utc::now(),
        );
        *mock.updated_recruitment_source.lock().expect("lock") = Some(updated.clone());
        let svc = make_service(mock);
        let req = UpdateRecruitmentSourceRequest {
            name: Some("Updated Name".into()),
            referrer_employee_id: None,
            referral_bonus_amount: None,
            referral_bonus_paid: None,
            agency_contact_name: None,
            agency_contact_email: None,
            agency_fee_amount: None,
            agency_fee_paid: None,
            notes: None,
        };
        let result = svc
            .update_recruitment_source(
                id,
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                req,
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name(), "Updated Name");
    }

    // ── Tests: delete_recruitment_source ─────────────────────────────

    #[tokio::test]
    async fn delete_recruitment_source_success() {
        let mock = MockRecruitmentRepo::empty();
        *mock.delete_rs_rows.lock().expect("lock") = 1;
        let svc = make_service(mock);

        let result = svc
            .delete_recruitment_source(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_recruitment_source_not_found() {
        let svc = make_service(MockRecruitmentRepo::empty());
        let result = svc
            .delete_recruitment_source(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Tests: get_recruitment_source_breakdown ──────────────────────

    #[tokio::test]
    async fn get_recruitment_source_breakdown_returns_data() {
        let ws = Uuid::new_v4();
        let mock = MockRecruitmentRepo::empty();
        let breakdown = RecruitmentSourceBreakdown {
            source_type: "direct".into(),
            source_name: "Indeed".into(),
            count: 5,
        };
        *mock.recruitment_source_breakdown.lock().expect("lock") = vec![breakdown];
        let svc = make_service(mock);
        let result = svc
            .get_recruitment_source_breakdown(
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
            )
            .await;
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].source_name, "Indeed");
    }

    // ── Tests: get_turnover_analytics ────────────────────────────────

    #[tokio::test]
    async fn get_turnover_analytics_returns_data() {
        let ws = Uuid::new_v4();
        let analytics = TurnoverAnalytics {
            tenure_distribution: vec![TurnoverTenureBucket {
                bucket: "0-6 months".into(),
                count: 5,
            }],
            by_source: vec![TurnoverBySource {
                source_type: "direct".into(),
                source_name: "Walk-in".into(),
                count: 3,
            }],
            by_department: vec![TurnoverByDepartment {
                department_name: "Engineering".into(),
                count: 2,
            }],
            total_terminated: 5,
            period_start: "2026-01-01".into(),
            period_end: "2026-06-30".into(),
        };

        let mock = MockRecruitmentRepo::empty();
        *mock.turnover.lock().expect("lock") = Some(analytics);

        let svc = make_service(mock);
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date");
        let end = NaiveDate::from_ymd_opt(2026, 6, 30).expect("valid date");

        let result = svc
            .get_turnover_analytics(
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                start,
                end,
            )
            .await
            .expect("ok");
        assert_eq!(result.total_terminated, 5);
        assert_eq!(result.tenure_distribution.len(), 1);
        assert_eq!(result.by_source.len(), 1);
        assert_eq!(result.by_department[0].department_name, "Engineering");
    }
}

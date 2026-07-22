use crate::errors::{AppError, AppResult};
use crate::models::{OtRequestStatus, OvertimeRequest, OvertimeRequestEmployee};
use crate::repositories::OvertimeRequestRepository;
use crate::utils::i18n::I18nKey;
use std::sync::Arc;
use uuid::Uuid;

pub struct OvertimeRequestService {
    overtime_request_repo: Arc<dyn OvertimeRequestRepository>,
}

impl OvertimeRequestService {
    pub fn new(overtime_request_repo: Arc<dyn OvertimeRequestRepository>) -> Self {
        Self {
            overtime_request_repo,
        }
    }

    pub async fn create_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        mut request: OvertimeRequest,
        mut employees: Vec<OvertimeRequestEmployee>,
    ) -> AppResult<OvertimeRequest> {
        request.set_workspace_id(auth_token.workspace_id());

        for emp in &mut employees {
            emp.set_overtime_request_id(request.id());
        }

        self.overtime_request_repo
            .create_request(auth_token, request, employees)
            .await
    }

    pub async fn update_request_status(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request_id: Uuid,
        new_status: OtRequestStatus,
        new_step: Option<i32>,
    ) -> AppResult<OvertimeRequest> {
        let workspace_id = auth_token.workspace_id();

        let mut req = self
            .overtime_request_repo
            .get_request(auth_token, workspace_id, request_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::OvertimeRequestNotFound))?;

        req.set_status(new_status);
        if let Some(s) = new_step {
            req.set_current_step(s);
        }

        self.overtime_request_repo
            .update_request(auth_token, req)
            .await
    }

    pub async fn get_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request_id: Uuid,
    ) -> AppResult<Option<OvertimeRequest>> {
        self.overtime_request_repo
            .get_request(auth_token, auth_token.workspace_id(), request_id)
            .await
    }

    pub async fn get_request_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request_id: Uuid,
    ) -> AppResult<Vec<OvertimeRequestEmployee>> {
        self.overtime_request_repo
            .get_request_employees(auth_token, request_id)
            .await
    }

    pub async fn update_employee_consent(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request_id: Uuid,
        employee_id: Uuid,
        consent_status: &str,
        reason: Option<String>,
    ) -> AppResult<OvertimeRequestEmployee> {
        self.overtime_request_repo
            .update_employee_consent(auth_token, request_id, employee_id, consent_status, reason)
            .await
    }

    pub async fn list_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> AppResult<(Vec<OvertimeRequest>, i64)> {
        self.overtime_request_repo
            .list_requests(auth_token, page, limit)
            .await
    }
}

#[async_trait::async_trait]
impl crate::services::approval_workflow::WorkflowCallbackHandler for OvertimeRequestService {
    async fn on_workflow_completed(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_id: Uuid,
        status: &str,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let ot_status = match status {
            "approved" => OtRequestStatus::Approved,
            "rejected" => OtRequestStatus::Rejected,
            "returned" => OtRequestStatus::Draft,
            _ => OtRequestStatus::PendingApproval,
        };
        self.update_request_status(auth_token, entity_id, ot_status, None)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::TenantAuthToken;
    use crate::errors::AppError;
    use async_trait::async_trait;
    use chrono::{NaiveDate, NaiveTime, Utc};

    struct MockOvertimeRequestRepository {
        pub create_request_called: std::sync::atomic::AtomicBool,
        pub update_request_called: std::sync::atomic::AtomicBool,
    }

    impl MockOvertimeRequestRepository {
        fn new() -> Self {
            Self {
                create_request_called: std::sync::atomic::AtomicBool::new(false),
                update_request_called: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    #[async_trait]
    impl OvertimeRequestRepository for MockOvertimeRequestRepository {
        async fn create_request(
            &self,
            _auth_token: &TenantAuthToken,
            request: OvertimeRequest,
            _employees: Vec<OvertimeRequestEmployee>,
        ) -> Result<OvertimeRequest, AppError> {
            self.create_request_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(request)
        }

        async fn update_request(
            &self,
            _auth_token: &TenantAuthToken,
            request: OvertimeRequest,
        ) -> Result<OvertimeRequest, AppError> {
            self.update_request_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(request)
        }

        async fn get_request(
            &self,
            _auth_token: &TenantAuthToken,
            workspace_id: Uuid,
            _request_id: Uuid,
        ) -> Result<Option<OvertimeRequest>, AppError> {
            let req = OvertimeRequest::new(
                Uuid::new_v4(),
                workspace_id,
                Uuid::new_v4(),
                None,
                NaiveDate::from_ymd_opt(2023, 10, 10).unwrap(),
                Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
                Some(NaiveTime::from_hms_opt(20, 0, 0).unwrap()),
                120,
                "Extra work".to_string(),
                OtRequestStatus::PendingApproval,
                None,
                1,
                None,
                None,
                None,
                Utc::now(),
                Utc::now(),
                None,
            );
            Ok(Some(req))
        }

        async fn get_request_employees(
            &self,
            _auth_token: &TenantAuthToken,
            _request_id: Uuid,
        ) -> Result<Vec<OvertimeRequestEmployee>, AppError> {
            Ok(vec![])
        }

        async fn update_employee_consent(
            &self,
            _auth_token: &TenantAuthToken,
            _request_id: Uuid,
            _employee_id: Uuid,
            _consent_status: &str,
            _reason: Option<String>,
        ) -> Result<OvertimeRequestEmployee, AppError> {
            unimplemented!()
        }

        async fn list_requests(
            &self,
            _auth_token: &TenantAuthToken,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<OvertimeRequest>, i64), AppError> {
            Ok((vec![], 0))
        }

        async fn get_requests_by_ids(
            &self,
            _auth_token: &TenantAuthToken,
            _ids: &[Uuid],
        ) -> Result<Vec<OvertimeRequest>, AppError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_create_request_success() {
        let repo = Arc::new(MockOvertimeRequestRepository::new());
        let service = OvertimeRequestService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let request = OvertimeRequest::new(
            Uuid::new_v4(),
            ws_id,
            Uuid::new_v4(),
            None,
            NaiveDate::from_ymd_opt(2023, 10, 10).unwrap(),
            Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(20, 0, 0).unwrap()),
            120,
            "Extra work".to_string(),
            OtRequestStatus::PendingApproval,
            None,
            1,
            None,
            None,
            None,
            Utc::now(),
            Utc::now(),
            None,
        );

        let employee = OvertimeRequestEmployee::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "pending".to_string(),
            None,
            None,
            None,
            None,
            Utc::now(),
        );

        let result = service
            .create_request(&token, request, vec![employee])
            .await
            .unwrap();

        // Ensure workspace_id is enforced
        assert_eq!(result.workspace_id(), ws_id);
        assert!(repo
            .create_request_called
            .load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_update_request_status() {
        let repo = Arc::new(MockOvertimeRequestRepository::new());
        let service = OvertimeRequestService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let request_id = Uuid::new_v4();

        let updated = service
            .update_request_status(&token, request_id, OtRequestStatus::Approved, Some(2))
            .await
            .unwrap();

        assert_eq!(updated.status(), OtRequestStatus::Approved);
        assert_eq!(updated.current_step(), 2);
        assert!(repo
            .update_request_called
            .load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_on_workflow_completed() {
        use crate::services::approval_workflow::WorkflowCallbackHandler;

        let repo = Arc::new(MockOvertimeRequestRepository::new());
        let service = OvertimeRequestService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let request_id = Uuid::new_v4();

        service
            .on_workflow_completed(&token, request_id, "approved", Uuid::new_v4())
            .await
            .unwrap();

        assert!(repo
            .update_request_called
            .load(std::sync::atomic::Ordering::SeqCst));
    }
}

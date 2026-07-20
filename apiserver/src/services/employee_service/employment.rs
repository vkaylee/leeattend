use super::EmployeeService;
use crate::errors::AppError;
use crate::models::employment::{CreateEmploymentRequest, Employment, UpdateEmploymentRequest};
use chrono::NaiveDate;
use uuid::Uuid;

fn dates_overlap(
    s1: NaiveDate,
    e1: Option<NaiveDate>,
    s2: NaiveDate,
    e2: Option<NaiveDate>,
) -> bool {
    let overlap_start = std::cmp::max(s1, s2);
    let overlap_end = match (e1, e2) {
        (Some(d1), Some(d2)) => Some(std::cmp::min(d1, d2)),
        (Some(d1), None) => Some(d1),
        (None, Some(d2)) => Some(d2),
        (None, None) => None,
    };

    match overlap_end {
        Some(end) => overlap_start <= end,
        None => true,
    }
}

impl EmployeeService {
    #[tracing::instrument(skip_all, err)]
    pub async fn create_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &CreateEmploymentRequest,
    ) -> Result<Employment, AppError> {
        let _workspace_id = auth_token.workspace_id();

        let existing = self
            .employment_repo
            .list_employee_employments(auth_token, req.employee_id)
            .await?;

        for employment in existing {
            if employment.employer_id() == req.employer_id
                && dates_overlap(
                    req.start_date,
                    req.end_date,
                    employment.start_date(),
                    employment.end_date(),
                )
            {
                return Err(AppError::conflict(
                    crate::utils::i18n::I18nKey::OverlappingEmployment,
                ));
            }
        }

        // Delegate to repo
        self.employment_repo
            .create_employment(auth_token, req)
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        req: &UpdateEmploymentRequest,
    ) -> Result<Employment, AppError> {
        let _workspace_id = auth_token.workspace_id();

        let current_emp = self
            .employment_repo
            .get_employment_by_id(auth_token, employment_id)
            .await?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmploymentNotFound))?;

        let target_employer_id = req.employer_id.unwrap_or_else(|| current_emp.employer_id());
        let target_start = req.start_date.unwrap_or_else(|| current_emp.start_date());
        let target_end = match &req.end_date {
            Some(opt_d) => *opt_d,
            None => current_emp.end_date(),
        };

        let existing = self
            .employment_repo
            .list_employee_employments(auth_token, current_emp.employee_id())
            .await?;

        for employment in existing {
            if employment.id() == employment_id {
                continue;
            }
            if employment.employer_id() == target_employer_id
                && dates_overlap(
                    target_start,
                    target_end,
                    employment.start_date(),
                    employment.end_date(),
                )
            {
                return Err(AppError::conflict(
                    crate::utils::i18n::I18nKey::OverlappingEmployment,
                ));
            }
        }

        self.employment_repo
            .update_employment(auth_token, employment_id, req)
            .await
    }
}

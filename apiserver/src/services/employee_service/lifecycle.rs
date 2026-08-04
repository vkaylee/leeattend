use super::EmployeeService;
use crate::errors::AppError;
use crate::models::{Employee, RehireEmployeeRequest};
use crate::utils::i18n::I18nKey;
use chrono::{Datelike, NaiveDate, Utc};
use uuid::Uuid;

impl EmployeeService {
    #[tracing::instrument(skip_all, err)]
    pub async fn rehire_employee(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: RehireEmployeeRequest,
    ) -> Result<Employee, AppError> {
        self.repo
            .rehire_employee(
                auth_token,
                employee_id,
                req.hire_date,
                req.employer_id,
                req.primary_location_id,
                req.department_id,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))
    }

    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub async fn terminate_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        termination_date: NaiveDate,
        termination_reason: String,
        notes: Option<String>,
    ) -> Result<crate::models::employment::Employment, AppError> {
        // 0. Validate termination_reason enum
        const VALID_REASONS: &[&str] = &[
            "voluntary_resignation",
            "involuntary_termination",
            "end_of_contract",
            "retirement",
            "mutual_agreement",
            "abandonment",
            "death",
            "other",
        ];
        if !VALID_REASONS.contains(&termination_reason.as_str()) {
            return Err(AppError::validation(
                crate::utils::i18n::I18nKey::Validation,
            ));
        }

        // 1. Get employment details
        let employment = self
            .employment_repo
            .get_employment_by_id(auth_token, employment_id)
            .await?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmploymentNotFound))?;

        // 2. Validate termination state & date
        if employment.status() == "terminated" {
            return Err(AppError::validation(
                crate::utils::i18n::I18nKey::Validation,
            ));
        }

        if termination_date < employment.start_date() {
            return Err(AppError::validation(
                crate::utils::i18n::I18nKey::Validation,
            ));
        }

        // 3. Calculate prorated leave balance for the year of termination
        let year = termination_date.year();
        let start_of_year = NaiveDate::from_ymd_opt(year, 1, 1)
            .ok_or_else(|| AppError::Internal("Invalid start of year".to_string()))?;
        let end_of_year = NaiveDate::from_ymd_opt(year, 12, 31)
            .ok_or_else(|| AppError::Internal("Invalid end of year".to_string()))?;

        // Effective start date is the maximum of start of year or employment start date
        let eff_start = std::cmp::max(start_of_year, employment.start_date());

        // Days worked in the year
        let days_worked = (termination_date - eff_start).num_days() + 1;
        let total_days = (end_of_year - start_of_year).num_days() + 1;

        let proration_fraction = days_worked as f64 / total_days as f64;

        // Fetch existing leave balances for this employee
        let balances = self
            .leave_balance_repo
            .get_employee_balances(auth_token, employment.employee_id(), year)
            .await?;

        // Prorate entitlement for each leave balance
        let mut balances_to_upsert = Vec::new();
        for emp_bal in balances {
            let prorated_entitlement =
                (f64::from(emp_bal.entitlement) * proration_fraction).round() as i32;

            let balance = crate::models::LeaveBalance::new(
                crate::models::NewLeaveBalanceParams {
                    id: Uuid::new_v4(),
                    employment_id,
                    leave_type_id: emp_bal.leave_type_id,
                    year,
                    entitlement: prorated_entitlement,
                    accrued_amount: rust_decimal::Decimal::from_f64_retain(
                        emp_bal.accrued_amount,
                    )
                    .unwrap_or_default(),
                    carried_over: emp_bal.carried_over,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            );
            balances_to_upsert.push(balance);
        }

        // 4. Update the employment status, end date, reason, notes, and prorated balances transactionally
        let updated_employment = self
            .employment_repo
            .terminate_employment_transactional(
                auth_token,
                employment_id,
                termination_date,
                termination_reason,
                notes,
                balances_to_upsert,
            )
            .await?;

        // Trigger offboarding checklist if lifecycle service is registered
        if let Some(ref lifecycle_service) = self.lifecycle_service {
            if let Err(e) = lifecycle_service
                .auto_trigger_offboarding(auth_token, employment_id, None)
                .await
            {
                tracing::warn!(
                    "Failed to auto-trigger offboarding checklist for employment {}: {:?}",
                    employment_id,
                    e
                );
            }
        }

        Ok(updated_employment)
    }
}

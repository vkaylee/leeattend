use crate::errors::AppError;
use crate::models::diesel::employee_policy_modifier::EmployeePolicyModifierEntity;
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EmployeePolicyModifierRepository: Send + Sync {
    /// Fetch all active modifiers for an employee on a given date.
    /// Ordered by priority_rank DESC (highest rank first, lowest runs last = wins).
    async fn get_active_modifiers(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: chrono::NaiveDate,
    ) -> Result<Vec<EmployeePolicyModifierEntity>, AppError>;
}

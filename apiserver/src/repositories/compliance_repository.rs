use async_trait::async_trait;

use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::compliance::ComplianceRules;

// ---------------------------------------------------------------------------
// Trait: ComplianceRepository — DIP abstraction for workspace compliance rules
// ---------------------------------------------------------------------------
#[async_trait]
pub trait ComplianceRepository: Send + Sync {
    /// Fetch the compliance rules for a workspace.
    /// Returns defaults (12h rest, 48h/wk, 40h/mo OT, override enabled) if no
    /// row exists yet.
    async fn get_rules(&self, auth_token: &TenantAuthToken) -> Result<ComplianceRules, AppError>;

    /// Insert or update compliance rules for a workspace.
    async fn upsert_rules(
        &self,
        auth_token: &TenantAuthToken,
        rules: &ComplianceRules,
    ) -> Result<ComplianceRules, AppError>;

    /// Quick check: are compliance rules enabled for this workspace?
    /// Returns false if no row exists.
    async fn is_enabled(&self, auth_token: &TenantAuthToken) -> Result<bool, AppError>;
}

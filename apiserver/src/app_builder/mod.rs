pub mod repositories;
pub mod services;
pub mod services_attendance;
pub mod services_core;
pub mod services_leave;
pub mod workers;

use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::agent_manager::AgentManager;
use crate::queue::RedisPool;
use crate::state::AppState;

use self::repositories::build_repositories;
use self::services::build_services;
use self::workers::build_workers;

pub struct AppWorkers {
    pub job_worker_handle: Option<JoinHandle<()>>,
    pub scheduler_handle: Option<JoinHandle<()>>,
    pub outbox_sweep_handle: Option<JoinHandle<()>>,
    pub file_scan_scheduler_handle: Option<JoinHandle<()>>,
    pub pairing_scheduler_handle: Option<JoinHandle<()>>,
    pub job_worker_token: CancellationToken,
}

impl AppState {
    #[allow(clippy::too_many_lines, clippy::similar_names)]
    #[allow(clippy::disallowed_methods)]
    pub async fn build(
        diesel_pools: crate::diesel_pool::DieselPools,
        redis_pool: Option<RedisPool>,
        registry: &crate::utils::env_registry::EnvRegistry,
    ) -> Result<(Arc<Self>, AppWorkers), Box<dyn std::error::Error>> {
        let config = crate::utils::config::get_config();
        let crypto_service = if let Some(keys_json) = &config.encryption_keys_json {
            let crypto = crate::crypto::CryptoService::from_json(keys_json)
                .map_err(|e| format!("Invalid ENCRYPTION_KEYS_JSON: {e}"))?;
            Some(Arc::new(crypto))
        } else {
            if config.env == "production" {
                return Err("ENCRYPTION_KEYS_JSON must be set in production".into());
            }
            // In development / testing, generate a dummy CryptoService
            let mut keys = std::collections::HashMap::new();
            keys.insert("v1".to_string(), [0u8; 32]);
            let crypto = crate::crypto::CryptoService::new("v1".to_string(), keys)
                .map_err(|e| format!("Failed to create dummy CryptoService: {e}"))?;
            Some(Arc::new(crypto))
        };

        let agent_manager = AgentManager::new(
            redis_pool
                .clone()
                .map(|p| Arc::new(p) as Arc<dyn crate::agent_manager::PresenceStore>),
        );

        let roster_cache = redis_pool
            .as_ref()
            .map(|rpool| Arc::new(crate::cache::RosterCache::new(rpool.clone())));

        let repos = build_repositories(&diesel_pools, &crypto_service, roster_cache.clone())?;

        // Create pairing notification channel: process_log sends lightweight
        // notifications after each INSERT; PairingScheduler drains them every
        // 2 s and calls reprocess_daily_attendance once per (emp, date) group.
        let (pairing_notifier, pairing_rx) = crate::worker::pairing_scheduler::pairing_channel();
        let pairing_notifier = Some(Arc::new(pairing_notifier));

        let services = build_services(
            &diesel_pools,
            &redis_pool,
            &crypto_service,
            &agent_manager,
            &repos,
            &pairing_notifier,
            registry,
        )
        .await?;

        let (coordinator, workers) = build_workers(
            &diesel_pools,
            &redis_pool,
            &repos,
            &services,
            roster_cache.clone(),
            pairing_rx,
            pairing_notifier.clone(),
            registry,
        );

        let state = Arc::new(AppState::new(
            crate::EmployeeState::new(
                services.employee,
                services.recruitment,
                repos.employee,
                repos.employment,
                repos.contract,
                repos.employment_phase,
            ),
            crate::AttendanceState::new(
                services.time_tracking,
                services.attendance_summary,
                services.attendance,
                services.pairing,
                services.adjustment,
                services.import,
                repos.time_entry,
                repos.staging,
                repos.raw_log,
                repos.import,
                repos.error,
                repos.attendance_policy,
                repos.attendance_daily_summary,
                repos.behalf_assignment,
            ),
            crate::ShiftState::new(
                services.shift,
                services.shift_swap,
                repos.roster,
                repos.shift,
                repos.shift_swap,
                repos.assignment_rule,
                repos.working_pattern,
                services.compliance,
            ),
            crate::LeaveState::new(
                services.leave,
                services.holiday_leave,
                services.leave_accrual,
                repos.leave_request,
                repos.leave_balance,
                repos.leave_policy,
                repos.holiday,
            ),
            crate::PayrollState::new(
                services.payroll,
                repos.payroll,
                repos.payroll_rule,
                repos.department,
                repos.feature_flag,
                repos.overtime_policy,
                repos.salary_base,
                repos.overtime_request,
                services.overtime_request,
            ),
            crate::ReportState::new(services.report, coordinator, repos.stats),
            crate::CoreState::new(
                services.auth,
                services.email,
                services.workspace,
                repos.workspace,
                repos.audit,
                repos.user,
                repos.session,
                config.jwt_secret.clone(),
                repos.workspace_config,
                repos.approval_workflow,
                services.workspace_config,
                services.approval_workflow,
                services.location,
                repos.location,
                services.employer,
                repos.employer,
                services.custom_field,
                repos.custom_field,
                services.lifecycle,
                repos.lifecycle,
                services.signature,
                Arc::clone(&repos.condition_type),
                Arc::clone(&services.condition_type),
            ),
            crate::AgentState::new(agent_manager, services.agent, repos.agent),
            crate::InfraState::new(
                diesel_pools,
                redis_pool.clone(),
                redis_pool.clone().map(|r| {
                    Arc::new(crate::cache::RedisCache::new(r))
                        as Arc<dyn crate::repositories::CacheRepository>
                }),
                roster_cache,
                redis_pool.map(|r| {
                    Arc::new(crate::repositories::queue_repository::RedisQueueRepository::new(r))
                        as Arc<dyn crate::repositories::queue_repository::QueueRepository>
                }),
                services.tenant_provisioner,
                crypto_service,
                repos.system_db_cluster,
                repos.system_settings,
            ),
            services.storage_resolver,
            services.file,
            services.scan,
        ));

        Ok((state, workers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_app_state_build_invalid_db_url() {
        std::env::set_var("DB_PRIMARY_URL", "postgres://u:p@localhost:5432/db");
        std::env::set_var("DB_POOLER_TX_URL", "postgres://u:p@localhost:6543/db");
        std::env::set_var("DB_POOLER_SESS_URL", "postgres://u:p@localhost:5452/db");
        std::env::set_var("S3_ENDPOINT_URL", "http://localhost:9000");
        std::env::set_var("S3_BUCKET_NAME", "test-bucket");
        std::env::set_var("S3_ACCESS_KEY", "test-access");
        std::env::set_var("S3_SECRET_KEY", "test-secret");
        std::env::set_var("S3_REGION", "us-east-1");

        let registry = crate::utils::env_registry::EnvRegistry::resolve()
            .expect("Should resolve with minimum env");

        let invalid_url = "postgres://invalid:invalid@localhost:12345/invalid_db";
        let diesel_pools = DieselPools::for_test(invalid_url);
        let result = AppState::build(diesel_pools, None, &registry).await;
        // Since database pools are initialized lazily without eager connection check,
        // constructing the AppState struct succeeds.
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_app_state_build_env_overrides() {
        std::env::set_var("DB_PRIMARY_URL", "postgres://u:p@localhost:5432/db");
        std::env::set_var("DB_POOLER_TX_URL", "postgres://u:p@localhost:6543/db");
        std::env::set_var("DB_POOLER_SESS_URL", "postgres://u:p@localhost:5452/db");
        std::env::set_var("APP_ENV", "testing");
        std::env::set_var("S3_ENDPOINT_URL", "http://localhost:9000");
        std::env::set_var("S3_BUCKET_NAME", "test-bucket");
        std::env::set_var("S3_ACCESS_KEY", "test-access");
        std::env::set_var("S3_SECRET_KEY", "test-secret");
        std::env::set_var("S3_REGION", "us-east-1");

        let registry = crate::utils::env_registry::EnvRegistry::resolve()
            .expect("Should resolve with minimum env");

        let invalid_url = "postgres://invalid:invalid@localhost:12345/invalid_db";
        let diesel_pools = DieselPools::for_test(invalid_url);
        let result = AppState::build(diesel_pools, None, &registry).await;
        assert!(result.is_ok());
        if let Ok((state, _workers)) = result {
            assert!(state.infra().redis().is_none());
        }
    }
}

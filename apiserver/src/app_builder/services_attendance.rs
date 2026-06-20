#![allow(clippy::clone_on_ref_ptr)]
use super::repositories::Repositories;
use crate::crypto::CryptoService;
use crate::diesel_pool::DieselPools;
use crate::queue::RedisPool;
use crate::services::AttendanceLogService;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct AttendanceServices {
    pub time_tracking: Arc<crate::services::TimeTrackingService>,
    pub attendance_summary: Arc<crate::services::AttendanceSummaryService>,
    pub pairing: Arc<dyn crate::services::pairing_service::PairingProcessor>,
    pub adjustment:
        Arc<crate::services::attendance_adjustment_service::AttendanceAdjustmentService>,
    pub attendance: Arc<crate::services::AttendanceLogService>,
    pub import: Arc<crate::services::attendance_import_service::AttendanceImportService>,
    pub report: Arc<crate::services::ReportService>,
}

#[allow(clippy::too_many_lines)]
pub async fn build_attendance_services(
    diesel_pools: &DieselPools,
    _redis_pool: &Option<RedisPool>,
    crypto_service: &Option<Arc<CryptoService>>,
    repos: &Repositories,
) -> Result<(AttendanceServices, JoinHandle<()>), Box<dyn std::error::Error>> {
    let time_tracking = Arc::new(crate::services::TimeTrackingService::new(
        Arc::clone(&repos.time_entry),
        Arc::clone(&repos.roster),
        Arc::clone(&repos.policy),
        Arc::clone(&repos.employee),
        Arc::clone(&repos.employment),
        Arc::clone(&repos.workspace),
        Arc::clone(&repos.shift),
        Arc::clone(&repos.holiday),
        Arc::clone(&repos.leave_request),
        Arc::clone(&repos.attendance_daily_summary),
    ));

    let attendance_summary = Arc::new(crate::services::AttendanceSummaryService::new(
        crate::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: Arc::clone(&repos.time_entry),
            roster_repo: Arc::clone(&repos.roster),
            policy_repo: Arc::clone(&repos.policy),
            workspace_repo: Arc::clone(&repos.workspace),
            employee_repo: Arc::clone(&repos.employee),
            employment_repo: Arc::clone(&repos.employment),
            holiday_repo: Arc::clone(&repos.holiday),
            leave_request_repo: Arc::clone(&repos.leave_request),
            daily_summary_repo: Arc::clone(&repos.attendance_daily_summary),
        },
    ));

    let pairing: Arc<dyn crate::services::pairing_service::PairingProcessor> =
        Arc::new(crate::services::pairing_service::PairingService::new(
            crate::services::pairing_service::PairingServiceArgs::new(
                Arc::clone(&repos.time_entry),
                Arc::clone(&repos.roster),
                Arc::clone(&repos.employee),
                Arc::clone(&repos.raw_log),
                Arc::new(crate::repositories::DieselShiftRepository::new(
                    diesel_pools.clone(),
                )),
                Arc::clone(&attendance_summary) as Arc<dyn crate::services::SummaryRecalculator>,
            ),
        ));

    let adjustment = Arc::new(
        crate::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
            Arc::clone(&repos.adj),
            Arc::clone(&repos.raw_log),
            Arc::clone(&pairing),
        ),
    );

    let buffer_capacity: usize = std::env::var("AGENT_LOG_BUFFER_CAPACITY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000);
    let batch_size: usize = std::env::var("AGENT_LOG_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let flush_interval_ms: u64 = std::env::var("AGENT_LOG_FLUSH_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);

    let (attendance, log_worker_handle) = AttendanceLogService::new(
        Arc::clone(&pairing),
        crate::services::attendance_log_service::AttendanceLogServiceConfig {
            buffer_capacity,
            batch_size,
            flush_interval_ms,
        },
        crate::services::attendance_log_service::AttendanceLogServiceDeps {
            employee_repo: Arc::clone(&repos.employee),
            raw_log_repo: Arc::clone(&repos.raw_log),
            staging_repo: Arc::clone(&repos.staging),
            error_repo: Arc::clone(&repos.error),
        },
    );
    let attendance = Arc::new(attendance);

    let import = Arc::new(
        crate::services::attendance_import_service::AttendanceImportService::new(
            Arc::clone(&repos.import),
            Arc::clone(&repos.adj),
            Arc::clone(&repos.employee),
            Arc::clone(&repos.time_entry),
            Arc::clone(&repos.workspace),
            Arc::clone(&repos.department),
            Arc::clone(&repos.staging),
            crate::worker::deps::WorkerDeps::new_with_crypto(diesel_pools, crypto_service.clone()),
            Some(Arc::clone(&attendance)),
        ),
    );
    import.resume_all_pending_jobs().await;

    let report = Arc::new(crate::services::ReportService::new(
        crate::services::report_service::ReportServiceArgs {
            time_entry_repo: Arc::clone(&repos.time_entry),
            roster_repo: Arc::clone(&repos.roster),
            policy_repo: Arc::clone(&repos.policy),
            employee_repo: Arc::clone(&repos.employee),
            employment_repo: Arc::clone(&repos.employment),
            attendance_daily_summary_repo: Arc::clone(&repos.attendance_daily_summary),
            leave_repo: Arc::clone(&repos.leave_request),
            holiday_repo: Arc::clone(&repos.holiday),
            workspace_repo: Arc::clone(&repos.workspace),
            employer_repo: Arc::clone(&repos.employer),
        },
    ));

    Ok((
        AttendanceServices {
            time_tracking,
            attendance_summary,
            pairing,
            adjustment,
            attendance,
            import,
            report,
        },
        log_worker_handle,
    ))
}

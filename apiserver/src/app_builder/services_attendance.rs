#![allow(clippy::clone_on_ref_ptr)]
use super::repositories::Repositories;
use crate::crypto::CryptoService;
use crate::diesel_pool::DieselPools;
use crate::queue::RedisPool;
use crate::services::AttendanceLogService;
use crate::worker::pairing_scheduler::PairingNotifier;
use std::sync::Arc;

pub struct AttendanceServices {
    pub time_tracking: Arc<crate::services::TimeTrackingService>,
    pub attendance_summary: Arc<crate::services::AttendanceSummaryService>,
    pub pairing: Arc<dyn crate::services::pairing::PairingProcessor>,
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
    notifier: &Option<Arc<PairingNotifier>>,
) -> Result<AttendanceServices, Box<dyn std::error::Error>> {
    let time_tracking = Arc::new(crate::services::TimeTrackingService::new(
        Arc::clone(&repos.time_entry),
        Arc::clone(&repos.roster),
        Arc::clone(&repos.policy),
        Arc::clone(&repos.condition_type),
        Arc::clone(&repos.employee),
        Arc::clone(&repos.employment),
        Arc::clone(&repos.workspace),
        Arc::clone(&repos.holiday),
        Arc::clone(&repos.leave_request),
        Arc::clone(&repos.attendance_daily_summary),
    ));

    let attendance_summary = Arc::new(crate::services::AttendanceSummaryService::new(
        crate::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: Arc::clone(&repos.time_entry),
            roster_repo: Arc::clone(&repos.roster),
            policy_repo: Arc::clone(&repos.policy),
            condition_type_repo: Arc::clone(&repos.condition_type),
            workspace_repo: Arc::clone(&repos.workspace),
            employee_repo: Arc::clone(&repos.employee),
            employment_repo: Arc::clone(&repos.employment),
            holiday_repo: Arc::clone(&repos.holiday),
            leave_request_repo: Arc::clone(&repos.leave_request),
            daily_summary_repo: Arc::clone(&repos.attendance_daily_summary),
        },
    )
    .with_modifier_repo(Arc::clone(&repos.employee_policy_modifier))
    .with_exception_repo(Arc::clone(&repos.attendance_exception)));

    let pairing: Arc<dyn crate::services::pairing::PairingProcessor> =
        Arc::new(crate::services::pairing::PairingService::new(
            crate::services::pairing::PairingServiceArgs::new(
                Arc::clone(&repos.time_entry),
                Arc::clone(&repos.roster),
                Arc::clone(&repos.employee),
                Arc::clone(&repos.raw_log),
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

    let deps = crate::services::attendance_log_service::AttendanceLogServiceDeps {
        employee_repo: Arc::clone(&repos.employee),
        employment_repo: Arc::clone(&repos.employment),
        raw_log_repo: Arc::clone(&repos.raw_log),
        staging_repo: Arc::clone(&repos.staging),
        error_repo: Arc::clone(&repos.error),
        pairing_service: Some(
            Arc::clone(&pairing) as Arc<dyn crate::services::pairing::PairingProcessor>
        ),
    };
    let attendance = AttendanceLogService::new(deps, notifier.clone());
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
            condition_type_repo: Arc::clone(&repos.condition_type),
            employment_repo: Arc::clone(&repos.employment),
            attendance_daily_summary_repo: Arc::clone(&repos.attendance_daily_summary),
            leave_repo: Arc::clone(&repos.leave_request),
            holiday_repo: Arc::clone(&repos.holiday),
            workspace_repo: Arc::clone(&repos.workspace),
            employer_repo: Arc::clone(&repos.employer),
        },
    ));

    Ok(AttendanceServices {
        time_tracking,
        attendance_summary,
        pairing,
        adjustment,
        attendance,
        import,
        report,
    })
}

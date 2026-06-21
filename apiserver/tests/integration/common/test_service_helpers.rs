use apiserver::diesel_pool::DieselPools;
use std::sync::Arc;

/// Build a PairingService with all required dependencies from DieselPools.
/// Reduces ~70 lines of boilerplate per test to 1 function call.
pub fn build_pairing_service(
    diesel_pools: &DieselPools,
) -> (
    apiserver::services::pairing::PairingService,
    Arc<dyn apiserver::repositories::time_entry_repository::TimeEntryRepository>,
) {
    let time_entry_repo = Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
        diesel_pools.clone(),
    ));
    let summary_service = Arc::new(apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            time_entry_repo: time_entry_repo.clone(),
            roster_repo: Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
                diesel_pools.clone(),
            )),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            employment_repo: Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
                diesel_pools.clone(),
            )),
            holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            )),
            leave_request_repo: Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        },
    ));

    let service = apiserver::services::pairing::PairingService::new(
        apiserver::services::pairing::PairingServiceArgs::new(
            time_entry_repo.clone(),
            Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            Arc::new(
                apiserver::repositories::DieselRawAttendanceLogRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            Arc::new(apiserver::repositories::DieselShiftRepository::new(
                diesel_pools.clone(),
            )),
            summary_service as Arc<dyn apiserver::services::SummaryRecalculator>,
        ),
    );

    (service, time_entry_repo)
}

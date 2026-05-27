#![allow(clippy::needless_for_each)]
use crate::models::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::get_devices,
        crate::handlers::get_agents,
        crate::handlers::attendance::time_tracking::punch_in,
        crate::handlers::attendance::time_tracking::punch_out,
        crate::handlers::attendance::time_tracking::get_time_history,
        crate::handlers::attendance::logs::get_attendance_logs,
        crate::handlers::attendance::logs::get_raw_attendance_logs,
    ),
    components(
        schemas(
            Device,
            CreateDeviceRequest,
            UpdateDeviceRequest,
            AuthorizedAgent,
            TimeEntry,
            PunchInRequest,
            PunchOutRequest,
            AgentStatus,
            DeviceStatus,
            GenericResponse,
            WorkspaceStats,
            EarlyArrivalStrategy,
            LeaveStatus,
            AdjustmentStatus,
            LogSource,
            ImportJobStatus,
            LeaveTransactionType
        )
    ),
    tags(
        (name = "leeattend", description = "LeeAttend API")
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;
    use utoipa::OpenApi;

    #[test]
    fn test_openapi_generation() {
        let openapi = ApiDoc::openapi();
        let json = openapi
            .to_json()
            .expect("Failed to serialize OpenAPI to JSON");

        // Basic sanity checks
        assert!(json.contains("\"openapi\":") || json.contains("\"openapi\" :"));
        assert!(json.contains("3.1.0"));
        assert!(json.contains("\"title\":") && json.contains("apiserver"));

        // Verify paths are present
        assert!(json.contains("/agents"));
        assert!(json.contains("/devices"));

        // Verify core schemas are present
        assert!(json.contains("Device"));
        assert!(json.contains("AgentStatus"));
        assert!(json.contains("AuthorizedAgent"));
    }
}

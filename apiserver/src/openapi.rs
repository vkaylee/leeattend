use crate::models::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::get_devices,
        crate::handlers::get_agents,
    ),
    components(
        schemas(
            Device,
            CreateDeviceRequest,
            UpdateDeviceRequest,
            AuthorizedAgent,
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
        println!("Generated JSON: {json}");

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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A workspace-scoped feature flag.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FeatureFlag {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub flag_key: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a new feature flag.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateFeatureFlagRequest {
    pub flag_key: String,
    #[serde(default)]
    pub enabled: bool,
    pub description: Option<String>,
    #[serde(default = "default_metadata")]
    pub metadata: serde_json::Value,
}

/// Request payload for updating a feature flag.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateFeatureFlagRequest {
    pub enabled: Option<bool>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Response payload for checking a feature flag.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FeatureFlagCheckResponse {
    pub key: String,
    pub enabled: bool,
}

/// ADR-005 dual-run window status response.
/// Computed from the `cascade_as_primary` feature flag metadata.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CascadeDualRunStatus {
    /// Whether the dual-run window is active (cascade_as_primary flag is enabled)
    pub enabled: bool,
    /// The ISO-8601 cutover timestamp from the flag metadata
    pub cutover_at: Option<String>,
    /// ISO-8601 timestamp when the 30-day window closes
    pub window_closes_at: Option<String>,
    /// Days remaining in the dual-run window (rounded to nearest integer)
    pub days_remaining: Option<i64>,
    /// Window phase: "active" | "closing_soon" (≤7d) | "critical" (≤3d) | "final" (≤1d) | "closed"
    pub window_phase: String,
    /// Human-readable message for the UI banner
    pub message: String,
}

fn default_metadata() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_request_defaults() {
        let json = r#"{ "flag_key": "dark_mode" }"#;
        let req: CreateFeatureFlagRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.flag_key, "dark_mode");
        assert!(!req.enabled);
        assert!(req.description.is_none());
        assert_eq!(req.metadata, serde_json::json!({}));
    }

    #[test]
    fn test_create_request_all_fields() {
        let json = r#"{
            "flag_key": "beta_payroll",
            "enabled": true,
            "description": "Beta payroll feature",
            "metadata": { "rollout_percent": 50 }
        }"#;
        let req: CreateFeatureFlagRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.flag_key, "beta_payroll");
        assert!(req.enabled);
        assert_eq!(req.description.as_deref(), Some("Beta payroll feature"));
        assert_eq!(req.metadata["rollout_percent"], 50);
    }

    #[test]
    fn test_update_request_empty() {
        let json = r"{}";
        let req: UpdateFeatureFlagRequest = serde_json::from_str(json).unwrap();
        assert!(req.enabled.is_none());
        assert!(req.description.is_none());
        assert!(req.metadata.is_none());
    }

    #[test]
    fn test_feature_flag_serde_roundtrip() {
        let flag = FeatureFlag {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            flag_key: "test_flag".to_string(),
            enabled: true,
            description: Some("A test flag".to_string()),
            metadata: serde_json::json!({"rollout": 100}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&flag).unwrap();
        let deserialized: FeatureFlag = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.flag_key, "test_flag");
        assert!(deserialized.enabled);
    }
}

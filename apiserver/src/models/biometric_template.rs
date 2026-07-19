use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Metadata view of a stored fingerprint template.
///
/// PII T1: the raw template bytes (`template_encrypted`) are NEVER exposed
/// through this DTO. Only metadata needed by HR to reason about coverage is
/// returned (which fingers exist, quality, provenance).
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BiometricTemplateInfo {
    pub id: Uuid,
    pub employee_id: Uuid,
    /// Finger index 0-9 (ZK fid).
    pub finger_id: i16,
    pub quality: Option<i16>,
    pub valid: bool,
    /// Device where this template was originally enrolled (provenance).
    pub source_device_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

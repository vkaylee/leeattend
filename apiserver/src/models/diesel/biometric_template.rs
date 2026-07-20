use crate::models::biometric_template::BiometricTemplateInfo;
use crate::schema::employee_biometric_templates;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

/// Full row including the encrypted template payload.
///
/// This is the storage-layer type; the raw `template_encrypted` bytes must be
/// decrypted via CryptoService only on the push path and NEVER serialized back
/// to API clients. Use `BiometricTemplateInfo` for anything client-facing.
#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = employee_biometric_templates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BiometricTemplateEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub finger_id: i16,
    pub template_encrypted: String,
    pub encryption_key_version: String,
    pub source_device_id: Option<Uuid>,
    pub quality: Option<i16>,
    pub valid: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<BiometricTemplateEntity> for BiometricTemplateInfo {
    fn from(entity: BiometricTemplateEntity) -> Self {
        // Deliberately drops template_encrypted / encryption_key_version — those
        // never leave the server.
        Self {
            id: entity.id,
            employee_id: entity.employee_id,
            finger_id: entity.finger_id,
            quality: entity.quality,
            valid: entity.valid,
            source_device_id: entity.source_device_id,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = employee_biometric_templates)]
pub struct NewBiometricTemplate {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub finger_id: i16,
    pub template_encrypted: String,
    pub encryption_key_version: String,
    pub source_device_id: Option<Uuid>,
    pub quality: Option<i16>,
    pub valid: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // PII T1 guard: the client-facing DTO must never carry the encrypted
    // template payload or the key version. This test fails if a future field
    // addition leaks either through the `From` conversion / serde output.
    #[test]
    fn info_conversion_drops_secret_fields() {
        let secret_template = "SENTINEL_TEMPLATE_BYTES_DO_NOT_LEAK";
        let secret_key_ver = "SENTINEL_KEY_VERSION_DO_NOT_LEAK";

        let entity = BiometricTemplateEntity {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            finger_id: 3,
            template_encrypted: secret_template.to_string(),
            encryption_key_version: secret_key_ver.to_string(),
            source_device_id: Some(Uuid::new_v4()),
            quality: Some(80),
            valid: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let info: BiometricTemplateInfo = entity.into();
        let json = serde_json::to_string(&info).unwrap();

        assert!(!json.contains(secret_template));
        assert!(!json.contains(secret_key_ver));
        assert!(!json.contains("template_encrypted"));
        assert!(!json.contains("encryption_key_version"));
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignatureStatus {
    Pending,
    Signed,
    Declined,
    Canceled,
}

impl AsRef<str> for SignatureStatus {
    fn as_ref(&self) -> &str {
        match self {
            SignatureStatus::Pending => "pending",
            SignatureStatus::Signed => "signed",
            SignatureStatus::Declined => "declined",
            SignatureStatus::Canceled => "canceled",
        }
    }
}

impl std::str::FromStr for SignatureStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(SignatureStatus::Pending),
            "signed" => Ok(SignatureStatus::Signed),
            "declined" => Ok(SignatureStatus::Declined),
            "canceled" => Ok(SignatureStatus::Canceled),
            _ => Err(format!("Invalid signature status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignatureRequest {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub signer_id: Uuid,
    pub status: SignatureStatus,
    pub signed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSignatureRequestPayload {
    pub document_id: Uuid,
    pub signer_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignDocumentPayload {
    pub signature_image_base64: String, // Canvas drawing or generated text image
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

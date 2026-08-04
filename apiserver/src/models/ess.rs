use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::employee::Employee;

pub struct NewEmployeeProfileParams {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub national_id: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_branch: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeeProfile {
    id: Uuid,
    employee_id: Uuid,
    phone: Option<String>,
    address: Option<String>,
    date_of_birth: Option<NaiveDate>,
    gender: Option<String>,
    national_id: Option<String>,
    bank_name: Option<String>,
    bank_account_number: Option<String>,
    bank_branch: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl EmployeeProfile {
    pub fn new(params: NewEmployeeProfileParams) -> Self {
        Self {
            id: params.id,
            employee_id: params.employee_id,
            phone: params.phone,
            address: params.address,
            date_of_birth: params.date_of_birth,
            gender: params.gender,
            national_id: params.national_id,
            bank_name: params.bank_name,
            bank_account_number: params.bank_account_number,
            bank_branch: params.bank_branch,
            created_at: params.created_at,
            updated_at: params.updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn employee_id(&self) -> Uuid {
        self.employee_id
    }

    pub fn phone(&self) -> Option<&str> {
        self.phone.as_deref()
    }

    pub fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }

    pub fn date_of_birth(&self) -> Option<NaiveDate> {
        self.date_of_birth
    }

    pub fn gender(&self) -> Option<&str> {
        self.gender.as_deref()
    }

    pub fn national_id(&self) -> Option<&str> {
        self.national_id.as_deref()
    }

    pub fn bank_name(&self) -> Option<&str> {
        self.bank_name.as_deref()
    }

    pub fn bank_account_number(&self) -> Option<&str> {
        self.bank_account_number.as_deref()
    }

    pub fn bank_branch(&self) -> Option<&str> {
        self.bank_branch.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl std::fmt::Debug for EmployeeProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmployeeProfile")
            .field("id", &self.id)
            .field("employee_id", &self.employee_id)
            .field(
                "phone",
                &self
                    .phone
                    .as_ref()
                    .map(|p| crate::utils::masking::mask_phone(p)),
            )
            .field("address", &self.address.as_ref().map(|_| "***"))
            .field("date_of_birth", &self.date_of_birth.as_ref().map(|_| "***"))
            .field("gender", &self.gender)
            .field("national_id", &self.national_id.as_ref().map(|_| "***"))
            .field("bank_name", &self.bank_name)
            .field(
                "bank_account_number",
                &self
                    .bank_account_number
                    .as_ref()
                    .map(|a| crate::utils::masking::mask_bank_account(a)),
            )
            .field("bank_branch", &self.bank_branch)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreateEmployeeProfileRequest {
    pub phone: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub national_id: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_branch: Option<String>,
}

impl std::fmt::Debug for CreateEmployeeProfileRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateEmployeeProfileRequest")
            .field(
                "phone",
                &self
                    .phone
                    .as_ref()
                    .map(|p| crate::utils::masking::mask_phone(p)),
            )
            .field("address", &self.address.as_ref().map(|_| "***"))
            .field("date_of_birth", &self.date_of_birth.as_ref().map(|_| "***"))
            .field("gender", &self.gender)
            .field("national_id", &self.national_id.as_ref().map(|_| "***"))
            .field("bank_name", &self.bank_name)
            .field(
                "bank_account_number",
                &self
                    .bank_account_number
                    .as_ref()
                    .map(|a| crate::utils::masking::mask_bank_account(a)),
            )
            .field("bank_branch", &self.bank_branch)
            .finish()
    }
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UpdateEmployeeProfileRequest {
    pub phone: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub national_id: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_branch: Option<String>,
}

impl std::fmt::Debug for UpdateEmployeeProfileRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateEmployeeProfileRequest")
            .field(
                "phone",
                &self
                    .phone
                    .as_ref()
                    .map(|p| crate::utils::masking::mask_phone(p)),
            )
            .field("address", &self.address.as_ref().map(|_| "***"))
            .field("date_of_birth", &self.date_of_birth.as_ref().map(|_| "***"))
            .field("gender", &self.gender)
            .field("national_id", &self.national_id.as_ref().map(|_| "***"))
            .field("bank_name", &self.bank_name)
            .field(
                "bank_account_number",
                &self
                    .bank_account_number
                    .as_ref()
                    .map(|a| crate::utils::masking::mask_bank_account(a)),
            )
            .field("bank_branch", &self.bank_branch)
            .finish()
    }
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeeEmergencyContact {
    id: Uuid,
    employee_id: Uuid,
    contact_name: String,
    relationship: String,
    phone: String,
    is_primary: bool,
    created_at: DateTime<Utc>,
}

impl EmployeeEmergencyContact {
    pub fn new(
        id: Uuid,
        employee_id: Uuid,
        contact_name: String,
        relationship: String,
        phone: String,
        is_primary: bool,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            employee_id,
            contact_name,
            relationship,
            phone,
            is_primary,
            created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn employee_id(&self) -> Uuid {
        self.employee_id
    }

    pub fn contact_name(&self) -> &str {
        &self.contact_name
    }

    pub fn relationship(&self) -> &str {
        &self.relationship
    }

    pub fn phone(&self) -> &str {
        &self.phone
    }

    pub fn is_primary(&self) -> bool {
        self.is_primary
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

impl std::fmt::Debug for EmployeeEmergencyContact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmployeeEmergencyContact")
            .field("id", &self.id)
            .field("employee_id", &self.employee_id)
            .field("contact_name", &self.contact_name)
            .field("relationship", &self.relationship)
            .field("phone", &crate::utils::masking::mask_phone(&self.phone))
            .field("is_primary", &self.is_primary)
            .field("created_at", &self.created_at)
            .finish()
    }
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreateEmergencyContactRequest {
    pub contact_name: String,
    pub relationship: String,
    pub phone: String,
    pub is_primary: Option<bool>,
}

impl std::fmt::Debug for CreateEmergencyContactRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateEmergencyContactRequest")
            .field("contact_name", &self.contact_name)
            .field("relationship", &self.relationship)
            .field("phone", &crate::utils::masking::mask_phone(&self.phone))
            .field("is_primary", &self.is_primary)
            .finish()
    }
}

pub struct NewEmployeeDocumentParams {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
    pub document_type: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i32,
    pub mime_type: String,
    pub expires_at: Option<NaiveDate>,
    pub is_active: bool,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeeDocument {
    id: Uuid,
    employee_id: Uuid,
    workspace_id: Uuid,
    document_type: String,
    file_name: String,
    file_path: String,
    file_size: i32,
    mime_type: String,
    expires_at: Option<NaiveDate>,
    is_active: bool,
    uploaded_by: Option<Uuid>,
    created_at: DateTime<Utc>,
}

impl EmployeeDocument {
    pub fn new(params: NewEmployeeDocumentParams) -> Self {
        Self {
            id: params.id,
            employee_id: params.employee_id,
            workspace_id: params.workspace_id,
            document_type: params.document_type,
            file_name: params.file_name,
            file_path: params.file_path,
            file_size: params.file_size,
            mime_type: params.mime_type,
            expires_at: params.expires_at,
            is_active: params.is_active,
            uploaded_by: params.uploaded_by,
            created_at: params.created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn employee_id(&self) -> Uuid {
        self.employee_id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn document_type(&self) -> &str {
        &self.document_type
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    pub fn file_size(&self) -> i32 {
        self.file_size
    }

    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }

    pub fn expires_at(&self) -> Option<NaiveDate> {
        self.expires_at
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn uploaded_by(&self) -> Option<Uuid> {
        self.uploaded_by
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

impl std::fmt::Debug for EmployeeDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmployeeDocument")
            .field("id", &self.id)
            .field("employee_id", &self.employee_id)
            .field("workspace_id", &self.workspace_id)
            .field("document_type", &self.document_type)
            .field("file_name", &self.file_name)
            .field("file_path", &self.file_path)
            .field("file_size", &self.file_size)
            .field("mime_type", &self.mime_type)
            .field("expires_at", &self.expires_at)
            .field("is_active", &self.is_active)
            .field("uploaded_by", &self.uploaded_by)
            .field("created_at", &self.created_at)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UploadDocumentRequest {
    pub document_type: String,
    pub expires_at: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct MyProfileResponse {
    pub employee: Employee,
    pub profile: Option<EmployeeProfile>,
    pub emergency_contacts: Vec<EmployeeEmergencyContact>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    #[test]
    fn test_employee_profile_debug_masks_pii() {
        let profile = EmployeeProfile::new(NewEmployeeProfileParams {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            phone: Some("0987654321".to_string()),
            address: Some("123 Street Name".to_string()),
            date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
            gender: Some("Male".to_string()),
            national_id: Some("123456789".to_string()),
            bank_name: Some("Test Bank".to_string()),
            bank_account_number: Some("1234567890".to_string()),
            bank_branch: Some("Branch Name".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        let debug_str = format!("{profile:?}");
        assert!(!debug_str.contains("0987654321"));
        assert!(debug_str.contains("098****321"));
        assert!(!debug_str.contains("123 Street Name"));
        assert!(!debug_str.contains("1990-01-01"));
        assert!(!debug_str.contains("123456789"));
        assert!(!debug_str.contains("1234567890"));
        assert!(debug_str.contains("******7890"));
        assert!(debug_str.contains("Male"));
        assert!(debug_str.contains("Test Bank"));
    }

    #[test]
    fn test_emergency_contact_debug_masks_pii() {
        let contact = EmployeeEmergencyContact::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "John Doe".to_string(),
            "Brother".to_string(),
            "0987654321".to_string(),
            true,
            Utc::now(),
        );

        let debug_str = format!("{contact:?}");
        assert!(debug_str.contains("John Doe"));
        assert!(!debug_str.contains("0987654321"));
        assert!(debug_str.contains("098****321"));
    }

    #[test]
    fn test_create_employee_profile_request_debug_masks_pii() {
        let req = CreateEmployeeProfileRequest {
            phone: Some("0987654321".to_string()),
            address: Some("123 Street Name".to_string()),
            date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
            gender: Some("Male".to_string()),
            national_id: Some("123456789".to_string()),
            bank_name: Some("Test Bank".to_string()),
            bank_account_number: Some("1234567890".to_string()),
            bank_branch: Some("Branch Name".to_string()),
        };

        let debug_str = format!("{req:?}");
        assert!(!debug_str.contains("0987654321"));
        assert!(debug_str.contains("098****321"));
        assert!(!debug_str.contains("123 Street Name"));
        assert!(!debug_str.contains("1990-01-01"));
        assert!(!debug_str.contains("123456789"));
        assert!(!debug_str.contains("1234567890"));
        assert!(debug_str.contains("******7890"));
    }

    #[test]
    fn test_update_employee_profile_request_debug_masks_pii() {
        let req = UpdateEmployeeProfileRequest {
            phone: Some("0987654321".to_string()),
            address: Some("123 Street Name".to_string()),
            date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
            gender: Some("Male".to_string()),
            national_id: Some("123456789".to_string()),
            bank_name: Some("Test Bank".to_string()),
            bank_account_number: Some("1234567890".to_string()),
            bank_branch: Some("Branch Name".to_string()),
        };

        let debug_str = format!("{req:?}");
        assert!(!debug_str.contains("0987654321"));
        assert!(debug_str.contains("098****321"));
        assert!(!debug_str.contains("123456789"));
        assert!(!debug_str.contains("1234567890"));
        assert!(debug_str.contains("******7890"));
    }

    #[test]
    fn test_create_emergency_contact_request_debug_masks_pii() {
        let req = CreateEmergencyContactRequest {
            contact_name: "John Doe".to_string(),
            relationship: "Brother".to_string(),
            phone: "0987654321".to_string(),
            is_primary: Some(true),
        };

        let debug_str = format!("{req:?}");
        assert!(debug_str.contains("John Doe"));
        assert!(!debug_str.contains("0987654321"));
        assert!(debug_str.contains("098****321"));
    }

    #[test]
    fn test_other_ess_models() {
        let doc = EmployeeDocument::new(NewEmployeeDocumentParams {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            document_type: "id_card".to_string(),
            file_name: "id.png".to_string(),
            file_path: "/path/to/file".to_string(),
            file_size: 1024,
            mime_type: "image/png".to_string(),
            expires_at: Some(NaiveDate::from_ymd_opt(2030, 1, 1).unwrap()),
            is_active: true,
            uploaded_by: Some(Uuid::new_v4()),
            created_at: Utc::now(),
        });

        assert_eq!(doc.file_name(), "id.png");

        let req = UploadDocumentRequest {
            document_type: "contract".to_string(),
            expires_at: None,
        };
        assert_eq!(req.document_type, "contract");
    }
}

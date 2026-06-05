use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::employee::Employee;

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeeProfile {
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
    pub id: Uuid,
    pub employee_id: Uuid,
    pub contact_name: String,
    pub relationship: String,
    pub phone: String,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeeDocument {
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
        let profile = EmployeeProfile {
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
        };

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
        let contact = EmployeeEmergencyContact {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            contact_name: "John Doe".to_string(),
            relationship: "Brother".to_string(),
            phone: "0987654321".to_string(),
            is_primary: true,
            created_at: Utc::now(),
        };

        let debug_str = format!("{contact:?}");
        assert!(debug_str.contains("John Doe"));
        assert!(!debug_str.contains("0987654321"));
        assert!(debug_str.contains("098****321"));
    }
}

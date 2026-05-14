use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::employee::Employee;

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeEmergencyContact {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub contact_name: String,
    pub relationship: String,
    pub phone: String,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateEmergencyContactRequest {
    pub contact_name: String,
    pub relationship: String,
    pub phone: String,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadDocumentRequest {
    pub document_type: String,
    pub expires_at: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyProfileResponse {
    pub employee: Employee,
    pub profile: Option<EmployeeProfile>,
    pub emergency_contacts: Vec<EmployeeEmergencyContact>,
}

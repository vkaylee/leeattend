use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct WorkspaceFile {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub file_name: String,
    pub storage_key: String,
    pub file_size: i64,
    pub mime_type: String,
    pub content_hash: Option<String>,
    pub category: FileCategory,
    pub folder_id: Option<Uuid>,
    pub owner_type: FileOwnerType,
    pub owner_id: Uuid,
    pub uploaded_by: Option<Uuid>,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub scan_status: ScanStatus,
    pub scanned_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub parent_file_id: Option<Uuid>,
    pub is_latest: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SystemFile {
    pub id: Uuid,
    pub file_name: String,
    pub storage_key: String,
    pub file_size: i64,
    pub mime_type: String,
    pub content_hash: Option<String>,
    pub category: FileCategory,
    pub owner_type: FileOwnerType,
    pub owner_id: Uuid,
    pub uploaded_by: Option<Uuid>,
    pub is_active: bool,
    pub scan_status: ScanStatus,
    pub version: i32,
    pub parent_file_id: Option<Uuid>,
    pub is_latest: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::AsRefStr,
    ToSchema,
)]
#[strum(serialize_all = "snake_case")]
pub enum FileCategory {
    EmployeeAvatar,
    WorkspaceLogo,
    AttendancePhoto,
    EmployeeDoc,
    Contract,
    LeaveAttachment,
    Payslip,
    ReportExport,
    OnboardingAttachment,
    ImportFile,
    Policy,
    TemplateOutput,
    UserAvatar,
    SystemAsset,
    AttendanceAdjustmentAttachment,
    OvertimeRequestAttachment,
    OtClaimAttachment,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::AsRefStr,
    ToSchema,
)]
#[strum(serialize_all = "snake_case")]
pub enum FileOwnerType {
    Employee,
    Employment,
    LeaveRequest,
    Workspace,
    User,
    System,
    Payroll,
    AttendanceAdjustment,
    OvertimeRequest,
    AttendanceDailySummary,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::AsRefStr,
    ToSchema,
)]
#[strum(serialize_all = "snake_case")]
pub enum ScanStatus {
    Pending,
    Clean,
    Infected,
    Skipped,
}

pub struct CategoryConfig {
    pub max_size: i64,
    pub allowed_mimes: &'static [&'static str],
}

impl FileCategory {
    pub fn config(&self) -> CategoryConfig {
        match self {
            Self::EmployeeAvatar | Self::UserAvatar => CategoryConfig {
                max_size: 5 * 1024 * 1024, // 5MB
                allowed_mimes: &["image/jpeg", "image/png", "image/webp"],
            },
            Self::WorkspaceLogo => CategoryConfig {
                max_size: 5 * 1024 * 1024, // 5MB
                allowed_mimes: &["image/jpeg", "image/png", "image/webp", "image/svg+xml"],
            },
            Self::AttendancePhoto => CategoryConfig {
                max_size: 10 * 1024 * 1024, // 10MB
                allowed_mimes: &["image/jpeg", "image/png", "image/webp"],
            },
            Self::EmployeeDoc | Self::OnboardingAttachment => CategoryConfig {
                max_size: 50 * 1024 * 1024, // 50MB
                allowed_mimes: &[
                    "application/pdf",
                    "image/jpeg",
                    "image/png",
                    "image/webp",
                    "application/msword",
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                ],
            },
            Self::Contract => CategoryConfig {
                max_size: 50 * 1024 * 1024, // 50MB
                allowed_mimes: &[
                    "application/pdf",
                    "application/msword",
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                ],
            },
            Self::LeaveAttachment
            | Self::AttendanceAdjustmentAttachment
            | Self::OvertimeRequestAttachment
            | Self::OtClaimAttachment => CategoryConfig {
                max_size: 20 * 1024 * 1024, // 20MB
                allowed_mimes: &["application/pdf"],
            },
            Self::Payslip => CategoryConfig {
                max_size: 10 * 1024 * 1024, // 10MB
                allowed_mimes: &["application/pdf"],
            },
            Self::ReportExport => CategoryConfig {
                max_size: 100 * 1024 * 1024, // 100MB
                allowed_mimes: &[
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    "text/csv",
                ],
            },
            Self::ImportFile => CategoryConfig {
                max_size: 20 * 1024 * 1024, // 20MB
                allowed_mimes: &[
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    "application/vnd.ms-excel",
                ],
            },
            Self::Policy => CategoryConfig {
                max_size: 50 * 1024 * 1024, // 50MB
                allowed_mimes: &["application/pdf"],
            },
            Self::TemplateOutput => CategoryConfig {
                max_size: 50 * 1024 * 1024, // 50MB
                allowed_mimes: &[
                    "application/pdf",
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                ],
            },
            Self::SystemAsset => CategoryConfig {
                max_size: 50 * 1024 * 1024, // 50MB
                allowed_mimes: &[
                    "image/jpeg",
                    "image/png",
                    "image/webp",
                    "image/svg+xml",
                    "application/pdf",
                ],
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UploadFileRequest {
    pub category: FileCategory,
    pub owner_type: FileOwnerType,
    pub owner_id: Uuid,
    pub expires_in_secs: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum FileDownloadResponse {
    ProxyBytes(Vec<u8>),
    PresignedUrl(String),
}

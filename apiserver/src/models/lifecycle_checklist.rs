use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::models::diesel::lifecycle_checklist::{
    LifecycleChecklistEntity, LifecycleChecklistItemEntity, LifecycleChecklistTemplateEntity,
    LifecycleChecklistTemplateItemEntity,
};

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleChecklistType {
    Onboarding,
    Offboarding,
    Transfer,
    ProbationEnd,
}

impl LifecycleChecklistType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Onboarding => "onboarding",
            Self::Offboarding => "offboarding",
            Self::Transfer => "transfer",
            Self::ProbationEnd => "probation_end",
        }
    }
}

impl FromStr for LifecycleChecklistType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "onboarding" => Ok(Self::Onboarding),
            "offboarding" => Ok(Self::Offboarding),
            "transfer" => Ok(Self::Transfer),
            "probation_end" => Ok(Self::ProbationEnd),
            _ => Err(format!("Unknown checklist type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChecklistCategory {
    HrAdmin,
    ItSetup,
    Documentation,
    Training,
    Equipment,
}

impl ChecklistCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HrAdmin => "hr_admin",
            Self::ItSetup => "it_setup",
            Self::Documentation => "documentation",
            Self::Training => "training",
            Self::Equipment => "equipment",
        }
    }
}

impl FromStr for ChecklistCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hr_admin" => Ok(Self::HrAdmin),
            "it_setup" => Ok(Self::ItSetup),
            "documentation" => Ok(Self::Documentation),
            "training" => Ok(Self::Training),
            "equipment" => Ok(Self::Equipment),
            _ => Err(format!("Unknown checklist category: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChecklistAssigneeType {
    HrManager,
    DirectManager,
    ItAdmin,
    Employee,
    SpecificUser,
}

impl ChecklistAssigneeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HrManager => "hr_manager",
            Self::DirectManager => "direct_manager",
            Self::ItAdmin => "it_admin",
            Self::Employee => "employee",
            Self::SpecificUser => "specific_user",
        }
    }
}

impl FromStr for ChecklistAssigneeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hr_manager" => Ok(Self::HrManager),
            "direct_manager" => Ok(Self::DirectManager),
            "it_admin" => Ok(Self::ItAdmin),
            "employee" => Ok(Self::Employee),
            "specific_user" => Ok(Self::SpecificUser),
            _ => Err(format!("Unknown assignee type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChecklistStatus {
    InProgress,
    Completed,
    Cancelled,
}

impl ChecklistStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl FromStr for ChecklistStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("Unknown checklist status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChecklistItemStatus {
    Pending,
    Completed,
    Skipped,
    Overdue,
}

impl ChecklistItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Skipped => "skipped",
            Self::Overdue => "overdue",
        }
    }
}

impl FromStr for ChecklistItemStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "completed" => Ok(Self::Completed),
            "skipped" => Ok(Self::Skipped),
            "overdue" => Ok(Self::Overdue),
            _ => Err(format!("Unknown item status: {s}")),
        }
    }
}

// --- Domain Models ---

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LifecycleChecklistTemplate {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    description: Option<String>,
    checklist_type: LifecycleChecklistType,
    is_default: bool,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl LifecycleChecklistTemplate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        checklist_type: LifecycleChecklistType,
        is_default: bool,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            name,
            description,
            checklist_type,
            is_default,
            is_active,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn checklist_type(&self) -> LifecycleChecklistType {
        self.checklist_type
    }
    pub fn is_default(&self) -> bool {
        self.is_default
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl From<LifecycleChecklistTemplateEntity> for LifecycleChecklistTemplate {
    fn from(entity: LifecycleChecklistTemplateEntity) -> Self {
        let checklist_type = LifecycleChecklistType::from_str(&entity.checklist_type)
            .unwrap_or(LifecycleChecklistType::Onboarding);
        Self::new(
            entity.id,
            entity.workspace_id,
            entity.name,
            entity.description,
            checklist_type,
            entity.is_default,
            entity.is_active,
            entity.created_at,
            entity.updated_at,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LifecycleChecklistTemplateItem {
    id: Uuid,
    template_id: Uuid,
    title: String,
    description: Option<String>,
    category: ChecklistCategory,
    assignee_type: ChecklistAssigneeType,
    assignee_user_id: Option<Uuid>,
    is_required: bool,
    due_days_offset: i32,
    display_order: i32,
}

impl LifecycleChecklistTemplateItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        template_id: Uuid,
        title: String,
        description: Option<String>,
        category: ChecklistCategory,
        assignee_type: ChecklistAssigneeType,
        assignee_user_id: Option<Uuid>,
        is_required: bool,
        due_days_offset: i32,
        display_order: i32,
    ) -> Self {
        Self {
            id,
            template_id,
            title,
            description,
            category,
            assignee_type,
            assignee_user_id,
            is_required,
            due_days_offset,
            display_order,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn template_id(&self) -> Uuid {
        self.template_id
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn category(&self) -> ChecklistCategory {
        self.category
    }
    pub fn assignee_type(&self) -> ChecklistAssigneeType {
        self.assignee_type
    }
    pub fn assignee_user_id(&self) -> Option<Uuid> {
        self.assignee_user_id
    }
    pub fn is_required(&self) -> bool {
        self.is_required
    }
    pub fn due_days_offset(&self) -> i32 {
        self.due_days_offset
    }
    pub fn display_order(&self) -> i32 {
        self.display_order
    }
}

impl From<LifecycleChecklistTemplateItemEntity> for LifecycleChecklistTemplateItem {
    fn from(entity: LifecycleChecklistTemplateItemEntity) -> Self {
        let category =
            ChecklistCategory::from_str(&entity.category).unwrap_or(ChecklistCategory::HrAdmin);
        let assignee_type = ChecklistAssigneeType::from_str(&entity.assignee_type)
            .unwrap_or(ChecklistAssigneeType::HrManager);
        Self::new(
            entity.id,
            entity.template_id,
            entity.title,
            entity.description,
            category,
            assignee_type,
            entity.assignee_user_id,
            entity.is_required,
            entity.due_days_offset,
            entity.display_order,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LifecycleChecklist {
    id: Uuid,
    workspace_id: Uuid,
    employment_id: Uuid,
    template_id: Option<Uuid>,
    checklist_type: LifecycleChecklistType,
    status: ChecklistStatus,
    progress_percent: i32,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl LifecycleChecklist {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        employment_id: Uuid,
        template_id: Option<Uuid>,
        checklist_type: LifecycleChecklistType,
        status: ChecklistStatus,
        progress_percent: i32,
        started_at: DateTime<Utc>,
        completed_at: Option<DateTime<Utc>>,
        created_by: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            employment_id,
            template_id,
            checklist_type,
            status,
            progress_percent,
            started_at,
            completed_at,
            created_by,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }
    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }
    pub fn template_id(&self) -> Option<Uuid> {
        self.template_id
    }
    pub fn checklist_type(&self) -> LifecycleChecklistType {
        self.checklist_type
    }
    pub fn status(&self) -> ChecklistStatus {
        self.status
    }
    pub fn progress_percent(&self) -> i32 {
        self.progress_percent
    }
    pub fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }
    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }
    pub fn created_by(&self) -> Option<Uuid> {
        self.created_by
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl From<LifecycleChecklistEntity> for LifecycleChecklist {
    fn from(entity: LifecycleChecklistEntity) -> Self {
        let checklist_type = LifecycleChecklistType::from_str(&entity.checklist_type)
            .unwrap_or(LifecycleChecklistType::Onboarding);
        let status =
            ChecklistStatus::from_str(&entity.status).unwrap_or(ChecklistStatus::InProgress);
        Self::new(
            entity.id,
            entity.workspace_id,
            entity.employment_id,
            entity.template_id,
            checklist_type,
            status,
            entity.progress_percent,
            entity.started_at,
            entity.completed_at,
            entity.created_by,
            entity.created_at,
            entity.updated_at,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LifecycleChecklistItem {
    id: Uuid,
    checklist_id: Uuid,
    template_item_id: Option<Uuid>,
    title: String,
    description: Option<String>,
    category: ChecklistCategory,
    assignee_user_id: Option<Uuid>,
    is_required: bool,
    due_date: Option<NaiveDate>,
    status: ChecklistItemStatus,
    completed_at: Option<DateTime<Utc>>,
    completed_by: Option<Uuid>,
    notes: Option<String>,
    display_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl LifecycleChecklistItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        checklist_id: Uuid,
        template_item_id: Option<Uuid>,
        title: String,
        description: Option<String>,
        category: ChecklistCategory,
        assignee_user_id: Option<Uuid>,
        is_required: bool,
        due_date: Option<NaiveDate>,
        status: ChecklistItemStatus,
        completed_at: Option<DateTime<Utc>>,
        completed_by: Option<Uuid>,
        notes: Option<String>,
        display_order: i32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            checklist_id,
            template_item_id,
            title,
            description,
            category,
            assignee_user_id,
            is_required,
            due_date,
            status,
            completed_at,
            completed_by,
            notes,
            display_order,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn checklist_id(&self) -> Uuid {
        self.checklist_id
    }
    pub fn template_item_id(&self) -> Option<Uuid> {
        self.template_item_id
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn category(&self) -> ChecklistCategory {
        self.category
    }
    pub fn assignee_user_id(&self) -> Option<Uuid> {
        self.assignee_user_id
    }
    pub fn is_required(&self) -> bool {
        self.is_required
    }
    pub fn due_date(&self) -> Option<NaiveDate> {
        self.due_date
    }
    pub fn status(&self) -> ChecklistItemStatus {
        self.status
    }
    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }
    pub fn completed_by(&self) -> Option<Uuid> {
        self.completed_by
    }
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
    pub fn display_order(&self) -> i32 {
        self.display_order
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl From<LifecycleChecklistItemEntity> for LifecycleChecklistItem {
    fn from(entity: LifecycleChecklistItemEntity) -> Self {
        let category =
            ChecklistCategory::from_str(&entity.category).unwrap_or(ChecklistCategory::HrAdmin);
        let status =
            ChecklistItemStatus::from_str(&entity.status).unwrap_or(ChecklistItemStatus::Pending);
        Self::new(
            entity.id,
            entity.checklist_id,
            entity.template_item_id,
            entity.title,
            entity.description,
            category,
            entity.assignee_user_id,
            entity.is_required,
            entity.due_date,
            status,
            entity.completed_at,
            entity.completed_by,
            entity.notes,
            entity.display_order,
            entity.created_at,
            entity.updated_at,
        )
    }
}

// --- Request/Response DTOs ---

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct CreateLifecycleChecklistTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub checklist_type: LifecycleChecklistType,
    pub is_default: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct UpdateLifecycleChecklistTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct CreateLifecycleChecklistTemplateItemRequest {
    pub title: String,
    pub description: Option<String>,
    pub category: ChecklistCategory,
    pub assignee_type: ChecklistAssigneeType,
    pub assignee_user_id: Option<Uuid>,
    pub is_required: bool,
    pub due_days_offset: i32,
    pub display_order: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct UpdateLifecycleChecklistTemplateItemRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<ChecklistCategory>,
    pub assignee_type: Option<ChecklistAssigneeType>,
    pub assignee_user_id: Option<Option<Uuid>>,
    pub is_required: Option<bool>,
    pub due_days_offset: Option<i32>,
    pub display_order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
pub struct UpdateChecklistItemStatusRequest {
    pub status: ChecklistItemStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LifecycleChecklistResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
    pub template_id: Option<Uuid>,
    pub checklist_type: LifecycleChecklistType,
    pub status: ChecklistStatus,
    pub progress_percent: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub items: Vec<LifecycleChecklistItem>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct PendingTaskResponse {
    pub id: Uuid,
    pub checklist_id: Uuid,
    pub employee_name: String,
    pub checklist_type: LifecycleChecklistType,
    pub title: String,
    pub description: Option<String>,
    pub category: ChecklistCategory,
    pub due_date: Option<NaiveDate>,
    pub is_required: bool,
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateChecklistRequest {
    pub template_id: Uuid,
}

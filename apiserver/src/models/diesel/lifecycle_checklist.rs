use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{
    lifecycle_checklist_items, lifecycle_checklist_template_items, lifecycle_checklist_templates,
    lifecycle_checklists,
};

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = lifecycle_checklist_templates)]
pub struct LifecycleChecklistTemplateEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub checklist_type: String,
    pub is_default: bool,
    pub is_active: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = lifecycle_checklist_template_items)]
pub struct LifecycleChecklistTemplateItemEntity {
    pub id: Uuid,
    pub template_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub assignee_type: String,
    pub assignee_user_id: Option<Uuid>,
    pub is_required: bool,
    pub due_days_offset: i32,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = lifecycle_checklists)]
pub struct LifecycleChecklistEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
    pub template_id: Option<Uuid>,
    pub checklist_type: String,
    pub status: String,
    pub progress_percent: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = lifecycle_checklist_items)]
pub struct LifecycleChecklistItemEntity {
    pub id: Uuid,
    pub checklist_id: Uuid,
    pub template_item_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub assignee_user_id: Option<Uuid>,
    pub is_required: bool,
    pub due_date: Option<NaiveDate>,
    pub status: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub completed_by: Option<Uuid>,
    pub notes: Option<String>,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

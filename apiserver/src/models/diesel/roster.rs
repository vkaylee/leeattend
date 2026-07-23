use crate::models::shift::{EmployeeShift, Shift, ShiftRoster};
use crate::models::shift_swap::{ShiftSwapRequest, ShiftSwapStatus};
use crate::schema::{
    employee_shifts, shift_audit_log, shift_default_intervals, shift_rosters, shift_swap_requests,
    shift_templates, shift_versions, shifts,
};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── New entities (Enterprise SCD Type-2) ───────────────────────────────────────

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = shift_templates)]
pub struct ShiftTemplateEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub current_version_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = shift_versions)]
pub struct ShiftVersionEntity {
    pub id: Uuid,
    pub template_id: Uuid,
    pub workspace_id: Uuid,
    pub version_number: i32,
    pub name: String,
    pub report_symbol: Option<String>,
    pub location_id: Option<Uuid>,
    pub employer_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub change_type: String,
    pub change_reason: Option<String>,
    pub superseded_by: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = shift_audit_log)]
pub struct ShiftAuditLogEntity {
    pub id: Uuid,
    pub template_id: Uuid,
    pub version_id: Uuid,
    pub action: String,
    pub change_type: String,
    pub changed_by: Uuid,
    pub change_reason: Option<String>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// ── Legacy entity (kept during transition, will be removed) ──────────────────────

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = shifts)]
#[allow(clippy::struct_excessive_bools)]
pub struct ShiftEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub parent_shift_id: Option<Uuid>,
    pub report_symbol: Option<String>,
    pub location_id: Option<Uuid>,
    pub employer_id: Option<Uuid>,
}

impl From<ShiftEntity> for Shift {
    fn from(entity: ShiftEntity) -> Self {
        Self::new(
            entity.id,
            entity.workspace_id,
            entity.name,
            entity.report_symbol,
            entity.is_active,
            entity.parent_shift_id,
            entity.location_id,
            entity.employer_id,
            entity.created_at.unwrap_or_else(Utc::now),
            entity.updated_at.unwrap_or_else(Utc::now),
        )
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = shift_rosters)]
pub struct ShiftRosterEntity {
    pub id: Uuid,
    pub template_id: Option<Uuid>,
    pub date: NaiveDate,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub employment_id: Uuid,
}

impl From<ShiftRosterEntity> for ShiftRoster {
    fn from(entity: ShiftRosterEntity) -> Self {
        Self {
            id: entity.id,
            employment_id: entity.employment_id,
            shift_id: entity.template_id,
            date: entity.date,
            status: entity.status.unwrap_or_else(|| "scheduled".to_string()),
            notes: entity.notes,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            updated_at: entity.updated_at.unwrap_or_else(Utc::now),
        }
    }
}

impl From<ShiftRoster> for ShiftRosterEntity {
    fn from(model: ShiftRoster) -> Self {
        Self {
            id: model.id,
            employment_id: model.employment_id,
            template_id: model.shift_id,
            date: model.date,
            status: Some(model.status),
            notes: model.notes,
            created_at: Some(model.created_at),
            updated_at: Some(model.updated_at),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = employee_shifts)]
pub struct EmployeeShiftEntity {
    pub id: Uuid,
    pub template_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub employment_id: Uuid,
}

impl From<EmployeeShiftEntity> for EmployeeShift {
    fn from(entity: EmployeeShiftEntity) -> Self {
        Self {
            id: entity.id,
            employment_id: entity.employment_id,
            shift_id: entity.template_id,
            start_date: entity.start_date,
            end_date: entity.end_date,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            updated_at: entity.updated_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = shift_swap_requests)]
pub struct ShiftSwapRequestEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub requester_id: Uuid,
    pub target_id: Uuid,
    pub swap_date: NaiveDate,
    pub requester_template_id: Option<Uuid>,
    pub target_template_id: Option<Uuid>,
    pub status: String,
    pub reason: Option<String>,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ShiftSwapRequestEntity> for ShiftSwapRequest {
    fn from(entity: ShiftSwapRequestEntity) -> Self {
        let status = match entity.status.as_str() {
            "pending_admin" => ShiftSwapStatus::PendingAdmin,
            "approved" => ShiftSwapStatus::Approved,
            "rejected" => ShiftSwapStatus::Rejected,
            "cancelled" => ShiftSwapStatus::Cancelled,
            _ => ShiftSwapStatus::PendingTarget,
        };
        ShiftSwapRequest::new(
            entity.id,
            entity.workspace_id,
            entity.requester_id,
            entity.target_id,
            entity.swap_date,
            entity.requester_template_id,
            entity.target_template_id,
            status,
            entity.reason,
            entity.requested_by_user_id,
            entity.approved_by_user_id,
            entity.created_at,
            entity.updated_at,
        )
    }
}
use crate::models::enums::AssignmentHistoryType;
use crate::models::enums::AssignmentHistoryTypeSql;
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, ToSql};
use std::io::Write;

impl FromSql<AssignmentHistoryTypeSql, Pg> for AssignmentHistoryType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"roster" => Ok(AssignmentHistoryType::Roster),
            b"pattern" => Ok(AssignmentHistoryType::Pattern),
            b"department_pattern" => Ok(AssignmentHistoryType::DepartmentPattern),
            b"fixed" => Ok(AssignmentHistoryType::Fixed),
            b"department_fixed" => Ok(AssignmentHistoryType::DepartmentFixed),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl ToSql<AssignmentHistoryTypeSql, Pg> for AssignmentHistoryType {
    fn to_sql(&self, out: &mut serialize::Output<'_, '_, Pg>) -> serialize::Result {
        match *self {
            AssignmentHistoryType::Roster => out.write_all(b"roster")?,
            AssignmentHistoryType::Pattern => out.write_all(b"pattern")?,
            AssignmentHistoryType::DepartmentPattern => out.write_all(b"department_pattern")?,
            AssignmentHistoryType::Fixed => out.write_all(b"fixed")?,
            AssignmentHistoryType::DepartmentFixed => out.write_all(b"department_fixed")?,
        }
        Ok(IsNull::No)
    }
}

/// Diesel entity for `shift_default_intervals`.
/// FK changed from `shift_id` (→ shifts.id) to `version_id` (→ shift_versions.id).
#[derive(
    Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize, utoipa::ToSchema,
)]
#[diesel(table_name = shift_default_intervals)]
#[allow(clippy::struct_excessive_bools)]
pub struct ShiftDefaultIntervalEntity {
    pub id: Uuid,
    pub version_id: Uuid,
    pub sort_order: i32,
    pub interval_name: Option<String>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub break_duration_minutes: i32,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: bool,
    pub break_is_overtime: bool,
    pub early_punch_in_grace_period_minutes: i32,
    pub allow_early_ot: bool,
    pub max_early_ot_minutes: Option<i32>,
    pub ot_aggregation_mode: String,
    pub is_night_shift: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

use crate::models::shift::{EmployeeShift, Shift, ShiftRoster};
use crate::models::shift_swap::{ShiftSwapRequest, ShiftSwapStatus};
use crate::schema::{employee_shifts, shift_rosters, shift_swap_requests, shifts};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = shifts)]
#[allow(clippy::struct_excessive_bools)]
pub struct ShiftEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub break_duration_minutes: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub break_is_overtime: bool,
    pub early_punch_in_grace_period_minutes: i32,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: bool,
    pub is_active: bool,
    pub parent_shift_id: Option<Uuid>,
    pub allow_early_ot: bool,
    pub report_symbol: Option<String>,
    pub max_early_ot_minutes: Option<i32>,
    pub location_id: Option<Uuid>,
    pub employer_id: Option<Uuid>,
}

impl From<ShiftEntity> for Shift {
    fn from(entity: ShiftEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            report_symbol: entity.report_symbol,
            start_time: entity.start_time,
            end_time: entity.end_time,
            break_duration_minutes: entity.break_duration_minutes,
            break_start: entity.break_start,
            break_end: entity.break_end,
            is_break_auto_deduct: entity.is_break_auto_deduct,
            break_is_overtime: entity.break_is_overtime,
            early_punch_in_grace_period_minutes: entity.early_punch_in_grace_period_minutes,
            allow_early_ot: entity.allow_early_ot,
            max_early_ot_minutes: entity.max_early_ot_minutes,
            is_active: entity.is_active,
            parent_shift_id: entity.parent_shift_id,
            location_id: entity.location_id,
            employer_id: entity.employer_id,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            updated_at: entity.updated_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = shift_rosters)]
pub struct ShiftRosterEntity {
    pub id: Uuid,
    pub shift_id: Option<Uuid>,
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
            shift_id: entity.shift_id,
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
            shift_id: model.shift_id,
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
    pub shift_id: Uuid,
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
            shift_id: entity.shift_id,
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
    pub requester_shift_id: Option<Uuid>,
    pub target_shift_id: Option<Uuid>,
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
            entity.requester_shift_id,
            entity.target_shift_id,
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

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;
use uuid::Uuid;

use crate::models::enums::LeaveStatus;
use crate::models::leave::*;
use crate::schema::sql_types::LeaveStatusType;
use crate::schema::*;

// ---------------------------------------------------------------------------
// Enum Mapping: LeaveStatusType
// ---------------------------------------------------------------------------

impl ToSql<LeaveStatusType, Pg> for LeaveStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            LeaveStatus::Pending => out.write_all(b"pending")?,
            LeaveStatus::Approved => out.write_all(b"approved")?,
            LeaveStatus::Rejected => out.write_all(b"rejected")?,
            LeaveStatus::Cancelled => out.write_all(b"cancelled")?,
            LeaveStatus::PreApproved => out.write_all(b"pre_approved")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<LeaveStatusType, Pg> for LeaveStatus {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"pending" => Ok(LeaveStatus::Pending),
            b"approved" => Ok(LeaveStatus::Approved),
            b"rejected" => Ok(LeaveStatus::Rejected),
            b"cancelled" => Ok(LeaveStatus::Cancelled),
            b"pre_approved" => Ok(LeaveStatus::PreApproved),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

// ---------------------------------------------------------------------------
// Leave Request
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = leave_requests)]
pub struct LeaveRequestEntity {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    pub status: LeaveStatus,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub leave_type_id: Uuid,
    pub duration_type: String,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
}

// ---------------------------------------------------------------------------
// Leave Type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = leave_types)]
pub struct LeaveTypeEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub code: String,
    pub name: String,
    pub is_paid: bool,
    pub created_at: DateTime<Utc>,
    pub policy_config: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Leave Balance
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = leave_balances)]
pub struct LeaveBalanceEntity {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub year: i32,
    pub entitlement: i32,
    pub carried_over: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub accrued_amount: BigDecimal,
    pub leave_type_id: Uuid,
}

// ---------------------------------------------------------------------------
// Leave Policy Definition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = leave_policy_definitions)]
pub struct LeavePolicyDefinitionEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rule_config: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub leave_type_id: Uuid,
}

// ---------------------------------------------------------------------------
// Leave Policy Assignment
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = leave_policy_assignments)]
pub struct LeavePolicyAssignmentEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub created_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Leave Accrual Log
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = leave_accrual_logs)]
pub struct LeaveAccrualLogEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub policy_assignment_id: Option<Uuid>,
    pub accrual_date: NaiveDate,
    pub amount: BigDecimal,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub leave_type_id: Uuid,
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

use std::str::FromStr;

fn bd_to_dec(bd: BigDecimal) -> rust_decimal::Decimal {
    rust_decimal::Decimal::from_str(&bd.to_string()).unwrap_or_default()
}

impl From<LeaveRequestEntity> for LeaveRequest {
    fn from(entity: LeaveRequestEntity) -> Self {
        Self {
            id: entity.id,
            employee_id: entity.employee_id,
            leave_type_id: entity.leave_type_id,
            start_date: entity.start_date,
            end_date: entity.end_date,
            reason: entity.reason,
            status: entity.status,
            duration_type: entity.duration_type.parse().unwrap_or_default(),
            start_time: entity.start_time,
            end_time: entity.end_time,
            requested_by_user_id: entity.requested_by_user_id,
            approved_by_user_id: entity.approved_by_user_id,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            updated_at: entity.updated_at.unwrap_or_else(Utc::now),
        }
    }
}

impl From<LeaveTypeEntity> for LeaveType {
    fn from(entity: LeaveTypeEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            code: entity.code,
            name: entity.name,
            is_paid: entity.is_paid,
            policy_config: sqlx::types::Json(
                serde_json::from_value(entity.policy_config).unwrap_or_default(),
            ),
            created_at: entity.created_at,
        }
    }
}

impl From<LeaveBalanceEntity> for LeaveBalance {
    fn from(entity: LeaveBalanceEntity) -> Self {
        Self {
            id: entity.id,
            employee_id: entity.employee_id,
            leave_type_id: entity.leave_type_id,
            year: entity.year,
            entitlement: entity.entitlement,
            accrued_amount: bd_to_dec(entity.accrued_amount),
            carried_over: entity.carried_over,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            updated_at: entity.updated_at.unwrap_or_else(Utc::now),
        }
    }
}

impl From<LeavePolicyDefinitionEntity> for LeavePolicyDefinition {
    fn from(entity: LeavePolicyDefinitionEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            description: entity.description,
            leave_type_id: entity.leave_type_id,
            rule_config: sqlx::types::Json(
                serde_json::from_value(entity.rule_config).unwrap_or_default(),
            ),
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            updated_at: entity.updated_at.unwrap_or_else(Utc::now),
        }
    }
}

impl From<LeavePolicyAssignmentEntity> for LeavePolicyAssignment {
    fn from(entity: LeavePolicyAssignmentEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            assignee_type: entity.assignee_type,
            assignee_id: entity.assignee_id,
            policy_id: entity.policy_id,
            effective_from: entity.effective_from,
            effective_to: entity.effective_to,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
        }
    }
}

impl From<LeaveAccrualLogEntity> for LeaveAccrualLog {
    fn from(entity: LeaveAccrualLogEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            employee_id: entity.employee_id,
            policy_assignment_id: entity.policy_assignment_id,
            accrual_date: entity.accrual_date,
            amount: bd_to_dec(entity.amount),
            leave_type_id: entity.leave_type_id,
            notes: entity.notes,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
        }
    }
}

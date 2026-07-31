use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use serde::{Deserialize, Serialize};
use std::io::Write;
use uuid::Uuid;

use crate::models::WorkSegment;

// ── SegmentType ──────────────────────────────────────────────────────────────

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum_macros::Display,
    strum_macros::EnumString,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = crate::schema::sql_types::SegmentType)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SegmentType {
    Work,
    Standby,
    OnCall,
    Training,
    Travel,
}

impl SegmentType {
    pub fn from_domain_str(s: &str) -> Self {
        match s {
            "work" => Self::Work,
            "standby" => Self::Standby,
            "on_call" => Self::OnCall,
            "training" => Self::Training,
            "travel" => Self::Travel,
            _ => Self::Work,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Work => "work",
            Self::Standby => "standby",
            Self::OnCall => "on_call",
            Self::Training => "training",
            Self::Travel => "travel",
        }
    }
}

impl ToSql<crate::schema::sql_types::SegmentType, Pg> for SegmentType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.as_str().as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::SegmentType, Pg> for SegmentType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"work" => Ok(Self::Work),
            b"standby" => Ok(Self::Standby),
            b"on_call" => Ok(Self::OnCall),
            b"training" => Ok(Self::Training),
            b"travel" => Ok(Self::Travel),
            _ => Err("Unrecognized SegmentType".into()),
        }
    }
}

// ── SegmentStatusType ────────────────────────────────────────────────────────

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum_macros::Display,
    strum_macros::EnumString,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = crate::schema::sql_types::SegmentStatusType)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SegmentStatusType {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SegmentStatusType {
    pub fn from_domain_str(s: &str) -> Self {
        match s {
            "scheduled" => Self::Scheduled,
            "in_progress" => Self::InProgress,
            "completed" => Self::Completed,
            "cancelled" => Self::Cancelled,
            _ => Self::Scheduled,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl ToSql<crate::schema::sql_types::SegmentStatusType, Pg> for SegmentStatusType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.as_str().as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::SegmentStatusType, Pg> for SegmentStatusType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"scheduled" => Ok(Self::Scheduled),
            b"in_progress" => Ok(Self::InProgress),
            b"completed" => Ok(Self::Completed),
            b"cancelled" => Ok(Self::Cancelled),
            _ => Err("Unrecognized SegmentStatusType".into()),
        }
    }
}

// ── GapCompensationType ──────────────────────────────────────────────────────

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum_macros::Display,
    strum_macros::EnumString,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = crate::schema::sql_types::GapCompensationType)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum GapCompensationType {
    Unpaid,
    Standby,
    Break,
}

impl GapCompensationType {
    pub fn from_domain_str(s: &str) -> Self {
        match s {
            "unpaid" => Self::Unpaid,
            "standby" => Self::Standby,
            "break" => Self::Break,
            _ => Self::Unpaid,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unpaid => "unpaid",
            Self::Standby => "standby",
            Self::Break => "break",
        }
    }
}

impl ToSql<crate::schema::sql_types::GapCompensationType, Pg> for GapCompensationType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.as_str().as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::GapCompensationType, Pg> for GapCompensationType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"unpaid" => Ok(Self::Unpaid),
            b"standby" => Ok(Self::Standby),
            b"break" => Ok(Self::Break),
            _ => Err("Unrecognized GapCompensationType".into()),
        }
    }
}

// ── WorkSegmentEntity ────────────────────────────────────────────────────────
// Pattern: follow EmployeeContractEntity — no per-field #[diesel(sql_type = ...)]
// because the schema table already defines column types.

#[derive(Debug, Clone, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::work_segments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkSegmentEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
    pub date: NaiveDate,
    pub template_id: Uuid,
    pub interval_id: Option<Uuid>,
    pub sort_order: i32,
    pub scheduled_start: NaiveTime,
    pub scheduled_end: NaiveTime,
    pub segment_type: SegmentType,
    pub segment_status: SegmentStatusType,
    pub compensation_rate: BigDecimal,
    pub gap_compensation_before: GapCompensationType,
    pub gap_compensation_rate_before: BigDecimal,
    pub break_config_override: Option<serde_json::Value>,
    pub ot_config_override: Option<serde_json::Value>,
    pub source: String,
    pub source_detail: serde_json::Value,
    pub notes: Option<String>,
    pub generation_metadata: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// ── NewWorkSegmentEntity (Insertable only) ───────────────────────────────────

#[derive(Debug, Clone, diesel::Insertable)]
#[diesel(table_name = crate::schema::work_segments)]
pub struct NewWorkSegmentEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
    pub date: NaiveDate,
    pub template_id: Uuid,
    pub interval_id: Option<Uuid>,
    pub sort_order: i32,
    pub scheduled_start: NaiveTime,
    pub scheduled_end: NaiveTime,
    pub segment_type: SegmentType,
    pub segment_status: SegmentStatusType,
    pub compensation_rate: BigDecimal,
    pub gap_compensation_before: GapCompensationType,
    pub gap_compensation_rate_before: BigDecimal,
    pub break_config_override: Option<serde_json::Value>,
    pub ot_config_override: Option<serde_json::Value>,
    pub source: String,
    pub source_detail: serde_json::Value,
    pub notes: Option<String>,
    pub generation_metadata: serde_json::Value,
}

// ── Conversions ──────────────────────────────────────────────────────────────

fn bigdecimal_to_f64(bd: &BigDecimal) -> f64 {
    bd.to_string().parse::<f64>().unwrap_or(0.0)
}

fn f64_to_bigdecimal(v: f64) -> BigDecimal {
    BigDecimal::try_from(v).unwrap_or_else(|_| BigDecimal::from(0))
}

impl From<WorkSegmentEntity> for WorkSegment {
    fn from(e: WorkSegmentEntity) -> Self {
        let now = Utc::now();
        WorkSegment {
            id: e.id,
            workspace_id: e.workspace_id,
            employment_id: e.employment_id,
            date: e.date,
            template_id: e.template_id,
            interval_id: e.interval_id,
            sort_order: e.sort_order,
            scheduled_start: e.scheduled_start,
            scheduled_end: e.scheduled_end,
            segment_type: e.segment_type.as_str().to_string(),
            segment_status: e.segment_status.as_str().to_string(),
            compensation_rate: bigdecimal_to_f64(&e.compensation_rate),
            gap_compensation_before: e.gap_compensation_before.as_str().to_string(),
            gap_compensation_rate_before: bigdecimal_to_f64(&e.gap_compensation_rate_before),
            break_config_override: e.break_config_override,
            ot_config_override: e.ot_config_override,
            source: e.source,
            source_detail: e.source_detail,
            notes: e.notes,
            generation_metadata: e.generation_metadata,
            created_at: e.created_at.unwrap_or(now),
            updated_at: e.updated_at.unwrap_or(now),
        }
    }
}

impl From<&crate::models::CreateWorkSegmentRequest> for NewWorkSegmentEntity {
    fn from(req: &crate::models::CreateWorkSegmentRequest) -> Self {
        NewWorkSegmentEntity {
            id: Uuid::new_v4(),
            workspace_id: Uuid::nil(), // filled by caller
            employment_id: req.employment_id,
            date: req.date,
            template_id: req.template_id,
            interval_id: req.interval_id,
            sort_order: req.sort_order,
            scheduled_start: req.scheduled_start,
            scheduled_end: req.scheduled_end,
            segment_type: SegmentType::from_domain_str(&req.segment_type),
            segment_status: SegmentStatusType::from_domain_str(&req.segment_status),
            compensation_rate: f64_to_bigdecimal(req.compensation_rate),
            gap_compensation_before: GapCompensationType::from_domain_str(
                &req.gap_compensation_before,
            ),
            gap_compensation_rate_before: f64_to_bigdecimal(
                req.gap_compensation_rate_before,
            ),
            break_config_override: req.break_config_override.clone(),
            ot_config_override: req.ot_config_override.clone(),
            source: req.source.clone(),
            source_detail: req.source_detail.clone(),
            notes: req.notes.clone(),
            generation_metadata: serde_json::json!({}),
        }
    }
}

/// Convert a `ResolvedSegment` into a `NewWorkSegmentEntity` for write-through persistence.
pub fn resolved_segment_to_new_entity(
    seg: &crate::models::ResolvedSegment,
    workspace_id: Uuid,
    employment_id: Uuid,
) -> NewWorkSegmentEntity {
    NewWorkSegmentEntity {
        id: Uuid::new_v4(),
        workspace_id,
        employment_id,
        date: seg.date.unwrap_or_default(),
        template_id: seg.template_id,
        interval_id: seg.interval_id,
        sort_order: seg.sort_order,
        scheduled_start: seg.scheduled_start,
        scheduled_end: seg.scheduled_end,
        segment_type: SegmentType::from_domain_str(&seg.segment_type),
        segment_status: SegmentStatusType::Scheduled,
        compensation_rate: BigDecimal::from(1),
        gap_compensation_before: GapCompensationType::Unpaid,
        gap_compensation_rate_before: BigDecimal::from(0),
        break_config_override: None,
        ot_config_override: None,
        source: seg.source.clone(),
        source_detail: serde_json::json!({
            "traces": seg.traces,
            "shift_name": seg.shift_name,
            "shift_symbol": seg.shift_symbol,
            "interval_name": seg.interval_name,
        }),
        notes: None,
        generation_metadata: serde_json::json!({}),
    }
}

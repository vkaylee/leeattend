use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkSegment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
    pub date: NaiveDate,
    pub shift_id: Uuid,
    pub interval_id: Option<Uuid>,
    pub sort_order: i32,
    pub scheduled_start: NaiveTime,
    pub scheduled_end: NaiveTime,
    pub segment_type: String, // 'work' | 'standby' | 'on_call' | 'training' | 'travel'
    pub segment_status: String, // 'scheduled' | 'in_progress' | 'completed' | 'cancelled'
    pub compensation_rate: f64,
    pub gap_compensation_before: String, // 'unpaid' | 'standby' | 'break'
    pub gap_compensation_rate_before: f64,
    pub break_config_override: Option<serde_json::Value>,
    pub ot_config_override: Option<serde_json::Value>,
    pub source: String, // 'roster' | 'pattern' | 'rule' | 'fixed' | 'manual' | 'auto'
    pub source_detail: serde_json::Value,
    pub notes: Option<String>,
    pub generation_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateWorkSegmentRequest {
    pub employment_id: Uuid,
    pub date: NaiveDate,
    pub shift_id: Uuid,
    #[serde(default)]
    pub interval_id: Option<Uuid>,
    #[serde(default)]
    pub sort_order: i32,
    pub scheduled_start: NaiveTime,
    pub scheduled_end: NaiveTime,
    #[serde(default = "default_segment_type")]
    pub segment_type: String,
    #[serde(default = "default_segment_status")]
    pub segment_status: String,
    #[serde(default = "default_rate_1")]
    pub compensation_rate: f64,
    #[serde(default = "default_gap_compensation")]
    pub gap_compensation_before: String,
    #[serde(default)]
    pub gap_compensation_rate_before: f64,
    #[serde(default)]
    pub break_config_override: Option<serde_json::Value>,
    #[serde(default)]
    pub ot_config_override: Option<serde_json::Value>,
    #[serde(default = "default_source_manual")]
    pub source: String,
    #[serde(default)]
    pub source_detail: serde_json::Value,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateWorkSegmentRequest {
    #[serde(default)]
    pub shift_id: Option<Uuid>,
    #[serde(default)]
    pub interval_id: Option<Uuid>,
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub scheduled_start: Option<NaiveTime>,
    #[serde(default)]
    pub scheduled_end: Option<NaiveTime>,
    #[serde(default)]
    pub segment_type: Option<String>,
    #[serde(default)]
    pub segment_status: Option<String>,
    #[serde(default)]
    pub compensation_rate: Option<f64>,
    #[serde(default)]
    pub gap_compensation_before: Option<String>,
    #[serde(default)]
    pub gap_compensation_rate_before: Option<f64>,
    #[serde(default)]
    pub break_config_override: Option<serde_json::Value>,
    #[serde(default)]
    pub ot_config_override: Option<serde_json::Value>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub source_detail: Option<serde_json::Value>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BulkCreateWorkSegmentsRequest {
    pub segments: Vec<CreateWorkSegmentRequest>,
    /// If true, replaces all existing segments for the given dates/employments
    #[serde(default)]
    pub replace_existing: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkSegmentListResponse {
    pub segments: Vec<WorkSegment>,
    pub total_count: i64,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct WorkSegmentFilter {
    pub employment_ids: Option<Vec<Uuid>>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub segment_type: Option<String>,
    pub segment_status: Option<String>,
    pub source: Option<String>,
}

// Default helpers
fn default_segment_type() -> String {
    "work".to_string()
}
fn default_segment_status() -> String {
    "scheduled".to_string()
}
fn default_rate_1() -> f64 {
    1.00
}
fn default_gap_compensation() -> String {
    "unpaid".to_string()
}
fn default_source_manual() -> String {
    "manual".to_string()
}

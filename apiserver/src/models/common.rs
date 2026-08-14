use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Maximum inclusive length (in days) of a custom analytics range.
pub const MAX_CUSTOM_RANGE_DAYS: i64 = 366;

/// Validate a custom date range for the operational dashboard endpoints.
///
/// Returns `Err(InvalidDateRange)` when `end < start`, `end > today`, or the
/// inclusive window exceeds [`MAX_CUSTOM_RANGE_DAYS`] — matching the executive
/// dashboard's `normalize()` semantics. Never silently falls back.
pub fn validate_custom_range(
    start: NaiveDate,
    end: NaiveDate,
    today: NaiveDate,
) -> Result<(), crate::errors::AppError> {
    if end < start
        || end > today
        || end.signed_duration_since(start).num_days() + 1 > MAX_CUSTOM_RANGE_DAYS
    {
        return Err(crate::errors::AppError::validation(
            crate::utils::i18n::I18nKey::InvalidDateRange,
        ));
    }
    Ok(())
}

/// A percentage rate together with the raw quantities it was derived from.
///
/// `value` is `None` (never a fabricated `0.0`) when `denominator <= 0`, mirroring the
/// executive dashboard contract's zero-denominator → not-applicable rule.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct RateQuantity {
    /// Percentage in `[0, 100]`, `None` when the denominator is zero.
    pub value: Option<rust_decimal::Decimal>,
    pub numerator: i64,
    pub denominator: i64,
    /// The unit the numerator/denominator are expressed in (e.g. `"employee_days"`, `"employees"`).
    pub unit: String,
}

impl RateQuantity {
    /// Build a rate from raw quantities. `value` is `None` when `denominator <= 0`.
    pub fn percent(numerator: i64, denominator: i64, unit: impl Into<String>) -> Self {
        let unit = unit.into();
        if denominator <= 0 {
            return Self {
                value: None,
                numerator: numerator.max(0),
                denominator,
                unit,
            };
        }
        let value = rust_decimal::Decimal::from(numerator.max(0))
            / rust_decimal::Decimal::from(denominator)
            * rust_decimal::Decimal::from(100);
        Self {
            value: Some(value),
            numerator: numerator.max(0),
            denominator,
            unit,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AttendanceTrendPoint {
    pub date: NaiveDate,
    pub present: i64,
    pub late: i64,
    pub absent: i64,
    pub on_leave: i64,
    /// Number of employees eligible to attend that day (denominator for rates).
    pub eligible: i64,
    pub attendance_rate: RateQuantity,
    pub absence_rate: RateQuantity,
    pub late_rate: RateQuantity,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct WorkspaceStats {
    pub total_employees: i64,
    pub present_today: i64,
    pub pending_leave_requests: i64,
    pub estimated_monthly_payroll: rust_decimal::Decimal,
    pub late_today: i64,
    pub absent_today: i64,
    pub on_leave_today: i64,
    pub early_departure_today: i64,
    pub avg_work_hours_today: f64,
    pub attendance_rate: RateQuantity,
    pub absence_rate: RateQuantity,
    pub late_rate: RateQuantity,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct DepartmentAttendancePoint {
    pub department_id: uuid::Uuid,
    pub department_name: String,
    pub on_time: i64,
    pub late: i64,
    pub absent: i64,
    pub on_leave: i64,
    pub total_employees: i64,
    pub attendance_rate: RateQuantity,
    pub absence_rate: RateQuantity,
    pub late_rate: RateQuantity,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UnexcusedAbsencesPoint {
    pub department_id: uuid::Uuid,
    pub department_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee_id: Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee_name: Option<String>,
    pub unexcused_count: i64,
    pub consecutive_count: i64,
    pub total_absent_count: i64,
    pub total_leave_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OvertimeTrendPoint {
    pub date: NaiveDate,
    pub system_ot_hours: f64,
    pub approved_ot_hours: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PeakTimeSlot {
    pub time_slot: String,
    pub check_in: i64,
    pub check_out: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LeaveTypeBreakdown {
    pub leave_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LatenessByDay {
    pub day: String,
    pub late_count: i64,
    pub avg_late_minutes: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EmployeeSearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, utoipa::IntoParams)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub q: Option<String>,
    pub year: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub department_id: Option<String>,
    pub permission: Option<String>,
    pub custom_field_filters: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GenericResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
    /// `Some(true)` when results are capped by a safety ceiling (e.g. computed-sort
    /// summary fetches at most 1 000 employees). `None` / absent in JSON otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_quantity_percent_computes_value() {
        let rate = RateQuantity::percent(1900, 2000, "employee_days");
        assert_eq!(rate.numerator, 1900);
        assert_eq!(rate.denominator, 2000);
        assert_eq!(rate.value, Some(rust_decimal::Decimal::new(95, 0)));
    }

    #[test]
    fn rate_quantity_zero_denominator_is_none_not_zero() {
        let rate = RateQuantity::percent(5, 0, "employees");
        assert_eq!(rate.value, None, "zero denominator must not fabricate 0%");
        assert_eq!(rate.numerator, 5);
        assert_eq!(rate.denominator, 0);
    }

    #[test]
    fn rate_quantity_clamps_negative_numerator() {
        let rate = RateQuantity::percent(-3, 10, "employee_days");
        assert_eq!(rate.numerator, 0);
        assert_eq!(rate.value, Some(rust_decimal::Decimal::new(0, 0)));
    }

    #[test]
    fn rate_quantity_serializes_value_as_decimal_string() {
        let rate = RateQuantity::percent(1, 3, "employees");
        let json = serde_json::to_value(&rate).unwrap();
        assert_eq!(json["numerator"], 1);
        assert_eq!(json["denominator"], 3);
        assert_eq!(json["unit"], "employees");
        // 1/3*100 → "33.33" (value is present, not null)
        assert!(
            json["value"].is_string(),
            "value should be a decimal string, got {}",
            json["value"]
        );
    }

    #[test]
    fn validate_custom_range_accepts_valid_window() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
        let start = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        assert!(validate_custom_range(start, today, today).is_ok());
    }

    #[test]
    fn validate_custom_range_rejects_end_before_start() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
        let start = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 8, 9).unwrap();
        assert!(validate_custom_range(start, end, today).is_err());
    }

    #[test]
    fn validate_custom_range_rejects_future_end() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
        let start = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
        assert!(validate_custom_range(start, end, today).is_err());
    }

    #[test]
    fn validate_custom_range_rejects_window_over_366_days() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
        let start = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();
        assert!(validate_custom_range(start, today, today).is_err());
    }

    #[test]
    fn validate_custom_range_accepts_exact_366_day_window() {
        // 2025-08-15..2026-08-14 = 364 days duration → 365 days inclusive → accepted.
        let today = NaiveDate::from_ymd_opt(2026, 8, 14).unwrap();
        let start_ok = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
        assert!(validate_custom_range(start_ok, today, today).is_ok());
        // 2025-08-14..2026-08-14 = 365 days duration → 366 days inclusive → the exact
        // maximum → accepted.
        let start_exact = NaiveDate::from_ymd_opt(2025, 8, 14).unwrap();
        assert!(validate_custom_range(start_exact, today, today).is_ok());
        // 2025-08-13..2026-08-14 = 366 days duration → 367 days inclusive → rejected.
        let start_over = NaiveDate::from_ymd_opt(2025, 8, 13).unwrap();
        assert!(validate_custom_range(start_over, today, today).is_err());
    }

    #[test]
    fn test_paginated_response_serialization_omits_truncated_none() {
        let resp = PaginatedResponse::<String> {
            data: vec!["a".into()],
            total: 1,
            page: 1,
            limit: 20,
            truncated: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(
            !json.contains("truncated"),
            "truncated: None should be absent from JSON, got: {json}"
        );
    }

    #[test]
    fn test_paginated_response_serialization_includes_truncated_true() {
        let resp = PaginatedResponse::<String> {
            data: vec!["a".into()],
            total: 1,
            page: 1,
            limit: 20,
            truncated: Some(true),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(
            json.contains("\"truncated\":true"),
            "truncated: true should appear in JSON, got: {json}"
        );
    }

    #[test]
    fn test_paginated_response_deserialization_defaults_truncated() {
        let json = r#"{"data":[],"total":0,"page":1,"limit":20}"#;
        let resp: PaginatedResponse<String> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.truncated, None);
    }

    #[test]
    fn test_paginated_response_deserialization_preserves_truncated_true() {
        let json = r#"{"data":[],"total":0,"page":1,"limit":20,"truncated":true}"#;
        let resp: PaginatedResponse<String> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.truncated, Some(true));
    }
}

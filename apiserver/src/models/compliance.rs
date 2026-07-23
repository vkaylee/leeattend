use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Compliance Rules — per-workspace thresholds
// ============================================================================

/// Domain-level compliance rule thresholds with sensible defaults.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ComplianceRules {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub min_rest_period_hours: i32,
    pub max_weekly_hours: i32,
    pub max_monthly_ot_hours: i32,
    pub allow_manager_override: bool,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for ComplianceRules {
    /// Returns defaults aligned with Vietnamese Labor Code:
    /// 12h rest, 48h/week, 40h/month OT, override enabled.
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            workspace_id: Uuid::nil(),
            min_rest_period_hours: 12,
            max_weekly_hours: 48,
            max_monthly_ot_hours: 40,
            allow_manager_override: true,
            is_enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

// ============================================================================
// Violation Types & Severities
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    /// Two shifts overlap in time on the same day
    Overlap,
    /// Insufficient rest period between consecutive shifts
    RestPeriod,
    /// Total weekly hours would exceed the cap
    WeeklyHours,
    /// Monthly overtime would exceed the cap
    MonthlyOvertime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    /// Hard violation — cannot be overridden (e.g., overlap)
    Hard,
    /// Soft violation — can be overridden with manager reason
    Soft,
}

// ============================================================================
// Violation Detail
// ============================================================================

/// Structured details for a single compliance violation.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ComplianceViolation {
    /// Type of violation detected
    pub violation_type: ViolationType,
    /// Hard (blocking) or Soft (warnable)
    pub severity: ViolationSeverity,
    /// Human-readable message (i18n key)
    pub message: String,
    /// Structured detail payload
    pub details: ComplianceViolationDetails,
}

/// Contextual data for a violation.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ComplianceViolationDetails {
    /// Employment being assigned
    pub employment_id: Uuid,
    /// Shift(s) involved in the violation
    pub shift_ids: Vec<Uuid>,
    /// Dates where the conflict occurs
    pub conflicting_dates: Vec<NaiveDate>,
    /// Current total (when applicable — weekly hours or monthly OT)
    pub total_hours: Option<f64>,
    /// The limit being exceeded
    pub limit_hours: Option<i32>,
}

// ============================================================================
// Request / Response DTOs
// ============================================================================

/// Request to check compliance for a set of shift assignments.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ComplianceCheckRequest {
    pub employment_id: Uuid,
    pub shift_id: Uuid,
    pub dates: Vec<NaiveDate>,
    /// If true, soft violations are treated as warnings only
    #[serde(default)]
    pub allow_warnings: bool,
    /// Manager-provided reason for overriding soft violations
    #[serde(default)]
    pub override_reason: Option<String>,
}

/// Response from a compliance check.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ComplianceCheckResponse {
    /// Whether the assignment is compliant
    pub compliant: bool,
    /// List of violations found (empty if compliant)
    pub violations: Vec<ComplianceViolation>,
    /// Whether an override reason is required to proceed
    pub requires_override: bool,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_rules_defaults() {
        let rules = ComplianceRules::default();
        assert_eq!(rules.min_rest_period_hours, 12);
        assert_eq!(rules.max_weekly_hours, 48);
        assert_eq!(rules.max_monthly_ot_hours, 40);
        assert!(rules.allow_manager_override);
        assert!(rules.is_enabled);
    }

    #[test]
    fn test_violation_type_serde() {
        assert_eq!(
            serde_json::to_string(&ViolationType::Overlap).unwrap(),
            r#""overlap""#
        );
        assert_eq!(
            serde_json::to_string(&ViolationType::RestPeriod).unwrap(),
            r#""rest_period""#
        );
        assert_eq!(
            serde_json::to_string(&ViolationSeverity::Hard).unwrap(),
            r#""hard""#
        );
        assert_eq!(
            serde_json::to_string(&ViolationSeverity::Soft).unwrap(),
            r#""soft""#
        );
    }

    #[test]
    fn test_compliance_violation_structure() {
        let employment_id = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 7, 20).unwrap();

        let details = ComplianceViolationDetails {
            employment_id,
            shift_ids: vec![shift_id],
            conflicting_dates: vec![date],
            total_hours: Some(52.0),
            limit_hours: Some(48),
        };

        let v = ComplianceViolation {
            violation_type: ViolationType::WeeklyHours,
            severity: ViolationSeverity::Soft,
            message: "Weekly hours exceeded".to_string(),
            details,
        };

        assert_eq!(v.violation_type, ViolationType::WeeklyHours);
        assert_eq!(v.severity, ViolationSeverity::Soft);
        assert_eq!(v.details.total_hours, Some(52.0));
        assert_eq!(v.details.limit_hours, Some(48));
    }

    #[test]
    fn test_compliance_check_response() {
        let resp = ComplianceCheckResponse {
            compliant: true,
            violations: vec![],
            requires_override: false,
        };
        assert!(resp.compliant);
        assert!(resp.violations.is_empty());
        assert!(!resp.requires_override);
    }
}

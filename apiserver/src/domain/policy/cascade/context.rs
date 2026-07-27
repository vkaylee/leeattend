//! Evaluation input — everything the cascade engine needs, pre-fetched.
//!
//! The caller (Phase 3 pairing service) loads rows from L2a → L3 tables
//! *before* calling `evaluate()`. This module is I/O-free: pure functions
//! only. See [[ADR-001]] for layer semantics.

use crate::domain::policy::cascade::base::BaseRules;
use crate::domain::policy::cascade::partial::PartialPolicyRules;
use uuid::Uuid;

/// One resolved L2.5 row, with the `priority_rank` needed to order it.
///
/// Multiple modifiers on the same employee cascade in ascending order of
/// `priority_rank` (higher rank runs first; the lowest rank runs last, so
/// it "wins" per ADR-001 semantics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmployeeModifier {
    pub modifier_id: Uuid,
    pub priority_rank: i32,
    pub contributions: PartialPolicyRules,
}

/// One resolved L3 row (rare — at most one exception per employee per date).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttendanceException {
    pub exception_id: Uuid,
    pub contributions: PartialPolicyRules,
}

/// Every input the cascade needs, pre-loaded.
///
/// `None` on any layer means "no contribution at this layer" — the engine
/// still emits an empty trace entry so the audit shape stays stable.
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub policy_version_id: Uuid,
    pub base_rules: BaseRules,

    pub day_type: Option<String>,
    pub day_type_contribution: Option<PartialPolicyRules>,

    pub shift_template_id: Option<Uuid>,
    pub shift_template_contribution: Option<PartialPolicyRules>,

    pub shift_version_id: Option<Uuid>,
    pub shift_version_contribution: Option<PartialPolicyRules>,

    pub employee_modifiers: Vec<EmployeeModifier>,
    pub exception: Option<AttendanceException>,
}

impl EvaluationContext {
    /// Sort modifiers so the cascade applies them from highest `priority_rank`
    /// down to lowest — lowest runs last and wins per-field conflicts.
    pub fn sort_modifiers(&mut self) {
        self.employee_modifiers
            .sort_by(|a, b| b.priority_rank.cmp(&a.priority_rank));
    }
}

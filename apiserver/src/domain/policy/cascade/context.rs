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
    modifier_id: Uuid,
    priority_rank: i32,
    contributions: PartialPolicyRules,
}

impl EmployeeModifier {
    #[must_use]
    pub fn new(modifier_id: Uuid, priority_rank: i32, contributions: PartialPolicyRules) -> Self {
        Self {
            modifier_id,
            priority_rank,
            contributions,
        }
    }

    pub fn modifier_id(&self) -> Uuid {
        self.modifier_id
    }
    pub fn priority_rank(&self) -> i32 {
        self.priority_rank
    }
    pub fn contributions(&self) -> &PartialPolicyRules {
        &self.contributions
    }

    /// Destructure by move for engine consumption.
    pub fn into_parts(self) -> (Uuid, i32, PartialPolicyRules) {
        (self.modifier_id, self.priority_rank, self.contributions)
    }
}

/// One resolved L3 row (rare — at most one exception per employee per date).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttendanceException {
    exception_id: Uuid,
    contributions: PartialPolicyRules,
}

impl AttendanceException {
    #[must_use]
    pub fn new(exception_id: Uuid, contributions: PartialPolicyRules) -> Self {
        Self {
            exception_id,
            contributions,
        }
    }

    #[must_use]
    pub fn exception_id(&self) -> Uuid {
        self.exception_id
    }
    pub fn contributions(&self) -> &PartialPolicyRules {
        &self.contributions
    }

    pub fn into_parts(self) -> (Uuid, PartialPolicyRules) {
        (self.exception_id, self.contributions)
    }
}

/// Every input the cascade needs, pre-loaded.
///
/// `None` on any layer means "no contribution at this layer" — the engine
/// still emits an empty trace entry so the audit shape stays stable.
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    policy_version_id: Uuid,
    base_rules: BaseRules,

    day_type: Option<String>,
    day_type_contribution: Option<PartialPolicyRules>,

    shift_template_id: Option<Uuid>,
    shift_template_contribution: Option<PartialPolicyRules>,

    shift_version_id: Option<Uuid>,
    shift_version_contribution: Option<PartialPolicyRules>,

    employee_modifiers: Vec<EmployeeModifier>,
    exception: Option<AttendanceException>,
}

impl EvaluationContext {
    #[must_use]
    pub fn new(policy_version_id: Uuid, base_rules: BaseRules) -> Self {
        Self {
            policy_version_id,
            base_rules,
            day_type: None,
            day_type_contribution: None,
            shift_template_id: None,
            shift_template_contribution: None,
            shift_version_id: None,
            shift_version_contribution: None,
            employee_modifiers: Vec::new(),
            exception: None,
        }
    }

    // Fluent builders.
    #[must_use]
    pub fn with_day_type(
        mut self,
        day_type: impl Into<String>,
        contribution: PartialPolicyRules,
    ) -> Self {
        self.day_type = Some(day_type.into());
        self.day_type_contribution = Some(contribution);
        self
    }
    #[must_use]
    pub fn with_shift_template(mut self, id: Uuid, contribution: PartialPolicyRules) -> Self {
        self.shift_template_id = Some(id);
        self.shift_template_contribution = Some(contribution);
        self
    }
    #[must_use]
    pub fn with_shift_version(mut self, id: Uuid, contribution: PartialPolicyRules) -> Self {
        self.shift_version_id = Some(id);
        self.shift_version_contribution = Some(contribution);
        self
    }
    #[must_use]
    pub fn push_modifier(mut self, modifier: EmployeeModifier) -> Self {
        self.employee_modifiers.push(modifier);
        self
    }
    #[must_use]
    pub fn with_exception(mut self, exception: AttendanceException) -> Self {
        self.exception = Some(exception);
        self
    }

    // Getters.
    pub fn policy_version_id(&self) -> Uuid {
        self.policy_version_id
    }
    pub fn base_rules(&self) -> &BaseRules {
        &self.base_rules
    }
    pub fn day_type(&self) -> Option<&str> {
        self.day_type.as_deref()
    }
    pub fn shift_template_id(&self) -> Option<Uuid> {
        self.shift_template_id
    }
    pub fn shift_version_id(&self) -> Option<Uuid> {
        self.shift_version_id
    }
    pub fn employee_modifiers(&self) -> &[EmployeeModifier] {
        &self.employee_modifiers
    }
    pub fn exception(&self) -> Option<&AttendanceException> {
        self.exception.as_ref()
    }

    /// Sort modifiers so the cascade applies them from highest `priority_rank`
    /// down to lowest — lowest runs last and wins per-field conflicts.
    pub fn sort_modifiers(&mut self) {
        self.employee_modifiers
            .sort_by(|a, b| b.priority_rank.cmp(&a.priority_rank));
    }

    /// Consume the context; hand every layer contribution to the engine.
    /// Ordering: `(policy_version_id, base_rules, day_type, l2a, shift_template_id,
    /// l2b, shift_version_id, l2c, modifiers, exception)`.
    #[allow(clippy::type_complexity)]
    pub fn into_parts(
        self,
    ) -> (
        Uuid,
        BaseRules,
        Option<String>,
        Option<PartialPolicyRules>,
        Option<Uuid>,
        Option<PartialPolicyRules>,
        Option<Uuid>,
        Option<PartialPolicyRules>,
        Vec<EmployeeModifier>,
        Option<AttendanceException>,
    ) {
        (
            self.policy_version_id,
            self.base_rules,
            self.day_type,
            self.day_type_contribution,
            self.shift_template_id,
            self.shift_template_contribution,
            self.shift_version_id,
            self.shift_version_contribution,
            self.employee_modifiers,
            self.exception,
        )
    }
}

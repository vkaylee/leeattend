//! Pure threshold evaluation for infrastructure monitoring alerts.
//!
//! This module deliberately has no database, Redis, scheduler, or handler dependencies.
//! A monitor snapshot can be evaluated in any caller without side effects.

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::fmt;

/// Canonical metrics understood by the monitor threshold catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MonitorMetric {
    QueueDepth,
    FailureRate,
    ActiveWorkers,
    StaleWorkers,
    LostJobs,
    DeadLetterJobs,
}

/// A value captured in a monitor snapshot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricValue {
    Number(f64),
    Boolean(bool),
}

impl MetricValue {
    fn number(self) -> Option<f64> {
        match self {
            Self::Number(value) if value.is_finite() => Some(value),
            Self::Number(_) | Self::Boolean(_) => None,
        }
    }
}

/// Immutable monitor input for one evaluation tick.
#[derive(Debug, Clone)]
pub struct MonitorSnapshot {
    observed_at: DateTime<Utc>,
    values: HashMap<MonitorMetric, MetricValue>,
}

impl MonitorSnapshot {
    pub fn new(observed_at: DateTime<Utc>) -> Self {
        Self {
            observed_at,
            values: HashMap::new(),
        }
    }

    pub fn observed_at(&self) -> DateTime<Utc> {
        self.observed_at
    }

    pub fn with_number(mut self, metric: MonitorMetric, value: f64) -> Self {
        self.values.insert(metric, MetricValue::Number(value));
        self
    }

    pub fn with_boolean(mut self, metric: MonitorMetric, value: bool) -> Self {
        self.values.insert(metric, MetricValue::Boolean(value));
        self
    }

    pub fn value(&self, metric: MonitorMetric) -> Option<MetricValue> {
        self.values.get(&metric).copied()
    }
}

/// Direction used for both breach and recovery comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl ThresholdOperator {
    fn matches(self, value: f64, threshold: f64) -> bool {
        match self {
            Self::GreaterThan => value > threshold,
            Self::GreaterThanOrEqual => value >= threshold,
            Self::LessThan => value < threshold,
            Self::LessThanOrEqual => value <= threshold,
        }
    }
}

/// A catalogued rule and its anti-flapping policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ThresholdRule {
    id: &'static str,
    metric: MonitorMetric,
    operator: ThresholdOperator,
    threshold: f64,
    recovery_threshold: f64,
    streak_required: u8,
    cooldown: Duration,
}

impl ThresholdRule {
    pub fn new(
        id: &'static str,
        metric: MonitorMetric,
        operator: ThresholdOperator,
        threshold: f64,
        recovery_threshold: f64,
    ) -> Result<Self, RuleConfigError> {
        if !threshold.is_finite() || !recovery_threshold.is_finite() {
            return Err(RuleConfigError::NonFiniteThreshold);
        }
        Ok(Self {
            id,
            metric,
            operator,
            threshold,
            recovery_threshold,
            streak_required: 3,
            cooldown: Duration::zero(),
        })
    }

    pub fn with_policy(mut self, streak_required: u8, cooldown: Duration) -> Result<Self, RuleConfigError> {
        if streak_required == 0 {
            return Err(RuleConfigError::InvalidStreak);
        }
        if cooldown < Duration::zero() {
            return Err(RuleConfigError::InvalidCooldown);
        }
        self.streak_required = streak_required;
        self.cooldown = cooldown;
        Ok(self)
    }

    pub fn id(&self) -> &'static str { self.id }
    pub fn metric(&self) -> MonitorMetric { self.metric }
    pub fn threshold(&self) -> f64 { self.threshold }
    pub fn recovery_threshold(&self) -> f64 { self.recovery_threshold }
    pub fn streak_required(&self) -> u8 { self.streak_required }
    pub fn cooldown(&self) -> Duration { self.cooldown }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleConfigError {
    NonFiniteThreshold,
    InvalidStreak,
    InvalidCooldown,
}

impl fmt::Display for RuleConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NonFiniteThreshold => "thresholds must be finite",
            Self::InvalidStreak => "streak must be greater than zero",
            Self::InvalidCooldown => "cooldown cannot be negative",
        };
        f.write_str(message)
    }
}

impl std::error::Error for RuleConfigError {}

/// Stable, versioned rule catalog. Thresholds are intentionally conservative defaults;
/// workspace-specific configuration can replace a rule without changing evaluator logic.
pub struct RuleCatalog;

impl RuleCatalog {
    pub const QUEUE_DEPTH_HIGH: &'static str = "queue_depth_high";
    pub const FAILURE_RATE_HIGH: &'static str = "failure_rate_high";
    pub const ACTIVE_WORKERS_LOW: &'static str = "active_workers_low";
    pub const STALE_WORKERS_HIGH: &'static str = "stale_workers_high";
    pub const LOST_JOBS_HIGH: &'static str = "lost_jobs_high";
    pub const DEAD_LETTER_JOBS_HIGH: &'static str = "dead_letter_jobs_high";

    pub fn defaults() -> Vec<ThresholdRule> {
        [
            (Self::QUEUE_DEPTH_HIGH, MonitorMetric::QueueDepth, 100.0, 80.0),
            (Self::FAILURE_RATE_HIGH, MonitorMetric::FailureRate, 0.10, 0.05),
            (Self::ACTIVE_WORKERS_LOW, MonitorMetric::ActiveWorkers, 1.0, 2.0),
            (Self::STALE_WORKERS_HIGH, MonitorMetric::StaleWorkers, 1.0, 0.0),
            (Self::LOST_JOBS_HIGH, MonitorMetric::LostJobs, 1.0, 0.0),
            (Self::DEAD_LETTER_JOBS_HIGH, MonitorMetric::DeadLetterJobs, 1.0, 0.0),
        ]
        .into_iter()
        .filter_map(|(id, metric, threshold, recovery)| {
            ThresholdRule::new(
                id,
                metric,
                match metric {
                    MonitorMetric::ActiveWorkers => ThresholdOperator::LessThan,
                    _ => ThresholdOperator::GreaterThanOrEqual,
                },
                threshold,
                recovery,
            )
            .ok()
        })
        .collect()
    }
}

/// Stable state that can be stored by a future alert repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleState {
    rule_id: &'static str,
    active: bool,
    breach_streak: u8,
    recovery_streak: u8,
    cooldown_until: Option<DateTime<Utc>>,
}

impl RuleState {
    pub fn new(rule_id: &'static str) -> Self {
        Self {
            rule_id,
            active: false,
            breach_streak: 0,
            recovery_streak: 0,
            cooldown_until: None,
        }
    }

    pub fn rule_id(&self) -> &'static str { self.rule_id }
    pub fn active(&self) -> bool { self.active }
    pub fn breach_streak(&self) -> u8 { self.breach_streak }
    pub fn recovery_streak(&self) -> u8 { self.recovery_streak }
    pub fn cooldown_until(&self) -> Option<DateTime<Utc>> { self.cooldown_until }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationStatus {
    Unknown,
    Pending,
    Suppressed,
    Fired,
    Active,
    Recovering,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    rule_id: &'static str,
    status: EvaluationStatus,
    state: RuleState,
}

impl Evaluation {
    pub fn rule_id(&self) -> &'static str { self.rule_id }
    pub fn status(&self) -> EvaluationStatus { self.status }
    pub fn state(&self) -> &RuleState { &self.state }
}

/// Stateful evaluator. State is per rule and can be exported to a repository after a tick.
#[derive(Debug, Default)]
pub struct ThresholdEvaluator {
    states: HashMap<&'static str, RuleState>,
}

impl ThresholdEvaluator {
    pub fn new() -> Self { Self::default() }

    pub fn state(&self, rule_id: &'static str) -> RuleState {
        self.states
            .get(rule_id)
            .cloned()
            .unwrap_or_else(|| RuleState::new(rule_id))
    }

    pub fn evaluate(&mut self, rule: &ThresholdRule, snapshot: &MonitorSnapshot) -> Evaluation {
        let mut state = self.state(rule.id());
        let Some(value) = snapshot.value(rule.metric()).and_then(MetricValue::number) else {
            return Evaluation { rule_id: rule.id(), status: EvaluationStatus::Unknown, state };
        };

        let breached = rule.operator().matches(value, rule.threshold());
        if state.active {
            if recovery_matches(rule, value) {
                state.recovery_streak = state.recovery_streak.saturating_add(1);
                state.breach_streak = 0;
                if state.recovery_streak >= rule.streak_required() {
                    state.active = false;
                    state.recovery_streak = 0;
                    state.cooldown_until = Some(snapshot.observed_at() + rule.cooldown());
                    self.states.insert(rule.id(), state.clone());
                    return Evaluation { rule_id: rule.id(), status: EvaluationStatus::Resolved, state };
                }
                self.states.insert(rule.id(), state.clone());
                return Evaluation { rule_id: rule.id(), status: EvaluationStatus::Recovering, state };
            }
            state.recovery_streak = 0;
            self.states.insert(rule.id(), state.clone());
            return Evaluation { rule_id: rule.id(), status: EvaluationStatus::Active, state };
        }

        state.recovery_streak = 0;
        if !breached {
            state.breach_streak = 0;
            self.states.insert(rule.id(), state.clone());
            return Evaluation { rule_id: rule.id(), status: EvaluationStatus::Pending, state };
        }

        state.breach_streak = state.breach_streak.saturating_add(1);
        if state.breach_streak < rule.streak_required() {
            self.states.insert(rule.id(), state.clone());
            return Evaluation { rule_id: rule.id(), status: EvaluationStatus::Pending, state };
        }

        if state.cooldown_until.is_some_and(|until| snapshot.observed_at() < until) {
            self.states.insert(rule.id(), state.clone());
            return Evaluation { rule_id: rule.id(), status: EvaluationStatus::Suppressed, state };
        }

        state.active = true;
        state.breach_streak = 0;
        self.states.insert(rule.id(), state.clone());
        Evaluation { rule_id: rule.id(), status: EvaluationStatus::Fired, state }
    }

    pub fn evaluate_all(&mut self, rules: &[ThresholdRule], snapshot: &MonitorSnapshot) -> Vec<Evaluation> {
        rules.iter().map(|rule| self.evaluate(rule, snapshot)).collect()
    }

}

impl ThresholdRule {
    fn operator(&self) -> ThresholdOperator { self.operator }
}

fn recovery_matches(rule: &ThresholdRule, value: f64) -> bool {
    match rule.operator() {
        ThresholdOperator::GreaterThan | ThresholdOperator::GreaterThanOrEqual => {
            value <= rule.recovery_threshold()
        }
        ThresholdOperator::LessThan | ThresholdOperator::LessThanOrEqual => {
            value >= rule.recovery_threshold()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule() -> ThresholdRule {
        ThresholdRule::new(
            "queue_depth_high",
            MonitorMetric::QueueDepth,
            ThresholdOperator::GreaterThanOrEqual,
            10.0,
            5.0,
        )
        .expect("valid rule")
        .with_policy(3, Duration::minutes(10))
        .expect("valid policy")
    }

    fn snapshot(at: i64, value: f64) -> MonitorSnapshot {
        MonitorSnapshot::new(DateTime::from_timestamp(at, 0).expect("valid timestamp"))
            .with_number(MonitorMetric::QueueDepth, value)
    }

    #[test]
    fn catalog_has_stable_rules_and_default_three_streak() {
        let rules = RuleCatalog::defaults();
        assert_eq!(rules.len(), 6);
        assert!(rules.iter().all(|rule| rule.streak_required() == 3));
    }

    #[test]
    fn missing_or_wrong_metric_is_unknown_and_does_not_advance_streak() {
        let mut evaluator = ThresholdEvaluator::new();
        let missing = evaluator.evaluate(&rule(), &MonitorSnapshot::new(Utc::now()));
        assert_eq!(missing.status(), EvaluationStatus::Unknown);
        assert_eq!(missing.state().breach_streak(), 0);

        let wrong_type = evaluator.evaluate(
            &rule(),
            &MonitorSnapshot::new(Utc::now()).with_boolean(MonitorMetric::QueueDepth, true),
        );
        assert_eq!(wrong_type.status(), EvaluationStatus::Unknown);
        assert_eq!(wrong_type.state().breach_streak(), 0);
    }

    #[test]
    fn fires_only_after_three_consecutive_breaches() {
        let mut evaluator = ThresholdEvaluator::new();
        assert_eq!(evaluator.evaluate(&rule(), &snapshot(1, 10.0)).status(), EvaluationStatus::Pending);
        assert_eq!(evaluator.evaluate(&rule(), &snapshot(2, 11.0)).status(), EvaluationStatus::Pending);
        let fired = evaluator.evaluate(&rule(), &snapshot(3, 12.0));
        assert_eq!(fired.status(), EvaluationStatus::Fired);
        assert!(fired.state().active());
    }

    #[test]
    fn breach_streak_resets_on_healthy_sample() {
        let mut evaluator = ThresholdEvaluator::new();
        evaluator.evaluate(&rule(), &snapshot(1, 10.0));
        evaluator.evaluate(&rule(), &snapshot(2, 11.0));
        let healthy = evaluator.evaluate(&rule(), &snapshot(3, 9.0));
        assert_eq!(healthy.status(), EvaluationStatus::Pending);
        assert_eq!(healthy.state().breach_streak(), 0);
    }

    #[test]
    fn hysteresis_requires_three_recovery_samples() {
        let mut evaluator = ThresholdEvaluator::new();
        for at in 1..=3 {
            evaluator.evaluate(&rule(), &snapshot(at, 10.0));
        }
        assert_eq!(evaluator.evaluate(&rule(), &snapshot(4, 5.5)).status(), EvaluationStatus::Active);
        assert_eq!(evaluator.evaluate(&rule(), &snapshot(5, 5.0)).status(), EvaluationStatus::Recovering);
        assert_eq!(evaluator.evaluate(&rule(), &snapshot(6, 4.0)).status(), EvaluationStatus::Recovering);
        let resolved = evaluator.evaluate(&rule(), &snapshot(7, 3.0));
        assert_eq!(resolved.status(), EvaluationStatus::Resolved);
        assert!(!resolved.state().active());
    }

    #[test]
    fn cooldown_suppresses_retrigger_after_resolution() {
        let mut evaluator = ThresholdEvaluator::new();
        for at in 1..=3 {
            evaluator.evaluate(&rule(), &snapshot(at, 10.0));
        }
        for at in 4..=6 {
            evaluator.evaluate(&rule(), &snapshot(at, 0.0));
        }
        assert_eq!(evaluator.evaluate(&rule(), &snapshot(7, 10.0)).status(), EvaluationStatus::Pending);
        assert_eq!(evaluator.evaluate(&rule(), &snapshot(8, 10.0)).status(), EvaluationStatus::Pending);
        let suppressed = evaluator.evaluate(&rule(), &snapshot(9, 10.0));
        assert_eq!(suppressed.status(), EvaluationStatus::Suppressed);
        assert!(!suppressed.state().active());

        let fired = evaluator.evaluate(&rule(), &snapshot(607, 10.0));
        assert_eq!(fired.status(), EvaluationStatus::Fired);
    }

    #[test]
    fn lower_than_rule_uses_inverse_hysteresis() {
        let rule = ThresholdRule::new(
            "active_workers_low",
            MonitorMetric::ActiveWorkers,
            ThresholdOperator::LessThan,
            2.0,
            3.0,
        )
        .expect("valid rule");
        let mut evaluator = ThresholdEvaluator::new();
        for at in 1..=3 {
            evaluator.evaluate(&rule, &MonitorSnapshot::new(DateTime::from_timestamp(at, 0).expect("timestamp")).with_number(MonitorMetric::ActiveWorkers, 1.0));
        }
        let resolved = evaluator.evaluate(&rule, &MonitorSnapshot::new(DateTime::from_timestamp(4, 0).expect("timestamp")).with_number(MonitorMetric::ActiveWorkers, 3.0));
        assert_eq!(resolved.status(), EvaluationStatus::Recovering);
    }
}

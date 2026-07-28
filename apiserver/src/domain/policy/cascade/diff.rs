//! Dual-run diff taxonomy per [[ADR-005]].
//!
//! When cascade engine output differs from legacy resolver output, the diff is
//! classified into one of three groups so operations can distinguish known bug
//! fixes from regressions without manual triage over every row.
//!
//! ## Diff groups
//!
//! - **Group A — Expected fix:** cascade engine intentionally fixes a known
//!   legacy bug.  These diffs are documented in
//!   `.attendagent/migration_expected_diffs.md` and are considered "good" diffs.
//! - **Group B — Unexpected new-engine bug:** cascade engine produces a result
//!   that does NOT match the policy's semantic intent → **must block cutover**.
//! - **Group C — Regression:** cascade result differs from legacy and legacy was
//!   correct → **must block cutover**.
//!
//! ## Magnitude buckets
//!
//! Per-field absolute minute deltas mapped to labelled buckets for aggregation.

use crate::models::PolicyRules;
use serde::Serialize;

/// Which ADR-005 diff group an observed delta belongs to.
///
/// The `serde_repr` serialization makes this a compact string in JSON logs
/// (`"expected_fix"`, `"unexpected_bug"`, `"regression"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DiffGroup {
    #[serde(rename = "expected_fix")]
    ExpectedFix,
    #[serde(rename = "unexpected_bug")]
    UnexpectedBug,
    #[serde(rename = "regression")]
    Regression,
}

/// Magnitude bucket for a single numeric field delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MagnitudeBucket {
    #[serde(rename = "0-5")]
    Minutes0to5,
    #[serde(rename = "5-15")]
    Minutes5to15,
    #[serde(rename = "15-plus")]
    Minutes15Plus,
}

impl MagnitudeBucket {
    /// Classify `delta_minutes` into one of the three magnitude buckets.
    #[must_use]
    pub fn classify(delta_minutes: i32) -> Self {
        let abs_min = delta_minutes.abs();
        if abs_min <= 5 {
            Self::Minutes0to5
        } else if abs_min <= 15 {
            Self::Minutes5to15
        } else {
            Self::Minutes15Plus
        }
    }
}

/// Structured event emitted when cascade final_rules ≠ legacy rules.
///
/// Designed so observability pipelines (OpenTelemetry, journald, log
/// aggregators) can consume every field as a well-typed attribute without
/// parsing unstructured message strings.
#[derive(Debug, Clone, Serialize)]
pub struct CascadeDiffEvent {
    /// Cascade final_rules — the new-engine output.
    pub cascade_rules: PolicyRules,
    /// Legacy resolver output — the old-engine values.
    pub legacy_rules: PolicyRules,
    /// Per-field delta classification and magnitude.
    pub diffs: Vec<FieldDiff>,
}

/// One field-level delta between cascade and legacy.
#[derive(Debug, Clone, Serialize)]
pub struct FieldDiff {
    pub field_name: &'static str,
    pub cascade_value: i32,
    pub legacy_value: i32,
    pub delta_minutes: i32,
    pub magnitude: MagnitudeBucket,
    /// Initial classification is always `ExpectedFix` — operators should
    /// reclassify to `UnexpectedBug` or `Regression` during shadow-run review
    /// if the delta is not in the expected-diffs manifest.
    pub group: DiffGroup,
}

impl CascadeDiffEvent {
    /// Build a diff event from the two rule-sets.
    ///
    /// Only fields whose numeric values differ are included. The initial
    /// classification for every diff is `ExpectedFix` — operators are
    /// responsible for reclassifying during the shadow-run review period.
    #[must_use]
    pub fn new(cascade: &PolicyRules, legacy: &PolicyRules) -> Self {
        let mut diffs: Vec<FieldDiff> = Vec::with_capacity(2);

        let grace_cascade = cascade.grace_period_minutes();
        let grace_legacy = legacy.grace_period_minutes();
        if grace_cascade != grace_legacy {
            diffs.push(FieldDiff {
                field_name: "grace_period_minutes",
                cascade_value: grace_cascade,
                legacy_value: grace_legacy,
                delta_minutes: grace_cascade - grace_legacy,
                magnitude: MagnitudeBucket::classify(grace_cascade - grace_legacy),
                group: DiffGroup::ExpectedFix,
            });
        }

        let ael_cascade = cascade.authorized_early_leave_minutes();
        let ael_legacy = legacy.authorized_early_leave_minutes();
        if ael_cascade != ael_legacy {
            diffs.push(FieldDiff {
                field_name: "authorized_early_leave_minutes",
                cascade_value: ael_cascade,
                legacy_value: ael_legacy,
                delta_minutes: ael_cascade - ael_legacy,
                magnitude: MagnitudeBucket::classify(ael_cascade - ael_legacy),
                group: DiffGroup::ExpectedFix,
            });
        }

        Self {
            cascade_rules: cascade.clone(),
            legacy_rules: legacy.clone(),
            diffs,
        }
    }

    /// Human-readable single-line summary of all field deltas.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.diffs.is_empty() {
            return "cascade == legacy (no numeric diff)".into();
        }
        let parts: Vec<String> = self
            .diffs
            .iter()
            .map(|d| {
                format!(
                    "{}: legacy={} cascade={} (Δ{} min) [{:?}]",
                    d.field_name,
                    d.legacy_value,
                    d.cascade_value,
                    d.delta_minutes,
                    d.magnitude
                )
            })
            .collect();
        parts.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::EarlyArrivalStrategy;

    fn rules(grace: i32, ael: i32) -> PolicyRules {
        PolicyRules::new(grace, ael, true, EarlyArrivalStrategy::Allow, true, false, false)
    }

    #[test]
    fn empty_diff_when_identical() {
        let event = CascadeDiffEvent::new(&rules(15, 30), &rules(15, 30));
        assert!(event.diffs.is_empty());
    }

    #[test]
    fn diff_detects_grace_delta() {
        let event = CascadeDiffEvent::new(&rules(20, 30), &rules(15, 30));
        assert_eq!(event.diffs.len(), 1);
        assert_eq!(event.diffs[0].field_name, "grace_period_minutes");
        assert_eq!(event.diffs[0].cascade_value, 20);
        assert_eq!(event.diffs[0].legacy_value, 15);
        assert_eq!(event.diffs[0].delta_minutes, 5);
        assert!(matches!(
            event.diffs[0].magnitude,
            MagnitudeBucket::Minutes0to5
        ));
    }

    #[test]
    fn diff_detects_both_fields() {
        let event = CascadeDiffEvent::new(&rules(25, 10), &rules(15, 0));
        assert_eq!(event.diffs.len(), 2);
    }

    #[test]
    fn magnitude_bucket_15_plus() {
        assert!(matches!(
            MagnitudeBucket::classify(20),
            MagnitudeBucket::Minutes15Plus
        ));
        assert!(matches!(
            MagnitudeBucket::classify(-30),
            MagnitudeBucket::Minutes15Plus
        ));
    }

    #[test]
    fn magnitude_bucket_5_to_15() {
        // edge: delta=6 → bucket 5-15
        assert!(matches!(
            MagnitudeBucket::classify(6),
            MagnitudeBucket::Minutes5to15
        ));
        assert!(matches!(
            MagnitudeBucket::classify(15),
            MagnitudeBucket::Minutes5to15
        ));
    }

    #[test]
    fn magnitude_bucket_0_to_5() {
        assert!(matches!(
            MagnitudeBucket::classify(0),
            MagnitudeBucket::Minutes0to5
        ));
        assert!(matches!(
            MagnitudeBucket::classify(5),
            MagnitudeBucket::Minutes0to5
        ));
    }

    #[test]
    fn event_serializes_compactly() {
        let event = CascadeDiffEvent::new(&rules(20, 30), &rules(15, 30));
        let json = serde_json::to_value(&event).unwrap();
        assert!(json.get("cascade_rules").is_some());
        assert!(json.get("legacy_rules").is_some());
        let diffs = json.get("diffs").unwrap().as_array().unwrap();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0]["field_name"], "grace_period_minutes");
        assert_eq!(diffs[0]["magnitude"], "0-5");
        assert_eq!(diffs[0]["group"], "expected_fix");
    }
}

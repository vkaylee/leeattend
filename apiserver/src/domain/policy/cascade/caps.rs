//! Base-policy field spec + clamp step (ADR-002 absolute caps).
//!
//! The L1 base policy owns the *ceiling* on every numeric field. Once the
//! full cascade has run and every layer merged, the accumulator is clamped
//! into `[absolute_min, absolute_max]` and any clamp is recorded as a
//! `ClampWarning` on the cascade trace ([[ADR-004]]).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaseFieldSpec<T> {
    default_value: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    absolute_min: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    absolute_max: Option<T>,
}

impl<T: Copy> BaseFieldSpec<T> {
    pub fn new(default_value: T, absolute_min: Option<T>, absolute_max: Option<T>) -> Self {
        Self {
            default_value,
            absolute_min,
            absolute_max,
        }
    }

    pub fn default_value(&self) -> T {
        self.default_value
    }
    pub fn absolute_min(&self) -> Option<T> {
        self.absolute_min
    }
    pub fn absolute_max(&self) -> Option<T> {
        self.absolute_max
    }
}

impl Default for BaseFieldSpec<i32> {
    fn default() -> Self {
        Self {
            default_value: 0,
            absolute_min: None,
            absolute_max: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClampWarning {
    field: String,
    before: i64,
    after: i64,
    reason: ClampReason,
}

impl ClampWarning {
    pub fn new(field: impl Into<String>, before: i64, after: i64, reason: ClampReason) -> Self {
        Self {
            field: field.into(),
            before,
            after,
            reason,
        }
    }

    pub fn field(&self) -> &str {
        &self.field
    }
    pub fn before(&self) -> i64 {
        self.before
    }
    pub fn after(&self) -> i64 {
        self.after
    }
    pub fn reason(&self) -> ClampReason {
        self.reason
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClampReason {
    BelowMin,
    AboveMax,
}

/// Clamp `value` to `[spec.absolute_min, spec.absolute_max]`. Returns the
/// clamped value and a `ClampWarning` when the value was moved.
pub fn clamp_i32(
    field: &str,
    value: i32,
    spec: &BaseFieldSpec<i32>,
) -> (i32, Option<ClampWarning>) {
    if let Some(min) = spec.absolute_min() {
        if value < min {
            return (
                min,
                Some(ClampWarning::new(
                    field,
                    i64::from(value),
                    i64::from(min),
                    ClampReason::BelowMin,
                )),
            );
        }
    }
    if let Some(max) = spec.absolute_max() {
        if value > max {
            return (
                max,
                Some(ClampWarning::new(
                    field,
                    i64::from(value),
                    i64::from(max),
                    ClampReason::AboveMax,
                )),
            );
        }
    }
    (value, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(min: Option<i32>, max: Option<i32>) -> BaseFieldSpec<i32> {
        BaseFieldSpec::new(0, min, max)
    }

    #[test]
    fn value_within_range_is_unchanged() {
        let (v, w) = clamp_i32("grace", 30, &spec(Some(0), Some(120)));
        assert_eq!(v, 30);
        assert!(w.is_none());
    }

    #[test]
    fn value_below_min_is_clamped_up_and_warned() {
        let (v, w) = clamp_i32("grace", -5, &spec(Some(0), Some(120)));
        assert_eq!(v, 0);
        let w = w.unwrap();
        assert_eq!(w.field(), "grace");
        assert_eq!(w.before(), -5);
        assert_eq!(w.after(), 0);
        assert_eq!(w.reason(), ClampReason::BelowMin);
    }

    #[test]
    fn value_above_max_is_clamped_down_and_warned() {
        let (v, w) = clamp_i32("grace", 999, &spec(Some(0), Some(120)));
        assert_eq!(v, 120);
        let w = w.unwrap();
        assert_eq!(w.after(), 120);
        assert_eq!(w.reason(), ClampReason::AboveMax);
    }

    #[test]
    fn no_bounds_means_no_clamp() {
        let (v, w) = clamp_i32("free", 9_999, &spec(None, None));
        assert_eq!(v, 9_999);
        assert!(w.is_none());
    }

    #[test]
    fn only_min_bound_leaves_upper_open() {
        let (v, w) = clamp_i32("f", 5_000, &spec(Some(0), None));
        assert_eq!(v, 5_000);
        assert!(w.is_none());
    }
}

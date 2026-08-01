use crate::models::{EarlyArrivalStrategy, PolicyRules, RoundingPolicy};
use chrono::{DateTime, Duration, TimeZone, Timelike};
use uuid::Uuid;

#[derive(Debug, Default, Clone)]
pub struct WorkBuckets {
    pub standard_minutes: i32,
    pub early_ot_minutes: i32,
    pub late_ot_minutes: i32,
    pub break_ot_minutes: i32,
}

/// Aggregation mode for OT calculation across multiple segments.
///
/// Persisted as lowercase snake_case (`"aggregate"`, `"per_segment"`, `"daily"`).
/// The DB stores three legacy values; both `aggregate` and `daily` collapse to
/// `Aggregate` here — the historical distinction was cosmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OtAggregationMode {
    #[default]
    Aggregate,
    PerSegment,
}

impl std::fmt::Display for OtAggregationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Aggregate => write!(f, "aggregate"),
            Self::PerSegment => write!(f, "per_segment"),
        }
    }
}

impl std::str::FromStr for OtAggregationMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "per_segment" => Ok(Self::PerSegment),
            "aggregate" | "daily" | "" => Ok(Self::Aggregate),
            _ => Err(()),
        }
    }
}

impl<'de> serde::Deserialize<'de> for OtAggregationMode {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(s.parse().unwrap_or_default())
    }
}

impl OtAggregationMode {
    /// Kept for legacy call-sites; prefer `str::parse` or `FromStr`.
    #[must_use]
    pub fn from_mode(s: &str) -> Self {
        s.parse().unwrap_or_default()
    }
}

/// Result of calculating units across multiple segments.
#[derive(Debug, Default)]
pub struct SegmentCalcResult {
    pub total_standard_units: i32,
    pub total_ot_units: i32,
    pub per_segment_buckets: Vec<(Option<Uuid>, WorkBuckets)>,
}

/// Encapsulates all inputs for break deduction calculation.
///
/// Replaces the 8-parameter signature of `calculate_break_deduction`,
/// improving readability at call sites.
pub struct BreakDeductionInput<'a, T: TimeZone> {
    pub actual_break_minutes: i32,
    pub scheduled_break_minutes: i32,
    pub is_break_auto_deduct: bool,
    pub is_break_overtime: bool,
    pub break_start: Option<&'a DateTime<T>>,
    pub break_end: Option<&'a DateTime<T>>,
    pub punch_in: &'a DateTime<T>,
    pub punch_out: &'a DateTime<T>,
}

/// Encapsulates all inputs for daily attendance calculation.
///
/// Replaces the 10-parameter function signatures in `calculate_daily_units`
/// and `calculate_buckets`, improving readability at call sites.
#[derive(Debug, Clone)]
pub struct DailyCalcInput<T: TimeZone> {
    pub punch_in: DateTime<T>,
    pub punch_out: DateTime<T>,
    pub shift_start: Option<DateTime<T>>,
    pub shift_end: Option<DateTime<T>>,
    pub scheduled_break_minutes: i32,
    pub actual_break_minutes: i32,
    /// Break frame start (Fixed mode only). When both break_start and break_end
    /// are present, the calculator uses Frame Intersection (overlap algorithm)
    /// instead of flat deduction for auto-deduct mode.
    pub break_start: Option<DateTime<T>>,
    /// Break frame end (Fixed mode only).
    pub break_end: Option<DateTime<T>>,
    pub work_unit: i32,
    pub rounding_policy: RoundingPolicy,
    pub rules: PolicyRules,
    pub ot_ceiling: Option<DateTime<T>>,
    pub max_early_ot_minutes: Option<i32>,
    /// Links this calculation to a specific work segment. None for legacy entries.
    pub segment_id: Option<Uuid>,
    /// How OT is aggregated when multiple segments exist in a day.
    pub ot_aggregation_mode: OtAggregationMode,
}

pub struct AttendanceCalculator;

impl AttendanceCalculator {
    fn truncate_to_minute<T: TimeZone>(dt: DateTime<T>) -> DateTime<T> {
        dt.with_second(0)
            .and_then(|d| d.with_nanosecond(0))
            .unwrap_or(dt)
    }

    /// Calculate standard and OT buckets using structured input.
    pub fn calculate_buckets<T: TimeZone>(input: &DailyCalcInput<T>) -> WorkBuckets {
        let input = DailyCalcInput {
            punch_in: Self::truncate_to_minute(input.punch_in.clone()),
            punch_out: Self::truncate_to_minute(input.punch_out.clone()),
            shift_start: input
                .shift_start
                .as_ref()
                .map(|dt| Self::truncate_to_minute(dt.clone())),
            shift_end: input
                .shift_end
                .as_ref()
                .map(|dt| Self::truncate_to_minute(dt.clone())),
            scheduled_break_minutes: input.scheduled_break_minutes,
            actual_break_minutes: input.actual_break_minutes,
            break_start: input
                .break_start
                .as_ref()
                .map(|dt| Self::truncate_to_minute(dt.clone())),
            break_end: input
                .break_end
                .as_ref()
                .map(|dt| Self::truncate_to_minute(dt.clone())),
            work_unit: input.work_unit,
            rounding_policy: input.rounding_policy,
            rules: input.rules.clone(),
            ot_ceiling: input
                .ot_ceiling
                .as_ref()
                .map(|dt| Self::truncate_to_minute(dt.clone())),
            max_early_ot_minutes: input.max_early_ot_minutes,
            segment_id: input.segment_id,
            ot_aggregation_mode: input.ot_aggregation_mode,
        };

        let punch_in = &input.punch_in;
        let punch_out = &input.punch_out;

        let mut buckets = WorkBuckets::default();

        if punch_out <= punch_in {
            return buckets;
        }

        let Some(start) = input.shift_start.clone() else {
            // No shift: everything is standard
            buckets.standard_minutes = Self::raw_duration_minutes(punch_in, punch_out);
            return buckets;
        };
        let Some(shift_end_dt) = input.shift_end.clone() else {
            buckets.standard_minutes = Self::raw_duration_minutes(punch_in, punch_out);
            return buckets;
        };

        // Handle Overnight Shift linearization
        let end = if shift_end_dt < start {
            shift_end_dt + Duration::days(1)
        } else {
            shift_end_dt
        };

        let effective_ot_start = Self::effective_ot_start(punch_in, &start, &input.rules);

        Self::fill_standard_bucket(&input, &start, &end, &mut buckets);
        Self::fill_early_ot(&input, &start, &effective_ot_start, &mut buckets);
        Self::fill_late_ot(&input, &end, &mut buckets);

        buckets
    }

    /// Compute the effective OT start time after applying grace period rules.
    fn effective_ot_start<T: TimeZone>(
        punch_in: &DateTime<T>,
        shift_start: &DateTime<T>,
        rules: &PolicyRules,
    ) -> DateTime<T> {
        if rules.grace_period_minutes() <= 0 {
            return punch_in.clone();
        }

        let grace_threshold =
            shift_start.clone() - Duration::minutes(i64::from(rules.grace_period_minutes()));

        match rules.early_arrival_strategy() {
            EarlyArrivalStrategy::Block | EarlyArrivalStrategy::Allow => {
                if punch_in >= &grace_threshold && punch_in < shift_start {
                    shift_start.clone()
                } else {
                    punch_in.clone()
                }
            }
            EarlyArrivalStrategy::Clamp => DateTime::max(punch_in.clone(), grace_threshold),
        }
    }

    /// Compute the standard work bucket (intersection of punch window and shift window).
    fn fill_standard_bucket<T: TimeZone>(
        input: &DailyCalcInput<T>,
        shift_start: &DateTime<T>,
        shift_end: &DateTime<T>,
        buckets: &mut WorkBuckets,
    ) {
        let punch_in = &input.punch_in;
        let punch_out = &input.punch_out;
        let rules = &input.rules;

        let standard_in = DateTime::max(punch_in.clone(), shift_start.clone());
        let mut standard_out = DateTime::min(punch_out.clone(), shift_end.clone());

        // Apply Authorized Early Leave (Paid)
        if rules.is_early_leave_paid() && rules.authorized_early_leave_minutes() > 0 {
            let missing_minutes = shift_end
                .clone()
                .signed_duration_since(standard_out.clone())
                .num_minutes();
            if missing_minutes > 0
                && missing_minutes <= i64::from(rules.authorized_early_leave_minutes())
            {
                standard_out = shift_end.clone();
            }
        }

        if standard_out <= standard_in {
            return;
        }

        let mut standard_raw = i32::try_from(
            standard_out
                .clone()
                .signed_duration_since(standard_in.clone())
                .num_minutes(),
        )
        .unwrap_or(0);

        // Calculate break deduction and break OT
        let (break_to_deduct, break_ot_minutes) = if rules.is_break_overtime() {
            let break_overlap = if let (Some(bs), Some(be)) =
                (input.break_start.clone(), input.break_end.clone())
            {
                let overlap_start = DateTime::max(standard_in, bs);
                let overlap_end = DateTime::min(standard_out, be);
                if overlap_end > overlap_start {
                    i32::try_from(
                        overlap_end
                            .signed_duration_since(overlap_start)
                            .num_minutes(),
                    )
                    .unwrap_or(0)
                } else {
                    0
                }
            } else {
                input.scheduled_break_minutes
            };
            (
                i32::max(input.actual_break_minutes, break_overlap),
                i32::max(0, break_overlap - input.actual_break_minutes),
            )
        } else {
            let deduct = Self::calculate_break_deduction(&BreakDeductionInput {
                actual_break_minutes: input.actual_break_minutes,
                scheduled_break_minutes: input.scheduled_break_minutes,
                is_break_auto_deduct: rules.is_break_auto_deduct(),
                is_break_overtime: rules.is_break_overtime(),
                break_start: input.break_start.as_ref(),
                break_end: input.break_end.as_ref(),
                punch_in,
                punch_out,
            });
            (deduct, 0)
        };

        standard_raw = i32::max(0, standard_raw - break_to_deduct);
        buckets.standard_minutes = standard_raw;
        buckets.break_ot_minutes = break_ot_minutes;
    }

    /// Compute early OT (time worked before shift start).
    fn fill_early_ot<T: TimeZone>(
        input: &DailyCalcInput<T>,
        shift_start: &DateTime<T>,
        effective_ot_start: &DateTime<T>,
        buckets: &mut WorkBuckets,
    ) {
        if !input.rules.allow_early_ot() || &input.punch_in >= shift_start {
            return;
        }

        let early_out = DateTime::min(input.punch_out.clone(), shift_start.clone());
        if early_out <= *effective_ot_start {
            return;
        }

        let mut capped = i32::try_from(
            early_out
                .signed_duration_since(effective_ot_start.clone())
                .num_minutes(),
        )
        .unwrap_or(0);
        if let Some(max_ot) = input.max_early_ot_minutes {
            capped = i32::min(capped, max_ot);
        }
        buckets.early_ot_minutes = capped;
    }

    /// Compute late OT (time worked after shift end, capped at ot_ceiling).
    fn fill_late_ot<T: TimeZone>(
        input: &DailyCalcInput<T>,
        shift_end: &DateTime<T>,
        buckets: &mut WorkBuckets,
    ) {
        if &input.punch_out <= shift_end {
            return;
        }

        let effective_punch_out = match &input.ot_ceiling {
            Some(ceil) => DateTime::min(input.punch_out.clone(), ceil.clone()),
            None => input.punch_out.clone(),
        };
        let late_in = DateTime::max(input.punch_in.clone(), shift_end.clone());
        if effective_punch_out > late_in {
            buckets.late_ot_minutes = i32::try_from(
                effective_punch_out
                    .signed_duration_since(late_in)
                    .num_minutes(),
            )
            .unwrap_or(0);
        }
    }

    /// Helper: compute raw duration in minutes between two timestamps.
    fn raw_duration_minutes<T: TimeZone>(from: &DateTime<T>, to: &DateTime<T>) -> i32 {
        i32::max(
            0,
            i32::try_from(to.clone().signed_duration_since(from.clone()).num_minutes())
                .unwrap_or(0),
        )
    }

    /// Break Deduction Logic
    ///
    /// Supports two modes:
    /// - **Flat deduction** (Flexible): subtracts `scheduled_break_minutes` wholesale.
    /// - **Frame Intersection** (Fixed): when `break_start`/`break_end` are provided,
    ///   computes the actual overlap between `[punch_in, punch_out]` and
    ///   `[break_start, break_end]`. This prevents over-deduction when an employee's
    ///   working period only partially overlaps with the configured break window.
    pub fn calculate_break_deduction<T: TimeZone>(input: &BreakDeductionInput<'_, T>) -> i32 {
        // 1. If there's actual break time recorded (4-punch mode), use it directly
        if input.actual_break_minutes > 0 {
            return input.actual_break_minutes;
        }

        // 2. No auto-deduct or break is OT → no deduction
        if !input.is_break_auto_deduct || input.is_break_overtime {
            return 0;
        }

        // 3. Frame Intersection: when break_start/break_end are provided (Fixed mode),
        //    calculate overlap between [punch_in, punch_out] ∩ [break_start, break_end]
        if let (Some(bs), Some(be)) = (input.break_start, input.break_end) {
            let overlap_start = DateTime::max(input.punch_in.clone(), bs.clone());
            let overlap_end = DateTime::min(input.punch_out.clone(), be.clone());
            let overlap_mins = overlap_end
                .signed_duration_since(overlap_start)
                .num_minutes();
            return i32::max(0, i32::try_from(overlap_mins).unwrap_or(0));
        }

        // 4. Fallback: flat deduction (Flexible mode — no break frame configured)
        input.scheduled_break_minutes
    }

    /// Unit Calculation Logic
    #[allow(clippy::cast_possible_truncation)]
    pub fn calculate_units(minutes: i32, unit_size: i32, policy: RoundingPolicy) -> i32 {
        let size = if unit_size <= 1 {
            1.0
        } else {
            f64::from(unit_size)
        };
        let mins = f64::from(minutes);

        match policy {
            RoundingPolicy::Floor => (mins / size).floor() as i32,
            RoundingPolicy::Ceil => (mins / size).ceil() as i32,
            RoundingPolicy::Round => (mins / size).round() as i32,
            RoundingPolicy::None => (mins / size) as i32,
        }
    }

    /// High-level method to calculate daily standard and OT units in one go.
    /// `ot_ceiling` caps late OT at the next shift's start (for split shifts).
    pub fn calculate_daily_units<T: TimeZone>(input: &DailyCalcInput<T>) -> (i32, i32) {
        let buckets = Self::calculate_buckets(input);

        let std_units = Self::calculate_units(
            buckets.standard_minutes,
            input.work_unit,
            input.rounding_policy,
        );
        let ot_units = Self::calculate_units(
            buckets.early_ot_minutes + buckets.late_ot_minutes + buckets.break_ot_minutes,
            input.work_unit,
            input.rounding_policy,
        );

        (std_units, ot_units)
    }

    /// Calculate units across multiple segments with configurable OT aggregation.
    ///
    /// - `Aggregate`: Sums all standard minutes across segments, then applies the
    ///   OT threshold once against the total. This is the default behavior matching
    ///   "1 ca/ngày" expectations where OT is computed on daily totals.
    /// - `PerSegment`: Computes OT independently for each segment. Late OT from
    ///   segment N is capped at segment N+1's start (via ot_ceiling). Each segment's
    ///   OT is summed to produce the daily total.
    pub fn calculate_segment_units<T: TimeZone>(inputs: &[DailyCalcInput<T>]) -> SegmentCalcResult {
        if inputs.is_empty() {
            return SegmentCalcResult::default();
        }

        // Determine mode from first input (all segments in a day share the same mode)
        let mode = &inputs[0].ot_aggregation_mode;
        let mut per_segment = Vec::with_capacity(inputs.len());

        match mode {
            OtAggregationMode::Aggregate => {
                // Sum all standard minutes and all OT minutes across segments
                let mut total_std_minutes = 0i32;
                let mut total_ot_minutes = 0i32;

                for input in inputs {
                    let buckets = Self::calculate_buckets(input);
                    total_std_minutes += buckets.standard_minutes;
                    total_ot_minutes += buckets.early_ot_minutes
                        + buckets.late_ot_minutes
                        + buckets.break_ot_minutes;
                    per_segment.push((input.segment_id, buckets));
                }

                let std_units = Self::calculate_units(
                    total_std_minutes,
                    inputs[0].work_unit,
                    inputs[0].rounding_policy,
                );
                let ot_units = Self::calculate_units(
                    total_ot_minutes,
                    inputs[0].work_unit,
                    inputs[0].rounding_policy,
                );

                SegmentCalcResult {
                    total_standard_units: std_units,
                    total_ot_units: ot_units,
                    per_segment_buckets: per_segment,
                }
            }
            OtAggregationMode::PerSegment => {
                let mut total_std_units = 0i32;
                let mut total_ot_units = 0i32;

                for input in inputs {
                    let (std_u, ot_u) = Self::calculate_daily_units(input);
                    total_std_units += std_u;
                    total_ot_units += ot_u;
                    let buckets = Self::calculate_buckets(input);
                    per_segment.push((input.segment_id, buckets));
                }

                SegmentCalcResult {
                    total_standard_units: total_std_units,
                    total_ot_units,
                    per_segment_buckets: per_segment,
                }
            }
        }
    }
}

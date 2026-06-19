use crate::models::{AccrualFrequency, LeavePolicyConfig};
use chrono::{Datelike, NaiveDate};

/// Default qualifying work days when threshold checking is disabled.
/// This value ensures `calculate_accrual_amount` always passes the
/// threshold check when no policy threshold is configured.
pub const DEFAULT_QUALIFYING_DAYS_BYPASS: f64 = 20.0;

impl super::LeaveAccrualService {
    /// Pure function to calculate how much leave should be accrued for a given policy and employee state.
    pub fn calculate_accrual_amount(
        config: &LeavePolicyConfig,
        months_service: i32,
        work_days_in_period: f64,
    ) -> f64 {
        // Check working days threshold if configured
        if let Some(threshold) = config.min_work_days_threshold {
            if work_days_in_period < threshold {
                return 0.0;
            }
        }

        let mut amount = config.accrual_amount;

        // Apply seniority bonus (highest milestone reached)
        let mut bonus_to_apply = 0.0;
        let mut max_milestone_months = -1;

        for bonus in &config.seniority_bonus {
            if months_service >= bonus.months && bonus.months > max_milestone_months {
                bonus_to_apply = bonus.bonus_amount;
                max_milestone_months = bonus.months;
            }
        }
        amount += bonus_to_apply;

        amount
    }

    /// Calculate how many days can be carried over to the next year
    pub fn calculate_carry_over_amount(remaining_days: f64, carry_over_limit: i32) -> f64 {
        let limit = f64::from(carry_over_limit);
        if remaining_days > limit {
            limit
        } else if remaining_days < 0.0 {
            0.0
        } else {
            remaining_days
        }
    }

    /// Helper to calculate full months of service across multiple contracts
    pub fn calculate_total_months_seniority(
        contracts: &[crate::models::contract::EmployeeContract],
        as_of: NaiveDate,
    ) -> i32 {
        use crate::models::enums::ContractStatus;

        let mut valid_intervals = Vec::new();

        for c in contracts {
            if c.status() == ContractStatus::Draft {
                continue;
            }

            if c.start_date() > as_of {
                continue;
            }

            let end_date = c.end_date().unwrap_or(as_of);
            let end_date = if end_date > as_of { as_of } else { end_date };

            if end_date >= c.start_date() {
                valid_intervals.push((c.start_date(), end_date));
            }
        }

        if valid_intervals.is_empty() {
            return 0;
        }

        valid_intervals.sort_by_key(|i| i.0);

        let mut total_days = 0;
        let mut current_start = valid_intervals[0].0;
        let mut current_end = valid_intervals[0].1;

        for interval in valid_intervals.into_iter().skip(1) {
            let threshold = current_end.succ_opt().unwrap_or(current_end);
            if interval.0 <= threshold {
                if interval.1 > current_end {
                    current_end = interval.1;
                }
            } else {
                total_days += (current_end - current_start).num_days() + 1;
                current_start = interval.0;
                current_end = interval.1;
            }
        }
        total_days += (current_end - current_start).num_days() + 1;

        i32::try_from(total_days / 30).unwrap_or(0) // Approximate month
    }

    /// Determines if an accrual should run on this date based on frequency.
    pub fn should_run_accrual(frequency: &AccrualFrequency, timing: &str, date: NaiveDate) -> bool {
        let is_start = timing == "start_of_period";

        match frequency {
            AccrualFrequency::Monthly => {
                if is_start {
                    date.day() == 1
                } else {
                    let next_day = date.succ_opt().unwrap_or(date);
                    next_day.month() != date.month()
                }
            }
            AccrualFrequency::Quarterly => {
                let is_quarter_month = matches!(date.month(), 1 | 4 | 7 | 10);
                if is_start {
                    date.day() == 1 && is_quarter_month
                } else {
                    let next_day = date.succ_opt().unwrap_or(date);
                    next_day.day() == 1 && matches!(next_day.month(), 4 | 7 | 10 | 1)
                }
            }
            AccrualFrequency::Yearly => {
                if is_start {
                    date.day() == 1 && date.month() == 1
                } else {
                    date.day() == 31 && date.month() == 12
                }
            }
        }
    }

    /// Pre-compute all accrual dates in a range instead of iterating day-by-day.
    pub fn get_accrual_dates(
        frequency: &AccrualFrequency,
        timing: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let is_start = timing == "start_of_period";

        match frequency {
            AccrualFrequency::Monthly => {
                let mut y = from.year();
                let mut m = from.month();
                if is_start {
                    while let Some(d) = NaiveDate::from_ymd_opt(y, m, 1) {
                        if d > to {
                            break;
                        }
                        if d >= from {
                            dates.push(d);
                        }
                        if m == 12 {
                            m = 1;
                            y += 1;
                        } else {
                            m += 1;
                        }
                    }
                } else {
                    loop {
                        let next_m = if m == 12 { 1 } else { m + 1 };
                        let next_y = if m == 12 { y + 1 } else { y };
                        if let Some(first_next) = NaiveDate::from_ymd_opt(next_y, next_m, 1) {
                            if let Some(last_day) = first_next.pred_opt() {
                                if last_day > to {
                                    break;
                                }
                                if last_day >= from {
                                    dates.push(last_day);
                                }
                            }
                        }
                        m = next_m;
                        y = next_y;
                        if NaiveDate::from_ymd_opt(y, m, 1).is_none_or(|d| d > to) {
                            break;
                        }
                    }
                }
            }
            AccrualFrequency::Quarterly => {
                let quarter_months: &[u32] = if is_start {
                    &[1, 4, 7, 10]
                } else {
                    &[3, 6, 9, 12]
                };
                let mut y = from.year();
                loop {
                    for &qm in quarter_months {
                        let d = if is_start {
                            NaiveDate::from_ymd_opt(y, qm, 1)
                        } else {
                            let next_m = if qm == 12 { 1 } else { qm + 1 };
                            let next_y = if qm == 12 { y + 1 } else { y };
                            NaiveDate::from_ymd_opt(next_y, next_m, 1).and_then(|d| d.pred_opt())
                        };
                        if let Some(d) = d {
                            if d > to {
                                return dates;
                            }
                            if d >= from {
                                dates.push(d);
                            }
                        }
                    }
                    y += 1;
                    if y > to.year() + 1 {
                        break;
                    }
                }
            }
            AccrualFrequency::Yearly => {
                let mut y = from.year();
                loop {
                    let d = if is_start {
                        NaiveDate::from_ymd_opt(y, 1, 1)
                    } else {
                        NaiveDate::from_ymd_opt(y, 12, 31)
                    };
                    if let Some(d) = d {
                        if d > to {
                            break;
                        }
                        if d >= from {
                            dates.push(d);
                        }
                    }
                    y += 1;
                    if y > to.year() + 1 {
                        break;
                    }
                }
            }
        }
        dates
    }
}

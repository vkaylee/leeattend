use crate::models::{
    AssignPatternRequest, BulkAssignPatternRequest, BulkAssignResponse,
    CreateWorkingPatternRequest, EmployeeWorkingPattern, WorkingPattern, WorkingPatternDetail,
};
use crate::repositories::working_pattern_repository as repo;
use chrono::{NaiveTime, Timelike};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Check whether two time ranges overlap.
/// Handles overnight shifts (end < start) by treating the range as wrapping past midnight.
fn times_overlap(s1: NaiveTime, e1: NaiveTime, s2: NaiveTime, e2: NaiveTime) -> bool {
    // Compare as seconds-from-midnight.
    // Overnight shifts (e.g., 22:00–06:00) are expanded to a wider range.
    let to_range = |s: NaiveTime, e: NaiveTime| -> (i64, i64) {
        let sm = s.num_seconds_from_midnight() as i64;
        let em = e.num_seconds_from_midnight() as i64;
        if em > sm {
            (sm, em)
        } else {
            // Overnight: treat end as next day
            (sm, em + 86400)
        }
    };
    let (a_start, a_end) = to_range(s1, e1);
    let (b_start, b_end) = to_range(s2, e2);

    // Check overlap in original position
    let overlaps = a_start < b_end && b_start < a_end;
    // Also check with b shifted by +24h (handles overnight A vs early-morning B)
    let overlaps_shifted_b = a_start < (b_end + 86400) && (b_start + 86400) < a_end;
    // Also check with a shifted by +24h (handles early-morning A vs overnight B)
    let overlaps_shifted_a = (a_start + 86400) < b_end && b_start < (a_end + 86400);

    overlaps || overlaps_shifted_b || overlaps_shifted_a
}

pub async fn create_working_pattern(
    pool: &PgPool,
    workspace_id: Uuid,
    req: CreateWorkingPatternRequest,
) -> Result<WorkingPattern, sqlx::Error> {
    // --- Split-shift overlap validation ---
    // Group details by day_of_week to find days with multiple shifts
    let mut day_shifts: HashMap<i32, Vec<Uuid>> = HashMap::new();
    for detail in &req.details {
        if let Some(shift_id) = detail.shift_id {
            day_shifts.entry(detail.day_of_week).or_default().push(shift_id);
        }
    }

    // For days with >1 shift, validate no time overlaps
    for shift_ids in day_shifts.values() {
        if shift_ids.len() < 2 {
            continue;
        }

        // Fetch shift times for all shift_ids on this day
        let rows = sqlx::query!(
            "SELECT id, start_time, end_time FROM shifts WHERE id = ANY($1) AND workspace_id = $2",
            &shift_ids[..],
            workspace_id
        )
        .fetch_all(pool)
        .await?;

        let times: Vec<(Uuid, NaiveTime, NaiveTime)> = rows
            .into_iter()
            .map(|r| (r.id, r.start_time, r.end_time))
            .collect();

        // Pairwise overlap check
        for i in 0..times.len() {
            for j in (i + 1)..times.len() {
                if times_overlap(times[i].1, times[i].2, times[j].1, times[j].2) {
                    return Err(sqlx::Error::Protocol(format!(
                        "Shifts overlap on the same day: shifts {} and {} have conflicting time ranges",
                        times[i].0, times[j].0
                    )));
                }
            }
        }
    }

    // --- Create pattern ---
    let mut tx = pool.begin().await?;

    // 1. Create Pattern Header
    let pattern = repo::create_pattern(
        &mut *tx,
        workspace_id,
        &req.name,
        req.description.as_deref(),
    )
    .await?;

    // 2. Add Details
    for detail in req.details {
        repo::add_pattern_detail(&mut *tx, pattern.id, detail.day_of_week, detail.shift_id).await?;
    }

    tx.commit().await?;
    Ok(pattern)
}

pub async fn list_patterns(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<WorkingPattern>, sqlx::Error> {
    repo::get_patterns(pool, workspace_id).await
}

pub async fn get_pattern_details(
    pool: &PgPool,
    pattern_id: Uuid,
) -> Result<Vec<WorkingPatternDetail>, sqlx::Error> {
    repo::get_pattern_details(pool, pattern_id).await
}

pub async fn assign_pattern(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    req: AssignPatternRequest,
) -> Result<EmployeeWorkingPattern, sqlx::Error> {
    repo::assign_pattern(
        pool,
        workspace_id,
        employee_id,
        req.pattern_id,
        req.start_date,
        req.end_date,
    )
    .await
}

pub async fn bulk_assign_pattern(
    pool: &PgPool,
    workspace_id: Uuid,
    req: BulkAssignPatternRequest,
) -> Result<BulkAssignResponse, sqlx::Error> {
    repo::bulk_assign_pattern(
        pool,
        workspace_id,
        &req.employee_ids,
        req.pattern_id,
        req.start_date,
        req.end_date,
    )
    .await
}

pub async fn get_employee_pattern_history(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
) -> Result<Vec<EmployeeWorkingPattern>, sqlx::Error> {
    repo::get_employee_pattern_history(pool, workspace_id, employee_id).await
}

pub async fn delete_working_pattern(pool: &PgPool, workspace_id: Uuid, pattern_id: Uuid) -> Result<(), sqlx::Error> {
    repo::delete_pattern(pool, workspace_id, pattern_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    fn t(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    #[test]
    fn test_adjacent_shifts_not_overlapping() {
        // Ca sáng 08:00-12:00, ca chiều 12:00-17:00
        assert!(!times_overlap(t(8, 0), t(12, 0), t(12, 0), t(17, 0)));
    }

    #[test]
    fn test_boundary_end_equals_start_not_overlapping() {
        // Ca 1 kết thúc 19:00, ca 2 bắt đầu 19:00
        assert!(!times_overlap(t(8, 0), t(19, 0), t(19, 0), t(22, 0)));
    }

    #[test]
    fn test_gap_between_shifts_not_overlapping() {
        // Ca sáng 08:00-12:00, ca chiều 13:00-17:00 (1h gap)
        assert!(!times_overlap(t(8, 0), t(12, 0), t(13, 0), t(17, 0)));
    }

    #[test]
    fn test_overlapping_shifts() {
        // Ca 1: 08:00-13:00, Ca 2: 12:00-17:00 (overlap 12:00-13:00)
        assert!(times_overlap(t(8, 0), t(13, 0), t(12, 0), t(17, 0)));
    }

    #[test]
    fn test_identical_shifts_overlap() {
        // Cùng giờ hoàn toàn
        assert!(times_overlap(t(8, 0), t(17, 0), t(8, 0), t(17, 0)));
    }

    #[test]
    fn test_one_shift_inside_another() {
        // Ca 2 nằm hoàn toàn trong ca 1
        assert!(times_overlap(t(8, 0), t(17, 0), t(10, 0), t(15, 0)));
    }

    #[test]
    fn test_overnight_shift_with_day_shift_no_overlap() {
        // Ca đêm 22:00-06:00, ca sáng 08:00-12:00 → no overlap
        assert!(!times_overlap(t(22, 0), t(6, 0), t(8, 0), t(12, 0)));
    }

    #[test]
    fn test_overnight_shift_overlapping_early_morning() {
        // Ca đêm 22:00-06:00, ca sáng sớm 05:00-09:00 → overlap 05:00-06:00
        assert!(times_overlap(t(22, 0), t(6, 0), t(5, 0), t(9, 0)));
    }

    #[test]
    fn test_two_overnight_shifts_overlap() {
        // Ca đêm 1: 20:00-04:00, ca đêm 2: 22:00-06:00 → overlap
        assert!(times_overlap(t(20, 0), t(4, 0), t(22, 0), t(6, 0)));
    }

    #[test]
    fn test_overnight_adjacent_not_overlapping() {
        // Ca chiều tối 14:00-22:00, ca đêm 22:00-06:00 → boundary touch
        assert!(!times_overlap(t(14, 0), t(22, 0), t(22, 0), t(6, 0)));
    }

    #[test]
    fn test_minute_overlap() {
        // Ca 1 kết thúc 12:01, ca 2 bắt đầu 12:00 → 1 minute overlap
        assert!(times_overlap(t(8, 0), t(12, 1), t(12, 0), t(17, 0)));
    }
}

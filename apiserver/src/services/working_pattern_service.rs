use crate::models::{
    AssignPatternRequest, BulkAssignPatternRequest, BulkAssignResponse,
    CreateWorkingPatternRequest, EmployeeWorkingPattern, WorkingPattern, WorkingPatternDetail,
};
use chrono::{NaiveTime, Timelike};
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

use std::sync::Arc;

#[tracing::instrument(skip_all, err)]
pub async fn create_working_pattern(
    repo: Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
    shift_repo: Arc<dyn crate::repositories::shift_repository::ShiftRepository>,
    workspace_id: Uuid,
    req: CreateWorkingPatternRequest,
) -> Result<WorkingPattern, crate::errors::AppError> {
    // --- Split-shift overlap validation ---
    // Group details by day_of_week to find days with multiple shifts
    let mut day_shifts: HashMap<i32, Vec<Uuid>> = HashMap::new();
    let mut details_to_insert = Vec::new();
    for detail in &req.details {
        details_to_insert.push((detail.day_of_week, detail.shift_id));
        if let Some(shift_id) = detail.shift_id {
            day_shifts
                .entry(detail.day_of_week)
                .or_default()
                .push(shift_id);
        }
    }

    // For days with >1 shift, validate no time overlaps
    if !day_shifts.is_empty() {
        let all_shifts = shift_repo.get_workspace_shifts(workspace_id).await?;
        let shift_map: HashMap<Uuid, _> = all_shifts.into_iter().map(|s| (s.id, s)).collect();

        for shift_ids in day_shifts.values() {
            if shift_ids.len() < 2 {
                continue;
            }

            let times: Vec<(Uuid, NaiveTime, NaiveTime)> = shift_ids
                .iter()
                .filter_map(|id| shift_map.get(id).map(|s| (*id, s.start_time, s.end_time)))
                .collect();

            // Pairwise overlap check
            for i in 0..times.len() {
                for j in (i + 1)..times.len() {
                    if times_overlap(times[i].1, times[i].2, times[j].1, times[j].2) {
                        return Err(crate::errors::AppError::Validation(format!(
                            "Shifts overlap on the same day: shifts {} and {} have conflicting time ranges",
                            times[i].0, times[j].0
                        )));
                    }
                }
            }
        }
    }

    // --- Create pattern ---
    let pattern = repo
        .create_pattern_with_details(
            workspace_id,
            &req.name,
            req.description.as_deref(),
            details_to_insert,
        )
        .await?;

    Ok(pattern)
}

#[tracing::instrument(skip_all, err)]
pub async fn list_patterns(
    repo: Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
    workspace_id: Uuid,
) -> Result<Vec<WorkingPattern>, crate::errors::AppError> {
    repo.get_patterns(workspace_id).await
}

#[tracing::instrument(skip_all, err)]
pub async fn get_pattern_details(
    repo: Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
    pattern_id: Uuid,
) -> Result<Vec<WorkingPatternDetail>, crate::errors::AppError> {
    repo.get_pattern_details(pattern_id).await
}

#[tracing::instrument(skip_all, err)]
pub async fn assign_pattern(
    repo: Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
    workspace_id: Uuid,
    employee_id: Uuid,
    req: AssignPatternRequest,
) -> Result<EmployeeWorkingPattern, crate::errors::AppError> {
    repo.assign_pattern(
        workspace_id,
        employee_id,
        req.pattern_id,
        req.start_date,
        req.end_date,
    )
    .await
}

#[tracing::instrument(skip_all, err)]
pub async fn bulk_assign_pattern(
    repo: Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
    workspace_id: Uuid,
    req: BulkAssignPatternRequest,
) -> Result<BulkAssignResponse, crate::errors::AppError> {
    repo.bulk_assign_pattern(
        workspace_id,
        &req.employee_ids,
        req.pattern_id,
        req.start_date,
        req.end_date,
    )
    .await
}

#[tracing::instrument(skip_all, err)]
pub async fn get_employee_pattern_history(
    repo: Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
    workspace_id: Uuid,
    employee_id: Uuid,
) -> Result<Vec<EmployeeWorkingPattern>, crate::errors::AppError> {
    repo.get_employee_pattern_history(workspace_id, employee_id)
        .await
}

#[tracing::instrument(skip_all, err)]
pub async fn delete_working_pattern(
    repo: Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
    workspace_id: Uuid,
    pattern_id: Uuid,
) -> Result<(), crate::errors::AppError> {
    repo.delete_pattern(workspace_id, pattern_id).await
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

    // ─── Async service tests with mock repositories ──────────────────────────

    use crate::errors::AppError;
    use crate::models::{
        BulkAssignPatternRequest, BulkAssignResponse, EmployeeWorkingPattern,
        WorkingPattern, WorkingPatternDetail,
    };
    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};
    use crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait;
    use crate::repositories::shift_repository::ShiftRepository;
    use crate::models::Shift;
    use uuid::Uuid;

    struct NopShiftRepo;
    #[async_trait]
    impl ShiftRepository for NopShiftRepo {
        async fn create_shift(&self, ws: Uuid, req: &crate::models::CreateShiftRequest, _a: bool, _p: Option<Uuid>) -> Result<Shift, AppError> {
            Ok(Shift {
                id: Uuid::new_v4(), workspace_id: ws,
                name: req.name.clone(), report_symbol: None,
                start_time: req.start_time, end_time: req.end_time,
                break_duration_minutes: req.break_duration_minutes,
                break_start: None, break_end: None,
                is_break_auto_deduct: true, break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false, is_active: true,
                parent_shift_id: _p,
                created_at: Utc::now(), updated_at: Utc::now(),
            })
        }
        async fn get_shift_by_id(&self, _w: Uuid, _id: Uuid) -> Result<Option<Shift>, AppError> { Ok(None) }
        async fn get_workspace_shifts(&self, _w: Uuid) -> Result<Vec<Shift>, AppError> { Ok(vec![]) }
        async fn update_shift_direct(&self, _w: Uuid, _id: Uuid, _r: &crate::models::UpdateShiftRequest) -> Result<Shift, AppError> { unimplemented!() }
        async fn archive_shift(&self, _w: Uuid, _id: Uuid) -> Result<(), AppError> { Ok(()) }
        async fn assign_employee_shift(&self, _e: Uuid, _s: Uuid, _sd: NaiveDate, _ed: Option<NaiveDate>) -> Result<(), AppError> { Ok(()) }
        async fn get_employee_active_shift(&self, _e: Uuid, _d: NaiveDate) -> Result<Option<Shift>, AppError> { Ok(None) }
        async fn update_assignments_to_new_version(&self, _o: Uuid, _n: Uuid, _d: NaiveDate) -> Result<(), AppError> { Ok(()) }
    }

    fn dummy_pattern() -> WorkingPattern {
        WorkingPattern {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            name: "5×8".into(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    struct MockPatternRepo {
        pattern: std::sync::Mutex<WorkingPattern>,
    }
    #[async_trait]
    impl WorkingPatternRepositoryTrait for MockPatternRepo {
        async fn create_pattern_with_details(&self, _w: Uuid, name: &str, _d: Option<&str>, _det: Vec<(i32, Option<Uuid>)>) -> Result<WorkingPattern, AppError> {
            let mut p = self.pattern.lock().unwrap().clone();
            p.name = name.to_string();
            Ok(p)
        }
        async fn get_patterns(&self, _w: Uuid) -> Result<Vec<WorkingPattern>, AppError> {
            Ok(vec![self.pattern.lock().unwrap().clone()])
        }
        async fn get_pattern_details(&self, _pid: Uuid) -> Result<Vec<WorkingPatternDetail>, AppError> {
            Ok(vec![WorkingPatternDetail { id: Uuid::new_v4(), pattern_id: _pid, day_of_week: 1, shift_id: None }])
        }
        async fn assign_pattern(&self, ws: Uuid, emp: Uuid, pat: Uuid, sd: NaiveDate, ed: Option<NaiveDate>) -> Result<EmployeeWorkingPattern, AppError> {
            Ok(EmployeeWorkingPattern { id: Uuid::new_v4(), workspace_id: ws, employee_id: emp, pattern_id: pat, start_date: sd, end_date: ed, created_at: Utc::now(), updated_at: Utc::now() })
        }
        async fn get_employee_pattern_history(&self, _w: Uuid, emp: Uuid) -> Result<Vec<EmployeeWorkingPattern>, AppError> {
            Ok(vec![EmployeeWorkingPattern {
                id: Uuid::new_v4(), workspace_id: _w, employee_id: emp,
                pattern_id: Uuid::new_v4(),
                start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                end_date: None, created_at: Utc::now(), updated_at: Utc::now(),
            }])
        }
        async fn bulk_assign_pattern(&self, _w: Uuid, emp: &[Uuid], _p: Uuid, _s: NaiveDate, _e: Option<NaiveDate>) -> Result<BulkAssignResponse, AppError> {
            Ok(BulkAssignResponse { success_count: emp.len(), failed_employees: vec![] })
        }
        async fn delete_pattern(&self, _w: Uuid, _p: Uuid) -> Result<(), AppError> { Ok(()) }
    }

    fn make_repos() -> (std::sync::Arc<MockPatternRepo>, std::sync::Arc<NopShiftRepo>) {
        let pat = std::sync::Arc::new(MockPatternRepo { pattern: std::sync::Mutex::new(dummy_pattern()) });
        let shift = std::sync::Arc::new(NopShiftRepo);
        (pat, shift)
    }

    #[tokio::test]
    async fn test_list_patterns() {
        let (repo, _) = make_repos();
        let result = list_patterns(repo, Uuid::new_v4()).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_get_pattern_details() {
        let (repo, _) = make_repos();
        let result = get_pattern_details(repo, Uuid::new_v4()).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_assign_pattern() {
        let (repo, _) = make_repos();
        let req = crate::models::AssignPatternRequest {
            pattern_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
        };
        let result = assign_pattern(repo, Uuid::new_v4(), Uuid::new_v4(), req).await.unwrap();
        assert!(result.end_date.is_none());
    }

    #[tokio::test]
    async fn test_bulk_assign_pattern() {
        let (repo, _) = make_repos();
        let req = BulkAssignPatternRequest {
            employee_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            pattern_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
        };
        let result = bulk_assign_pattern(repo, Uuid::new_v4(), req).await.unwrap();
        assert_eq!(result.success_count, 2);
        assert!(result.failed_employees.is_empty());
    }

    #[tokio::test]
    async fn test_get_employee_pattern_history() {
        let (repo, _) = make_repos();
        let result = get_employee_pattern_history(repo, Uuid::new_v4(), Uuid::new_v4()).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_working_pattern() {
        let (repo, _) = make_repos();
        delete_working_pattern(repo, Uuid::new_v4(), Uuid::new_v4()).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_working_pattern_no_overlaps() {
        let (repo, shift_repo) = make_repos();
        let req = crate::models::CreateWorkingPatternRequest {
            name: "Standard 5×8".into(),
            description: Some("Mon-Fri".into()),
            details: vec![
                crate::models::WorkingPatternDetailRequest { day_of_week: 1, shift_id: None },
                crate::models::WorkingPatternDetailRequest { day_of_week: 2, shift_id: None },
            ],
        };
        let result = create_working_pattern(repo, shift_repo, Uuid::new_v4(), req).await.unwrap();
        assert_eq!(result.name, "Standard 5×8");
    }
}

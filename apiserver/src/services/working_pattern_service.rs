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
        let sm = i64::from(s.num_seconds_from_midnight());
        let em = i64::from(e.num_seconds_from_midnight());
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
    // Group details by schedule_day to find days with multiple shifts
    let mut day_shifts: HashMap<i32, Vec<Uuid>> = HashMap::new();
    let mut details_to_insert = Vec::new();
    for detail in &req.details {
        details_to_insert.push((detail.schedule_day, detail.shift_id));
        if let Some(shift_id) = detail.shift_id {
            day_shifts
                .entry(detail.schedule_day)
                .or_default()
                .push(shift_id);
        }
    }

    // For days with >1 shift, validate no time overlaps
    if !day_shifts.is_empty() {
        let all_shifts = shift_repo
            .get_workspace_shifts(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                None,
                None,
            )
            .await?;
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

    // --- Determine pattern_type ---
    let pattern_type = req.pattern_type.as_deref().unwrap_or("weekly");
    if pattern_type != "weekly" && pattern_type != "rotation" {
        return Err(crate::errors::AppError::Validation(
            "pattern_type must be 'weekly' or 'rotation'".to_string(),
        ));
    }
    if pattern_type == "rotation" {
        let cycle_len = req.cycle_length_days.unwrap_or(0);
        if !(1..=366).contains(&cycle_len) {
            return Err(crate::errors::AppError::Validation(
                "cycle_length_days must be between 1 and 366 for rotation patterns".to_string(),
            ));
        }
        // Validate schedule_day range
        for detail in &req.details {
            if detail.schedule_day < 0 || detail.schedule_day >= cycle_len {
                return Err(crate::errors::AppError::Validation(format!(
                    "schedule_day {} is out of range [0, {}) for cycle_length_days {}",
                    detail.schedule_day, cycle_len, cycle_len,
                )));
            }
        }
    }

    // --- Create pattern ---
    let pattern = repo
        .create_pattern_with_details(
            workspace_id,
            &req.name,
            req.description.as_deref(),
            pattern_type,
            req.cycle_length_days,
            details_to_insert,
        )
        .await?;

    Ok(pattern)
}

#[tracing::instrument(skip_all, err)]
pub async fn update_working_pattern(
    repo: Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
    shift_repo: Arc<dyn crate::repositories::shift_repository::ShiftRepository>,
    workspace_id: Uuid,
    pattern_id: Uuid,
    req: CreateWorkingPatternRequest,
) -> Result<WorkingPattern, crate::errors::AppError> {
    let mut day_shifts: HashMap<i32, Vec<Uuid>> = HashMap::new();
    let mut details_to_insert = Vec::new();
    for detail in &req.details {
        details_to_insert.push((detail.schedule_day, detail.shift_id));
        if let Some(shift_id) = detail.shift_id {
            day_shifts
                .entry(detail.schedule_day)
                .or_default()
                .push(shift_id);
        }
    }

    if !day_shifts.is_empty() {
        let all_shifts = shift_repo
            .get_workspace_shifts(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                None,
                None,
            )
            .await?;
        let shift_map: HashMap<Uuid, _> = all_shifts.into_iter().map(|s| (s.id, s)).collect();

        for shift_ids in day_shifts.values() {
            if shift_ids.len() < 2 {
                continue;
            }

            let times: Vec<(Uuid, NaiveTime, NaiveTime)> = shift_ids
                .iter()
                .filter_map(|id| shift_map.get(id).map(|s| (*id, s.start_time, s.end_time)))
                .collect();

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

    let pattern_type = req.pattern_type.as_deref().unwrap_or("weekly");
    if pattern_type != "weekly" && pattern_type != "rotation" {
        return Err(crate::errors::AppError::Validation(
            "pattern_type must be 'weekly' or 'rotation'".to_string(),
        ));
    }
    if pattern_type == "rotation" {
        let cycle_len = req.cycle_length_days.unwrap_or(0);
        if !(1..=366).contains(&cycle_len) {
            return Err(crate::errors::AppError::Validation(
                "cycle_length_days must be between 1 and 366 for rotation patterns".to_string(),
            ));
        }
        for detail in &req.details {
            if detail.schedule_day < 0 || detail.schedule_day >= cycle_len {
                return Err(crate::errors::AppError::Validation(format!(
                    "schedule_day {} is out of range [0, {}) for cycle_length_days {}",
                    detail.schedule_day, cycle_len, cycle_len,
                )));
            }
        }
    }

    let pattern = repo
        .update_pattern_with_details(
            workspace_id,
            pattern_id,
            &req.name,
            req.description.as_deref(),
            pattern_type,
            req.cycle_length_days,
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
    workspace_id: Uuid,
    pattern_id: Uuid,
) -> Result<Vec<WorkingPatternDetail>, crate::errors::AppError> {
    repo.get_pattern_details(workspace_id, pattern_id).await
}

#[tracing::instrument(skip_all, err)]
pub async fn assign_pattern(
    repo: Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
    workspace_id: Uuid,
    employment_id: Uuid,
    req: AssignPatternRequest,
) -> Result<EmployeeWorkingPattern, crate::errors::AppError> {
    repo.assign_pattern(
        workspace_id,
        employment_id,
        req.pattern_id,
        req.start_date,
        req.end_date,
        req.anchor_date,
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
        &req.employment_ids,
        req.pattern_id,
        req.start_date,
        req.end_date,
        req.anchor_date,
    )
    .await
}

#[tracing::instrument(skip_all, err)]
pub async fn get_employment_pattern_history(
    repo: Arc<dyn crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait>,
    workspace_id: Uuid,
    employment_id: Uuid,
) -> Result<Vec<EmployeeWorkingPattern>, crate::errors::AppError> {
    repo.get_employment_pattern_history(workspace_id, employment_id)
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
    use crate::models::Shift;
    use crate::models::{
        BulkAssignPatternRequest, BulkAssignResponse, EmployeeWorkingPattern, WorkingPattern,
        WorkingPatternDetail,
    };
    use crate::repositories::shift_repository::ShiftRepository;
    use crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait;
    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    struct NopShiftRepo;
    #[async_trait]
    impl ShiftRepository for NopShiftRepo {
        async fn create_shift(
            &self,
            auth_token: &crate::diesel_pool::TenantAuthToken,
            req: &crate::models::CreateShiftRequest,
            _a: bool,
            _p: Option<Uuid>,
        ) -> Result<Shift, AppError> {
            Ok(Shift {
                id: Uuid::new_v4(),
                workspace_id: auth_token.0,
                name: req.name.clone(),
                report_symbol: None,
                start_time: req.start_time,
                end_time: req.end_time,
                break_duration_minutes: req.break_duration_minutes,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                is_active: true,
                parent_shift_id: _p,
                location_id: req.location_id,
                employer_id: req.employer_id,
                max_early_ot_minutes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
        async fn get_shift_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<Option<Shift>, AppError> {
            Ok(None)
        }
        async fn get_workspace_shifts(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _location_id: Option<Uuid>,
            _employer_id: Option<Uuid>,
        ) -> Result<Vec<Shift>, AppError> {
            // Provide a few mock shifts for overlap testing
            let s1 = Shift {
                id: Uuid::from_u128(1),
                workspace_id: Uuid::new_v4(),
                name: "S1".into(),
                report_symbol: None,
                start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end_time: chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                break_duration_minutes: 0,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: false,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                is_active: true,
                parent_shift_id: None,
                location_id: None,
                employer_id: None,
                max_early_ot_minutes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let s2 = Shift {
                id: Uuid::from_u128(2),
                workspace_id: Uuid::new_v4(),
                name: "S2".into(),
                report_symbol: None,
                start_time: chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                end_time: chrono::NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
                break_duration_minutes: 0,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: false,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                is_active: true,
                parent_shift_id: None,
                location_id: None,
                employer_id: None,
                max_early_ot_minutes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            Ok(vec![s1, s2])
        }
        async fn update_shift_direct(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _r: &crate::models::UpdateShiftRequest,
        ) -> Result<Shift, AppError> {
            unimplemented!()
        }
        async fn archive_shift(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn assign_employment_shift(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: Uuid,
            _sd: NaiveDate,
            _ed: Option<NaiveDate>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        async fn get_employment_active_shift(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _d: NaiveDate,
        ) -> Result<Option<Shift>, AppError> {
            Ok(None)
        }
        async fn update_assignments_to_new_version(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _o: Uuid,
            _n: Uuid,
            _d: NaiveDate,
        ) -> Result<(), AppError> {
            Ok(())
        }
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
            pattern_type: "weekly".to_string(),
            cycle_length_days: None,
        }
    }

    struct MockPatternRepo {
        pattern: std::sync::Mutex<WorkingPattern>,
    }
    #[async_trait]
    impl WorkingPatternRepositoryTrait for MockPatternRepo {
        async fn create_pattern_with_details(
            &self,
            _w: Uuid,
            name: &str,
            _d: Option<&str>,
            _pt: &str,
            _cl: Option<i32>,
            _det: Vec<(i32, Option<Uuid>)>,
        ) -> Result<WorkingPattern, AppError> {
            let mut p = self.pattern.lock().unwrap().clone();
            p.name = name.to_string();
            Ok(p)
        }
        #[allow(clippy::too_many_arguments)]
        async fn update_pattern_with_details(
            &self,
            _w: Uuid,
            _pid: Uuid,
            name: &str,
            _d: Option<&str>,
            _pt: &str,
            _cl: Option<i32>,
            _det: Vec<(i32, Option<Uuid>)>,
        ) -> Result<WorkingPattern, AppError> {
            let mut p = self.pattern.lock().unwrap().clone();
            p.name = name.to_string();
            Ok(p)
        }
        async fn get_patterns(&self, _w: Uuid) -> Result<Vec<WorkingPattern>, AppError> {
            Ok(vec![self.pattern.lock().unwrap().clone()])
        }
        async fn get_pattern_details(
            &self,
            _ws: Uuid,
            _pid: Uuid,
        ) -> Result<Vec<WorkingPatternDetail>, AppError> {
            Ok(vec![WorkingPatternDetail {
                id: Uuid::new_v4(),
                pattern_id: _pid,
                schedule_day: 1,
                shift_id: None,
            }])
        }
        async fn assign_pattern(
            &self,
            ws: Uuid,
            emp: Uuid,
            pat: Uuid,
            sd: NaiveDate,
            ed: Option<NaiveDate>,
            anchor: Option<NaiveDate>,
        ) -> Result<EmployeeWorkingPattern, AppError> {
            Ok(EmployeeWorkingPattern {
                id: Uuid::new_v4(),
                workspace_id: ws,
                employment_id: emp,
                pattern_id: pat,
                start_date: sd,
                end_date: ed,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                anchor_date: anchor,
            })
        }
        async fn get_employment_pattern_history(
            &self,
            _w: Uuid,
            emp: Uuid,
        ) -> Result<Vec<EmployeeWorkingPattern>, AppError> {
            Ok(vec![EmployeeWorkingPattern {
                id: Uuid::new_v4(),
                workspace_id: _w,
                employment_id: emp,
                pattern_id: Uuid::new_v4(),
                start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                end_date: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                anchor_date: None,
            }])
        }
        async fn bulk_assign_pattern(
            &self,
            _w: Uuid,
            emp: &[Uuid],
            _p: Uuid,
            _s: NaiveDate,
            _e: Option<NaiveDate>,
            _a: Option<NaiveDate>,
        ) -> Result<BulkAssignResponse, AppError> {
            Ok(BulkAssignResponse {
                success_count: emp.len(),
                failed_employees: vec![],
            })
        }
        async fn delete_pattern(&self, _w: Uuid, _p: Uuid) -> Result<(), AppError> {
            Ok(())
        }
    }

    fn make_repos() -> (
        std::sync::Arc<MockPatternRepo>,
        std::sync::Arc<NopShiftRepo>,
    ) {
        let pat = std::sync::Arc::new(MockPatternRepo {
            pattern: std::sync::Mutex::new(dummy_pattern()),
        });
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
        let result = get_pattern_details(repo, Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_assign_pattern() {
        let (repo, _) = make_repos();
        let req = crate::models::AssignPatternRequest {
            pattern_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            anchor_date: None,
        };
        let result = assign_pattern(repo, Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap();
        assert!(result.end_date.is_none());
    }

    #[tokio::test]
    async fn test_bulk_assign_pattern() {
        let (repo, _) = make_repos();
        let req = BulkAssignPatternRequest {
            employment_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            pattern_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            anchor_date: None,
        };
        let result = bulk_assign_pattern(repo, Uuid::new_v4(), req)
            .await
            .unwrap();
        assert_eq!(result.success_count, 2);
        assert!(result.failed_employees.is_empty());
    }

    #[tokio::test]
    async fn test_get_employment_pattern_history() {
        let (repo, _) = make_repos();
        let result = get_employment_pattern_history(repo, Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_working_pattern() {
        let (repo, _) = make_repos();
        delete_working_pattern(repo, Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_working_pattern_no_overlaps() {
        let (repo, shift_repo) = make_repos();
        let req = crate::models::CreateWorkingPatternRequest {
            name: "Standard 5×8".into(),
            description: Some("Mon-Fri".into()),
            pattern_type: None,
            cycle_length_days: None,
            details: vec![
                crate::models::WorkingPatternDetailRequest {
                    schedule_day: 1,
                    shift_id: None,
                },
                crate::models::WorkingPatternDetailRequest {
                    schedule_day: 2,
                    shift_id: None,
                },
            ],
        };
        let result = create_working_pattern(repo, shift_repo, Uuid::new_v4(), req)
            .await
            .unwrap();
        assert_eq!(result.name, "Standard 5×8");
    }

    #[tokio::test]
    async fn test_create_working_pattern_validation_errors() {
        let (repo, shift_repo) = make_repos();

        // 1. Invalid pattern type
        let req1 = crate::models::CreateWorkingPatternRequest {
            name: "Invalid Type".into(),
            description: None,
            pattern_type: Some("invalid_type".into()),
            cycle_length_days: None,
            details: vec![],
        };
        let res1 =
            create_working_pattern(repo.clone(), shift_repo.clone(), Uuid::new_v4(), req1).await;
        assert!(res1.is_err());
        assert!(res1
            .unwrap_err()
            .to_string()
            .contains("pattern_type must be"));

        // 2. Rotation type with missing/invalid cycle_length_days
        let req2 = crate::models::CreateWorkingPatternRequest {
            name: "Rotation missing length".into(),
            description: None,
            pattern_type: Some("rotation".into()),
            cycle_length_days: Some(400),
            details: vec![],
        };
        let res2 =
            create_working_pattern(repo.clone(), shift_repo.clone(), Uuid::new_v4(), req2).await;
        assert!(res2.is_err());
        assert!(res2
            .unwrap_err()
            .to_string()
            .contains("cycle_length_days must be between 1 and 366"));

        // 3. Rotation type with schedule_day out of range
        let req3 = crate::models::CreateWorkingPatternRequest {
            name: "Rotation out of range".into(),
            description: None,
            pattern_type: Some("rotation".into()),
            cycle_length_days: Some(10),
            details: vec![crate::models::WorkingPatternDetailRequest {
                schedule_day: 10, // out of range [0, 10)
                shift_id: None,
            }],
        };
        let res3 =
            create_working_pattern(repo.clone(), shift_repo.clone(), Uuid::new_v4(), req3).await;
        assert!(res3.is_err());
        assert!(res3
            .unwrap_err()
            .to_string()
            .contains("schedule_day 10 is out of range"));

        // 4. Shifts overlap
        let req4 = crate::models::CreateWorkingPatternRequest {
            name: "Overlap".into(),
            description: None,
            pattern_type: Some("weekly".into()),
            cycle_length_days: None,
            details: vec![
                crate::models::WorkingPatternDetailRequest {
                    schedule_day: 1,
                    shift_id: Some(Uuid::from_u128(1)),
                },
                crate::models::WorkingPatternDetailRequest {
                    schedule_day: 1,                    // Same day
                    shift_id: Some(Uuid::from_u128(2)), // Overlaps with 1
                },
            ],
        };
        let res4 =
            create_working_pattern(repo.clone(), shift_repo.clone(), Uuid::new_v4(), req4).await;
        assert!(res4.is_err());
        assert!(res4
            .unwrap_err()
            .to_string()
            .contains("Shifts overlap on the same day"));
    }

    #[tokio::test]
    async fn test_update_working_pattern_validation_errors() {
        let (repo, shift_repo) = make_repos();

        // 1. Invalid pattern type
        let req1 = crate::models::CreateWorkingPatternRequest {
            name: "Invalid Type".into(),
            description: None,
            pattern_type: Some("invalid_type".into()),
            cycle_length_days: None,
            details: vec![],
        };
        let res1 = update_working_pattern(
            repo.clone(),
            shift_repo.clone(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            req1,
        )
        .await;
        assert!(res1.is_err());

        // 2. Rotation type with missing/invalid cycle_length_days
        let req2 = crate::models::CreateWorkingPatternRequest {
            name: "Rotation missing length".into(),
            description: None,
            pattern_type: Some("rotation".into()),
            cycle_length_days: Some(400),
            details: vec![],
        };
        let res2 = update_working_pattern(
            repo.clone(),
            shift_repo.clone(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            req2,
        )
        .await;
        assert!(res2.is_err());

        // 3. Rotation type with schedule_day out of range
        let req3 = crate::models::CreateWorkingPatternRequest {
            name: "Rotation out of range".into(),
            description: None,
            pattern_type: Some("rotation".into()),
            cycle_length_days: Some(10),
            details: vec![crate::models::WorkingPatternDetailRequest {
                schedule_day: 10, // out of range [0, 10)
                shift_id: None,
            }],
        };
        let res3 = update_working_pattern(
            repo.clone(),
            shift_repo.clone(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            req3,
        )
        .await;
        assert!(res3.is_err());

        // 4. Shifts overlap
        let req4 = crate::models::CreateWorkingPatternRequest {
            name: "Overlap".into(),
            description: None,
            pattern_type: Some("weekly".into()),
            cycle_length_days: None,
            details: vec![
                crate::models::WorkingPatternDetailRequest {
                    schedule_day: 1,
                    shift_id: Some(Uuid::from_u128(1)),
                },
                crate::models::WorkingPatternDetailRequest {
                    schedule_day: 1,                    // Same day
                    shift_id: Some(Uuid::from_u128(2)), // Overlaps with 1
                },
            ],
        };
        let res4 = update_working_pattern(
            repo.clone(),
            shift_repo.clone(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            req4,
        )
        .await;
        assert!(res4.is_err());
    }
}

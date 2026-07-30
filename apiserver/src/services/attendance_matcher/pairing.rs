use super::*;
use crate::cache::RosterInterval;
use crate::models::employee_assignment::AssignmentStatusEnum;
use crate::repositories::roster::shift_resolution::resolve_effective_shift_from_data;

impl AttendanceMatcher {
    /// Resolve active employment, employer, and assignment/location for a
    /// single calendar date.
    ///
    /// Returns `(active_employer_id, active_employment_id, active_location_id)`.
    pub(super) fn resolve_day_context(
        ctx: &AttendanceContext,
        current_date: NaiveDate,
    ) -> (Option<Uuid>, Uuid, Uuid) {
        // Find active employment on current_date
        let active_employment = ctx
            .active_employments()
            .iter()
            .filter(|e| {
                e.status() == "active"
                    && e.start_date() <= current_date
                    && e.end_date().is_none_or(|end| end >= current_date)
            })
            .max_by_key(|e| e.start_date());
        let active_employer_id = active_employment.map(crate::models::Employment::employer_id);
        let active_employment_id = active_employment
            .map(crate::models::Employment::id)
            .unwrap_or_else(|| {
                ctx.active_employments()
                    .first()
                    .map(crate::models::Employment::id)
                    .unwrap_or_else(Uuid::nil)
            });

        // Find active assignment on current_date
        let active_assignment = ctx
            .active_assignments()
            .iter()
            .filter(|a| {
                a.status == AssignmentStatusEnum::Active
                    && a.start_date <= current_date
                    && a.end_date.is_none_or(|end| end >= current_date)
            })
            .max_by_key(|a| a.start_date);
        let active_location_id = active_assignment
            .map(|a| a.host_location_id)
            .unwrap_or_else(|| ctx.primary_location_id());

        (active_employer_id, active_employment_id, active_location_id)
    }

    /// Resolve shifts from pre-loaded roster data for a calendar date, then
    /// return the shift-id / first-interval pairs sorted by `start_time`.
    ///
    /// Uses `resolve_effective_shift_from_data` which is zero-DB — all data
    /// comes from the in-memory `ctx.roster_data` and `ctx.payroll_roster_map`.
    pub(super) fn collect_sorted_day_shifts(
        ctx: &AttendanceContext,
        active_employment_id: Uuid,
        employee_id: Uuid,
        active_location_id: Uuid,
        active_employer_id: Option<Uuid>,
        current_date: NaiveDate,
    ) -> Vec<(Uuid, &RosterInterval)> {
        let (shift_ids, _, _) = resolve_effective_shift_from_data(
            ctx.roster_data(),
            ctx.payroll_roster_map(),
            active_employment_id,
            employee_id,
            ctx.emp_id_number(),
            ctx.emp_tags(),
            current_date,
            Some(active_location_id),
            active_employer_id,
        );

        let mut day_shifts: Vec<(Uuid, &RosterInterval)> = shift_ids
            .iter()
            .filter_map(|opt| {
                let id = (*opt)?;
                let interval = ctx.roster_data().shift_intervals.get(&id)?.first()?;
                Some((id, interval))
            })
            .collect();
        day_shifts.sort_by_key(|(_, i)| i.start_time);
        day_shifts
    }
}

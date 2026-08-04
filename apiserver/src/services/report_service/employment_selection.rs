use crate::errors::AppError;
use crate::models::Employment;
use crate::utils::i18n::I18nKey;
use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EmploymentGroup {
    pub(super) employee_id: Uuid,
    pub(super) employer_id: Uuid,
    pub(super) employment_ids: Vec<Uuid>,
    pub(super) representative_employment_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EmploymentSelection {
    pub(super) employment_ids: Vec<Uuid>,
    pub(super) groups: Vec<EmploymentGroup>,
}

fn overlaps_range(employment: &Employment, start_date: NaiveDate, end_date: NaiveDate) -> bool {
    employment.start_date() <= end_date
        && employment
            .end_date()
            .is_none_or(|employment_end| employment_end >= start_date)
}

fn validate_no_group_overlap(employments: &[&Employment]) -> Result<(), AppError> {
    for pair in employments.windows(2) {
        let previous = pair[0];
        let next = pair[1];
        if previous
            .end_date()
            .is_none_or(|previous_end| previous_end >= next.start_date())
        {
            return Err(AppError::conflict(I18nKey::OverlappingEmployment));
        }
    }
    Ok(())
}

pub(super) fn select_report_employments(
    employments: &[Employment],
    employee_order: &[Uuid],
    start_date: NaiveDate,
    end_date: NaiveDate,
    employer_id: Option<Uuid>,
) -> Result<EmploymentSelection, AppError> {
    let employee_positions: HashMap<Uuid, usize> = employee_order
        .iter()
        .enumerate()
        .map(|(position, employee_id)| (*employee_id, position))
        .collect();

    let mut seen_ids = HashSet::new();
    let mut selected: Vec<&Employment> = employments
        .iter()
        .filter(|employment| employee_positions.contains_key(&employment.employee_id()))
        .filter(|employment| employer_id.is_none_or(|id| employment.employer_id() == id))
        .filter(|employment| overlaps_range(employment, start_date, end_date))
        .filter(|employment| seen_ids.insert(employment.id()))
        .collect();

    selected.sort_by_key(|employment| {
        (
            employee_positions
                .get(&employment.employee_id())
                .copied()
                .unwrap_or(usize::MAX),
            employment.employer_id(),
            employment.start_date(),
            employment.id(),
        )
    });

    let mut grouped: Vec<Vec<&Employment>> = Vec::new();
    for employment in selected {
        let belongs_to_last_group = grouped.last().is_some_and(|group| {
            group.first().is_some_and(|first| {
                first.employee_id() == employment.employee_id()
                    && first.employer_id() == employment.employer_id()
            })
        });
        if belongs_to_last_group {
            if let Some(group) = grouped.last_mut() {
                group.push(employment);
            }
        } else {
            grouped.push(vec![employment]);
        }
    }

    let mut employment_ids = Vec::new();
    let mut groups = Vec::new();
    for group in grouped {
        validate_no_group_overlap(&group)?;
        let Some(first) = group.first() else {
            continue;
        };
        let Some(representative) = group.last() else {
            continue;
        };
        let group_employment_ids: Vec<Uuid> =
            group.iter().map(|employment| employment.id()).collect();
        employment_ids.extend(group_employment_ids.iter().copied());
        groups.push(EmploymentGroup {
            employee_id: first.employee_id(),
            employer_id: first.employer_id(),
            employment_ids: group_employment_ids,
            representative_employment_id: representative.id(),
        });
    }

    Ok(EmploymentSelection {
        employment_ids,
        groups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use chrono::Utc;

    fn employment(
        id: Uuid,
        employee_id: Uuid,
        employer_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Employment {
        Employment::new(NewEmploymentParams {
            id: id,
            workspace_id: Uuid::new_v4(),
            employee_id: employee_id,
            employer_id: employer_id,
            primary_location_id: None,
            department_id: None,
            status: "active".to_string(),
            start_date: start_date,
            end_date: end_date,
            job_title: None,
            employment_type: "full_time".to_string(),
            manager_employment_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            termination_reason: None,
            notes: None,
        })
    }

    #[test]
    fn groups_adjacent_employments_for_same_employer() {
        let employee_id = Uuid::new_v4();
        let employer_id = Uuid::new_v4();
        let old_id = Uuid::new_v4();
        let new_id = Uuid::new_v4();
        let employments = vec![
            employment(
                old_id,
                employee_id,
                employer_id,
                NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
                Some(NaiveDate::from_ymd_opt(2026, 7, 14).unwrap()),
            ),
            employment(
                new_id,
                employee_id,
                employer_id,
                NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
                None,
            ),
        ];

        let selection = select_report_employments(
            &employments,
            &[employee_id],
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 7, 31).unwrap(),
            None,
        )
        .unwrap();

        assert_eq!(selection.employment_ids, vec![old_id, new_id]);
        assert_eq!(selection.groups.len(), 1);
        assert_eq!(selection.groups[0].employment_ids, vec![old_id, new_id]);
        assert_eq!(selection.groups[0].representative_employment_id, new_id);
    }

    #[test]
    fn preserves_separate_employers() {
        let employee_id = Uuid::new_v4();
        let employer_a = Uuid::new_v4();
        let employer_b = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let employments = vec![
            employment(Uuid::new_v4(), employee_id, employer_a, date, None),
            employment(Uuid::new_v4(), employee_id, employer_b, date, None),
        ];

        let selection =
            select_report_employments(&employments, &[employee_id], date, date, None).unwrap();

        assert_eq!(selection.groups.len(), 2);
    }

    #[test]
    fn rejects_true_overlap_for_same_employer() {
        let employee_id = Uuid::new_v4();
        let employer_id = Uuid::new_v4();
        let employments = vec![
            employment(
                Uuid::new_v4(),
                employee_id,
                employer_id,
                NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
                Some(NaiveDate::from_ymd_opt(2026, 7, 15).unwrap()),
            ),
            employment(
                Uuid::new_v4(),
                employee_id,
                employer_id,
                NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
                None,
            ),
        ];

        let error = select_report_employments(
            &employments,
            &[employee_id],
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 7, 31).unwrap(),
            None,
        )
        .unwrap_err();

        assert_eq!(error.error_code(), "CONFLICT");
    }

    #[test]
    fn ignores_employments_outside_report_range() {
        let employee_id = Uuid::new_v4();
        let employer_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let employments = vec![employment(
            employment_id,
            employee_id,
            employer_id,
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
        )];

        let selection = select_report_employments(
            &employments,
            &[employee_id],
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 7, 31).unwrap(),
            None,
        )
        .unwrap();

        assert!(selection.employment_ids.is_empty());
        assert!(selection.groups.is_empty());
    }
}

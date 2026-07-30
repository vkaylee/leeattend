//! Bulk termination preview service.
//!
//! Extracted from `handlers/employees::import_termination_preview` to move
//! business logic out of the HTTP layer and fix the N+1 leave-balance query.

use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppResult;
use crate::models::{Employee, EmployeeLeaveBalance, Employment};
use crate::repositories::employee::EmployeeRepository;
use crate::repositories::employment_repository::EmploymentRepository;
use crate::repositories::leave_balance_repository::LeaveBalanceRepository;
use chrono::Datelike;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

use super::super::handlers::employees::{
    InvalidTerminationRow, ProratedLeavePreview, TerminationPreviewResponse, ValidTerminationRow,
};

const VALID_TERMINATION_REASONS: &[&str] = &[
    "voluntary_resignation",
    "involuntary_termination",
    "end_of_contract",
    "retirement",
    "mutual_agreement",
    "abandonment",
    "death",
    "other",
];

/// Holds parsed-and-validated data for a single CSV row before enrichment.
struct ParsedRow {
    row_number: usize,
    raw_code: String,
    normalized_code: String,
    raw_date: String,
    termination_date: chrono::NaiveDate,
    termination_reason: String,
    termination_notes: Option<String>,
}

pub struct TerminationService {
    employee_repo: Arc<dyn EmployeeRepository>,
    employment_repo: Arc<dyn EmploymentRepository>,
    leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
}

impl TerminationService {
    pub fn new(
        employee_repo: Arc<dyn EmployeeRepository>,
        employment_repo: Arc<dyn EmploymentRepository>,
        leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
    ) -> Self {
        Self {
            employee_repo,
            employment_repo,
            leave_balance_repo,
        }
    }

    /// Build a termination preview from already-parsed CSV mapped rows.
    #[allow(clippy::too_many_lines)]
    pub async fn build_preview(
        &self,
        auth_token: &TenantAuthToken,
        mapped_rows: Vec<(usize, HashMap<String, String>)>,
    ) -> AppResult<TerminationPreviewResponse> {
        // ── 1. Parse & validate rows ──────────────────────────────────────
        let (parsed_rows, mut invalid_rows, normalized_codes) =
            Self::parse_and_validate_rows(mapped_rows);

        if parsed_rows.is_empty() {
            return Ok(TerminationPreviewResponse {
                valid_rows: vec![],
                invalid_rows,
                can_commit: false,
            });
        }

        // ── 2. Batch-fetch Employees ─────────────────────────────────────
        let employees_list = self
            .employee_repo
            .get_employees_by_id_numbers(&normalized_codes, auth_token)
            .await?;

        let employees_by_code: HashMap<String, Employee> = employees_list
            .into_iter()
            .map(|emp| (emp.employee_id_number().to_string(), emp))
            .collect();

        let employee_ids: Vec<Uuid> = employees_by_code.values().map(Employee::id).collect();

        // ── 3. Batch-fetch Employments ───────────────────────────────────
        let employments = self
            .employment_repo
            .get_employments_for_employees(auth_token, &employee_ids)
            .await?;

        let employments_by_employee_id: HashMap<Uuid, Vec<Employment>> = {
            let mut map: HashMap<Uuid, Vec<Employment>> = HashMap::new();
            for emp in employments {
                map.entry(emp.employee_id()).or_default().push(emp);
            }
            map
        };

        // ── 4. Batch-fetch Leave Balances (SINGLE query for ALL years) ──
        let years: Vec<i32> = {
            let mut set: HashSet<i32> = parsed_rows
                .iter()
                .map(|r| r.termination_date.year())
                .collect();
            let mut v: Vec<i32> = set.drain().collect();
            v.sort_unstable();
            v
        };

        let balances_by_year_and_employee = self
            .leave_balance_repo
            .get_employee_balances_by_years(auth_token, &employee_ids, &years)
            .await?;

        // ── 5. Enrich & build proration previews (pure in-memory) ───────
        let mut valid_rows = Vec::new();

        for row in parsed_rows {
            let Some(employee) = employees_by_code.get(&row.normalized_code) else {
                invalid_rows.push(InvalidTerminationRow {
                    row_number: row.row_number,
                    employee_code: Some(row.raw_code.clone()),
                    raw_termination_date: Some(row.raw_date.clone()),
                    reason: crate::utils::i18n::translate(
                        &crate::utils::i18n::get_locale(),
                        "error.employee_not_found",
                    ),
                });
                continue;
            };

            let employee_employments = employments_by_employee_id
                .get(&employee.id())
                .cloned()
                .unwrap_or_default();

            // Resolve active employment: latest non-"terminated", preferring
            // most-recent start_date when multiple candidates exist.
            let mut warnings = Vec::new();
            let active_employment = {
                let mut candidates: Vec<_> = employee_employments
                    .clone()
                    .into_iter()
                    .filter(|emp| emp.status() != "terminated")
                    .collect();
                candidates.sort_by(|a, b| b.start_date().cmp(&a.start_date()));
                if candidates.len() > 1 {
                    let count = candidates.len().to_string();
                    let start_date = candidates[0].start_date().format("%d/%m/%Y").to_string();
                    warnings.push(crate::utils::i18n::t_args(
                        "termination.multiple_non_terminated_warning",
                        &[("count", &count), ("start_date", &start_date)],
                    ));
                }
                candidates.into_iter().next()
            };

            let Some(active_employment) = active_employment else {
                let error_key = if employee_employments.is_empty() {
                    "error.no_active_employment"
                } else {
                    "error.employee_already_terminated"
                };
                invalid_rows.push(InvalidTerminationRow {
                    row_number: row.row_number,
                    employee_code: Some(row.raw_code.clone()),
                    raw_termination_date: Some(row.raw_date.clone()),
                    reason: crate::utils::i18n::translate(
                        &crate::utils::i18n::get_locale(),
                        error_key,
                    ),
                });
                continue;
            };

            if row.termination_date < active_employment.start_date() {
                let start_date = active_employment
                    .start_date()
                    .format("%d/%m/%Y")
                    .to_string();
                let reason = crate::utils::i18n::t_args(
                    "error.termination_date_before_start",
                    &[("start_date", &start_date)],
                );
                invalid_rows.push(InvalidTerminationRow {
                    row_number: row.row_number,
                    employee_code: Some(row.raw_code.clone()),
                    raw_termination_date: Some(row.raw_date.clone()),
                    reason,
                });
                continue;
            }

            // Proration
            let year = row.termination_date.year();
            let prorated_leaves = Self::compute_proration(
                year,
                row.termination_date,
                active_employment.start_date(),
                &balances_by_year_and_employee,
                employee.id(),
            );

            valid_rows.push(ValidTerminationRow {
                row_number: row.row_number,
                employee_id: employee.id(),
                employee_code: row.raw_code.clone(),
                full_name: employee.full_name().to_string(),
                start_date: active_employment.start_date(),
                termination_date: row.termination_date,
                employment_id: active_employment.id(),
                prorated_leaves,
                termination_reason: row.termination_reason,
                termination_notes: row.termination_notes,
                warnings,
            });
        }

        let can_commit = !valid_rows.is_empty();

        Ok(TerminationPreviewResponse {
            valid_rows,
            invalid_rows,
            can_commit,
        })
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Parse raw CSV mapped rows, validate field-level constraints,
    /// and separate into parsed rows, invalid rows, and gathered normalized codes.
    fn parse_and_validate_rows(
        mapped_rows: Vec<(usize, HashMap<String, String>)>,
    ) -> (Vec<ParsedRow>, Vec<InvalidTerminationRow>, Vec<String>) {
        let mut parsed_rows = Vec::new();
        let mut invalid_rows = Vec::new();
        let mut normalized_codes = Vec::new();
        let mut seen_codes = HashSet::new();

        for (line_number, mapped_row) in mapped_rows {
            let raw_code = mapped_row.get("employee_code").cloned().unwrap_or_default();
            let raw_date = mapped_row
                .get("termination_date")
                .cloned()
                .unwrap_or_default();
            let raw_reason = mapped_row
                .get("termination_reason")
                .cloned()
                .unwrap_or_default();
            let termination_notes = mapped_row
                .get("termination_notes")
                .cloned()
                .filter(|s| !s.trim().is_empty());

            // Empty employee code
            if raw_code.trim().is_empty() {
                invalid_rows.push(InvalidTerminationRow {
                    row_number: line_number,
                    employee_code: Some(raw_code),
                    raw_termination_date: Some(raw_date),
                    reason: crate::utils::i18n::translate(
                        &crate::utils::i18n::get_locale(),
                        "error.full_name_required",
                    ),
                });
                continue;
            }

            // Invalid date
            let Some(termination_date) = crate::utils::datetime::parse_date(&raw_date) else {
                invalid_rows.push(InvalidTerminationRow {
                    row_number: line_number,
                    employee_code: Some(raw_code.clone()),
                    raw_termination_date: Some(raw_date.clone()),
                    reason: crate::utils::i18n::translate(
                        &crate::utils::i18n::get_locale(),
                        "error.invalid_date_format",
                    ),
                });
                continue;
            };

            // Invalid reason
            let termination_reason = raw_reason.trim().to_lowercase();
            if !VALID_TERMINATION_REASONS.contains(&termination_reason.as_str()) {
                let valid = VALID_TERMINATION_REASONS.join(", ");
                let reason = crate::utils::i18n::t_args(
                    "error.termination_reason_invalid",
                    &[("reason", &raw_reason), ("valid", &valid)],
                );
                invalid_rows.push(InvalidTerminationRow {
                    row_number: line_number,
                    employee_code: Some(raw_code.clone()),
                    raw_termination_date: Some(raw_date.clone()),
                    reason,
                });
                continue;
            }

            // Duplicate code
            let normalized_code = crate::utils::normalize_employee_id(&raw_code);
            if !seen_codes.insert(normalized_code.clone()) {
                let reason = crate::utils::i18n::t("error.termination_duplicate_code");
                invalid_rows.push(InvalidTerminationRow {
                    row_number: line_number,
                    employee_code: Some(raw_code),
                    raw_termination_date: Some(raw_date),
                    reason,
                });
                continue;
            }
            normalized_codes.push(normalized_code.clone());

            parsed_rows.push(ParsedRow {
                row_number: line_number,
                raw_code,
                normalized_code,
                raw_date,
                termination_date,
                termination_reason,
                termination_notes,
            });
        }

        (parsed_rows, invalid_rows, normalized_codes)
    }

    /// Compute the proration preview for a single employee in a given year.
    fn compute_proration(
        year: i32,
        termination_date: chrono::NaiveDate,
        employment_start_date: chrono::NaiveDate,
        balances_by_year: &HashMap<i32, HashMap<Uuid, Vec<EmployeeLeaveBalance>>>,
        employee_id: Uuid,
    ) -> Vec<ProratedLeavePreview> {
        // Year boundaries are infallible for years in the Gregorian calendar,
        // but guard against edge cases gracefully.
        let Some(start_of_year) = chrono::NaiveDate::from_ymd_opt(year, 1, 1) else {
            return vec![];
        };
        let Some(end_of_year) = chrono::NaiveDate::from_ymd_opt(year, 12, 31) else {
            return vec![];
        };

        let eff_start = std::cmp::max(start_of_year, employment_start_date);
        let days_worked = (termination_date - eff_start).num_days() + 1;
        let total_days = (end_of_year - start_of_year).num_days() + 1;
        #[allow(clippy::cast_precision_loss)]
        let proration_fraction = days_worked as f64 / total_days as f64;

        let balances = balances_by_year
            .get(&year)
            .and_then(|m| m.get(&employee_id))
            .cloned()
            .unwrap_or_default();

        balances
            .into_iter()
            .map(|emp_bal| {
                #[allow(clippy::cast_possible_truncation)]
                let new_entitlement =
                    (f64::from(emp_bal.entitlement) * proration_fraction).round() as i32;
                ProratedLeavePreview {
                    leave_type_id: emp_bal.leave_type_id,
                    leave_type_code: emp_bal.leave_type_code,
                    leave_type_name: emp_bal.leave_type_name,
                    old_entitlement: emp_bal.entitlement,
                    new_entitlement,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    // ── Helpers ──────────────────────────────────────────────────────────

    fn mapped_row(
        line: usize,
        code: &str,
        date: &str,
        reason: &str,
        notes: Option<&str>,
    ) -> (usize, HashMap<String, String>) {
        let mut map = HashMap::new();
        map.insert("employee_code".to_string(), code.to_string());
        map.insert("termination_date".to_string(), date.to_string());
        map.insert("termination_reason".to_string(), reason.to_string());
        if let Some(n) = notes {
            map.insert("termination_notes".to_string(), n.to_string());
        }
        (line, map)
    }

    fn row(line: usize, code: &str, date: &str, reason: &str) -> (usize, HashMap<String, String>) {
        mapped_row(line, code, date, reason, None)
    }

    fn make_balance(id: Uuid, code: &str, name: &str, entitlement: i32) -> EmployeeLeaveBalance {
        EmployeeLeaveBalance {
            leave_type_id: id,
            leave_type_code: code.to_string(),
            leave_type_name: name.to_string(),
            is_paid: true,
            entitlement,
            accrued_amount: 0.0,
            carried_over: 0,
            used: 0,
            remaining: f64::from(entitlement),
        }
    }

    // ── parse_and_validate_rows ──────────────────────────────────────────

    #[test]
    fn parse_empty_rows() {
        let (parsed, invalid, codes) = TerminationService::parse_and_validate_rows(vec![]);
        assert!(parsed.is_empty());
        assert!(invalid.is_empty());
        assert!(codes.is_empty());
    }

    #[test]
    fn parse_empty_employee_code() {
        let rows = vec![row(1, "", "2026-06-30", "voluntary_resignation")];
        let (parsed, invalid, _) = TerminationService::parse_and_validate_rows(rows);
        assert!(parsed.is_empty());
        assert_eq!(invalid.len(), 1);
    }

    #[test]
    fn parse_invalid_date() {
        let rows = vec![row(1, "EMP001", "invalid-date", "voluntary_resignation")];
        let (parsed, invalid, _) = TerminationService::parse_and_validate_rows(rows);
        assert!(parsed.is_empty());
        assert_eq!(invalid.len(), 1);
    }

    #[test]
    fn parse_invalid_reason() {
        let rows = vec![row(1, "EMP001", "2026-06-30", "nonexistent_reason")];
        let (parsed, invalid, _) = TerminationService::parse_and_validate_rows(rows);
        assert!(parsed.is_empty());
        assert_eq!(invalid.len(), 1);
    }

    #[test]
    fn parse_duplicate_code() {
        let rows = vec![
            row(1, "EMP001", "2026-06-30", "voluntary_resignation"),
            row(2, "EMP001", "2026-07-15", "retirement"),
        ];
        let (parsed, invalid, codes) = TerminationService::parse_and_validate_rows(rows);
        assert_eq!(parsed.len(), 1);
        assert_eq!(invalid.len(), 1);
        assert_eq!(codes.len(), 1);
    }

    #[test]
    fn parse_all_eight_valid_reasons() {
        let reasons = [
            "voluntary_resignation",
            "involuntary_termination",
            "end_of_contract",
            "retirement",
            "mutual_agreement",
            "abandonment",
            "death",
            "other",
        ];
        let rows: Vec<_> = reasons
            .iter()
            .enumerate()
            .map(|(i, r)| row(i + 1, &format!("EMP{i:03}"), "2026-06-30", r))
            .collect();

        let (parsed, invalid, codes) = TerminationService::parse_and_validate_rows(rows);
        assert_eq!(parsed.len(), 8);
        assert!(invalid.is_empty());
        assert_eq!(codes.len(), 8);
    }

    #[test]
    fn parse_valid_single_row() {
        let rows = vec![row(1, "EMP001", "2026-06-30", "voluntary_resignation")];
        let (parsed, invalid, codes) = TerminationService::parse_and_validate_rows(rows);

        assert_eq!(parsed.len(), 1);
        assert!(invalid.is_empty());
        assert_eq!(codes.len(), 1);

        let p = &parsed[0];
        assert_eq!(p.row_number, 1);
        assert_eq!(p.raw_code, "EMP001");
        // normalize_employee_id extracts trailing digits; just verify non-empty
        assert!(!p.normalized_code.is_empty());
        assert_eq!(p.termination_reason, "voluntary_resignation");
        assert!(p.termination_notes.is_none());
    }

    #[test]
    fn parse_with_notes() {
        let rows = vec![mapped_row(
            1,
            "EMP001",
            "2026-06-30",
            "other",
            Some("Some notes"),
        )];
        let (parsed, invalid, _) = TerminationService::parse_and_validate_rows(rows);

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].termination_notes.as_deref(), Some("Some notes"));
        assert!(invalid.is_empty());
    }

    #[test]
    fn parse_empty_notes_become_none() {
        let rows = vec![mapped_row(1, "EMP001", "2026-06-30", "other", Some("   "))];
        let (parsed, _invalid, _) = TerminationService::parse_and_validate_rows(rows);

        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].termination_notes.is_none());
    }

    // ── compute_proration ─────────────────────────────────────────────────

    #[test]
    fn proration_full_year_equals_full_entitlement() {
        let lt = Uuid::new_v4();
        let emp = Uuid::new_v4();
        let term = chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let b = make_balance(lt, "AL", "Annual Leave", 12);
        let mut by_year = HashMap::new();
        let mut by_emp = HashMap::new();
        by_emp.insert(emp, vec![b]);
        by_year.insert(2026, by_emp);

        let result = TerminationService::compute_proration(2026, term, start, &by_year, emp);
        assert_eq!(result[0].new_entitlement, 12);
        assert_eq!(result[0].old_entitlement, 12);
    }

    #[test]
    fn proration_half_year() {
        let lt = Uuid::new_v4();
        let emp = Uuid::new_v4();
        let term = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
        let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let b = make_balance(lt, "AL", "Annual Leave", 12);
        let mut by_year = HashMap::new();
        let mut by_emp = HashMap::new();
        by_emp.insert(emp, vec![b]);
        by_year.insert(2026, by_emp);

        let result = TerminationService::compute_proration(2026, term, start, &by_year, emp);
        assert_eq!(result[0].new_entitlement, 6);
    }

    #[test]
    fn proration_multiple_leave_types() {
        let lt1 = Uuid::new_v4();
        let lt2 = Uuid::new_v4();
        let emp = Uuid::new_v4();
        let term = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut by_year = HashMap::new();
        let mut by_emp = HashMap::new();
        by_emp.insert(
            emp,
            vec![
                make_balance(lt1, "AL", "Annual Leave", 12),
                make_balance(lt2, "SL", "Sick Leave", 30),
            ],
        );
        by_year.insert(2026, by_emp);

        let result = TerminationService::compute_proration(2026, term, start, &by_year, emp);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].new_entitlement, 6);
        assert_eq!(result[1].new_entitlement, 15);
    }

    #[test]
    fn proration_no_balances() {
        let emp = Uuid::new_v4();
        let term = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let by_year: HashMap<i32, HashMap<Uuid, Vec<EmployeeLeaveBalance>>> = HashMap::new();

        let result = TerminationService::compute_proration(2026, term, start, &by_year, emp);
        assert!(result.is_empty());
    }

    #[test]
    fn proration_wrong_year_has_no_balances() {
        let emp = Uuid::new_v4();
        let term = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let b = make_balance(Uuid::new_v4(), "AL", "AL", 12);
        let mut by_year = HashMap::new();
        let mut by_emp = HashMap::new();
        by_emp.insert(emp, vec![b]);
        by_year.insert(2025, by_emp);

        let result = TerminationService::compute_proration(2026, term, start, &by_year, emp);
        assert!(result.is_empty());
    }
}

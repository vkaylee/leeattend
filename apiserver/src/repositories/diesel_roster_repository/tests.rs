use super::*;
use crate::diesel_pool::DieselPools;

#[test]
fn employment_range_is_clipped_to_report_window() {
    let report_start = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let report_end = NaiveDate::from_ymd_opt(2026, 7, 31).unwrap();

    assert_eq!(
        effective_employment_range(
            report_start,
            report_end,
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2026, 7, 14).unwrap()),
        ),
        Some((report_start, NaiveDate::from_ymd_opt(2026, 7, 14).unwrap()))
    );
    assert_eq!(
        effective_employment_range(
            report_start,
            report_end,
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
            None,
        ),
        Some((NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(), report_end))
    );
    assert_eq!(
        effective_employment_range(
            report_start,
            report_end,
            NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            None,
        ),
        None
    );
}

#[tokio::test]
async fn test_repo_initialization() {
    let test_db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/leeattend_test_db".to_string());

    let pools = DieselPools::for_test(&test_db_url);
    let _repo = DieselRosterRepository::new(pools);
}

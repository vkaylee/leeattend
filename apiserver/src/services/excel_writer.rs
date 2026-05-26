//! Excel Writer Service (Facade)
//!
//! Generates multi-sheet Excel files for attendance reports.
//! Delegates implementation to the `excel` module to maintain Single Responsibility Principle.

use chrono::NaiveDate;
use chrono_tz::Tz;
use rust_xlsxwriter::Workbook;

use crate::models::{AttendanceSummary, DailyAttendanceReport};
use crate::services::excel::{raw_details_generator, summary_generator, timesheet_generator};
use crate::services::import_messages::ExcelReportMessages;

/// Excel file content type for HTTP responses.
pub const EXCEL_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/// Excel writer for attendance reports.
pub struct ExcelWriter;

impl ExcelWriter {
    /// Generate Excel file for attendance summary report.
    pub fn write_summary(
        summaries: &[AttendanceSummary],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        msgs: &ExcelReportMessages,
    ) -> crate::errors::AppResult<Vec<u8>> {
        let mut workbook = Workbook::new();

        summary_generator::generate(&mut workbook, summaries, workspace_name, date_range, msgs)?;

        workbook
            .save_to_buffer()
            .map_err(|e| crate::errors::AppError::Internal(format!("Excel write error: {e}")))
    }

    /// Multi-sheet detail report: "Timesheet View" (matrix) + "Raw Details" (list).
    pub fn write_detail(
        reports: &[DailyAttendanceReport],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        timezone: &str,
        work_unit_minutes: i32,
        msgs: &ExcelReportMessages,
    ) -> crate::errors::AppResult<Vec<u8>> {
        let tz: Tz = crate::utils::tz::parse_workspace_tz(timezone);
        let mut workbook = Workbook::new();

        timesheet_generator::generate(
            &mut workbook,
            reports,
            workspace_name,
            date_range,
            tz,
            work_unit_minutes,
            msgs,
        )?;

        raw_details_generator::generate(
            &mut workbook,
            reports,
            workspace_name,
            date_range,
            tz,
            msgs,
        )?;

        workbook
            .save_to_buffer()
            .map_err(|e| crate::errors::AppError::Internal(format!("Excel write error: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::excel::utils::{
        ceil_units_to_hours, fmt_h, format_datetime, format_time, units_to_hours,
    };
    use chrono::{NaiveDate, NaiveTime};
    use uuid::Uuid;

    /// Helper: default English messages for test convenience.
    fn en_msgs() -> ExcelReportMessages {
        ExcelReportMessages::for_lang("en")
    }

    #[test]
    fn test_write_summary_empty() -> Result<(), Box<dyn std::error::Error>> {
        let result = ExcelWriter::write_summary(
            &[],
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).ok_or("Invalid start date")?,
                NaiveDate::from_ymd_opt(2026, 1, 31).ok_or("Invalid end date")?,
            ),
            &en_msgs(),
        );
        assert!(result.is_ok());
        let data = result?;
        assert!(data.len() > 100);
        assert_eq!(&data[0..2], b"PK");
        Ok(())
    }

    // --- units_to_hours / ceil_units_to_hours tests ---

    #[test]
    fn test_units_to_hours_basic() {
        assert!((units_to_hours(16, 30) - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_units_to_hours_zero_units() {
        assert!((units_to_hours(0, 30) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_units_to_hours_15min_unit() {
        assert!((units_to_hours(32, 15) - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ceil_units_to_hours_exact() {
        assert!((ceil_units_to_hours(480, 30) - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ceil_units_to_hours_rounds_up() {
        assert!((ceil_units_to_hours(475, 30) - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ceil_units_to_hours_zero() {
        assert!((ceil_units_to_hours(0, 30) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_write_summary_with_data() -> Result<(), Box<dyn std::error::Error>> {
        let summaries = vec![
            AttendanceSummary {
                employee_id: Uuid::new_v4(),
                employee_name: "Alice Smith".to_string(),
                employee_id_number: String::new(),
                total_std_units: 140,
                total_ot_units: 20,
                total_leave_days: 2,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 140.0,
                computed_ot_hours: 20.0,
            },
            AttendanceSummary {
                employee_id: Uuid::new_v4(),
                employee_name: "Bob Jones".to_string(),
                employee_id_number: String::new(),
                total_std_units: 152,
                total_ot_units: 0,
                total_leave_days: 4,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 152.0,
                computed_ot_hours: 0.0,
            },
        ];

        let result = ExcelWriter::write_summary(
            &summaries,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).ok_or("Invalid start date")?,
                NaiveDate::from_ymd_opt(2026, 1, 31).ok_or("Invalid end date")?,
            ),
            &en_msgs(),
        );
        assert!(result.is_ok());
        assert!(result?.len() > 100);
        Ok(())
    }

    #[test]
    fn test_write_detail_empty() -> Result<(), Box<dyn std::error::Error>> {
        let result = ExcelWriter::write_detail(
            &[],
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).ok_or("Invalid start date")?,
                NaiveDate::from_ymd_opt(2026, 1, 31).ok_or("Invalid end date")?,
            ),
            "UTC",
            30,
            &en_msgs(),
        );
        assert!(result.is_ok());
        let data = result?;
        assert!(data.len() > 100);
        assert_eq!(&data[0..2], b"PK");
        Ok(())
    }

    #[test]
    fn test_write_detail_with_data() -> Result<(), Box<dyn std::error::Error>> {
        let emp_alice = Uuid::new_v4();
        let reports = vec![
            DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("Invalid date")?,
                employee_id: emp_alice,
                employee_name: "Alice Smith".to_string(),
                employee_id_number: "EMP001".to_string(),
                shift_symbol: None,
                scheduled_start: Some(NaiveTime::from_hms_opt(8, 0, 0).ok_or("Invalid time")?),
                scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).ok_or("Invalid time")?),
                break_duration_minutes: Some(60),
                break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).ok_or("Invalid time")?),
                break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).ok_or("Invalid time")?),
                is_break_auto_deduct: Some(true),
                actual_punch_in: None,
                actual_punch_out: None,
                std_work_units: 16,
                ot_work_units: 2,
                ot_claim_status: Some("approved".to_string()),
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 8.0,
                computed_ot_hours: 0.0,
                status: crate::services::attendance_utils::AttendanceStatus::Present,
                is_holiday: false,
                is_unscheduled: false,
                holiday_name: None,
                generation_metadata: None,
            },
            DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 16).ok_or("Invalid date")?,
                employee_id: emp_alice,
                employee_name: "Alice Smith".to_string(),
                employee_id_number: "EMP001".to_string(),
                shift_symbol: None,
                scheduled_start: Some(NaiveTime::from_hms_opt(8, 0, 0).ok_or("Invalid time")?),
                scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).ok_or("Invalid time")?),
                break_duration_minutes: Some(60),
                break_start: None,
                break_end: None,
                is_break_auto_deduct: Some(true),
                actual_punch_in: None,
                actual_punch_out: None,
                std_work_units: 8,
                ot_work_units: 0,
                ot_claim_status: None,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 8.0,
                computed_ot_hours: 0.0,
                status: crate::services::attendance_utils::AttendanceStatus::Present,
                is_holiday: false,
                is_unscheduled: false,
                holiday_name: None,
                generation_metadata: None,
            },
        ];

        let result = ExcelWriter::write_detail(
            &reports,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("Invalid start date")?,
                NaiveDate::from_ymd_opt(2026, 1, 16).ok_or("Invalid end date")?,
            ),
            "Asia/Ho_Chi_Minh",
            30,
            &en_msgs(),
        );
        assert!(result.is_ok());
        let data = result?;
        assert!(data.len() > 100);
        assert_eq!(&data[0..2], b"PK");
        Ok(())
    }

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(None), "");
        let time = NaiveTime::from_hms_opt(9, 30, 0);
        assert_eq!(format_time(time), "09:30");
    }

    #[test]
    fn test_format_datetime() {
        assert_eq!(format_datetime(None, chrono_tz::UTC), "");
    }

    #[test]
    fn test_format_datetime_timezone_conversion() {
        use chrono::TimeZone;
        let utc_time = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 1, 0, 0).unwrap();
        let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
        let result = format_datetime(Some(utc_time), tz);
        assert_eq!(result, "08:00");
    }

    #[test]
    fn test_format_datetime_utc_unchanged() {
        use chrono::TimeZone;
        let utc_time = chrono::Utc
            .with_ymd_and_hms(2026, 1, 15, 14, 30, 0)
            .unwrap();
        let result = format_datetime(Some(utc_time), chrono_tz::UTC);
        assert_eq!(result, "14:30");
    }

    #[test]
    fn test_format_datetime_crosses_midnight() {
        use chrono::TimeZone;
        let utc_time = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 21, 0, 0).unwrap();
        let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
        let result = format_datetime(Some(utc_time), tz);
        assert_eq!(result, "04:00");
    }

    #[test]
    fn test_format_datetime_negative_offset() {
        use chrono::TimeZone;
        let utc_time = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 15, 0, 0).unwrap();
        let tz: chrono_tz::Tz = "US/Eastern".parse().unwrap();
        let result = format_datetime(Some(utc_time), tz);
        assert_eq!(result, "10:00");
    }

    #[test]
    fn test_write_detail_with_different_timezones() -> Result<(), Box<dyn std::error::Error>> {
        let reports = vec![DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("Invalid date")?,
            employee_id: Uuid::new_v4(),
            employee_name: "Test Employee".to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: 8,
            ot_work_units: 0,
            ot_claim_status: None,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 8.0,
            computed_ot_hours: 0.0,
            status: crate::services::attendance_utils::AttendanceStatus::Present,
            is_holiday: false,
            is_unscheduled: false,
            holiday_name: None,
            generation_metadata: None,
        }];

        let date_range = (
            NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("d")?,
            NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("d")?,
        );

        let result = ExcelWriter::write_detail(
            &reports,
            "W",
            date_range,
            "Asia/Ho_Chi_Minh",
            30,
            &en_msgs(),
        );
        assert!(result.is_ok());

        let result =
            ExcelWriter::write_detail(&reports, "W", date_range, "Invalid/TZ", 30, &en_msgs());
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_fmt_h_integer_value() {
        assert_eq!(fmt_h(0.0), "0");
        assert_eq!(fmt_h(8.0), "8");
        assert_eq!(fmt_h(1.0), "1");
    }

    #[test]
    fn test_fmt_h_decimal_value() {
        assert_eq!(fmt_h(0.5), "0.5");
        assert_eq!(fmt_h(1.5), "1.5");
        assert_eq!(fmt_h(8.5), "8.5");
    }

    #[test]
    fn test_std_hours_from_work_units() {
        assert!((units_to_hours(16, 30) - 8.0).abs() < f64::EPSILON);
    }
}

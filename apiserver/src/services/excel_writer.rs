//! Excel Writer Service (Facade)
//!
//! Generates multi-sheet Excel files for attendance reports.
//! Delegates implementation to the `excel` module to maintain Single Responsibility Principle.

use chrono::NaiveDate;
use chrono_tz::Tz;
use rust_xlsxwriter::Workbook;

use crate::models::{AttendanceSummary, DailyAttendanceReport};
use crate::services::excel::{
    guide_generator, raw_details_generator, summary_generator, timesheet_generator,
};
use crate::services::import_messages::ExcelReportMessages;

/// Excel file content type for HTTP responses.
pub const EXCEL_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

use uuid::Uuid;

/// Excel writer for attendance reports.
pub struct ExcelWriter;

impl ExcelWriter {
    /// Generate Excel file for attendance summary report.
    pub fn write_summary(
        summaries: &[AttendanceSummary],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        msgs: &ExcelReportMessages,
        custom_field_headers: &[String],
        custom_field_values_map: &std::collections::HashMap<Uuid, Vec<String>>,
    ) -> crate::errors::AppResult<Vec<u8>> {
        let mut workbook = Workbook::new();

        summary_generator::generate(
            &mut workbook,
            summaries,
            workspace_name,
            date_range,
            msgs,
            custom_field_headers,
            custom_field_values_map,
        )?;

        workbook
            .save_to_buffer()
            .map_err(|e| crate::errors::AppError::Internal(format!("Excel write error: {e}")))
    }

    /// Multi-sheet detail report: "Timesheet View" (matrix) + "Raw Details" (list).
    #[allow(clippy::too_many_arguments)]
    pub fn write_detail(
        reports: &[DailyAttendanceReport],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        timezone: &str,
        work_unit_minutes: i32,
        msgs: &ExcelReportMessages,
        custom_field_headers: &[String],
        custom_field_values_map: &std::collections::HashMap<Uuid, Vec<String>>,
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
            custom_field_headers,
            custom_field_values_map,
        )?;

        guide_generator::generate(&mut workbook, msgs)?;

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
            &[],
            &std::collections::HashMap::new(),
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
                employment_id: Uuid::new_v4(),
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
                employer_id: None,
                employer_name: None,
            },
            AttendanceSummary {
                employment_id: Uuid::new_v4(),
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
                employer_id: None,
                employer_name: None,
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
            &[],
            &std::collections::HashMap::new(),
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
            &[],
            &std::collections::HashMap::new(),
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
                employment_id: emp_alice,
                employee_id: emp_alice,
                employee_name: "Alice Smith".to_string(),
                employee_id_number: "EMP001".to_string(),
                shift_symbol: None,
                shift_name: None,
                scheduled_start: Some(NaiveTime::from_hms_opt(8, 0, 0).ok_or("Invalid time")?),
                scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).ok_or("Invalid time")?),
                grace_period_minutes: None,
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
                is_leave_paid: None,
                employer_name: None,
                timezone: None,
                segment_id: None,
                shift_id: None,
                interval_id: None,
                interval_name: None,
                segment_type: None,
                sort_order: None,
                segment_count: None,
                break_split: None,
            },
            DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 16).ok_or("Invalid date")?,
                employment_id: emp_alice,
                employee_id: emp_alice,
                employee_name: "Alice Smith".to_string(),
                employee_id_number: "EMP001".to_string(),
                shift_symbol: None,
                shift_name: None,
                scheduled_start: Some(NaiveTime::from_hms_opt(8, 0, 0).ok_or("Invalid time")?),
                scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).ok_or("Invalid time")?),
                grace_period_minutes: None,
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
                is_leave_paid: None,
                employer_name: None,
                timezone: None,
                segment_id: None,
                shift_id: None,
                interval_id: None,
                interval_name: None,
                segment_type: None,
                sort_order: None,
                segment_count: None,
                break_split: None,
            },
            DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 17).ok_or("Invalid date")?,
                employment_id: emp_alice,
                employee_id: emp_alice,
                employee_name: "Alice Smith".to_string(),
                employee_id_number: "EMP001".to_string(),
                shift_symbol: Some("HC".to_string()),
                shift_name: Some("Day Shift".to_string()),
                scheduled_start: Some(NaiveTime::from_hms_opt(8, 0, 0).ok_or("Invalid time")?),
                scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).ok_or("Invalid time")?),
                grace_period_minutes: None,
                break_duration_minutes: Some(60),
                break_start: None,
                break_end: None,
                is_break_auto_deduct: Some(true),
                actual_punch_in: None,
                actual_punch_out: None,
                std_work_units: 0,
                ot_work_units: 0,
                ot_claim_status: None,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 0.0,
                computed_ot_hours: 0.0,
                status: crate::services::attendance_utils::AttendanceStatus::Leave,
                is_holiday: false,
                is_unscheduled: false,
                holiday_name: None,
                generation_metadata: None,
                is_leave_paid: Some(true),
                employer_name: None,
                timezone: None,
                segment_id: None,
                shift_id: None,
                interval_id: None,
                interval_name: None,
                segment_type: None,
                sort_order: None,
                segment_count: None,
                break_split: None,
            },
            DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 18).ok_or("Invalid date")?,
                employment_id: emp_alice,
                employee_id: emp_alice,
                employee_name: "Alice Smith".to_string(),
                employee_id_number: "EMP001".to_string(),
                shift_symbol: Some("HC".to_string()),
                shift_name: Some("Day Shift".to_string()),
                scheduled_start: Some(NaiveTime::from_hms_opt(8, 0, 0).ok_or("Invalid time")?),
                scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).ok_or("Invalid time")?),
                grace_period_minutes: None,
                break_duration_minutes: Some(60),
                break_start: None,
                break_end: None,
                is_break_auto_deduct: Some(true),
                actual_punch_in: None,
                actual_punch_out: None,
                std_work_units: 0,
                ot_work_units: 0,
                ot_claim_status: None,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 0.0,
                computed_ot_hours: 0.0,
                status: crate::services::attendance_utils::AttendanceStatus::Leave,
                is_holiday: false,
                is_unscheduled: false,
                holiday_name: None,
                generation_metadata: None,
                is_leave_paid: Some(false),
                employer_name: None,
                timezone: None,
                segment_id: None,
                shift_id: None,
                interval_id: None,
                interval_name: None,
                segment_type: None,
                sort_order: None,
                segment_count: None,
                break_split: None,
            },
        ];

        let result = ExcelWriter::write_detail(
            &reports,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("Invalid start date")?,
                NaiveDate::from_ymd_opt(2026, 1, 18).ok_or("Invalid end date")?,
            ),
            "Asia/Ho_Chi_Minh",
            30,
            &en_msgs(),
            &[],
            &std::collections::HashMap::new(),
        );
        assert!(result.is_ok());
        let data = result?;
        assert!(data.len() > 100);
        assert_eq!(&data[0..2], b"PK");
        Ok(())
    }

    #[test]
    fn test_write_detail_with_holidays() -> Result<(), Box<dyn std::error::Error>> {
        let emp_id = Uuid::new_v4();
        let reports = vec![
            DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 1).ok_or("Invalid date")?,
                employment_id: emp_id,
                employee_id: emp_id,
                employee_name: "John Holiday".to_string(),
                employee_id_number: "EMP002".to_string(),
                shift_symbol: Some("N".to_string()),
                shift_name: Some("Normal Shift".to_string()),
                scheduled_start: Some(NaiveTime::from_hms_opt(8, 0, 0).ok_or("Invalid time")?),
                scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).ok_or("Invalid time")?),
                grace_period_minutes: None,
                break_duration_minutes: Some(60),
                break_start: None,
                break_end: None,
                is_break_auto_deduct: Some(true),
                actual_punch_in: None,
                actual_punch_out: None,
                std_work_units: 0,
                ot_work_units: 0,
                ot_claim_status: None,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 0.0,
                computed_ot_hours: 0.0,
                status: crate::services::attendance_utils::AttendanceStatus::HolidayPaid,
                is_holiday: true,
                is_unscheduled: false,
                holiday_name: Some("New Year's Day".to_string()),
                generation_metadata: None,
                is_leave_paid: None,
                employer_name: None,
                timezone: None,
                segment_id: None,
                shift_id: None,
                interval_id: None,
                interval_name: None,
                segment_type: None,
                sort_order: None,
                segment_count: None,
                break_split: None,
            },
            DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 2).ok_or("Invalid date")?,
                employment_id: emp_id,
                employee_id: emp_id,
                employee_name: "John Holiday".to_string(),
                employee_id_number: "EMP002".to_string(),
                shift_symbol: Some("N".to_string()),
                shift_name: Some("Normal Shift".to_string()),
                scheduled_start: Some(NaiveTime::from_hms_opt(8, 0, 0).ok_or("Invalid time")?),
                scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).ok_or("Invalid time")?),
                grace_period_minutes: None,
                break_duration_minutes: Some(60),
                break_start: None,
                break_end: None,
                is_break_auto_deduct: Some(true),
                actual_punch_in: None,
                actual_punch_out: None,
                std_work_units: 0,
                ot_work_units: 0,
                ot_claim_status: None,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 0.0,
                computed_ot_hours: 0.0,
                status: crate::services::attendance_utils::AttendanceStatus::HolidayUnpaid,
                is_holiday: true,
                is_unscheduled: false,
                holiday_name: Some("Unpaid Holiday".to_string()),
                generation_metadata: None,
                is_leave_paid: None,
                employer_name: None,
                timezone: None,
                segment_id: None,
                shift_id: None,
                interval_id: None,
                interval_name: None,
                segment_type: None,
                sort_order: None,
                segment_count: None,
                break_split: None,
            },
            DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 3).ok_or("Invalid date")?,
                employment_id: emp_id,
                employee_id: emp_id,
                employee_name: "John Holiday".to_string(),
                employee_id_number: "EMP002".to_string(),
                shift_symbol: Some("N".to_string()),
                shift_name: Some("Normal Shift".to_string()),
                scheduled_start: Some(NaiveTime::from_hms_opt(8, 0, 0).ok_or("Invalid time")?),
                scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).ok_or("Invalid time")?),
                grace_period_minutes: None,
                break_duration_minutes: Some(60),
                break_start: None,
                break_end: None,
                is_break_auto_deduct: Some(true),
                actual_punch_in: None,
                actual_punch_out: None,
                std_work_units: 16,
                ot_work_units: 4,
                ot_claim_status: Some("approved".to_string()),
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 8.0,
                computed_ot_hours: 2.0,
                status: crate::services::attendance_utils::AttendanceStatus::HolidayWork,
                is_holiday: true,
                is_unscheduled: false,
                holiday_name: Some("New Year Working".to_string()),
                generation_metadata: None,
                is_leave_paid: None,
                employer_name: None,
                timezone: None,
                segment_id: None,
                shift_id: None,
                interval_id: None,
                interval_name: None,
                segment_type: None,
                sort_order: None,
                segment_count: None,
                break_split: None,
            },
        ];

        let result = ExcelWriter::write_detail(
            &reports,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).ok_or("Invalid start date")?,
                NaiveDate::from_ymd_opt(2026, 1, 3).ok_or("Invalid end date")?,
            ),
            "Asia/Ho_Chi_Minh",
            30,
            &en_msgs(),
            &[],
            &std::collections::HashMap::new(),
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
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Test Employee".to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
            shift_name: None,
            scheduled_start: None,
            scheduled_end: None,
            grace_period_minutes: None,
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
            is_leave_paid: None,
            employer_name: None,
            timezone: None,
            segment_id: None,
            shift_id: None,
            interval_id: None,
            interval_name: None,
            segment_type: None,
            sort_order: None,
            segment_count: None,
            break_split: None,
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
            &[],
            &std::collections::HashMap::new(),
        );
        assert!(result.is_ok());

        let result = ExcelWriter::write_detail(
            &reports,
            "W",
            date_range,
            "Invalid/TZ",
            30,
            &en_msgs(),
            &[],
            &std::collections::HashMap::new(),
        );
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

    #[test]
    fn test_detail_sheet_hours_and_headers() -> Result<(), Box<dyn std::error::Error>> {
        use calamine::{Data, Reader, Xlsx};
        use std::io::Cursor;
        use uuid::Uuid;

        let reports = vec![DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "John Doe".to_string(),
            employee_id_number: "EMP101".to_string(),
            shift_symbol: Some("HC".to_string()),
            shift_name: Some("Day".to_string()),
            scheduled_start: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
            scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
            grace_period_minutes: None,
            break_duration_minutes: Some(60),
            break_start: None,
            break_end: None,
            is_break_auto_deduct: Some(true),
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: 16,
            ot_work_units: 3,
            ot_claim_status: Some("approved".to_string()),
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 8.0,
            computed_ot_hours: 1.5,
            status: crate::services::attendance_utils::AttendanceStatus::Present,
            is_holiday: false,
            is_unscheduled: false,
            holiday_name: None,
            generation_metadata: None,
            is_leave_paid: None,
            employer_name: Some("Google".to_string()),
            timezone: None,
            segment_id: None,
            shift_id: None,
            interval_id: None,
            interval_name: None,
            segment_type: None,
            sort_order: None,
            segment_count: None,
            break_split: None,
        }];

        let date_range = (
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        );

        // 1. Verify English report headers and numeric values
        let bytes_en = ExcelWriter::write_detail(
            &reports,
            "EN Workspace",
            date_range,
            "UTC",
            30,
            &ExcelReportMessages::for_lang("en"),
            &[],
            &std::collections::HashMap::new(),
        )?;

        let mut wb_en: Xlsx<_> = Xlsx::new(Cursor::new(&bytes_en))?;
        let sheet_en = wb_en.worksheet_range("Raw Details")?;

        // Assert header is present and columns 14 & 15 are "Std Hours" & "OT Hours"
        // (multi-punch columns In 2/Out 2/In 3/Out 3 occupy 10-13)
        // Header row in raw_details_generator.rs is at row 3 (0-indexed)
        let header_row: Vec<_> = sheet_en.rows().nth(3).unwrap().to_vec();
        assert_eq!(header_row[3], Data::String("Employer".to_string()));
        assert_eq!(header_row[14], Data::String("Std Hours".to_string()));
        assert_eq!(header_row[15], Data::String("OT Hours".to_string()));

        // Assert data row has correct float values
        // Data rows start at row 4
        let data_row: Vec<_> = sheet_en.rows().nth(4).unwrap().to_vec();
        assert_eq!(data_row[3], Data::String("Google".to_string()));
        assert_eq!(data_row[14], Data::Float(8.0));
        assert_eq!(data_row[15], Data::Float(1.5));

        // Assert timesheet headers and values
        let sheet_ts = wb_en.worksheet_range("Timesheet View")?;
        let ts_header: Vec<_> = sheet_ts.rows().nth(3).unwrap().to_vec();
        assert_eq!(ts_header[0], Data::String("Employee ID".to_string()));
        assert_eq!(ts_header[1], Data::String("Employee".to_string()));
        assert_eq!(ts_header[2], Data::String("Employer".to_string()));
        assert_eq!(ts_header[3], Data::String("15".to_string()));

        let ts_row: Vec<_> = sheet_ts.rows().nth(4).unwrap().to_vec();
        assert_eq!(ts_row[0], Data::String("EMP101".to_string()));
        assert_eq!(ts_row[1], Data::String("John Doe".to_string()));
        assert_eq!(ts_row[2], Data::String("Google".to_string()));
        assert_eq!(ts_row[3], Data::String("HC8/0/0/1.5".to_string()));

        // 2. Verify Vietnamese report headers (should remain translated)
        let bytes_vi = ExcelWriter::write_detail(
            &reports,
            "VI Workspace",
            date_range,
            "Asia/Ho_Chi_Minh",
            30,
            &ExcelReportMessages::for_lang("vi"),
            &[],
            &std::collections::HashMap::new(),
        )?;

        let mut wb_vi: Xlsx<_> = Xlsx::new(Cursor::new(&bytes_vi))?;
        // Vietnamese sheet name is "Chi tiết"
        let sheet_vi = wb_vi.worksheet_range("Chi tiết")?;
        let header_row_vi: Vec<_> = sheet_vi.rows().nth(3).unwrap().to_vec();
        assert_eq!(header_row_vi[3], Data::String("Công ty".to_string()));
        assert_eq!(header_row_vi[14], Data::String("Giờ chuẩn".to_string()));
        assert_eq!(header_row_vi[15], Data::String("Giờ tăng ca".to_string()));

        let data_row_vi: Vec<_> = sheet_vi.rows().nth(4).unwrap().to_vec();
        assert_eq!(data_row_vi[3], Data::String("Google".to_string()));
        assert_eq!(data_row_vi[14], Data::Float(8.0));
        assert_eq!(data_row_vi[15], Data::Float(1.5));

        // Assert timesheet headers in Vietnamese
        let sheet_ts_vi = wb_vi.worksheet_range("Bảng chấm công")?;
        let ts_header_vi: Vec<_> = sheet_ts_vi.rows().nth(3).unwrap().to_vec();
        assert_eq!(ts_header_vi[0], Data::String("Mã NV".to_string()));
        assert_eq!(ts_header_vi[1], Data::String("Nhân viên".to_string()));
        assert_eq!(ts_header_vi[2], Data::String("Công ty".to_string()));

        // 3. Verify Chinese report headers (should remain translated)
        let bytes_zh = ExcelWriter::write_detail(
            &reports,
            "ZH Workspace",
            date_range,
            "Asia/Shanghai",
            30,
            &ExcelReportMessages::for_lang("zh"),
            &[],
            &std::collections::HashMap::new(),
        )?;

        let mut wb_zh: Xlsx<_> = Xlsx::new(Cursor::new(&bytes_zh))?;
        // Chinese sheet name is "明细"
        let sheet_zh = wb_zh.worksheet_range("明细")?;
        let header_row_zh: Vec<_> = sheet_zh.rows().nth(3).unwrap().to_vec();
        assert_eq!(header_row_zh[3], Data::String("雇主".to_string()));
        assert_eq!(header_row_zh[14], Data::String("标准工时".to_string()));
        assert_eq!(header_row_zh[15], Data::String("加班工时".to_string()));

        // Assert timesheet headers in Chinese
        let sheet_ts_zh = wb_zh.worksheet_range("考勤表")?;
        let ts_header_zh: Vec<_> = sheet_ts_zh.rows().nth(3).unwrap().to_vec();
        assert_eq!(ts_header_zh[0], Data::String("员工编号".to_string()));
        assert_eq!(ts_header_zh[1], Data::String("员工".to_string()));
        assert_eq!(ts_header_zh[2], Data::String("雇主".to_string()));

        Ok(())
    }

    #[test]
    fn test_summary_sheet_hours_and_headers() -> Result<(), Box<dyn std::error::Error>> {
        use calamine::{Data, Reader, Xlsx};
        use std::io::Cursor;
        use uuid::Uuid;

        let summaries = vec![AttendanceSummary {
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Alice Smith".to_string(),
            employee_id_number: "EMP001".to_string(),
            total_std_units: 160,
            total_ot_units: 20,
            total_leave_days: 2,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 80.0,
            computed_ot_hours: 10.0,
            employer_id: None,
            employer_name: None,
        }];

        let date_range = (
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        );

        // 1. English Summary
        let bytes_en = ExcelWriter::write_summary(
            &summaries,
            "EN Workspace",
            date_range,
            &ExcelReportMessages::for_lang("en"),
            &[],
            &std::collections::HashMap::new(),
        )?;

        let mut wb_en: Xlsx<_> = Xlsx::new(Cursor::new(&bytes_en))?;
        let sheet_en = wb_en.worksheet_range("Summary")?;

        // Header row is row 3
        let header_row: Vec<_> = sheet_en.rows().nth(3).unwrap().to_vec();
        assert_eq!(header_row[2], Data::String("Std Hours".to_string()));
        assert_eq!(header_row[3], Data::String("OT Hours".to_string()));

        // Data row is row 4
        let data_row: Vec<_> = sheet_en.rows().nth(4).unwrap().to_vec();
        assert_eq!(data_row[2], Data::Float(80.0));
        assert_eq!(data_row[3], Data::Float(10.0));

        // 2. Vietnamese Summary
        let bytes_vi = ExcelWriter::write_summary(
            &summaries,
            "VI Workspace",
            date_range,
            &ExcelReportMessages::for_lang("vi"),
            &[],
            &std::collections::HashMap::new(),
        )?;

        let mut wb_vi: Xlsx<_> = Xlsx::new(Cursor::new(&bytes_vi))?;
        // Vietnamese sheet name is "Tổng hợp"
        let sheet_vi = wb_vi.worksheet_range("Tổng hợp")?;

        let header_row_vi: Vec<_> = sheet_vi.rows().nth(3).unwrap().to_vec();
        assert_eq!(header_row_vi[2], Data::String("Giờ công chuẩn".to_string()));
        assert_eq!(header_row_vi[3], Data::String("Giờ tăng ca".to_string()));

        let data_row_vi: Vec<_> = sheet_vi.rows().nth(4).unwrap().to_vec();
        assert_eq!(data_row_vi[2], Data::Float(80.0));
        assert_eq!(data_row_vi[3], Data::Float(10.0));

        Ok(())
    }
}

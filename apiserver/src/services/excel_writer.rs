//! Excel Writer Service
//!
//! Generates multi-sheet Excel files for attendance reports.
//! - Summary: single "Summary" sheet.
//! - Detail: "Timesheet View" (matrix) + "Raw Details" (list) sheets.

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};
use std::collections::BTreeMap;

use crate::models::{AttendanceSummary, DailyAttendanceReport};

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
    ) -> Result<Vec<u8>, String> {
        let mut workbook = Workbook::new();
        let worksheet = workbook
            .add_worksheet()
            .set_name("Summary")
            .map_err(|e| e.to_string())?;

        let title_format = Format::new()
            .set_bold()
            .set_font_size(14)
            .set_font_color(Color::RGB(0x1F4E79));

        let subtitle_format = Format::new()
            .set_italic()
            .set_font_color(Color::RGB(0x5B5B5B));

        let header_format = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::RGB(0xFFFFFF));

        let number_format = Format::new().set_align(FormatAlign::Right);

        worksheet
            .write_string_with_format(
                0,
                0,
                format!("Attendance Summary Report - {workspace_name}"),
                &title_format,
            )
            .map_err(|e| e.to_string())?;

        worksheet
            .write_string_with_format(
                1,
                0,
                format!("Period: {} to {}", date_range.0, date_range.1),
                &subtitle_format,
            )
            .map_err(|e| e.to_string())?;

        let headers = ["Employee Name", "Total Work Units", "Total Leave Days"];
        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(3, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        for (idx, summary) in summaries.iter().enumerate() {
            let row = (idx + 4) as u32;
            worksheet
                .write_string(row, 0, &summary.employee_name)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number_with_format(row, 1, summary.total_work_units as f64, &number_format)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number_with_format(row, 2, summary.total_leave_days as f64, &number_format)
                .map_err(|e| e.to_string())?;
        }

        worksheet
            .set_column_width(0, 35)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(1, 18)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(2, 18)
            .map_err(|e| e.to_string())?;

        workbook
            .save_to_buffer()
            .map_err(|e| format!("Excel write error: {e}"))
    }

    /// Multi-sheet detail report: "Timesheet View" (matrix) + "Raw Details" (list).
    pub fn write_detail(
        reports: &[DailyAttendanceReport],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        timezone: &str,
    ) -> Result<Vec<u8>, String> {
        let tz: Tz = timezone
            .parse()
            .unwrap_or(chrono_tz::UTC);
        let mut workbook = Workbook::new();
        Self::write_timesheet_matrix(&mut workbook, reports, workspace_name, date_range)?;
        Self::write_raw_details(&mut workbook, reports, workspace_name, date_range, tz)?;
        workbook
            .save_to_buffer()
            .map_err(|e| format!("Excel write error: {e}"))
    }

    /// Sheet 1 "Timesheet View": Matrix — rows = employees (sorted), cols = days.
    /// Cell format: "Std / OT". OT > 0 cells get light yellow background.
    /// Freeze panes: header row + employee name column.
    fn write_timesheet_matrix(
        workbook: &mut Workbook,
        reports: &[DailyAttendanceReport],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
    ) -> Result<(), String> {
        let worksheet = workbook
            .add_worksheet()
            .set_name("Timesheet View")
            .map_err(|e| e.to_string())?;

        let title_fmt = Format::new()
            .set_bold()
            .set_font_size(14)
            .set_font_color(Color::RGB(0x1F4E79));
        let subtitle_fmt = Format::new()
            .set_italic()
            .set_font_color(Color::RGB(0x5B5B5B));
        let header_fmt = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::RGB(0xFFFFFF));
        let cell_fmt = Format::new().set_align(FormatAlign::Center);
        let ot_fmt = Format::new()
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0xFFF9C4)); // light yellow

        // Title (row 0) and subtitle (row 1)
        worksheet
            .write_string_with_format(
                0,
                0,
                format!("Timesheet View - {workspace_name}"),
                &title_fmt,
            )
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(
                1,
                0,
                format!("Period: {} to {}", date_range.0, date_range.1),
                &subtitle_fmt,
            )
            .map_err(|e| e.to_string())?;

        // Date columns: start_date .. end_date inclusive
        let mut dates: Vec<NaiveDate> = Vec::new();
        let mut cur = date_range.0;
        while cur <= date_range.1 {
            dates.push(cur);
            cur += Duration::days(1);
        }

        // Group by employee name (BTreeMap keys are sorted)
        let mut emp_days: BTreeMap<&str, BTreeMap<NaiveDate, (i32, i32)>> = BTreeMap::new();
        for r in reports {
            emp_days
                .entry(r.employee_name.as_str())
                .or_default()
                .insert(r.date, (r.std_work_units, r.ot_work_units));
        }

        // Header row (row 3): "Employee" then day numbers
        worksheet
            .write_string_with_format(3, 0, "Employee", &header_fmt)
            .map_err(|e| e.to_string())?;
        for (i, date) in dates.iter().enumerate() {
            worksheet
                .write_string_with_format(3, (i + 1) as u16, date.day().to_string(), &header_fmt)
                .map_err(|e| e.to_string())?;
        }

        // Data rows (row 4+)
        for (row_idx, (name, day_map)) in emp_days.iter().enumerate() {
            let row = (row_idx + 4) as u32;
            worksheet
                .write_string(row, 0, *name)
                .map_err(|e| e.to_string())?;

            for (day_idx, date) in dates.iter().enumerate() {
                if let Some(&(std_u, ot_u)) = day_map.get(date) {
                    let text = format!("{std_u} / {ot_u}");
                    let fmt = if ot_u > 0 { &ot_fmt } else { &cell_fmt };
                    worksheet
                        .write_string_with_format(row, (day_idx + 1) as u16, &text, fmt)
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        // Column widths
        worksheet
            .set_column_width(0, 35)
            .map_err(|e| e.to_string())?;
        for i in 1..=(dates.len() as u16) {
            worksheet
                .set_column_width(i, 8)
                .map_err(|e| e.to_string())?;
        }

        // Freeze panes: row 4 (first data), col 1 (first day col)
        let _ = worksheet.set_freeze_panes(4, 1);

        Ok(())
    }

    /// Sheet 2 "Raw Details": flat list with Std/OT split columns.
    fn write_raw_details(
        workbook: &mut Workbook,
        reports: &[DailyAttendanceReport],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        tz: Tz,
    ) -> Result<(), String> {
        let worksheet = workbook
            .add_worksheet()
            .set_name("Raw Details")
            .map_err(|e| e.to_string())?;

        let title_fmt = Format::new()
            .set_bold()
            .set_font_size(14)
            .set_font_color(Color::RGB(0x1F4E79));
        let subtitle_fmt = Format::new()
            .set_italic()
            .set_font_color(Color::RGB(0x5B5B5B));
        let header_fmt = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::RGB(0xFFFFFF));
        let time_fmt = Format::new().set_align(FormatAlign::Center);
        let status_present = Format::new()
            .set_font_color(Color::RGB(0x2E7D32))
            .set_align(FormatAlign::Center);
        let status_late = Format::new()
            .set_font_color(Color::RGB(0xF57C00))
            .set_align(FormatAlign::Center);
        let status_absent = Format::new()
            .set_font_color(Color::RGB(0xD32F2F))
            .set_align(FormatAlign::Center);
        let status_off = Format::new()
            .set_font_color(Color::RGB(0x757575))
            .set_align(FormatAlign::Center);

        worksheet
            .write_string_with_format(0, 0, format!("Raw Details - {workspace_name}"), &title_fmt)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(
                1,
                0,
                format!("Period: {} to {}", date_range.0, date_range.1),
                &subtitle_fmt,
            )
            .map_err(|e| e.to_string())?;

        // Headers (row 3)
        let headers = [
            "Date",
            "Employee Name",
            "Scheduled Start",
            "Scheduled End",
            "Actual In",
            "Actual Out",
            "Std Units",
            "OT Units",
            "OT Status",
            "Status",
        ];
        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(3, col as u16, *header, &header_fmt)
                .map_err(|e| e.to_string())?;
        }

        // Data rows (row 4+)
        for (idx, report) in reports.iter().enumerate() {
            let row = (idx + 4) as u32;
            worksheet
                .write_string(row, 0, report.date.to_string())
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 1, &report.employee_name)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string_with_format(row, 2, format_time(report.scheduled_start), &time_fmt)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string_with_format(row, 3, format_time(report.scheduled_end), &time_fmt)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string_with_format(
                    row,
                    4,
                    format_datetime(report.actual_punch_in, tz),
                    &time_fmt,
                )
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string_with_format(
                    row,
                    5,
                    format_datetime(report.actual_punch_out, tz),
                    &time_fmt,
                )
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 6, report.std_work_units as f64)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 7, report.ot_work_units as f64)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 8, report.ot_claim_status.as_deref().unwrap_or("-"))
                .map_err(|e| e.to_string())?;

            let status_fmt = match report.status.as_str() {
                "present" => &status_present,
                "late" => &status_late,
                "absent" => &status_absent,
                "off" => &status_off,
                _ => &time_fmt,
            };
            worksheet
                .write_string_with_format(row, 9, &report.status, status_fmt)
                .map_err(|e| e.to_string())?;
        }

        // Column widths
        worksheet
            .set_column_width(0, 12)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(1, 30)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(2, 15)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(3, 15)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(4, 12)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(5, 12)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(6, 12)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(7, 12)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(8, 12)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(9, 12)
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

/// Format optional NaiveTime to HH:MM string.
fn format_time(time: Option<NaiveTime>) -> String {
    time.map(|t| t.format("%H:%M").to_string())
        .unwrap_or_default()
}

/// Format optional DateTime to HH:MM string in the given timezone.
fn format_datetime(dt: Option<DateTime<Utc>>, tz: Tz) -> String {
    dt.map(|t| t.with_timezone(&tz).format("%H:%M").to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};
    use uuid::Uuid;

    #[test]
    fn test_write_summary_empty() -> Result<(), Box<dyn std::error::Error>> {
        let result = ExcelWriter::write_summary(
            &[],
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).ok_or("Invalid start date")?,
                NaiveDate::from_ymd_opt(2026, 1, 31).ok_or("Invalid end date")?,
            ),
        );
        assert!(result.is_ok());
        let data = result?;
        assert!(data.len() > 100);
        assert_eq!(&data[0..2], b"PK");
        Ok(())
    }

    #[test]
    fn test_write_summary_with_data() -> Result<(), Box<dyn std::error::Error>> {
        let summaries = vec![
            AttendanceSummary {
                employee_id: Uuid::new_v4(),
                employee_name: "Alice Smith".to_string(),
                total_work_units: 160,
                total_leave_days: 2,
            },
            AttendanceSummary {
                employee_id: Uuid::new_v4(),
                employee_name: "Bob Jones".to_string(),
                total_work_units: 152,
                total_leave_days: 4,
            },
        ];

        let result = ExcelWriter::write_summary(
            &summaries,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).ok_or("Invalid start date")?,
                NaiveDate::from_ymd_opt(2026, 1, 31).ok_or("Invalid end date")?,
            ),
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
                status: "present".to_string(),
                generation_metadata: None,
            },
            DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 16).ok_or("Invalid date")?,
                employee_id: emp_alice,
                employee_name: "Alice Smith".to_string(),
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
                status: "present".to_string(),
                generation_metadata: None,
            },
            DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("Invalid date")?,
                employee_id: Uuid::new_v4(),
                employee_name: "Bob Jones".to_string(),
                scheduled_start: Some(NaiveTime::from_hms_opt(9, 0, 0).ok_or("Invalid time")?),
                scheduled_end: Some(NaiveTime::from_hms_opt(18, 0, 0).ok_or("Invalid time")?),
                break_duration_minutes: Some(60),
                break_start: None,
                break_end: None,
                is_break_auto_deduct: Some(false),
                actual_punch_in: None,
                actual_punch_out: None,
                std_work_units: 8,
                ot_work_units: 3,
                ot_claim_status: Some("pending".to_string()),
                status: "present".to_string(),
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

    /// Optimization #12: format_datetime must convert UTC to workspace timezone.
    /// A user in Asia/Ho_Chi_Minh (UTC+7) punching in at UTC 01:00 should see 08:00.
    #[test]
    fn test_format_datetime_timezone_conversion() {
        use chrono::TimeZone;
        // UTC 01:00 → Ho_Chi_Minh 08:00 (UTC+7)
        let utc_time = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 1, 0, 0).unwrap();
        let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();

        let result = format_datetime(Some(utc_time), tz);
        assert_eq!(result, "08:00", "UTC 01:00 should display as 08:00 in UTC+7");
    }

    /// Verify UTC timezone produces unchanged time.
    #[test]
    fn test_format_datetime_utc_unchanged() {
        use chrono::TimeZone;
        let utc_time = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 14, 30, 0).unwrap();

        let result = format_datetime(Some(utc_time), chrono_tz::UTC);
        assert_eq!(result, "14:30");
    }

    /// Test timezone conversion that crosses midnight (day boundary).
    #[test]
    fn test_format_datetime_crosses_midnight() {
        use chrono::TimeZone;
        // UTC 21:00 → Ho_Chi_Minh 04:00 next day
        let utc_time = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 21, 0, 0).unwrap();
        let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();

        let result = format_datetime(Some(utc_time), tz);
        assert_eq!(result, "04:00", "UTC 21:00 should display as 04:00 in UTC+7");
    }

    /// Test negative offset timezone (US/Eastern = UTC-5).
    #[test]
    fn test_format_datetime_negative_offset() {
        use chrono::TimeZone;
        // UTC 15:00 → US/Eastern 10:00 (UTC-5)
        let utc_time = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 15, 0, 0).unwrap();
        let tz: chrono_tz::Tz = "US/Eastern".parse().unwrap();

        let result = format_datetime(Some(utc_time), tz);
        assert_eq!(result, "10:00", "UTC 15:00 should display as 10:00 in US/Eastern");
    }

    /// Test write_detail accepts timezone string and produces valid Excel.
    #[test]
    fn test_write_detail_with_different_timezones() -> Result<(), Box<dyn std::error::Error>> {
        let reports = vec![DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("Invalid date")?,
            employee_id: Uuid::new_v4(),
            employee_name: "Test Employee".to_string(),
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
            status: "present".to_string(),
            generation_metadata: None,
        }];

        let date_range = (
            NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("d")?,
            NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("d")?,
        );

        // Valid timezone
        let result = ExcelWriter::write_detail(&reports, "W", date_range, "Asia/Ho_Chi_Minh");
        assert!(result.is_ok());

        // Invalid timezone falls back to UTC (shouldn't panic)
        let result = ExcelWriter::write_detail(&reports, "W", date_range, "Invalid/TZ");
        assert!(result.is_ok());

        Ok(())
    }
}

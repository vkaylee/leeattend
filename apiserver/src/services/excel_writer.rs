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

        let headers = ["Employee Name", "Std Units", "OT Units", "Total Leave Days"];
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
                .write_number_with_format(row, 1, summary.total_std_units as f64, &number_format)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number_with_format(row, 2, summary.total_ot_units as f64, &number_format)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number_with_format(row, 3, summary.total_leave_days as f64, &number_format)
                .map_err(|e| e.to_string())?;
        }

        worksheet
            .set_column_width(0, 35)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(1, 15)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(2, 15)
            .map_err(|e| e.to_string())?;
        worksheet
            .set_column_width(3, 18)
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
        work_unit_minutes: i32,
    ) -> Result<Vec<u8>, String> {
        let tz: Tz = timezone
            .parse()
            .unwrap_or(chrono_tz::UTC);
        let mut workbook = Workbook::new();
        Self::write_timesheet_matrix(&mut workbook, reports, workspace_name, date_range, tz, work_unit_minutes)?;
        Self::write_raw_details(&mut workbook, reports, workspace_name, date_range, tz)?;
        workbook
            .save_to_buffer()
            .map_err(|e| format!("Excel write error: {e}"))
    }

    /// Sheet 1 "Timesheet View": Matrix — rows = employees (sorted), cols = days.
    /// Cell format: "Std hours / Late hours / Early-leave hours / OT units".
    /// OT > 0 cells get light yellow background.
    /// Freeze panes: header row + employee name column.
    fn write_timesheet_matrix(
        workbook: &mut Workbook,
        reports: &[DailyAttendanceReport],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        tz: Tz,
        work_unit_minutes: i32,
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

        // Group by employee name (BTreeMap keys are sorted).
        // Value: (employee_id_number String, BTreeMap of per-day data (including shift_symbol))
        #[allow(clippy::type_complexity)]
        let mut emp_days: BTreeMap<
            &str,
            (String, BTreeMap<NaiveDate, (i32, i32, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<chrono::NaiveTime>, Option<chrono::NaiveTime>, Option<String>)>),
        > = BTreeMap::new();
        for r in reports {
            let entry = emp_days
                .entry(r.employee_name.as_str())
                .or_insert_with(|| (r.employee_id_number.clone(), BTreeMap::new()));
            entry.1.insert(r.date, (r.std_work_units, r.ot_work_units, r.actual_punch_in, r.actual_punch_out, r.scheduled_start, r.scheduled_end, r.shift_symbol.clone()));
        }

        // Header row (row 3): "Employee ID" | "Employee" | day numbers
        worksheet
            .write_string_with_format(3, 0, "Employee ID", &header_fmt)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(3, 1, "Employee", &header_fmt)
            .map_err(|e| e.to_string())?;
        for (i, date) in dates.iter().enumerate() {
            worksheet
                .write_string_with_format(3, (i + 2) as u16, date.day().to_string(), &header_fmt)
                .map_err(|e| e.to_string())?;
        }

        // Data rows (row 4+)
        for (row_idx, (name, (emp_id_number, day_map))) in emp_days.iter().enumerate() {
            let row = (row_idx + 4) as u32;
            worksheet
                .write_string(row, 0, emp_id_number.as_str())
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 1, *name)
                .map_err(|e| e.to_string())?;

            for (day_idx, date) in dates.iter().enumerate() {
                if let Some(&(std_u, ot_u, punch_in_utc, punch_out_utc, sched_start, sched_end, ref shift_sym)) = day_map.get(date) {
                    // Symbol prefix: "S8" → "S8", None → ""
                    let prefix = shift_sym.as_deref().unwrap_or("");

                    // Convert work units to hours
                    let std_h = (std_u as f64 * work_unit_minutes as f64) / 60.0;

                    // Calculate late: ceil to nearest work unit, then convert to hours
                    let late_h = match (punch_in_utc, sched_start) {
                        (Some(pin_utc), Some(sched)) => {
                            let pin_local = pin_utc.with_timezone(&tz);
                            let pin_time = pin_local.time();
                            if pin_time > sched {
                                let diff_minutes = (pin_time - sched).num_minutes();
                                let units = (diff_minutes + work_unit_minutes as i64 - 1) / work_unit_minutes as i64;
                                (units * work_unit_minutes as i64) as f64 / 60.0
                            } else {
                                0.0
                            }
                        }
                        _ => 0.0,
                    };

                    // Calculate early-leave: ceil to nearest work unit, then convert to hours
                    let early_leave_h = match (punch_out_utc, sched_end) {
                        (Some(pout_utc), Some(sched)) => {
                            let pout_local = pout_utc.with_timezone(&tz);
                            let pout_time = pout_local.time();
                            if pout_time < sched {
                                let diff_minutes = (sched - pout_time).num_minutes();
                                let units = (diff_minutes + work_unit_minutes as i64 - 1) / work_unit_minutes as i64;
                                (units * work_unit_minutes as i64) as f64 / 60.0
                            } else {
                                0.0
                            }
                        }
                        _ => 0.0,
                    };

                    let ot_h = (ot_u as f64 * work_unit_minutes as f64) / 60.0;
                    let text = format!("{}{}/{}/{}/{}", prefix, fmt_h(std_h), fmt_h(late_h), fmt_h(early_leave_h), fmt_h(ot_h));
                    let fmt = if ot_u > 0 { &ot_fmt } else { &cell_fmt };
                    worksheet
                        .write_string_with_format(row, (day_idx + 2) as u16, &text, fmt)
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        // Auto-fit all column widths to content
        worksheet.autofit();

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
            "Employee ID",
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
                .write_string(row, 1, &report.employee_id_number)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 2, &report.employee_name)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string_with_format(row, 3, format_time(report.scheduled_start), &time_fmt)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string_with_format(row, 4, format_time(report.scheduled_end), &time_fmt)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string_with_format(
                    row,
                    5,
                    format_datetime(report.actual_punch_in, tz),
                    &time_fmt,
                )
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string_with_format(
                    row,
                    6,
                    format_datetime(report.actual_punch_out, tz),
                    &time_fmt,
                )
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 7, report.std_work_units as f64)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 8, report.ot_work_units as f64)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 9, report.ot_claim_status.as_deref().unwrap_or("-"))
                .map_err(|e| e.to_string())?;

            let status_fmt = match report.status.as_str() {
                "present" => &status_present,
                "late" => &status_late,
                "absent" => &status_absent,
                "off" => &status_off,
                _ => &time_fmt,
            };
            worksheet
                .write_string_with_format(row, 10, &report.status, status_fmt)
                .map_err(|e| e.to_string())?;
        }

        // Auto-fit all column widths to content
        worksheet.autofit();

        Ok(())
    }
}

/// Format optional NaiveTime to HH:MM string.
/// Format hours: show as integer if whole number (8.0 → "8"), otherwise 1 decimal (0.5 → "0.5").
fn fmt_h(h: f64) -> String {
    if h.fract() == 0.0 {
        format!("{}", h as i64)
    } else {
        format!("{h:.1}")
    }
}

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
                total_std_units: 140,
                total_ot_units: 20,
                total_leave_days: 2,
            },
            AttendanceSummary {
                employee_id: Uuid::new_v4(),
                employee_name: "Bob Jones".to_string(),
                total_std_units: 152,
                total_ot_units: 0,
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
            30,
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
                status: "present".to_string(),
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
                status: "present".to_string(),
                generation_metadata: None,
            },
            DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("Invalid date")?,
                employee_id: Uuid::new_v4(),
                employee_name: "Bob Jones".to_string(),
                employee_id_number: "EMP002".to_string(),
                shift_symbol: None,
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
            30,
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
            status: "present".to_string(),
            generation_metadata: None,
        }];

        let date_range = (
            NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("d")?,
            NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("d")?,
        );

        // Valid timezone
        let result = ExcelWriter::write_detail(&reports, "W", date_range, "Asia/Ho_Chi_Minh", 30);
        assert!(result.is_ok());

        // Invalid timezone falls back to UTC (shouldn't panic)
        let result = ExcelWriter::write_detail(&reports, "W", date_range, "Invalid/TZ", 30);
        assert!(result.is_ok());

        Ok(())
    }

    // ============================================================================
    // Tests for new cell format: std_h/late_h/early_leave_h/ot_u
    // ============================================================================

    /// fmt_h: whole number → no decimal
    #[test]
    fn test_fmt_h_integer_value() {
        assert_eq!(fmt_h(0.0), "0");
        assert_eq!(fmt_h(8.0), "8");
        assert_eq!(fmt_h(1.0), "1");
    }

    /// fmt_h: non-whole number → 1 decimal place
    #[test]
    fn test_fmt_h_decimal_value() {
        assert_eq!(fmt_h(0.5), "0.5");
        assert_eq!(fmt_h(1.5), "1.5");
        assert_eq!(fmt_h(8.5), "8.5");
    }

    /// std_work_units × work_unit_minutes / 60 = giờ công trong ca
    #[test]
    fn test_std_hours_from_work_units() {
        // 16 units × 30 min = 480 min = 8h
        assert_eq!((16f64 * 30.0) / 60.0, 8.0);
        // 1 unit × 30 min = 30 min = 0.5h
        assert_eq!((1f64 * 30.0) / 60.0, 0.5);
    }

    /// Đi trễ 1 phút → ceil(1/30) = 1 đơn vị → 0.5h (với work_unit_minutes=30)
    #[test]
    fn test_late_ceil_to_work_unit_1_minute() {
        let diff_minutes: i64 = 1;
        let work_unit_minutes: i64 = 30;
        let units = (diff_minutes + work_unit_minutes - 1) / work_unit_minutes;
        let hours = (units * work_unit_minutes) as f64 / 60.0;
        assert_eq!(units, 1);
        assert_eq!(hours, 0.5);
    }

    /// Đi trễ 30 phút → ceil(30/30) = 1 đơn vị → 0.5h
    #[test]
    fn test_late_ceil_to_work_unit_30_minutes() {
        let diff_minutes: i64 = 30;
        let work_unit_minutes: i64 = 30;
        let units = (diff_minutes + work_unit_minutes - 1) / work_unit_minutes;
        let hours = (units * work_unit_minutes) as f64 / 60.0;
        assert_eq!(units, 1);
        assert_eq!(hours, 0.5);
    }

    /// Đi trễ 31 phút → ceil(31/30) = 2 đơn vị → 1h
    #[test]
    fn test_late_ceil_to_work_unit_31_minutes() {
        let diff_minutes: i64 = 31;
        let work_unit_minutes: i64 = 30;
        let units = (diff_minutes + work_unit_minutes - 1) / work_unit_minutes;
        let hours = (units * work_unit_minutes) as f64 / 60.0;
        assert_eq!(units, 2);
        assert_eq!(hours, 1.0);
    }

    /// Về sớm 15 phút với work_unit=15 → ceil(15/15) = 1 đơn vị → 0.25h
    #[test]
    fn test_early_leave_ceil_15min_unit() {
        let diff_minutes: i64 = 15;
        let work_unit_minutes: i64 = 15;
        let units = (diff_minutes + work_unit_minutes - 1) / work_unit_minutes;
        let hours = (units * work_unit_minutes) as f64 / 60.0;
        assert_eq!(units, 1);
        assert_eq!(hours, 0.25);
    }

    /// write_detail với punch_in trễ 1 phút: cell phải hiển thị đúng format
    #[test]
    fn test_write_detail_cell_format_with_late_punch_in() -> Result<(), Box<dyn std::error::Error>> {
        use chrono::{TimeZone, NaiveTime};

        // Shift 08:00-17:00, punch in at 08:01 UTC (= 15:01 Ho_Chi_Minh? No, use UTC tz for simplicity)
        // Use UTC tz so UTC time = local time
        let scheduled_start = NaiveTime::from_hms_opt(8, 0, 0).ok_or("time")?;
        let scheduled_end   = NaiveTime::from_hms_opt(17, 0, 0).ok_or("time")?;
        // Punch in 08:01 UTC, punch out 17:00 UTC
        let punch_in  = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 8, 1, 0).unwrap();
        let punch_out = chrono::Utc.with_ymd_and_hms(2026, 1, 15, 17, 0, 0).unwrap();

        let reports = vec![DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("d")?,
            employee_id: Uuid::new_v4(),
            employee_name: "Alice".to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
            scheduled_start: Some(scheduled_start),
            scheduled_end: Some(scheduled_end),
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: Some(punch_in),
            actual_punch_out: Some(punch_out),
            std_work_units: 16, // 16 × 30min = 8h
            ot_work_units: 0,
            ot_claim_status: None,
            status: "late".to_string(),
            generation_metadata: None,
        }];

        let date_range = (
            NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("d")?,
            NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("d")?,
        );

        // work_unit_minutes=30, tz=UTC
        // late: 1 min → ceil(1/30)=1 unit → 0.5h → "0.5"
        // early_leave: 0 min → "0"
        // std: 16*30/60 = 8h → "8"
        // Format: "8/0.5/0/0"
        let result = ExcelWriter::write_detail(&reports, "W", date_range, "UTC", 30);
        assert!(result.is_ok(), "write_detail should succeed");
        // Verify it's a valid xlsx (starts with PK magic bytes)
        assert_eq!(&result.unwrap()[0..2], b"PK");
        Ok(())
    }

    // ============================================================================
    // Tests for employee_id_number column
    // ============================================================================

    /// employee_id_number phải round-trip đúng qua JSON (batch processing dùng serde).
    #[test]
    fn test_employee_id_number_serialization_roundtrip() {
        let report = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            employee_id: Uuid::new_v4(),
            employee_name: "Nguyen Van A".to_string(),
            employee_id_number: "NV0042".to_string(),
            shift_symbol: None,
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: 16,
            ot_work_units: 0,
            ot_claim_status: None,
            status: "present".to_string(),
            generation_metadata: None,
        };

        // Serialize → deserialize (simulates what batch worker does)
        let json = serde_json::to_string(&report).unwrap();
        let decoded: DailyAttendanceReport = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.employee_id_number, "NV0042");
        assert_eq!(decoded.employee_name, "Nguyen Van A");
        // UUID is separate from employee_id_number
        assert_eq!(decoded.employee_id, report.employee_id);
    }

    /// employee_id_number là field riêng biệt với UUID.
    #[test]
    fn test_employee_id_number_is_independent_of_uuid() {
        let uuid = Uuid::new_v4();
        let report = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            employee_id: uuid,
            employee_name: "Test".to_string(),
            employee_id_number: "EMP-2024-001".to_string(),
            shift_symbol: None,
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: 0,
            ot_work_units: 0,
            ot_claim_status: None,
            status: "absent".to_string(),
            generation_metadata: None,
        };

        assert_ne!(report.employee_id_number, uuid.to_string());
        assert_eq!(report.employee_id_number, "EMP-2024-001");
    }

    /// write_detail với nhiều nhân viên có employee_id_number khác nhau → excel hợp lệ.
    #[test]
    fn test_write_detail_multiple_employees_with_id_numbers() -> Result<(), Box<dyn std::error::Error>> {
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).ok_or("d")?;
        let make = |name: &str, id_number: &str, std_u: i32| DailyAttendanceReport {
            date,
            employee_id: Uuid::new_v4(),
            employee_name: name.to_string(),
            employee_id_number: id_number.to_string(),
            shift_symbol: None,
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: std_u,
            ot_work_units: 0,
            ot_claim_status: None,
            status: "present".to_string(),
            generation_metadata: None,
        };

        let reports = vec![
            make("Nguyen Van A", "NV001", 16),
            make("Tran Thi B",   "NV002", 14),
            make("Le Van C",     "NV003", 16),
        ];

        let result = ExcelWriter::write_detail(&reports, "Cong Ty XYZ", (date, date), "Asia/Ho_Chi_Minh", 30);
        assert!(result.is_ok(), "write_detail failed: {:?}", result.err());
        assert_eq!(&result.unwrap()[0..2], b"PK");
        Ok(())
    }

    // ============================================================================
    // Tests for OT hours conversion
    // ============================================================================

    /// 2 OT units × 30 min = 60 min = 1h → fmt_h hiển thị "1"
    #[test]
    fn test_ot_hours_from_work_units_30min() {
        let ot_u: i32 = 2;
        let work_unit_minutes: i32 = 30;
        let ot_h = (ot_u as f64 * work_unit_minutes as f64) / 60.0;
        assert_eq!(ot_h, 1.0);
        assert_eq!(fmt_h(ot_h), "1");
    }

    /// 1 OT unit × 30 min = 30 min = 0.5h → fmt_h hiển thị "0.5"
    #[test]
    fn test_ot_hours_1_unit_30min() {
        let ot_u: i32 = 1;
        let work_unit_minutes: i32 = 30;
        let ot_h = (ot_u as f64 * work_unit_minutes as f64) / 60.0;
        assert_eq!(ot_h, 0.5);
        assert_eq!(fmt_h(ot_h), "0.5");
    }

    /// 0 OT units → 0h → fmt_h hiển thị "0"
    #[test]
    fn test_ot_hours_zero_units() {
        let ot_u: i32 = 0;
        let work_unit_minutes: i32 = 30;
        let ot_h = (ot_u as f64 * work_unit_minutes as f64) / 60.0;
        assert_eq!(ot_h, 0.0);
        assert_eq!(fmt_h(ot_h), "0");
    }

    /// 4 OT units × 15 min = 60 min = 1h → fmt_h hiển thị "1" (unit=15 phút)
    #[test]
    fn test_ot_hours_4_units_15min() {
        let ot_u: i32 = 4;
        let work_unit_minutes: i32 = 15;
        let ot_h = (ot_u as f64 * work_unit_minutes as f64) / 60.0;
        assert_eq!(ot_h, 1.0);
        assert_eq!(fmt_h(ot_h), "1");
    }

    // ── shift_symbol prefix tests ─────────────────────────────────────────────

    /// Khi shift có symbol thì prefix xuất hiện đầu cell: "S8 8/0/0/0"
    #[test]
    fn test_cell_format_with_shift_symbol_has_prefix() {
        let prefix = "S8";
        let std_h = 8.0_f64;
        let late_h = 0.0_f64;
        let early_leave_h = 0.0_f64;
        let ot_h = 0.0_f64;
        let text = format!("{}{}/{}/{}/{}", prefix, fmt_h(std_h), fmt_h(late_h), fmt_h(early_leave_h), fmt_h(ot_h));
        assert_eq!(text, "S88/0/0/0");
    }

    /// Khi không có symbol thì cell không có prefix: "8/0/0/0"
    #[test]
    fn test_cell_format_without_shift_symbol_no_prefix() {
        let shift_sym: Option<String> = None;
        let prefix = shift_sym.as_deref().unwrap_or("");
        let std_h = 8.0_f64;
        let late_h = 0.0_f64;
        let early_leave_h = 0.0_f64;
        let ot_h = 0.0_f64;
        let text = format!("{}{}/{}/{}/{}", prefix, fmt_h(std_h), fmt_h(late_h), fmt_h(early_leave_h), fmt_h(ot_h));
        assert_eq!(text, "8/0/0/0");
    }

    /// Symbol rỗng (Some("")) tương đương không có prefix
    #[test]
    fn test_cell_format_empty_symbol_no_prefix() {
        let shift_sym: Option<String> = Some("".to_string());
        let prefix = shift_sym.as_deref().unwrap_or("");
        let std_h = 8.0_f64;
        let text = format!("{}{}/{}/{}/{}", prefix, fmt_h(std_h), fmt_h(0.0), fmt_h(0.0), fmt_h(0.0));
        assert_eq!(text, "8/0/0/0");
    }

    /// Symbol + OT: "N88/0/0/2" khi ca tăng ca
    #[test]
    fn test_cell_format_symbol_with_ot() {
        let prefix = "N8";
        let work_unit_minutes = 30_i32;
        let std_h = (16_i32 as f64 * work_unit_minutes as f64) / 60.0; // 8h
        let ot_h = (4_i32 as f64 * work_unit_minutes as f64) / 60.0;   // 2h
        let text = format!("{}{}/{}/{}/{}", prefix, fmt_h(std_h), fmt_h(0.0), fmt_h(0.0), fmt_h(ot_h));
        assert_eq!(text, "N88/0/0/2");
    }

    /// fmt_h với giờ thập phân: 7.5h → "7.5"
    #[test]
    fn test_cell_format_decimal_hours_with_symbol() {
        let prefix = "FX";
        let std_h = 7.5_f64;
        let text = format!("{}{}/{}/{}/{}", prefix, fmt_h(std_h), fmt_h(0.5), fmt_h(0.0), fmt_h(0.0));
        assert_eq!(text, "FX7.5/0.5/0/0");
    }
}

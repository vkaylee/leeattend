//! Excel Writer Service
//!
//! Generates Excel files for attendance reports with proper formatting.
//! Uses rust_xlsxwriter for streaming write to minimize memory usage.

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};

use crate::models::{AttendanceSummary, DailyAttendanceReport};

/// Excel file content type for HTTP responses.
pub const EXCEL_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/// Excel writer for attendance reports.
pub struct ExcelWriter;

impl ExcelWriter {
    /// Generate Excel file for attendance summary report.
    ///
    /// # Arguments
    /// * `summaries` - List of attendance summaries sorted by employee name
    /// * `workspace_name` - Name of the workspace for report title
    /// * `date_range` - (start_date, end_date) tuple for the report period
    ///
    /// # Returns
    /// Excel file as byte vector.
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

        // Title format
        let title_format = Format::new()
            .set_bold()
            .set_font_size(14)
            .set_font_color(Color::RGB(0x1F4E79));

        // Subtitle format
        let subtitle_format = Format::new()
            .set_italic()
            .set_font_color(Color::RGB(0x5B5B5B));

        // Header format
        let header_format = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::RGB(0xFFFFFF));

        // Number format for alignment
        let number_format = Format::new().set_align(FormatAlign::Right);

        // Write title (row 0)
        worksheet
            .write_string_with_format(
                0,
                0,
                &format!("Attendance Summary Report - {}", workspace_name),
                &title_format,
            )
            .map_err(|e| e.to_string())?;

        // Write date range (row 1)
        worksheet
            .write_string_with_format(
                1,
                0,
                &format!("Period: {} to {}", date_range.0, date_range.1),
                &subtitle_format,
            )
            .map_err(|e| e.to_string())?;

        // Headers (row 3)
        let headers = [
            "Employee Name",
            "Total Work Units",
            "Total Leave Days",
        ];
        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(3, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        // Data rows (starting row 4)
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

        // Set column widths
        worksheet.set_column_width(0, 35).map_err(|e| e.to_string())?;
        worksheet.set_column_width(1, 18).map_err(|e| e.to_string())?;
        worksheet.set_column_width(2, 18).map_err(|e| e.to_string())?;

        // Write to buffer
        let buffer = workbook
            .save_to_buffer()
            .map_err(|e| format!("Excel write error: {}", e))?;

        Ok(buffer)
    }

    /// Generate Excel file for daily attendance detail report.
    ///
    /// # Arguments
    /// * `reports` - List of daily reports sorted by employee name then date
    /// * `workspace_name` - Name of the workspace for report title
    /// * `date_range` - (start_date, end_date) tuple for the report period
    ///
    /// # Returns
    /// Excel file as byte vector.
    pub fn write_detail(
        reports: &[DailyAttendanceReport],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
    ) -> Result<Vec<u8>, String> {
        let mut workbook = Workbook::new();
        let worksheet = workbook
            .add_worksheet()
            .set_name("Detail")
            .map_err(|e| e.to_string())?;

        // Title format
        let title_format = Format::new()
            .set_bold()
            .set_font_size(14)
            .set_font_color(Color::RGB(0x1F4E79));

        // Subtitle format
        let subtitle_format = Format::new()
            .set_italic()
            .set_font_color(Color::RGB(0x5B5B5B));

        // Header format
        let header_format = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::RGB(0xFFFFFF));

        // Time format
        let time_format = Format::new().set_align(FormatAlign::Center);

        // Status formats
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

        // Write title (row 0)
        worksheet
            .write_string_with_format(
                0,
                0,
                &format!("Attendance Detail Report - {}", workspace_name),
                &title_format,
            )
            .map_err(|e| e.to_string())?;

        // Write date range (row 1)
        worksheet
            .write_string_with_format(
                1,
                0,
                &format!("Period: {} to {}", date_range.0, date_range.1),
                &subtitle_format,
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
            "Work Units",
            "Status",
        ];
        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(3, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        // Data rows (starting row 4)
        for (idx, report) in reports.iter().enumerate() {
            let row = (idx + 4) as u32;

            worksheet
                .write_string(row, 0, &report.date.to_string())
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string(row, 1, &report.employee_name)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 2, &format_time(report.scheduled_start), &time_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 3, &format_time(report.scheduled_end), &time_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 4, &format_datetime(report.actual_punch_in), &time_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_string_with_format(row, 5, &format_datetime(report.actual_punch_out), &time_format)
                .map_err(|e| e.to_string())?;

            worksheet
                .write_number(row, 6, report.work_units as f64)
                .map_err(|e| e.to_string())?;

            // Status with conditional formatting
            let status_fmt = match report.status.as_str() {
                "present" => &status_present,
                "late" => &status_late,
                "absent" => &status_absent,
                "off" => &status_off,
                _ => &time_format,
            };
            worksheet
                .write_string_with_format(row, 7, &report.status, status_fmt)
                .map_err(|e| e.to_string())?;
        }

        // Set column widths
        worksheet.set_column_width(0, 12).map_err(|e| e.to_string())?;
        worksheet.set_column_width(1, 30).map_err(|e| e.to_string())?;
        worksheet.set_column_width(2, 15).map_err(|e| e.to_string())?;
        worksheet.set_column_width(3, 15).map_err(|e| e.to_string())?;
        worksheet.set_column_width(4, 12).map_err(|e| e.to_string())?;
        worksheet.set_column_width(5, 12).map_err(|e| e.to_string())?;
        worksheet.set_column_width(6, 12).map_err(|e| e.to_string())?;
        worksheet.set_column_width(7, 12).map_err(|e| e.to_string())?;

        // Write to buffer
        let buffer = workbook
            .save_to_buffer()
            .map_err(|e| format!("Excel write error: {}", e))?;

        Ok(buffer)
    }
}

/// Format optional NaiveTime to HH:MM string.
fn format_time(time: Option<NaiveTime>) -> String {
    time.map(|t| t.format("%H:%M").to_string())
        .unwrap_or_default()
}

/// Format optional DateTime to HH:MM string.
fn format_datetime(dt: Option<DateTime<Utc>>) -> String {
    dt.map(|t| t.format("%H:%M").to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn test_write_summary_empty() {
        let result = ExcelWriter::write_summary(
            &[],
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
        );
        assert!(result.is_ok());
        // Check it's a valid xlsx (starts with PK for zip)
        let data = result.unwrap();
        assert!(data.len() > 100);
        assert_eq!(&data[0..2], b"PK");
    }

    #[test]
    fn test_write_summary_with_data() {
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
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
        );
        assert!(result.is_ok());
        assert!(result.unwrap().len() > 100);
    }

    #[test]
    fn test_write_detail_empty() {
        let result = ExcelWriter::write_detail(
            &[],
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
        );
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.len() > 100);
        assert_eq!(&data[0..2], b"PK");
    }

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(None), "");
        let time = NaiveTime::from_hms_opt(9, 30, 0);
        assert_eq!(format_time(time), "09:30");
    }

    #[test]
    fn test_format_datetime() {
        assert_eq!(format_datetime(None), "");
    }
}

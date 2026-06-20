use chrono::NaiveDate;
use chrono_tz::Tz;
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};
use uuid::Uuid;

use super::utils::{format_datetime, format_period_label, format_time};
use crate::models::DailyAttendanceReport;
use crate::services::import_messages::ExcelReportMessages;
use crate::utils::excel_format_utils::ExcelFormatUtils;

#[allow(clippy::too_many_lines, clippy::implicit_hasher)]
pub fn generate(
    workbook: &mut Workbook,
    reports: &[DailyAttendanceReport],
    workspace_name: &str,
    date_range: (NaiveDate, NaiveDate),
    tz: Tz,
    msgs: &ExcelReportMessages,
    custom_field_headers: &[String],
    custom_field_values_map: &std::collections::HashMap<Uuid, Vec<String>>,
) -> crate::errors::AppResult<()> {
    let worksheet = workbook
        .add_worksheet()
        .set_name(msgs.raw_details_sheet_name)?;

    let title_fmt = ExcelFormatUtils::title_format();
    let subtitle_fmt = ExcelFormatUtils::subtitle_format();
    let header_fmt = ExcelFormatUtils::header_format();
    let time_fmt = Format::new().set_align(FormatAlign::Center);
    let status_present = Format::new()
        .set_font_color(Color::RGB(0x002E_7D32))
        .set_align(FormatAlign::Center);
    let status_late = Format::new()
        .set_font_color(Color::RGB(0x00F5_7C00))
        .set_align(FormatAlign::Center);
    let status_absent = Format::new()
        .set_font_color(Color::RGB(0x00D3_2F2F))
        .set_align(FormatAlign::Center);
    let status_off = Format::new()
        .set_font_color(Color::RGB(0x0075_7575))
        .set_align(FormatAlign::Center);
    let status_holiday = Format::new()
        .set_font_color(Color::RGB(0x001B_5E20))
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(0x00C8_E6C9)); // light green
    let status_holiday_work = Format::new()
        .set_font_color(Color::RGB(0x00E6_5100))
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(0x00FF_E0B2)); // light orange
    let status_leave_paid = Format::new()
        .set_font_color(Color::RGB(0x000D_47A1))
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(0x00E3_F2FD));
    let status_leave_unpaid = Format::new()
        .set_font_color(Color::RGB(0x0037_474F))
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(0x00EC_EFF1));

    worksheet.write_string_with_format(
        0,
        0,
        msgs.raw_details_title_tpl
            .replace("{workspace}", workspace_name),
        &title_fmt,
    )?;
    worksheet.write_string_with_format(
        1,
        0,
        format_period_label(msgs.period_label, date_range),
        &subtitle_fmt,
    )?;

    // Headers (row 3)
    let headers = msgs.detail_headers;
    let last_col = u16::try_from(headers.len()).unwrap_or(0);
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(
            3,
            u16::try_from(col).unwrap_or(0),
            *header,
            &header_fmt,
        )?;
    }

    for (col_idx, cf_header) in custom_field_headers.iter().enumerate() {
        worksheet.write_string_with_format(
            3,
            last_col + u16::try_from(col_idx).unwrap_or(0),
            cf_header,
            &header_fmt,
        )?;
    }

    // Data rows (row 4+)
    for (idx, report) in reports.iter().enumerate() {
        let row = u32::try_from(idx + 4).unwrap_or(0);
        worksheet.write_string(row, 0, report.date.to_string())?;
        worksheet.write_string(row, 1, &report.employee_id_number)?;
        worksheet.write_string(row, 2, &report.employee_name)?;
        worksheet.write_string(row, 3, report.employer_name.as_deref().unwrap_or(""))?;
        worksheet.write_string_with_format(
            row,
            4,
            format_time(report.scheduled_start),
            &time_fmt,
        )?;
        worksheet.write_string_with_format(row, 5, format_time(report.scheduled_end), &time_fmt)?;
        let (actual_in_str, actual_out_str) = get_punch_times(report, tz);
        worksheet.write_string_with_format(row, 6, actual_in_str, &time_fmt)?;
        worksheet.write_string_with_format(row, 7, actual_out_str, &time_fmt)?;
        worksheet.write_number(row, 8, report.computed_std_hours)?;
        worksheet.write_number(row, 9, report.computed_ot_hours)?;
        let ot_status_label = match report.ot_claim_status.as_deref() {
            Some("pending_employee" | "pendingemployee") => msgs.ot_status_pending_employee,
            Some("pending_manager" | "pendingmanager") => msgs.ot_status_pending_manager,
            Some("approved") => msgs.ot_status_approved,
            Some("rejected") => msgs.ot_status_rejected,
            _ => "-",
        };
        worksheet.write_string(row, 10, ot_status_label)?;

        let (status_label, status_fmt) = match report.status.as_str() {
            "present" => (msgs.status_present, &status_present),
            "late" => (msgs.status_late, &status_late),
            "absent" => (msgs.status_absent, &status_absent),
            "off" => (msgs.status_off, &status_off),
            "holiday" | "holiday_paid" => (msgs.status_holiday, &status_holiday),
            "holiday_unpaid" => (msgs.status_leave_unpaid, &status_leave_unpaid),
            "holiday_work" => (msgs.status_holiday_work, &status_holiday_work),
            "leave" | "holiday_leave" => {
                if report.is_leave_paid == Some(true) {
                    (msgs.status_leave_paid, &status_leave_paid)
                } else {
                    (msgs.status_leave_unpaid, &status_leave_unpaid)
                }
            }
            "unscheduled" => (msgs.status_unscheduled, &time_fmt),
            other => (other, &time_fmt),
        };
        worksheet.write_string_with_format(row, 11, status_label, status_fmt)?;

        // Holiday column (col 12)
        worksheet.write_string(row, 12, report.holiday_name.as_deref().unwrap_or(""))?;

        // Write custom field values
        if let Some(cf_vals) = custom_field_values_map.get(&report.employee_id) {
            for (col_idx, val) in cf_vals.iter().enumerate() {
                worksheet.write_string(row, last_col + u16::try_from(col_idx).unwrap_or(0), val)?;
            }
        }
    }

    // Auto-fit all column widths to content
    worksheet.autofit();

    Ok(())
}

fn get_punch_times(report: &DailyAttendanceReport, tz: Tz) -> (String, String) {
    if let Some(entries_val) = report
        .generation_metadata
        .as_ref()
        .and_then(|m| m.get("time_entries"))
        .and_then(|v| v.as_array())
    {
        let mut parsed_entries = Vec::new();
        for entry in entries_val {
            if let Some(in_time_str) = entry.get("punch_in_time").and_then(|t| t.as_str()) {
                if let Ok(in_time) = chrono::DateTime::parse_from_rfc3339(in_time_str) {
                    let out_time = entry
                        .get("punch_out_time")
                        .and_then(|t| t.as_str())
                        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                        .map(|t| t.with_timezone(&chrono::Utc));
                    parsed_entries.push((in_time.with_timezone(&chrono::Utc), out_time));
                }
            }
        }
        parsed_entries.sort_by_key(|e| e.0);

        if !parsed_entries.is_empty() {
            let ins = parsed_entries
                .iter()
                .map(|e| format_datetime(Some(e.0), tz))
                .collect::<Vec<_>>()
                .join(" | ");
            let outs = parsed_entries
                .iter()
                .map(|e| format_datetime(e.1, tz))
                .collect::<Vec<_>>()
                .join(" | ");
            return (ins, outs);
        }
    }
    (
        format_datetime(report.actual_punch_in, tz),
        format_datetime(report.actual_punch_out, tz),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    #[test]
    fn test_get_punch_times_fallback() {
        let tz = chrono_tz::Asia::Ho_Chi_Minh;
        let report = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Test Employee".to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
            shift_name: None,
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: Some(Utc.with_ymd_and_hms(2026, 6, 20, 1, 0, 0).unwrap()), // 08:00 ICT
            actual_punch_out: Some(Utc.with_ymd_and_hms(2026, 6, 20, 10, 0, 0).unwrap()), // 17:00 ICT
            std_work_units: 0,
            ot_work_units: 0,
            ot_claim_status: None,
            unit_type: crate::models::enums::TimekeepingUnit::HOUR,
            unit_coefficient_hours: 1.0,
            computed_std_hours: 8.0,
            computed_ot_hours: 0.0,
            status: crate::services::attendance_utils::AttendanceStatus::Present,
            is_holiday: false,
            is_unscheduled: false,
            holiday_name: None,
            generation_metadata: None,
            is_leave_paid: None,
            employer_name: None,
        };

        let (in_str, out_str) = get_punch_times(&report, tz);
        assert_eq!(in_str, "08:00");
        assert_eq!(out_str, "17:00");
    }

    #[test]
    fn test_get_punch_times_multi_entries() {
        let tz = chrono_tz::Asia::Ho_Chi_Minh;
        let metadata = json!({
            "time_entries": [
                {
                    "punch_in_time": "2026-06-20T01:00:00Z", // 08:00 ICT
                    "punch_out_time": "2026-06-20T05:00:00Z" // 12:00 ICT
                },
                {
                    "punch_in_time": "2026-06-20T06:00:00Z", // 13:00 ICT
                    "punch_out_time": "2026-06-20T10:00:00Z" // 17:00 ICT
                }
            ]
        });

        let report = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Test Employee".to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
            shift_name: None,
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: Some(Utc.with_ymd_and_hms(2026, 6, 20, 1, 0, 0).unwrap()),
            actual_punch_out: Some(Utc.with_ymd_and_hms(2026, 6, 20, 10, 0, 0).unwrap()),
            std_work_units: 0,
            ot_work_units: 0,
            ot_claim_status: None,
            unit_type: crate::models::enums::TimekeepingUnit::HOUR,
            unit_coefficient_hours: 1.0,
            computed_std_hours: 8.0,
            computed_ot_hours: 0.0,
            status: crate::services::attendance_utils::AttendanceStatus::Present,
            is_holiday: false,
            is_unscheduled: false,
            holiday_name: None,
            generation_metadata: Some(metadata),
            is_leave_paid: None,
            employer_name: None,
        };

        let (in_str, out_str) = get_punch_times(&report, tz);
        assert_eq!(in_str, "08:00 | 13:00");
        assert_eq!(out_str, "12:00 | 17:00");
    }
}

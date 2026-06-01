use chrono::NaiveDate;
use chrono_tz::Tz;
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};

use super::utils::{format_datetime, format_period_label, format_time};
use crate::models::DailyAttendanceReport;
use crate::services::import_messages::ExcelReportMessages;
use crate::utils::excel_format_utils::ExcelFormatUtils;



#[allow(clippy::too_many_lines)]
pub fn generate(
    workbook: &mut Workbook,
    reports: &[DailyAttendanceReport],
    workspace_name: &str,
    date_range: (NaiveDate, NaiveDate),
    tz: Tz,
    msgs: &ExcelReportMessages,
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
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(3, col as u16, *header, &header_fmt)?;
    }

    // Data rows (row 4+)
    for (idx, report) in reports.iter().enumerate() {
        let row = (idx + 4) as u32;
        worksheet.write_string(row, 0, report.date.to_string())?;
        worksheet.write_string(row, 1, &report.employee_id_number)?;
        worksheet.write_string(row, 2, &report.employee_name)?;
        worksheet.write_string_with_format(
            row,
            3,
            format_time(report.scheduled_start),
            &time_fmt,
        )?;
        worksheet.write_string_with_format(row, 4, format_time(report.scheduled_end), &time_fmt)?;
        worksheet.write_string_with_format(
            row,
            5,
            format_datetime(report.actual_punch_in, tz),
            &time_fmt,
        )?;
        worksheet.write_string_with_format(
            row,
            6,
            format_datetime(report.actual_punch_out, tz),
            &time_fmt,
        )?;
        worksheet.write_number(row, 7, report.computed_std_hours)?;
        worksheet.write_number(row, 8, report.computed_ot_hours)?;
        let ot_status_label = match report.ot_claim_status.as_deref() {
            Some("pending_employee" | "pendingemployee") => msgs.ot_status_pending_employee,
            Some("pending_manager" | "pendingmanager") => msgs.ot_status_pending_manager,
            Some("approved") => msgs.ot_status_approved,
            Some("rejected") => msgs.ot_status_rejected,
            _ => "-",
        };
        worksheet.write_string(row, 9, ot_status_label)?;

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
        worksheet.write_string_with_format(row, 10, status_label, status_fmt)?;

        // Holiday column (col 11)
        worksheet.write_string(row, 11, report.holiday_name.as_deref().unwrap_or(""))?;
    }

    // Auto-fit all column widths to content
    worksheet.autofit();

    Ok(())
}

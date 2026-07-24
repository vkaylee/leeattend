use chrono::{Datelike, Duration, NaiveDate};
use chrono_tz::Tz;
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};
use std::collections::BTreeMap;

use super::utils::{
    ceil_units_to_hours, fmt_h, format_period_label, units_to_hours, DayCell, SegmentCell,
};
use crate::models::DailyAttendanceReport;
use crate::services::attendance_utils::AttendanceStatus;
use crate::services::import_messages::ExcelReportMessages;
use crate::utils::excel_format_utils::ExcelFormatUtils;

/// Render a single day cell — returns the display text and the format to use.
fn render_day_cell<'a>(
    cell: &DayCell,
    work_unit_minutes: i32,
    tz: Tz,
    cell_fmt: &'a Format,
    ot_fmt: &'a Format,
    holiday_fmt: &'a Format,
    leave_paid_fmt: &'a Format,
    leave_unpaid_fmt: &'a Format,
) -> (String, &'a Format) {
    let prefix = cell.shift_symbol.as_deref().unwrap_or("");

    if cell.status == AttendanceStatus::Leave || cell.status == AttendanceStatus::HolidayLeave {
        let suffix = match cell.is_leave_paid {
            Some(true) => "(PL)",
            _ => "(UL)",
        };
        let t = format!("{}{}", prefix, suffix);
        let f = match cell.is_leave_paid {
            Some(true) => leave_paid_fmt,
            _ => leave_unpaid_fmt,
        };
        (t, f)
    } else if cell.status == AttendanceStatus::HolidayPaid {
        (format!("{}(HP)", prefix), holiday_fmt)
    } else if cell.status == AttendanceStatus::HolidayUnpaid {
        (format!("{}(HU)", prefix), leave_unpaid_fmt)
    } else {
        // Standard work day with late/early-leave/OT calculation
        let std_h = units_to_hours(cell.std_work_units, work_unit_minutes);

        let late_h = match (cell.punch_in, cell.scheduled_start) {
            (Some(pin_utc), Some(sched)) => {
                let pin_time = pin_utc.with_timezone(&tz).time();
                if pin_time > sched {
                    ceil_units_to_hours(
                        (pin_time - sched).num_minutes(),
                        i64::from(work_unit_minutes),
                    )
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };

        let early_leave_h = match (cell.punch_out, cell.scheduled_end) {
            (Some(pout_utc), Some(sched)) => {
                let pout_time = pout_utc.with_timezone(&tz).time();
                if pout_time < sched {
                    ceil_units_to_hours(
                        (sched - pout_time).num_minutes(),
                        i64::from(work_unit_minutes),
                    )
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };

        let ot_h = units_to_hours(cell.ot_work_units, work_unit_minutes);
        let suffix = if cell.status == AttendanceStatus::HolidayWork {
            "(HW)"
        } else {
            ""
        };
        let t = format!(
            "{}{}/{}/{}/{}{}",
            prefix,
            fmt_h(std_h),
            fmt_h(late_h),
            fmt_h(early_leave_h),
            fmt_h(ot_h),
            suffix,
        );

        let f = if cell.is_holiday {
            holiday_fmt
        } else if cell.ot_work_units > 0 {
            ot_fmt
        } else {
            cell_fmt
        };
        (t, f)
    }
}

pub fn generate(
    workbook: &mut Workbook,
    reports: &[DailyAttendanceReport],
    workspace_name: &str,
    date_range: (NaiveDate, NaiveDate),
    tz: Tz,
    work_unit_minutes: i32,
    msgs: &ExcelReportMessages,
) -> crate::errors::AppResult<()> {
    let worksheet = workbook
        .add_worksheet()
        .set_name(msgs.timesheet_sheet_name)?;

    let title_fmt = ExcelFormatUtils::title_format();
    let subtitle_fmt = ExcelFormatUtils::subtitle_format();
    let header_fmt = ExcelFormatUtils::header_format();
    let cell_fmt = Format::new().set_align(FormatAlign::Center);
    let ot_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(0x00FF_F9C4)); // light yellow
    let holiday_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(0x00C8_E6C9)); // light green for holidays
    let leave_paid_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(0x00E3_F2FD)); // light blue for paid leave
    let leave_unpaid_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(0x00EC_EFF1)); // light gray for unpaid leave

    // Title (row 0) and subtitle (row 1)
    worksheet.write_string_with_format(
        0,
        0,
        msgs.timesheet_title_tpl
            .replace("{workspace}", workspace_name),
        &title_fmt,
    )?;
    worksheet.write_string_with_format(
        1,
        0,
        format_period_label(msgs.period_label, date_range),
        &subtitle_fmt,
    )?;

    // Date columns: start_date .. end_date inclusive
    let mut dates: Vec<NaiveDate> = Vec::new();
    let mut cur = date_range.0;
    while cur <= date_range.1 {
        dates.push(cur);
        cur += Duration::days(1);
    }

    // Group by employee name and employer name (BTreeMap keys are sorted).
    // Value: BTreeMap of per-day DayCell data
    let mut emp_days: BTreeMap<(String, Option<String>, String), BTreeMap<NaiveDate, DayCell>> =
        BTreeMap::new();
    for r in reports {
        let key = (
            r.employee_name.clone(),
            r.employer_name.clone(),
            r.employee_id_number.clone(),
        );
        let day_map = emp_days.entry(key).or_default();
        let segment_cell = SegmentCell {
            segment_id: r.segment_id,
            sort_order: r.sort_order.unwrap_or(0),
            interval_name: r.interval_name.clone(),
            segment_type: r.segment_type.clone().unwrap_or_else(|| "work".to_string()),
            scheduled_start: r.scheduled_start,
            scheduled_end: r.scheduled_end,
            std_work_units: r.std_work_units,
            ot_work_units: r.ot_work_units,
            punch_in: r.actual_punch_in,
            punch_out: r.actual_punch_out,
        };

        day_map
            .entry(r.date)
            .and_modify(|cell| {
                cell.std_work_units += r.std_work_units;
                cell.ot_work_units += r.ot_work_units;
                cell.segments.push(segment_cell.clone());
            })
            .or_insert_with(|| DayCell {
                std_work_units: r.std_work_units,
                ot_work_units: r.ot_work_units,
                punch_in: r.actual_punch_in,
                punch_out: r.actual_punch_out,
                scheduled_start: r.scheduled_start,
                scheduled_end: r.scheduled_end,
                shift_symbol: r.shift_symbol.clone(),
                is_holiday: r.is_holiday,
                status: r.status,
                is_leave_paid: r.is_leave_paid,
                segments: vec![segment_cell],
            });
    }

    // Header row (row 3): Employee ID | Employee | Employer | day numbers
    worksheet.write_string_with_format(3, 0, msgs.timesheet_emp_id_header, &header_fmt)?;
    worksheet.write_string_with_format(3, 1, msgs.timesheet_emp_header, &header_fmt)?;
    worksheet.write_string_with_format(3, 2, msgs.timesheet_employer_header, &header_fmt)?;
    for (i, date) in dates.iter().enumerate() {
        worksheet.write_string_with_format(
            3,
            u16::try_from(i + 3).unwrap_or(0),
            date.day().to_string(),
            &header_fmt,
        )?;
    }

    // Data rows (row 4+)
    for (row_idx, ((name, employer_name, emp_id_number), day_map)) in emp_days.iter().enumerate() {
        let row = u32::try_from(row_idx + 4).unwrap_or(0);
        worksheet.write_string(row, 0, emp_id_number.as_str())?;
        worksheet.write_string(row, 1, name.as_str())?;
        worksheet.write_string(row, 2, employer_name.as_deref().unwrap_or(""))?;

        for (day_idx, date) in dates.iter().enumerate() {
            if let Some(cell) = day_map.get(date) {
                let (text, fmt) = render_day_cell(
                    cell,
                    work_unit_minutes,
                    tz,
                    &cell_fmt,
                    &ot_fmt,
                    &holiday_fmt,
                    &leave_paid_fmt,
                    &leave_unpaid_fmt,
                );

                worksheet.write_string_with_format(
                    row,
                    u16::try_from(day_idx + 3).unwrap_or(0),
                    &text,
                    fmt,
                )?;
            }
        }
    }

    // Auto-fit all column widths to content
    worksheet.autofit();

    // Freeze panes: row 4 (first data), col 3 (first day col)
    let _ = worksheet.set_freeze_panes(4, 3);

    Ok(())
}

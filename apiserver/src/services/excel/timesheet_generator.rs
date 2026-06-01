use chrono::{Datelike, Duration, NaiveDate};
use chrono_tz::Tz;
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};
use std::collections::BTreeMap;

use super::utils::{ceil_units_to_hours, fmt_h, format_period_label, units_to_hours, DayCell};
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

    // Group by employee name (BTreeMap keys are sorted).
    // Value: (employee_id_number, BTreeMap of per-day DayCell data)
    let mut emp_days: BTreeMap<&str, (String, BTreeMap<NaiveDate, DayCell>)> = BTreeMap::new();
    for r in reports {
        let entry = emp_days
            .entry(r.employee_name.as_str())
            .or_insert_with(|| (r.employee_id_number.clone(), BTreeMap::new()));
        entry.1.insert(
            r.date,
            DayCell {
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
            },
        );
    }

    // Header row (row 3): Employee ID | Employee | day numbers
    worksheet.write_string_with_format(3, 0, msgs.timesheet_emp_id_header, &header_fmt)?;
    worksheet.write_string_with_format(3, 1, msgs.timesheet_emp_header, &header_fmt)?;
    for (i, date) in dates.iter().enumerate() {
        worksheet.write_string_with_format(
            3,
            (i + 2) as u16,
            date.day().to_string(),
            &header_fmt,
        )?;
    }

    // Data rows (row 4+)
    for (row_idx, (name, (emp_id_number, day_map))) in emp_days.iter().enumerate() {
        let row = (row_idx + 4) as u32;
        worksheet.write_string(row, 0, emp_id_number.as_str())?;
        worksheet.write_string(row, 1, *name)?;

        for (day_idx, date) in dates.iter().enumerate() {
            if let Some(cell) = day_map.get(date) {
                let prefix = cell.shift_symbol.as_deref().unwrap_or("");
                let (text, fmt) = if cell.status
                    == crate::services::attendance_utils::AttendanceStatus::Leave
                    || cell.status
                        == crate::services::attendance_utils::AttendanceStatus::HolidayLeave
                {
                    let suffix = match cell.is_leave_paid {
                        Some(true) => "(PL)",
                        _ => "(UL)",
                    };
                    let t = format!("{prefix}{suffix}");
                    let f = match cell.is_leave_paid {
                        Some(true) => &leave_paid_fmt,
                        _ => &leave_unpaid_fmt,
                    };
                    (t, f)
                } else if cell.status
                    == crate::services::attendance_utils::AttendanceStatus::HolidayPaid
                {
                    let t = format!("{prefix}(HP)");
                    (t, &holiday_fmt)
                } else if cell.status
                    == crate::services::attendance_utils::AttendanceStatus::HolidayUnpaid
                {
                    let t = format!("{prefix}(HU)");
                    (t, &leave_unpaid_fmt)
                } else {
                    // Convert work units to hours
                    let std_h = units_to_hours(cell.std_work_units, work_unit_minutes);

                    // Calculate late: ceil to nearest work unit, then convert to hours
                    let late_h = match (cell.punch_in, cell.scheduled_start) {
                        (Some(pin_utc), Some(sched)) => {
                            let pin_local = pin_utc.with_timezone(&tz);
                            let pin_time = pin_local.time();
                            if pin_time > sched {
                                let diff_minutes = (pin_time - sched).num_minutes();
                                ceil_units_to_hours(diff_minutes, i64::from(work_unit_minutes))
                            } else {
                                0.0
                            }
                        }
                        _ => 0.0,
                    };

                    // Calculate early-leave: ceil to nearest work unit, then convert to hours
                    let early_leave_h = match (cell.punch_out, cell.scheduled_end) {
                        (Some(pout_utc), Some(sched)) => {
                            let pout_local = pout_utc.with_timezone(&tz);
                            let pout_time = pout_local.time();
                            if pout_time < sched {
                                let diff_minutes = (sched - pout_time).num_minutes();
                                ceil_units_to_hours(diff_minutes, i64::from(work_unit_minutes))
                            } else {
                                0.0
                            }
                        }
                        _ => 0.0,
                    };

                    let ot_h = units_to_hours(cell.ot_work_units, work_unit_minutes);
                    let suffix = if cell.status
                        == crate::services::attendance_utils::AttendanceStatus::HolidayWork
                    {
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
                        suffix
                    );
                    let f = if cell.is_holiday {
                        &holiday_fmt
                    } else if cell.ot_work_units > 0 {
                        &ot_fmt
                    } else {
                        &cell_fmt
                    };
                    (t, f)
                };

                worksheet.write_string_with_format(row, (day_idx + 2) as u16, &text, fmt)?;
            }
        }
    }

    // Auto-fit all column widths to content
    worksheet.autofit();

    // Freeze panes: row 4 (first data), col 1 (first day col)
    let _ = worksheet.set_freeze_panes(4, 1);

    Ok(())
}

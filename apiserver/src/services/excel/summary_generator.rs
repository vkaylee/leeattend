use super::utils::format_period_label;
use crate::models::AttendanceSummary;
use crate::services::import_messages::ExcelReportMessages;
use crate::utils::excel_format_utils::ExcelFormatUtils;
use chrono::NaiveDate;
use rust_xlsxwriter::Workbook;

pub fn generate(
    workbook: &mut Workbook,
    summaries: &[AttendanceSummary],
    workspace_name: &str,
    date_range: (NaiveDate, NaiveDate),
    msgs: &ExcelReportMessages,
) -> crate::errors::AppResult<()> {
    let worksheet = workbook.add_worksheet().set_name(msgs.summary_sheet_name)?;

    let title_format = ExcelFormatUtils::title_format();
    let subtitle_format = ExcelFormatUtils::subtitle_format();
    let header_format = ExcelFormatUtils::header_format();
    let number_format = ExcelFormatUtils::number_format();

    worksheet.write_string_with_format(
        0,
        0,
        msgs.summary_title_tpl
            .replace("{workspace}", workspace_name),
        &title_format,
    )?;

    worksheet.write_string_with_format(
        1,
        0,
        format_period_label(msgs.period_label, date_range),
        &subtitle_format,
    )?;

    let headers = msgs.summary_headers;
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(3, col as u16, *header, &header_format)?;
    }

    for (idx, summary) in summaries.iter().enumerate() {
        let row = (idx + 4) as u32;
        worksheet.write_string(row, 0, &summary.employee_name)?;
        worksheet.write_number_with_format(row, 1, summary.computed_std_hours, &number_format)?;
        worksheet.write_number_with_format(row, 2, summary.computed_ot_hours, &number_format)?;
        worksheet.write_number_with_format(
            row,
            3,
            summary.total_leave_days as f64,
            &number_format,
        )?;
    }

    worksheet.set_column_width(0, 35)?;
    worksheet.set_column_width(1, 15)?;
    worksheet.set_column_width(2, 15)?;
    worksheet.set_column_width(3, 18)?;

    Ok(())
}

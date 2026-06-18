use super::utils::format_period_label;
use crate::models::AttendanceSummary;
use crate::services::import_messages::ExcelReportMessages;
use crate::utils::excel_format_utils::ExcelFormatUtils;
use chrono::NaiveDate;
use rust_xlsxwriter::Workbook;
use uuid::Uuid;

#[allow(clippy::implicit_hasher)]
pub fn generate(
    workbook: &mut Workbook,
    summaries: &[AttendanceSummary],
    workspace_name: &str,
    date_range: (NaiveDate, NaiveDate),
    msgs: &ExcelReportMessages,
    custom_field_headers: &[String],
    custom_field_values_map: &std::collections::HashMap<Uuid, Vec<String>>,
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
    let last_col = u16::try_from(headers.len()).unwrap_or(0);
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(
            3,
            u16::try_from(col).unwrap_or(0),
            *header,
            &header_format,
        )?;
    }

    for (col_idx, cf_header) in custom_field_headers.iter().enumerate() {
        worksheet.write_string_with_format(
            3,
            last_col + u16::try_from(col_idx).unwrap_or(0),
            cf_header,
            &header_format,
        )?;
    }

    for (idx, summary) in summaries.iter().enumerate() {
        let row = u32::try_from(idx + 4).unwrap_or(0);
        worksheet.write_string(row, 0, &summary.employee_name)?;
        worksheet.write_string(row, 1, summary.employer_name.as_deref().unwrap_or(""))?;
        worksheet.write_number_with_format(row, 2, summary.computed_std_hours, &number_format)?;
        worksheet.write_number_with_format(row, 3, summary.computed_ot_hours, &number_format)?;
        worksheet.write_number_with_format(
            row,
            4,
            f64::from(i32::try_from(summary.total_leave_days).unwrap_or(0)),
            &number_format,
        )?;

        if let Some(cf_vals) = custom_field_values_map.get(&summary.employee_id) {
            for (col_idx, val) in cf_vals.iter().enumerate() {
                worksheet.write_string(row, last_col + u16::try_from(col_idx).unwrap_or(0), val)?;
            }
        }
    }

    worksheet.set_column_width(0, 35)?;
    worksheet.set_column_width(1, 25)?;
    worksheet.set_column_width(2, 15)?;
    worksheet.set_column_width(3, 15)?;
    worksheet.set_column_width(4, 18)?;

    for col_idx in 0..custom_field_headers.len() {
        worksheet.set_column_width(last_col + u16::try_from(col_idx).unwrap_or(0), 18)?;
    }

    Ok(())
}

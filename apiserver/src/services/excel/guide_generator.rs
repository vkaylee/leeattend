use crate::services::import_messages::ExcelReportMessages;
use crate::utils::excel_format_utils::ExcelFormatUtils;
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};

#[allow(clippy::similar_names)]
pub fn generate(
    workbook: &mut Workbook,
    msgs: &ExcelReportMessages,
) -> crate::errors::AppResult<()> {
    let worksheet = workbook.add_worksheet().set_name(msgs.guide_sheet_name)?;

    let title_fmt = ExcelFormatUtils::title_format();
    let header_fmt = ExcelFormatUtils::ref_header_format();

    let section_title_fmt = Format::new()
        .set_bold()
        .set_font_size(12)
        .set_font_color(Color::RGB(0x001F_4E79));

    let italic_text_fmt = Format::new()
        .set_italic()
        .set_font_color(Color::RGB(0x005B_5B5B));

    // Style formats matching the actual sheets
    let pl_cell_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_bold()
        .set_background_color(Color::RGB(0x00E3_F2FD)); // light blue

    let ul_cell_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_bold()
        .set_background_color(Color::RGB(0x00EC_EFF1)); // light gray

    let hp_cell_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_bold()
        .set_background_color(Color::RGB(0x00C8_E6C9)); // light green

    let hu_cell_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_bold()
        .set_background_color(Color::RGB(0x00EC_EFF1)); // light gray

    let hw_cell_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_bold()
        .set_background_color(Color::RGB(0x00C8_E6C9)); // light green

    // Title (row 0)
    worksheet.write_string_with_format(0, 0, msgs.guide_title, &title_fmt)?;

    // Section 1: Timesheet Cell Format (row 2)
    worksheet.write_string_with_format(2, 0, msgs.guide_explanation_title, &section_title_fmt)?;
    worksheet.write_string_with_format(
        3,
        0,
        msgs.guide_explanation_format,
        &Format::new().set_bold(),
    )?;
    worksheet.write_string_with_format(4, 0, msgs.guide_explanation_example, &italic_text_fmt)?;

    // Section 2: Abbreviations (row 6)
    let headers = msgs.guide_headers;
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(6, col as u16, *header, &header_fmt)?;
    }

    // Write abbreviations table (rows 7 to 11)
    for (idx, row_data) in msgs.guide_abbreviations.iter().enumerate() {
        let row = (idx + 7) as u32;
        let symbol = row_data[0];
        let meaning = row_data[1];
        let desc = row_data[2];

        // Format symbol cell to match actual sheet visualization
        let sym_fmt = match symbol {
            "(PL)" => &pl_cell_fmt,
            "(UL)" => &ul_cell_fmt,
            "(HP)" => &hp_cell_fmt,
            "(HU)" => &hu_cell_fmt,
            "(HW)" => &hw_cell_fmt,
            _ => &Format::default(),
        };

        worksheet.write_string_with_format(row, 0, symbol, sym_fmt)?;
        worksheet.write_string(row, 1, meaning)?;
        worksheet.write_string(row, 2, desc)?;
    }

    // Adjust column widths
    worksheet.set_column_width(0, 20)?;
    worksheet.set_column_width(1, 25)?;
    worksheet.set_column_width(2, 60)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guide_generation() {
        let mut workbook = Workbook::new();
        let msgs = ExcelReportMessages::for_lang("en");
        let result = generate(&mut workbook, &msgs);
        assert!(result.is_ok());
        let data = workbook.save_to_buffer();
        assert!(data.is_ok());
        assert!(data.unwrap().len() > 100);
    }
}

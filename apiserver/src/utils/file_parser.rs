use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use calamine::{Data, Reader, Xls, Xlsx};
use std::collections::HashMap;
use std::io::Cursor;

/// Convert a Calamine Excel cell into a trimmed String
pub fn cell_to_string(cell: Option<&Data>) -> String {
    match cell {
        Some(Data::String(s)) => s.trim().to_string(),
        Some(Data::Float(f)) => f.to_string(),
        Some(Data::Int(i)) => i.to_string(),
        Some(Data::DateTime(d)) => {
            // Very basic handle for Excel dates if needed
            if let Some(dt) = d.as_datetime() {
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            } else {
                d.to_string()
            }
        }
        Some(Data::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn is_zip_file(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04])
}

fn is_xls_file(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])
}

/// Extract rows from XLSX, XLS, or CSV.
/// Returns a Vec of (line_number, columns)
/// - line_number is 1-indexed, but usually we start at line 2 because of header skip.
/// - The header (first row) is skipped automatically.
pub fn extract_file_rows(file_bytes: &[u8]) -> Result<Vec<(usize, Vec<String>)>, AppError> {
    let mut rows = Vec::new();

    // 1. Try XLSX
    if is_zip_file(file_bytes) {
        let mut reader = Xlsx::new(Cursor::new(file_bytes)).map_err(|e| {
            AppError::validation_args(I18nKey::CannotOpenXlsx, vec![("error", e.to_string())])
        })?;
        let sheet_names = reader.sheet_names();
        for sheet in sheet_names {
            if let Ok(range) = reader.worksheet_range(&sheet) {
                if range.get_size().0 > 1 {
                    for (idx, row) in range.rows().enumerate() {
                        if idx == 0 {
                            continue; // Skip header
                        }
                        let line_number = idx + 1;
                        let cols: Vec<String> =
                            row.iter().map(|c| cell_to_string(Some(c))).collect();
                        rows.push((line_number, cols));
                    }
                    return Ok(rows);
                }
            }
        }
        return Err(AppError::validation(I18nKey::EmptyExcelSheet));
    }

    // 2. Try XLS
    if is_xls_file(file_bytes) {
        let mut reader = Xls::new(Cursor::new(file_bytes)).map_err(|e| {
            AppError::validation_args(I18nKey::CannotOpenXls, vec![("error", e.to_string())])
        })?;
        let sheet_names = reader.sheet_names();
        for sheet in sheet_names {
            if let Ok(range) = reader.worksheet_range(&sheet) {
                if range.get_size().0 > 1 {
                    for (idx, row) in range.rows().enumerate() {
                        if idx == 0 {
                            continue; // Skip header
                        }
                        let line_number = idx + 1;
                        let cols: Vec<String> =
                            row.iter().map(|c| cell_to_string(Some(c))).collect();
                        rows.push((line_number, cols));
                    }
                    return Ok(rows);
                }
            }
        }
        return Err(AppError::validation(I18nKey::EmptyExcelSheet));
    }

    Err(AppError::validation(I18nKey::InvalidFileFormat))
}

/// Normalize header string for matching (lowercase, trim whitespace, and strip BOM)
fn normalize_header(h: &str) -> String {
    h.trim()
        .trim_start_matches('\u{feff}')
        .trim()
        .to_lowercase()
}

/// Extract rows and map them using normalized headers.
///
/// Returns a list of (line_number, mapped_row).
/// The mapped_row uses the EXACT expected_headers strings as keys, but matched flexibly.
/// Only scans the FIRST sheet.
#[allow(clippy::type_complexity, clippy::too_many_lines)]
pub fn extract_mapped_rows(
    file_bytes: &[u8],
    expected_headers: &[&str],
) -> Result<Vec<(usize, HashMap<String, String>)>, AppError> {
    let expected_normalized: Vec<String> = expected_headers
        .iter()
        .map(|&s| normalize_header(s))
        .collect();
    let mut header_map: HashMap<String, usize> = HashMap::new(); // normalized header -> col index
    let mut rows = Vec::new();

    // Helper closure to process headers and rows
    let process_sheet = |mut row_iter: Box<dyn Iterator<Item = (usize, Vec<String>)>>| -> Result<Vec<(usize, HashMap<String, String>)>, AppError> {
        let mut missing_headers = Vec::new();
        let mut duplicate_headers = Vec::new();

        // Process Header
        if let Some((_, header_cols)) = row_iter.next() {
            for (idx, col) in header_cols.into_iter().enumerate() {
                let norm = normalize_header(&col);
                if !norm.is_empty() {
                    // Only care if an EXPECTED header is duplicated
                    if expected_normalized.contains(&norm) && header_map.contains_key(&norm) {
                        duplicate_headers.push(col.clone());
                    }
                    header_map.insert(norm, idx);
                }
            }
        } else {
            return Err(AppError::validation(I18nKey::EmptyExcelSheet));
        }

        if !duplicate_headers.is_empty() {
            return Err(AppError::validation_args(
                I18nKey::DuplicateHeaders,
                vec![("headers", duplicate_headers.join(", "))],
            ));
        }

        // Validate Expected Headers
        for (i, expected) in expected_normalized.iter().enumerate() {
            if !header_map.contains_key(expected) {
                missing_headers.push(expected_headers[i]);
            }
        }

        if !missing_headers.is_empty() {
            return Err(AppError::validation_args(
                I18nKey::MissingMandatoryHeaders,
                vec![("headers", missing_headers.join(", "))],
            ));
        }

        // Process Data Rows
        for (line_number, cols) in row_iter {
            let mut mapped_row = HashMap::new();
            for (i, expected_norm) in expected_normalized.iter().enumerate() {
                if let Some(&col_idx) = header_map.get(expected_norm) {
                    let val = cols.get(col_idx).cloned().unwrap_or_default();
                    mapped_row.insert(expected_headers[i].to_string(), val);
                }
            }
            // Check if the whole mapped row is empty (skip completely empty rows)
            if mapped_row.values().any(|v| !v.is_empty()) {
                rows.push((line_number, mapped_row));
            }
        }

        if rows.is_empty() {
            return Err(AppError::validation(I18nKey::EmptyExcelSheet));
        }

        Ok(rows)
    };

    // 1. Try XLSX
    if is_zip_file(file_bytes) {
        let mut reader = Xlsx::new(Cursor::new(file_bytes)).map_err(|e| {
            AppError::validation_args(I18nKey::CannotOpenXlsx, vec![("error", e.to_string())])
        })?;
        let sheet_names = reader.sheet_names();
        let mut best_sheet_range = None;
        let mut max_matched_headers = 0;

        for name in &sheet_names {
            if let Ok(range) = reader.worksheet_range(name) {
                if let Some(first_row) = range.rows().next() {
                    let mut match_count = 0;
                    for cell in first_row {
                        let cell_str = cell_to_string(Some(cell));
                        let norm = normalize_header(&cell_str);
                        if !norm.is_empty() && expected_normalized.contains(&norm) {
                            match_count += 1;
                        }
                    }
                    if match_count > max_matched_headers {
                        max_matched_headers = match_count;
                        best_sheet_range = Some(range);
                    }
                }
            }
        }

        let range = if let Some(r) = best_sheet_range {
            r
        } else if let Some(name) = sheet_names.first() {
            reader.worksheet_range(name).map_err(|e| {
                AppError::validation_args(
                    I18nKey::CannotReadSheet,
                    vec![("name", name.clone()), ("error", e.to_string())],
                )
            })?
        } else {
            return Err(AppError::validation(I18nKey::NoSheetsFound));
        };

        let iter = range.rows().enumerate().map(|(idx, r)| {
            let cols: Vec<String> = r.iter().map(|c| cell_to_string(Some(c))).collect();
            (idx + 1, cols)
        });
        return process_sheet(Box::new(iter));
    }

    // 2. Try XLS
    if is_xls_file(file_bytes) {
        let mut reader = Xls::new(Cursor::new(file_bytes)).map_err(|e| {
            AppError::validation_args(I18nKey::CannotOpenXls, vec![("error", e.to_string())])
        })?;
        let sheet_names = reader.sheet_names();
        let mut best_sheet_range = None;
        let mut max_matched_headers = 0;

        for name in &sheet_names {
            if let Ok(range) = reader.worksheet_range(name) {
                if let Some(first_row) = range.rows().next() {
                    let mut match_count = 0;
                    for cell in first_row {
                        let cell_str = cell_to_string(Some(cell));
                        let norm = normalize_header(&cell_str);
                        if !norm.is_empty() && expected_normalized.contains(&norm) {
                            match_count += 1;
                        }
                    }
                    if match_count > max_matched_headers {
                        max_matched_headers = match_count;
                        best_sheet_range = Some(range);
                    }
                }
            }
        }

        let range = if let Some(r) = best_sheet_range {
            r
        } else if let Some(name) = sheet_names.first() {
            reader.worksheet_range(name).map_err(|e| {
                AppError::validation_args(
                    I18nKey::CannotReadSheet,
                    vec![("name", name.clone()), ("error", e.to_string())],
                )
            })?
        } else {
            return Err(AppError::validation(I18nKey::NoSheetsFound));
        };

        let iter = range.rows().enumerate().map(|(idx, r)| {
            let cols: Vec<String> = r.iter().map(|c| cell_to_string(Some(c))).collect();
            (idx + 1, cols)
        });
        return process_sheet(Box::new(iter));
    }

    Err(AppError::validation(I18nKey::InvalidFileFormat))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_xlsxwriter::Workbook;

    fn create_xlsx_bytes(headers: &[&str], rows: &[Vec<&str>]) -> Vec<u8> {
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        for (col, header) in headers.iter().enumerate() {
            sheet.write_string(0, col as u16, *header).unwrap();
        }
        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, val) in row.iter().enumerate() {
                sheet
                    .write_string((row_idx + 1) as u32, col_idx as u16, *val)
                    .unwrap();
            }
        }
        workbook.save_to_buffer().unwrap()
    }

    #[test]
    fn test_cell_to_string() {
        assert_eq!(
            cell_to_string(Some(&Data::String(" Hello ".to_string()))),
            "Hello"
        );
        assert_eq!(cell_to_string(Some(&Data::Float(123.45))), "123.45");
        assert_eq!(cell_to_string(Some(&Data::Int(42))), "42");
        assert_eq!(cell_to_string(Some(&Data::Bool(true))), "true");
        assert_eq!(cell_to_string(None), "");
        assert_eq!(cell_to_string(Some(&Data::Empty)), "");
    }

    #[test]
    fn test_extract_file_rows_xlsx() {
        let headers = ["emp_id", "leave_code", "start_date"];
        let rows = vec![
            vec!["E001", "AL", "2023-01-01"],
            vec!["E002", "SL", "2023-01-02"],
        ];
        let bytes = create_xlsx_bytes(&headers, &rows);
        let result = extract_file_rows(&bytes).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 2); // line 2
        assert_eq!(result[0].1, vec!["E001", "AL", "2023-01-01"]);

        assert_eq!(result[1].0, 3); // line 3
        assert_eq!(result[1].1, vec!["E002", "SL", "2023-01-02"]);
    }

    #[test]
    fn test_extract_file_rows_empty_xlsx() {
        let headers = ["emp_id", "leave_code", "start_date"];
        let rows: Vec<Vec<&str>> = vec![];
        let bytes = create_xlsx_bytes(&headers, &rows);
        let result = extract_file_rows(&bytes);
        assert!(result.is_err(), "Empty file should return error");
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "Validation error: error.empty_excel_sheet");
    }

    #[test]
    fn test_extract_file_rows_invalid_format() {
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03]; // Random binary data
        let result = extract_file_rows(&invalid_data);
        assert!(result.is_err(), "Invalid binary file should return error");
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Validation error: error.invalid_file_format"
        );
    }

    #[test]
    fn test_extract_mapped_rows() {
        let headers = ["employee_code", "full_name", "hire_date", "department_name"];
        let rows = vec![
            vec!["E001", "Nguyen Van A", "2023-01-01", "IT"],
            vec!["E002", "Tran Thi B", "2023-01-02", "HR"],
        ];
        let bytes = create_xlsx_bytes(&headers, &rows);
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(&bytes, &expected).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1.get("employee_code").unwrap(), "E001");
        assert_eq!(result[0].1.get("full_name").unwrap(), "Nguyen Van A");
        assert_eq!(result[0].1.get("department_name").unwrap(), "IT");

        assert_eq!(result[1].1.get("employee_code").unwrap(), "E002");
    }

    #[test]
    fn test_extract_mapped_rows_missing_header() {
        let headers = ["employee_code", "full_name", "hire_date"];
        let rows = vec![vec!["E001", "Nguyen Van A", "2023-01-01"]];
        let bytes = create_xlsx_bytes(&headers, &rows);
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(&bytes, &expected);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Validation error: error.missing_mandatory_headers:headers=department_name"
        );
    }

    #[test]
    fn test_extract_mapped_rows_duplicate_expected_header() {
        let headers = [
            "employee_code",
            "employee_code",
            "full_name",
            "hire_date",
            "department_name",
        ];
        let rows = vec![vec!["E001", "E001X", "Nguyen Van A", "2023-01-01", "IT"]];
        let bytes = create_xlsx_bytes(&headers, &rows);
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(&bytes, &expected);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Validation error: error.duplicate_headers:headers=employee_code"
        );
    }

    #[test]
    fn test_extract_mapped_rows_duplicate_ignored_header() {
        let headers = [
            "employee_code",
            "full_name",
            "hire_date",
            "department_name",
            "ignoreMe",
            "ignoreMe",
        ];
        let rows = vec![vec!["E001", "Nguyen Van A", "2023-01-01", "IT", "1", "2"]];
        let bytes = create_xlsx_bytes(&headers, &rows);
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(&bytes, &expected);

        assert!(
            result.is_ok(),
            "Should ignore duplicates of headers we don't care about"
        );
        let rows = result.unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_extract_mapped_rows_normalization() {
        let headers = [
            "  EMPLOYEE_code ",
            " full_name  ",
            " HIRE_date",
            "  Department_Name  ",
        ];
        let rows = vec![vec!["E001", "Nguyen Van A", "2023-01-01", "IT"]];
        let bytes = create_xlsx_bytes(&headers, &rows);
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(&bytes, &expected).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.get("employee_code").unwrap(), "E001");
        assert_eq!(result[0].1.get("department_name").unwrap(), "IT");
    }

    #[test]
    fn test_extract_mapped_rows_empty_data() {
        let headers = ["employee_code", "full_name", "hire_date", "department_name"];
        let rows: Vec<Vec<&str>> = vec![];
        let bytes = create_xlsx_bytes(&headers, &rows);
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(&bytes, &expected);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "Validation error: error.empty_excel_sheet");
    }

    #[test]
    fn test_extract_mapped_rows_scrambled_columns() {
        let headers = [
            "department_name",
            "extra1",
            "full_name",
            "extra2",
            "employee_code",
            "hire_date",
        ];
        let rows = vec![vec![
            "IT",
            "ignore1",
            "Nguyen Van A",
            "ignore2",
            "E001",
            "2023-01-01",
        ]];
        let bytes = create_xlsx_bytes(&headers, &rows);
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(&bytes, &expected).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.get("employee_code").unwrap(), "E001");
        assert_eq!(result[0].1.get("department_name").unwrap(), "IT");
        assert_eq!(result[0].1.get("full_name").unwrap(), "Nguyen Van A");
    }

    #[test]
    fn test_corrupted_xlsx_returns_xlsx_error() {
        let corrupted_zip = vec![0x50, 0x4B, 0x03, 0x04, 0x01, 0x02, 0x03];
        let expected = ["employee_code", "full_name"];
        let result = extract_mapped_rows(&corrupted_zip, &expected);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("error.cannot_open_xlsx"),
            "Error msg was: {err_msg}"
        );
    }

    #[test]
    fn test_extract_mapped_rows_multiple_sheets_matches_correct_one() {
        let mut workbook = Workbook::new();

        let ref_sheet = workbook.add_worksheet().set_name("Reference").unwrap();
        ref_sheet.write_string(0, 0, "leave_type_code").unwrap();
        ref_sheet.write_string(0, 1, "leave_type_name").unwrap();
        ref_sheet.write_string(1, 0, "vacation").unwrap();
        ref_sheet.write_string(1, 1, "Annual Leave").unwrap();

        let tpl_sheet = workbook.add_worksheet().set_name("Template").unwrap();
        tpl_sheet.write_string(0, 0, "employee_code").unwrap();
        tpl_sheet.write_string(0, 1, "leave_type_code").unwrap();
        tpl_sheet.write_string(0, 2, "start_date").unwrap();
        tpl_sheet.write_string(0, 3, "end_date").unwrap();
        tpl_sheet.write_string(1, 0, "E001").unwrap();
        tpl_sheet.write_string(1, 1, "vacation").unwrap();
        tpl_sheet.write_string(1, 2, "2026-04-01").unwrap();
        tpl_sheet.write_string(1, 3, "2026-04-02").unwrap();

        let bytes = workbook.save_to_buffer().unwrap();
        let expected = ["employee_code", "leave_type_code", "start_date", "end_date"];
        let result = extract_mapped_rows(&bytes, &expected).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.get("employee_code").unwrap(), "E001");
        assert_eq!(result[0].1.get("leave_type_code").unwrap(), "vacation");
    }
}

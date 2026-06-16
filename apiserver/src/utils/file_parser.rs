use calamine::{Data, Reader, Xls, Xlsx};
use std::collections::HashMap;
use std::io::Cursor;
use tracing::warn;

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

/// Extract rows from XLSX, XLS, or CSV.
/// Returns a Vec of (line_number, columns)
/// - line_number is 1-indexed, but usually we start at line 2 because of header skip.
/// - The header (first row) is skipped automatically.
pub fn extract_file_rows(file_bytes: &[u8]) -> Result<Vec<(usize, Vec<String>)>, String> {
    let mut rows = Vec::new();

    // 1. Try XLSX
    if let Ok(mut reader) = Xlsx::new(Cursor::new(file_bytes)) {
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
        return Err("File is empty or contains only headers.".to_string());
    }

    // 2. Try XLS
    if let Ok(mut reader) = Xls::new(Cursor::new(file_bytes)) {
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
        return Err("File is empty or contains only headers.".to_string());
    }

    // 3. Try CSV
    let decoded_opt = decode_utf16_bytes(file_bytes);

    let first_line_storage;
    let first_line = if let Some(ref decoded) = decoded_opt {
        decoded.lines().next().unwrap_or_default()
    } else {
        first_line_storage = String::from_utf8_lossy(file_bytes);
        first_line_storage.lines().next().unwrap_or_default()
    };

    let delimiter = detect_delimiter(first_line);

    let mut reader_builder = csv::ReaderBuilder::new();
    reader_builder.has_headers(true);
    reader_builder.delimiter(delimiter);

    let mut reader = if let Some(ref decoded) = decoded_opt {
        reader_builder.from_reader(Cursor::new(decoded.as_bytes()))
    } else {
        reader_builder.from_reader(Cursor::new(file_bytes))
    };

    let mut has_records = false;
    for (idx, result) in reader.records().enumerate() {
        has_records = true;
        let line_number = idx + 2; // 1-indexed + header skip
        match result {
            Ok(record) => {
                let cols: Vec<String> = record.iter().map(|s| s.trim().to_string()).collect();
                rows.push((line_number, cols));
            }
            Err(e) => {
                warn!("CSV parse error at line {}: {}", line_number, e);
                rows.push((line_number, vec![]));
            }
        }
    }

    if has_records && !rows.is_empty() {
        return Ok(rows);
    }

    Err("Invalid file format or file is empty (contains only headers). Supported formats: XLSX, XLS, CSV.".to_string())
}

/// Normalize header string for matching (lowercase, trim whitespace, and strip BOM)
fn normalize_header(h: &str) -> String {
    h.trim()
        .trim_start_matches('\u{feff}')
        .trim()
        .to_lowercase()
}

/// Decode UTF-16 bytes (LE or BE) to a String if a BOM is present
fn decode_utf16_bytes(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 2 {
        return None;
    }

    let is_le = bytes[0] == 0xFF && bytes[1] == 0xFE;
    let is_be = bytes[0] == 0xFE && bytes[1] == 0xFF;

    if !is_le && !is_be {
        return None;
    }

    let data_bytes = &bytes[2..];
    let mut u16_data = Vec::with_capacity(data_bytes.len() / 2);

    for chunk in data_bytes.chunks_exact(2) {
        let val = if is_le {
            u16::from_le_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], chunk[1]])
        };
        u16_data.push(val);
    }

    Some(
        char::decode_utf16(u16_data)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect::<String>(),
    )
}

/// Detect CSV delimiter by counting occurrences in the header line
fn detect_delimiter(header_line: &str) -> u8 {
    let comma_count = header_line.chars().filter(|&c| c == ',').count();
    let semicolon_count = header_line.chars().filter(|&c| c == ';').count();
    let tab_count = header_line.chars().filter(|&c| c == '\t').count();

    if semicolon_count > comma_count && semicolon_count > tab_count {
        b';'
    } else if tab_count > comma_count && tab_count > semicolon_count {
        b'\t'
    } else {
        b','
    }
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
) -> Result<Vec<(usize, HashMap<String, String>)>, String> {
    let expected_normalized: Vec<String> = expected_headers
        .iter()
        .map(|&s| normalize_header(s))
        .collect();
    let mut header_map: HashMap<String, usize> = HashMap::new(); // normalized header -> col index
    let mut rows = Vec::new();

    // Helper closure to process headers and rows
    let process_sheet = |mut row_iter: Box<dyn Iterator<Item = (usize, Vec<String>)>>| -> Result<Vec<(usize, HashMap<String, String>)>, String> {
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
            return Err("File is completely empty.".to_string());
        }

        if !duplicate_headers.is_empty() {
            return Err(format!("Phát hiện các cột bị trùng lặp: {}", duplicate_headers.join(", ")));
        }

        // Validate Expected Headers
        for (i, expected) in expected_normalized.iter().enumerate() {
            if !header_map.contains_key(expected) {
                missing_headers.push(expected_headers[i]);
            }
        }

        if !missing_headers.is_empty() {
            return Err(format!("Thiếu các cột bắt buộc: {}", missing_headers.join(", ")));
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
            return Err("File is empty or contains only headers.".to_string());
        }

        Ok(rows)
    };

    // 1. Try XLSX
    if let Ok(mut reader) = Xlsx::new(Cursor::new(file_bytes)) {
        if let Some(Ok(range)) = reader.worksheet_range_at(0) {
            let iter = range.rows().enumerate().map(|(idx, r)| {
                let cols: Vec<String> = r.iter().map(|c| cell_to_string(Some(c))).collect();
                (idx + 1, cols)
            });
            return process_sheet(Box::new(iter));
        }
        return Err("Không thể đọc được sheet đầu tiên của file Excel.".to_string());
    }

    // 2. Try XLS
    if let Ok(mut reader) = Xls::new(Cursor::new(file_bytes)) {
        if let Some(Ok(range)) = reader.worksheet_range_at(0) {
            let iter = range.rows().enumerate().map(|(idx, r)| {
                let cols: Vec<String> = r.iter().map(|c| cell_to_string(Some(c))).collect();
                (idx + 1, cols)
            });
            return process_sheet(Box::new(iter));
        }
        return Err("Không thể đọc được sheet đầu tiên của file Excel.".to_string());
    }

    // 3. Try CSV
    let decoded_opt = decode_utf16_bytes(file_bytes);

    let first_line_storage;
    let first_line = if let Some(ref decoded) = decoded_opt {
        decoded.lines().next().unwrap_or_default()
    } else {
        first_line_storage = String::from_utf8_lossy(file_bytes);
        first_line_storage.lines().next().unwrap_or_default()
    };

    let delimiter = detect_delimiter(first_line);

    let mut reader_builder = csv::ReaderBuilder::new();
    reader_builder.has_headers(false);
    reader_builder.delimiter(delimiter);

    let mut reader = if let Some(ref decoded) = decoded_opt {
        reader_builder.from_reader(Cursor::new(decoded.as_bytes()))
    } else {
        reader_builder.from_reader(Cursor::new(file_bytes))
    };

    let mut iter_data = Vec::new();
    for (idx, res) in reader.records().enumerate() {
        let cols = match res {
            Ok(record) => record.iter().map(|s| s.trim().to_string()).collect(),
            Err(e) => {
                warn!("CSV parse error at line {}: {}", idx + 1, e);
                vec![]
            }
        };
        iter_data.push((idx + 1, cols));
    }

    process_sheet(Box::new(iter_data.into_iter())).map_err(|e| {
        if e == "File is completely empty." {
            "Định dạng file không hợp lệ hoặc file trống. Hỗ trợ: XLSX, XLS, CSV.".to_string()
        } else {
            e
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_extract_file_rows_csv() {
        let csv_data = "emp_id,leave_code,start_date\nE001,AL,2023-01-01\nE002,SL,2023-01-02";
        let result = extract_file_rows(csv_data.as_bytes()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 2); // line 2
        assert_eq!(result[0].1, vec!["E001", "AL", "2023-01-01"]);

        assert_eq!(result[1].0, 3); // line 3
        assert_eq!(result[1].1, vec!["E002", "SL", "2023-01-02"]);
    }

    #[test]
    fn test_extract_file_rows_empty_csv() {
        let csv_data = "emp_id,leave_code,start_date\n"; // Only header
        let result = extract_file_rows(csv_data.as_bytes());
        assert!(result.is_err(), "Empty file should return error");
    }

    #[test]
    fn test_extract_file_rows_invalid_format() {
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03]; // Random binary data
        let result = extract_file_rows(&invalid_data);
        assert!(result.is_err(), "Invalid binary file should return error");
    }
    #[test]
    fn test_extract_mapped_rows() {
        let csv_data = "employee_code,full_name,hire_date,department_name\nE001,Nguyen Van A,2023-01-01,IT\nE002,Tran Thi B,2023-01-02,HR";
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(csv_data.as_bytes(), &expected).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1.get("employee_code").unwrap(), "E001");
        assert_eq!(result[0].1.get("full_name").unwrap(), "Nguyen Van A");
        assert_eq!(result[0].1.get("department_name").unwrap(), "IT");

        assert_eq!(result[1].1.get("employee_code").unwrap(), "E002");
    }

    #[test]
    fn test_extract_mapped_rows_missing_header() {
        let csv_data = "employee_code,full_name,hire_date\nE001,Nguyen Van A,2023-01-01";
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(csv_data.as_bytes(), &expected);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Thiếu các cột bắt buộc: department_name"));
    }

    #[test]
    fn test_extract_mapped_rows_duplicate_expected_header() {
        // Two employee_code columns
        let csv_data = "employee_code,employee_code,full_name,hire_date,department_name\nE001,E001X,Nguyen Van A,2023-01-01,IT";
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(csv_data.as_bytes(), &expected);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Phát hiện các cột bị trùng lặp: employee_code"));
    }

    #[test]
    fn test_extract_mapped_rows_duplicate_ignored_header() {
        // Two ignoreMe columns, shouldn't trigger duplicate error for expected headers
        let csv_data = "employee_code,full_name,hire_date,department_name,ignoreMe,ignoreMe\nE001,Nguyen Van A,2023-01-01,IT,1,2";
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(csv_data.as_bytes(), &expected);

        assert!(
            result.is_ok(),
            "Should ignore duplicates of headers we don't care about"
        );
        let rows = result.unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_extract_mapped_rows_normalization() {
        // Messy headers: uppercase, surrounding spaces
        let csv_data = "  EMPLOYEE_code , full_name  , HIRE_date,  Department_Name  \nE001,Nguyen Van A,2023-01-01,IT";
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(csv_data.as_bytes(), &expected).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.get("employee_code").unwrap(), "E001");
        assert_eq!(result[0].1.get("department_name").unwrap(), "IT");
    }

    #[test]
    fn test_extract_mapped_rows_bom() {
        // First column header starts with UTF-8 BOM: \u{feff}employee_code
        let csv_data = "\u{feff}employee_code,full_name,hire_date,department_name\nE001,Nguyen Van A,2023-01-01,IT";
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(csv_data.as_bytes(), &expected).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.get("employee_code").unwrap(), "E001");
    }

    #[test]
    fn test_extract_mapped_rows_semicolon_csv() {
        // Semicolon delimited CSV
        let csv_data =
            "employee_code;full_name;hire_date;department_name\nE001;Nguyen Van A;2023-01-01;IT";
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(csv_data.as_bytes(), &expected).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.get("employee_code").unwrap(), "E001");
        assert_eq!(result[0].1.get("department_name").unwrap(), "IT");
    }

    #[test]
    fn test_extract_mapped_rows_utf16_csv() {
        // UTF-16LE encoded semicolon CSV with BOM
        let csv_data =
            "employee_code;full_name;hire_date;department_name\nE001;Nguyen Van A;2023-01-01;IT\n";
        let utf16_bytes = csv_data.encode_utf16().collect::<Vec<u16>>();
        let mut file_bytes = vec![0xFF, 0xFE]; // LE BOM
        for val in utf16_bytes {
            let b = val.to_le_bytes();
            file_bytes.push(b[0]);
            file_bytes.push(b[1]);
        }

        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(&file_bytes, &expected).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.get("employee_code").unwrap(), "E001");
        assert_eq!(result[0].1.get("department_name").unwrap(), "IT");
    }

    #[test]
    fn test_extract_mapped_rows_empty_data() {
        // Only headers, no data
        let csv_data = "employee_code,full_name,hire_date,department_name\n";
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(csv_data.as_bytes(), &expected);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("File is empty or contains only headers"));
    }

    #[test]
    fn test_extract_mapped_rows_scrambled_columns() {
        // Order of columns is completely mixed up, with extra unneeded columns
        let csv_data = "department_name,extra1,full_name,extra2,employee_code,hire_date\nIT,ignore1,Nguyen Van A,ignore2,E001,2023-01-01";
        let expected = ["employee_code", "full_name", "hire_date", "department_name"];
        let result = extract_mapped_rows(csv_data.as_bytes(), &expected).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.get("employee_code").unwrap(), "E001");
        assert_eq!(result[0].1.get("department_name").unwrap(), "IT");
        assert_eq!(result[0].1.get("full_name").unwrap(), "Nguyen Van A");
    }
}

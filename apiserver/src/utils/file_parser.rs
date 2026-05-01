use calamine::{Data, DataType, Reader, Xls, Xlsx};
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
        if let Some(Ok(range)) = reader.worksheet_range_at(0) {
            for (idx, row) in range.rows().enumerate() {
                if idx == 0 {
                    continue; // Skip header
                }
                let line_number = idx + 1;
                let cols: Vec<String> = row.iter().map(|c| cell_to_string(Some(c))).collect();
                rows.push((line_number, cols));
            }
            return Ok(rows);
        }
    }

    // 2. Try XLS
    if let Ok(mut reader) = Xls::new(Cursor::new(file_bytes)) {
        if let Some(Ok(range)) = reader.worksheet_range_at(0) {
            for (idx, row) in range.rows().enumerate() {
                if idx == 0 {
                    continue; // Skip header
                }
                let line_number = idx + 1;
                let cols: Vec<String> = row.iter().map(|c| cell_to_string(Some(c))).collect();
                rows.push((line_number, cols));
            }
            return Ok(rows);
        }
    }

    // 3. Try CSV
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(Cursor::new(file_bytes));

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
                // Return an error or just skip? We'll return error if the format is fully broken
                // But for now, let's just collect the ones we can, or return error to let the caller know.
                // We will collect it as empty string so it gets handled as a blank row/error row
                rows.push((line_number, vec![]));
            }
        }
    }

    if has_records || !rows.is_empty() {
        return Ok(rows);
    }

    Err("Invalid file format or empty file. Supported formats: XLSX, XLS, CSV.".to_string())
}

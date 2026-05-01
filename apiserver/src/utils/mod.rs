pub mod chrono_helpers;
pub mod datetime;
pub mod debug;
pub mod excel_format_utils;
pub mod file_parser;
pub mod serde_helpers;
pub mod tz;

/// Normalizes an employee ID by trimming whitespace and removing leading zeros if the ID is purely numeric.
/// Empty or whitespace-only strings return "0" (consistent with agent-main behavior).
pub fn normalize_employee_id(id: &str) -> String {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return "0".to_string();
    }

    // Check if the ID is purely numeric
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        let normalized = trimmed.trim_start_matches('0');
        if normalized.is_empty() {
            "0".to_string()
        } else {
            normalized.to_string()
        }
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_employee_id() {
        // Trimming & Whitespace
        assert_eq!(normalize_employee_id("  123  "), "123");
        assert_eq!(normalize_employee_id("\t456\n"), "456");

        // Numeric Normalization (Leading Zeros)
        assert_eq!(normalize_employee_id("000123"), "123");
        assert_eq!(normalize_employee_id("0000"), "0");
        assert_eq!(normalize_employee_id("0"), "0");
        assert_eq!(normalize_employee_id("  00055  "), "55");
        assert_eq!(normalize_employee_id("0026"), "26"); // Issue: ensure 0026 becomes 26

        // Alphanumeric & Special (Protection)
        assert_eq!(normalize_employee_id("EMP001"), "EMP001");
        assert_eq!(normalize_employee_id("  EMP001  "), "EMP001");
        assert_eq!(normalize_employee_id("00A1"), "00A1");
        assert_eq!(normalize_employee_id("123-456"), "123-456");

        // Edge cases — empty/whitespace → "0" (consistent with agent-main)
        assert_eq!(normalize_employee_id(""), "0");
        assert_eq!(normalize_employee_id("   "), "0");
        assert_eq!(normalize_employee_id("00 123"), "00 123"); // Space breaks numeric check
    }
}

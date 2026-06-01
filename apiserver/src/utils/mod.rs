pub mod bigdecimal_helpers;
pub mod chrono_helpers;
pub mod datetime;
pub mod debug;
pub mod excel_format_utils;
pub mod file_parser;
pub mod i18n;
pub mod id;
pub mod masking;
pub mod serde_helpers;
pub mod tz;

/// Returns the base storage directory from the STORAGE_DIR environment variable, defaulting to "storage".
pub fn get_storage_dir() -> String {
    std::env::var("STORAGE_DIR").unwrap_or_else(|_| "storage".to_string())
}

/// Normalizes an employee ID by trimming whitespace and removing leading zeros if the ID is purely numeric.
/// Empty or whitespace-only strings return "0" (consistent with agent-main behavior).
pub fn normalize_employee_id(id: &str) -> String {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return "0".to_string();
    }

    let mut digits = String::new();
    for c in trimmed.chars().rev() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else {
            break;
        }
    }

    if digits.is_empty() {
        return trimmed.to_string();
    }

    let reversed: String = digits.chars().rev().collect();
    if let Ok(num) = reversed.parse::<u64>() {
        num.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_storage_dir() {
        // We only test that it returns a valid string without panicking,
        // to avoid race conditions with std::env::set_var in concurrent tests.
        let dir = get_storage_dir();
        assert!(!dir.is_empty());
    }

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
        assert_eq!(normalize_employee_id("0026"), "26");

        // Alphanumeric & Special (New right-to-left logic)
        assert_eq!(normalize_employee_id("EMP001"), "1");
        assert_eq!(normalize_employee_id("  EMP001  "), "1");
        assert_eq!(normalize_employee_id("00A1"), "1");
        assert_eq!(normalize_employee_id("123-456"), "456");
        assert_eq!(normalize_employee_id("1EH000254"), "254"); // User's requirement

        // Fallback when no trailing digits
        assert_eq!(normalize_employee_id("ABC"), "ABC");
        assert_eq!(normalize_employee_id("EMP-"), "EMP-");

        // Edge cases — empty/whitespace → "0"
        assert_eq!(normalize_employee_id(""), "0");
        assert_eq!(normalize_employee_id("   "), "0");
    }
}

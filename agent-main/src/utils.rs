/// Normalize employee ID by trimming whitespace and removing leading zeros if purely numeric
///
/// The central server expects normalized IDs without leading zeros for numeric IDs.
/// This function matches the normalization logic in the API server.
///
/// Examples:
/// - "0058" -> "58"
/// - "0026" -> "26"
/// - "  123  " -> "123"
/// - "1234" -> "1234"
/// - "0000" -> "0" (special case: all zeros becomes single zero)
/// - "EMP001" -> "EMP001" (alphanumeric: preserved as-is)
/// - "00A1" -> "00A1" (not purely numeric: preserved)
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
    fn test_normalize_employee_id_removes_leading_zeros() {
        assert_eq!(normalize_employee_id("0058"), "58");
        assert_eq!(normalize_employee_id("0026"), "26");
        assert_eq!(normalize_employee_id("0001"), "1");
        assert_eq!(normalize_employee_id("000123"), "123");
    }

    #[test]
    fn test_normalize_employee_id_no_leading_zeros() {
        assert_eq!(normalize_employee_id("1234"), "1234");
        assert_eq!(normalize_employee_id("58"), "58");
        assert_eq!(normalize_employee_id("9"), "9");
    }

    #[test]
    fn test_normalize_employee_id_all_zeros() {
        assert_eq!(normalize_employee_id("0000"), "0");
        assert_eq!(normalize_employee_id("00"), "0");
        assert_eq!(normalize_employee_id("0"), "0");
    }

    #[test]
    fn test_normalize_employee_id_single_digit() {
        assert_eq!(normalize_employee_id("5"), "5");
        assert_eq!(normalize_employee_id("0"), "0");
    }

    #[test]
    fn test_normalize_employee_id_mixed() {
        assert_eq!(normalize_employee_id("0100"), "100");
        assert_eq!(normalize_employee_id("00500"), "500");
        assert_eq!(normalize_employee_id("00050"), "50");
    }

    #[test]
    fn test_normalize_employee_id_whitespace() {
        assert_eq!(normalize_employee_id("  123  "), "123");
        assert_eq!(normalize_employee_id("\t456\n"), "456");
        assert_eq!(normalize_employee_id("  00055  "), "55");
    }

    #[test]
    fn test_normalize_employee_id_alphanumeric_preserved() {
        // Alphanumeric IDs are preserved as-is (not normalized)
        assert_eq!(normalize_employee_id("EMP001"), "EMP001");
        assert_eq!(normalize_employee_id("00A1"), "00A1");
        assert_eq!(normalize_employee_id("123-456"), "123-456");
        assert_eq!(normalize_employee_id("  EMP001  "), "EMP001");
    }

    #[test]
    fn test_normalize_employee_id_empty() {
        assert_eq!(normalize_employee_id(""), "0");
        assert_eq!(normalize_employee_id("   "), "0");
    }
}

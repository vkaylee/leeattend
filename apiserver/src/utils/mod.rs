pub mod datetime;
pub mod debug;
pub mod serde_helpers;

/// Initializes the environment variables from the .env file, overriding existing ones.
pub fn init_env() {
    init_env_from_path(".env");
}

/// Initializes the environment variables from a specific path, overriding existing ones.
pub fn init_env_from_path(path: impl AsRef<std::path::Path>) {
    if let Ok(entries) = dotenvy::from_path_iter(path) {
        for (key, value) in entries.flatten() {
            unsafe {
                std::env::set_var(key, value);
            }
        }
    } else {
        dotenvy::dotenv().ok();
    }
}

/// Normalizes an employee ID by trimming whitespace and removing leading zeros if the ID is purely numeric.
pub fn normalize_employee_id(id: &str) -> String {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return "".to_string();
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
    use std::io::Write;

    #[test]
    fn test_init_env_override() {
        let mut tmp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp_file, "TEST_VAR_OVERRIDE=new_value").unwrap();
        
        // Set a system variable
        std::env::set_var("TEST_VAR_OVERRIDE", "system_value");
        assert_eq!(std::env::var("TEST_VAR_OVERRIDE").unwrap(), "system_value");
        
        // Override it
        init_env_from_path(tmp_file.path());
        
        // Verify it was changed
        assert_eq!(std::env::var("TEST_VAR_OVERRIDE").unwrap(), "new_value");
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
        assert_eq!(normalize_employee_id("0026"), "26"); // Issue: ensure 0026 becomes 26
        
        // Alphanumeric & Special (Protection)
        assert_eq!(normalize_employee_id("EMP001"), "EMP001");
        assert_eq!(normalize_employee_id("  EMP001  "), "EMP001");
        assert_eq!(normalize_employee_id("00A1"), "00A1");
        assert_eq!(normalize_employee_id("123-456"), "123-456");
        
        // Edge cases
        assert_eq!(normalize_employee_id(""), "");
        assert_eq!(normalize_employee_id("   "), "");
        assert_eq!(normalize_employee_id("00 123"), "00 123"); // Space breaks numeric check
    }
}

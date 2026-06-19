use super::allowlist::ALLOWLIST;
use super::tables::TENANT_TABLES;
use std::fs;
use std::path::PathBuf;

/// Extract SQL string literals from Rust source code.
/// Handles both `r#"..."#` raw strings and `"..."` regular strings.
pub fn strip_comments(content: &str) -> String {
    let mut res = String::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut in_raw_string = false;

    while i < chars.len() {
        if in_line_comment {
            if chars[i] == '\n' {
                in_line_comment = false;
                res.push('\n');
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if chars[i] == '\n' {
                res.push('\n');
            } else if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if !in_string && !in_raw_string {
            if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                in_line_comment = true;
                i += 2;
                continue;
            }
            if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                in_block_comment = true;
                i += 2;
                continue;
            }
        }

        if in_raw_string {
            if chars[i] == '"' && i + 1 < chars.len() && chars[i + 1] == '#' {
                in_raw_string = false;
                res.push('"');
                res.push('#');
                i += 2;
                continue;
            }
        } else if in_string {
            if chars[i] == '\\' && i + 1 < chars.len() {
                res.push(chars[i]);
                res.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if chars[i] == '"' {
                in_string = false;
            }
        } else if i + 2 < chars.len()
            && chars[i] == 'r'
            && chars[i + 1] == '#'
            && chars[i + 2] == '"'
        {
            in_raw_string = true;
        } else if chars[i] == '"' {
            in_string = true;
        }

        res.push(chars[i]);
        i += 1;
    }
    res
}

/// Extract SQL string literals from Rust source code.
/// Handles both `r#"..."#` raw strings and `"..."` regular strings.
pub fn extract_sql_strings(content: &str) -> Vec<(usize, String)> {
    let stripped = strip_comments(content);
    let mut results = Vec::new();
    let chars: Vec<char> = stripped.chars().collect();
    let mut i = 0;
    let mut line_no = 1;

    while i < chars.len() {
        if chars[i] == '\n' {
            line_no += 1;
            i += 1;
            continue;
        }

        // Check for raw string: r#"" ... ""#
        if i + 2 < chars.len() && chars[i] == 'r' && chars[i + 1] == '#' && chars[i + 2] == '"' {
            let start_line = line_no;
            let mut s = String::new();
            i += 3;
            let mut found = false;
            while i < chars.len() {
                if chars[i] == '\n' {
                    line_no += 1;
                }
                if chars[i] == '"' && i + 1 < chars.len() && chars[i + 1] == '#' {
                    found = true;
                    i += 2;
                    break;
                }
                s.push(chars[i]);
                i += 1;
            }
            if found && has_sql_keyword(&s) {
                results.push((start_line, s));
            }
            continue;
        }

        // Check for raw string: r" ... "
        if i + 1 < chars.len() && chars[i] == 'r' && chars[i + 1] == '"' {
            let start_line = line_no;
            let mut s = String::new();
            i += 2;
            let mut found = false;
            while i < chars.len() {
                if chars[i] == '\n' {
                    line_no += 1;
                }
                if chars[i] == '"' {
                    found = true;
                    i += 1;
                    break;
                }
                s.push(chars[i]);
                i += 1;
            }
            if found && has_sql_keyword(&s) {
                results.push((start_line, s));
            }
            continue;
        }

        // Check for regular string: " ... "
        if chars[i] == '"' {
            let start_line = line_no;
            let mut s = String::new();
            i += 1;
            let mut found = false;
            let mut escaped = false;
            while i < chars.len() {
                if chars[i] == '\n' {
                    line_no += 1;
                }
                if escaped {
                    s.push(chars[i]);
                    escaped = false;
                } else if chars[i] == '\\' {
                    escaped = true;
                } else if chars[i] == '"' {
                    found = true;
                    i += 1;
                    break;
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if found && has_sql_keyword(&s) {
                results.push((start_line, s));
            }
            continue;
        }

        i += 1;
    }
    results
}

pub fn has_sql_keyword(s: &str) -> bool {
    let upper = s.to_uppercase();
    upper.contains("SELECT")
        || upper.contains("INSERT INTO")
        || upper.contains("UPDATE ")
        || upper.contains("DELETE FROM")
}

/// Check if a SQL string references a tenant-scoped table.
pub fn find_tenant_tables_in_query(sql: &str) -> Vec<&'static str> {
    let sql_upper = sql.to_uppercase();
    let mut found = Vec::new();

    for &table in TENANT_TABLES {
        let table_upper = table.to_uppercase();
        // Check for table reference in FROM, INTO, UPDATE, JOIN
        let patterns = [
            format!("FROM {table_upper}"),
            format!("FROM {table_upper} "),
            format!("JOIN {table_upper}"),
            format!("JOIN {table_upper} "),
            format!("INTO {table_upper}"),
            format!("INTO {table_upper} "),
            format!("UPDATE {table_upper}"),
            format!("UPDATE {table_upper} "),
            format!("DELETE FROM {table_upper}"),
        ];

        for pat in &patterns {
            if sql_upper.contains(pat.as_str()) {
                found.push(table);
                break;
            }
        }
    }
    found
}

pub fn has_workspace_id(sql: &str) -> bool {
    let lower = sql.to_lowercase();
    lower.contains("workspace_id")
}

pub fn is_allowlisted(file_name: &str, table: &str) -> bool {
    ALLOWLIST
        .iter()
        .any(|(f, t, _)| file_name.ends_with(f) && *t == table)
}

pub fn get_rs_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(get_rs_files(&path));
            } else if path.extension().is_some_and(|e| e == "rs") {
                files.push(path);
            }
        }
    }
    files
}

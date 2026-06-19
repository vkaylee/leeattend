use super::allowlist::ALLOWLIST;
use super::helpers::{
    extract_sql_strings, find_tenant_tables_in_query, get_rs_files, has_workspace_id,
    is_allowlisted,
};
use super::tables::TENANT_TABLES;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[test]
fn all_tenant_queries_must_include_workspace_id() {
    // Resolve the source directory relative to the test binary
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    assert!(src_dir.exists(), "Source directory not found: {src_dir:?}");

    let rs_files = get_rs_files(&src_dir);
    assert!(!rs_files.is_empty(), "No .rs files found in {src_dir:?}");

    let mut violations: Vec<String> = Vec::new();
    let mut scanned_queries = 0u32;
    let mut checked_tables = HashSet::new();

    for file_path in &rs_files {
        let content = fs::read_to_string(file_path).expect("Failed to read file");
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip test-only files and this guard test itself
        if file_name == "workspace_id_guard.rs"
            || file_name == "query_tests.rs"
            || file_name == "handler_tests.rs"
            || file_name == "builder_tests.rs"
        {
            continue;
        }

        let sql_strings = extract_sql_strings(&content);

        for (line_no, sql) in &sql_strings {
            scanned_queries += 1;
            let tenant_tables = find_tenant_tables_in_query(sql);

            for table in tenant_tables {
                checked_tables.insert(table);

                if !has_workspace_id(sql) && !is_allowlisted(file_name, table) {
                    violations.push(format!(
                        "  {file_name}:{line_no} — table `{table}` queried without workspace_id"
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔴 WORKSPACE_ID GUARD FAILURE\n\
             ════════════════════════════════════════════════════════════\n\
             {} query/queries touch tenant-scoped tables WITHOUT workspace_id:\n\n{}\n\n\
             FIX: Add `WHERE workspace_id = $N` to each query, or add\n\
             an entry to ALLOWLIST in workspace_id_guard/allowlist.rs with a reason.\n\
             ════════════════════════════════════════════════════════════\n",
        violations.len(),
        violations.join("\n")
    );

    // Sanity check: ensure we scanned a meaningful number of queries
    assert!(
        scanned_queries > 20,
        "Only scanned {scanned_queries} queries — SQL extraction may be broken"
    );

    eprintln!(
        "✅ workspace_id guard: scanned {} SQL queries, {} tenant tables covered",
        scanned_queries,
        checked_tables.len()
    );
}

/// Ensure ALLOWLIST entries reference valid tenant tables
/// (no stale/typo entries).
#[test]
fn allowlist_entries_reference_valid_tables() {
    let tenant_set: HashSet<&str> = TENANT_TABLES.iter().copied().collect();

    for (file, table, reason) in ALLOWLIST {
        assert!(
            tenant_set.contains(table)
                || [
                    "shift_rosters",
                    "employee_shifts",
                    "payslip_segments",
                    "payslip_incentives",
                    "payroll_incentive_tiers",
                    "employee_salary_components",
                    "overtime_rules",
                ]
                .contains(table),
            "ALLOWLIST entry ({file}, {table}) references unknown table. Reason: {reason}"
        );
    }
}

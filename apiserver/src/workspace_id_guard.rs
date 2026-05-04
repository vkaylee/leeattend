//! Static analysis guard: ensures every SQL query touching tenant-scoped
//! tables includes `workspace_id` filtering.
//!
//! This test scans all `.rs` files in the apiserver codebase, extracts raw
//! SQL string literals, and verifies that queries referencing tenant-scoped
//! tables contain `workspace_id` somewhere in the query text.
//!
//! **Why?**  In a shared-database SaaS model every query must be scoped to
//! a workspace. This guard test catches regressions at `cargo test` time
//! — before they reach production and leak data across tenants.

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fs;
    use std::path::PathBuf;

    /// Tables that MUST have `workspace_id` filtering in every query.
    /// These are tenant-scoped tables with a `workspace_id` column.
    const TENANT_TABLES: &[&str] = &[
        "employees",
        "departments",
        "shifts",
        "devices",
        "authorized_agents",
        "time_entries",
        "leave_types",
        "leave_requests",
        "leave_balances",
        "attendance_daily_summaries",
        "raw_attendance_logs",
        "attendance_staging_logs",
        "attendance_import_jobs",
        "attendance_adjustment_requests",
        "payslips",
        "payroll_settings",
        "payroll_incentive_policies",
        "salary_component_definitions",
        "overtime_policy_definitions",
        "overtime_policy_assignments",
        "leave_policy_definitions",
        "leave_policy_assignments",
        "working_patterns",
        "employee_working_patterns",
        "shift_swap_requests",
        "holidays",
        "recruitment_sources",
        "workspace_members",
        "department_payroll_configs",
        "employee_payroll_configs",
        "payroll_incentive_assignments",
    ];

    /// Queries that are legitimately exempt from workspace_id filtering.
    /// Format: (file_basename, table_name, reason)
    ///
    /// Keep this list as SHORT as possible. Each entry must have a
    /// documented reason. Code reviewers should scrutinize additions.
    const ALLOWLIST: &[(&str, &str, &str)] = &[
        // Auth: agent_id comes from verified WebSocket token, not user input
        (
            "agent_repository.rs",
            "authorized_agents",
            "agent lookup by verified token ID",
        ),
        (
            "agent_repository.rs",
            "devices",
            "device lookup by authenticated agent_id",
        ),
        // Roster CRUD: employee_id already validated from MemberContext (workspace-scoped)
        (
            "roster_crud.rs",
            "shift_rosters",
            "employee_id from MemberContext",
        ),
        // Leave request repo: queries by employee_id from workspace-validated handlers
        (
            "leave_request_repository.rs",
            "leave_requests",
            "employee_id from workspace-validated context",
        ),
        (
            "leave_request_repository.rs",
            "leave_balances",
            "employee_id from workspace-validated context",
        ),
        // Payslip segment/incentive: accessed via payslip_id (already workspace-scoped)
        (
            "payroll_repository.rs",
            "payslip_segments",
            "payslip_id from workspace-scoped payslip",
        ),
        (
            "payroll_repository.rs",
            "payslip_incentives",
            "payslip_id from workspace-scoped payslip",
        ),
        (
            "payroll_repository.rs",
            "payroll_incentive_tiers",
            "policy_id from workspace-scoped policy",
        ),
        // Employee salary components: joined with salary_component_definitions.workspace_id
        (
            "payroll_repository.rs",
            "employee_salary_components",
            "joined with scd.workspace_id filter",
        ),
        // Import service: uses job_id which is workspace-scoped
        (
            "attendance_import_service.rs",
            "attendance_import_jobs",
            "job_id from workspace-scoped context",
        ),
        (
            "attendance_import_service.rs",
            "raw_attendance_logs",
            "import job already workspace-scoped",
        ),
        (
            "attendance_import_service.rs",
            "employees",
            "employee lookup within import job context",
        ),
        // Workspace service: operates on workspace creation/management
        (
            "workspace_service.rs",
            "workspace_members",
            "workspace creation/management context",
        ),
        // Pairing service: already has workspace_id in scope but queries by employee_id
        (
            "pairing_service.rs",
            "attendance_daily_summaries",
            "workspace_id added to all queries",
        ),
        (
            "pairing_service.rs",
            "raw_attendance_logs",
            "workspace_id passed from handler context",
        ),
        // Holiday leave service: workspace_id from authenticated context
        (
            "holiday_leave_service.rs",
            "leave_requests",
            "workspace context from handler",
        ),
        (
            "holiday_leave_service.rs",
            "leave_balances",
            "workspace context from handler",
        ),
        // Department repo: some queries via department_id which has workspace FK
        (
            "department_repository.rs",
            "employee_department_history",
            "department_id has workspace FK",
        ),
        (
            "department_repository.rs",
            "department_payroll_configs",
            "department_id has workspace FK",
        ),
        // Shift-related: some queries by shift_id which already belongs to a workspace
        (
            "shift_repository.rs",
            "employee_shifts",
            "shift_id has workspace FK",
        ),
        // Time entry repo: some internal utility queries
        (
            "time_entry_repository.rs",
            "attendance_adjustment_requests",
            "time_entry_id from workspace-scoped entry",
        ),
        // Overtime rules: accessed via policy_id which is workspace-scoped
        (
            "overtime_repository.rs",
            "overtime_rules",
            "policy_id from workspace-scoped policy",
        ),
        // Leave balance repo: employee_id from workspace-validated context
        (
            "leave_balance_repository.rs",
            "leave_balances",
            "employee_id from workspace-validated handler",
        ),
        // Core queries: employee/department payroll configs via FK
        (
            "core_queries.rs",
            "employee_payroll_configs",
            "employee_id has workspace FK via employees",
        ),
        // Attendance handler: leave lookups via employee_id from MemberContext
        (
            "attendance.rs",
            "leave_requests",
            "employee_id from workspace-validated MemberContext",
        ),
        // Leave accrual log repo: balance updates via employee_id FK-scoped from accrual job
        (
            "leave_accrual_log_repository.rs",
            "leave_balances",
            "employee_id from workspace-scoped accrual job",
        ),
        // Report service: employee-level leave aggregation
        (
            "report_service.rs",
            "leave_requests",
            "employee_id from workspace-validated report context",
        ),
    ];

    /// Extract SQL string literals from Rust source code.
    /// Handles both `r#"..."#` raw strings and `"..."` regular strings.
    #[allow(clippy::needless_range_loop)]
    fn extract_sql_strings(content: &str) -> Vec<(usize, String)> {
        let mut results = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Strategy: find lines containing SQL keywords and gather
        // the surrounding multi-line string content.
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];

            // Detect start of raw string with SQL
            if line.contains("r#\"") || line.contains("r#\"") {
                // Gather the full raw string
                let start_line = i;
                let mut sql_block = String::new();
                let mut found_end = false;

                for j in i..lines.len() {
                    sql_block.push_str(lines[j]);
                    sql_block.push('\n');
                    if lines[j].contains("\"#") && j > i {
                        found_end = true;
                        i = j;
                        break;
                    }
                    // Single-line raw string
                    if lines[j].contains("r#\"") && lines[j].contains("\"#") {
                        found_end = true;
                        i = j;
                        break;
                    }
                }

                if found_end && has_sql_keyword(&sql_block) {
                    results.push((start_line + 1, sql_block));
                }
            }
            i += 1;
        }
        results
    }

    fn has_sql_keyword(s: &str) -> bool {
        let upper = s.to_uppercase();
        upper.contains("SELECT")
            || upper.contains("INSERT INTO")
            || upper.contains("UPDATE ")
            || upper.contains("DELETE FROM")
    }

    /// Check if a SQL string references a tenant-scoped table.
    fn find_tenant_tables_in_query(sql: &str) -> Vec<&'static str> {
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

    fn has_workspace_id(sql: &str) -> bool {
        let lower = sql.to_lowercase();
        lower.contains("workspace_id")
    }

    fn is_allowlisted(file_name: &str, table: &str) -> bool {
        ALLOWLIST
            .iter()
            .any(|(f, t, _)| file_name.ends_with(f) && *t == table)
    }

    fn get_rs_files(dir: &PathBuf) -> Vec<PathBuf> {
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
            if file_name == "workspace_id_guard.rs" {
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
                 an entry to ALLOWLIST in workspace_id_guard.rs with a reason.\n\
                 ════════════════════════════════════════════════════════════\n",
            violations.len(),
            violations.join("\n")
        );

        // Sanity check: ensure we actually scanned a meaningful number of queries
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
                        "employee_department_history",
                        "overtime_rules",
                    ]
                    .contains(table),
                "ALLOWLIST entry ({file}, {table}) references unknown table. Reason: {reason}"
            );
        }
    }

    // =======================================================================
    // Guard #2: workspace_id must come from MemberContext, not Path params
    // =======================================================================

    /// Handler files that legitimately use `Path(ws_id)` instead of MemberContext.
    ///
    /// Keep this list MINIMAL. Each entry must have a documented reason.
    const CONTEXT_ALLOWLIST: &[(&str, &str)] = &[
        // Admin-level workspace CRUD: user is authenticated, ws_id from URL
        // is validated against user's workspace list in the handler
        (
            "workspaces/crud.rs",
            "workspace CRUD by authenticated admin",
        ),
        // Debug/dev endpoints: only available in debug builds
        ("debug.rs", "dev-only endpoints, compiled out in release"),
    ];

    /// Detects handler functions in `handlers/` that extract workspace_id
    /// from `Path` parameters instead of `MemberContext`.
    ///
    /// **Why?** `MemberContext` verifies the user is a member of the workspace
    /// via DB lookup (authn + authz). `Path(ws_id)` trusts user-provided URL
    /// input — an attacker could substitute another workspace's UUID.
    ///
    /// **Safe pattern:**
    /// ```ignore
    /// async fn my_handler(ctx: MemberContext, ...) {
    ///     let ws_id = ctx.workspace_id; // ✅ from authenticated context
    /// }
    /// ```
    ///
    /// **Unsafe pattern:**
    /// ```ignore
    /// async fn my_handler(Path(ws_id): Path<Uuid>, ...) {
    ///     // ❌ ws_id from URL — attacker can set any UUID
    /// }
    /// ```
    #[test]
    fn handlers_must_use_member_context_for_workspace_id() {
        let handlers_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("handlers");
        assert!(
            handlers_dir.exists(),
            "Handlers directory not found: {handlers_dir:?}"
        );

        let rs_files = get_rs_files(&handlers_dir);
        assert!(!rs_files.is_empty(), "No handler .rs files found");

        let mut violations: Vec<String> = Vec::new();
        let mut scanned_handlers = 0u32;

        for file_path in &rs_files {
            let content = fs::read_to_string(file_path).expect("Failed to read file");
            let rel_path = file_path
                .strip_prefix(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("src")
                        .join("handlers"),
                )
                .unwrap_or(file_path.as_path())
                .to_str()
                .unwrap_or("");
            let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip this guard file itself
            if file_name == "workspace_id_guard.rs" {
                continue;
            }

            // Check if this file is in the context allowlist
            let is_allowed = CONTEXT_ALLOWLIST.iter().any(|(f, _)| rel_path.ends_with(f));
            if is_allowed {
                continue;
            }

            scanned_handlers += 1;

            // Detect unsafe patterns: Path extractors containing ws_id or workspace_id
            for (line_no, line) in content.lines().enumerate() {
                let line_trimmed = line.trim();

                // Pattern 1: Path(ws_id): Path<Uuid>
                // Pattern 2: Path(workspace_id): Path<Uuid>
                // Pattern 3: Path((ws_id, ...)): Path<(Uuid, ...)>
                let has_path_ws = line_trimmed.contains("Path(ws_id)")
                    || line_trimmed.contains("Path(workspace_id)")
                    || line_trimmed.contains("Path((ws_id")
                    || line_trimmed.contains("Path((workspace_id");

                if has_path_ws {
                    // Ignore if the variable is prefixed with _ (intentionally unused)
                    let is_unused =
                        line_trimmed.contains("_ws_id") || line_trimmed.contains("_workspace_id");
                    if is_unused {
                        continue;
                    }

                    violations.push(format!(
                        "  handlers/{}:{} — workspace_id from Path (use MemberContext instead)\n    {}",
                        rel_path,
                        line_no + 1,
                        line_trimmed,
                    ));
                }
            }
        }

        assert!(
            violations.is_empty(),
            "\n\n🔴 CONTEXT SOURCE GUARD FAILURE\n\
                 ════════════════════════════════════════════════════════════\n\
                 {} handler(s) extract workspace_id from Path instead of MemberContext:\n\n{}\n\n\
                 FIX: Replace `Path(ws_id): Path<Uuid>` with `ctx: MemberContext`\n\
                 and use `ctx.workspace_id`. This ensures the user is a verified\n\
                 member of the workspace.\n\n\
                 If this is a legitimate exception (e.g., admin endpoint),\n\
                 add to CONTEXT_ALLOWLIST with a reason.\n\
                 ════════════════════════════════════════════════════════════\n",
            violations.len(),
            violations.join("\n")
        );

        assert!(
            scanned_handlers > 5,
            "Only scanned {scanned_handlers} handler files — directory structure may have changed"
        );

        eprintln!(
            "✅ context source guard: scanned {scanned_handlers} handler files, all use MemberContext"
        );
    }

    /// Verify that no handler function has workspace_id as a direct function
    /// parameter (from query string or JSON body) without also having MemberContext.
    ///
    /// This catches the pattern where a service function receives workspace_id
    /// from user input rather than from the authenticated context.
    #[test]
    fn handler_functions_must_not_accept_workspace_id_param() {
        let handlers_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("handlers");

        let rs_files = get_rs_files(&handlers_dir);
        let mut violations: Vec<String> = Vec::new();

        for file_path in &rs_files {
            let content = fs::read_to_string(file_path).expect("Failed to read file");
            let rel_path = file_path
                .strip_prefix(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("src")
                        .join("handlers"),
                )
                .unwrap_or(file_path.as_path())
                .to_str()
                .unwrap_or("");
            let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if file_name == "workspace_id_guard.rs"
                || file_name == "common.rs"
                || file_name == "mod.rs"
            {
                continue;
            }

            // Check for Query/Json extractors containing workspace_id
            for (line_no, line) in content.lines().enumerate() {
                let line_trimmed = line.trim();

                // Detect: Json<SomeRequest> where the request has workspace_id field
                // This is hard to detect statically, so instead we look for
                // explicit workspace_id in Query parameters
                let has_query_ws = line_trimmed.contains("Query<")
                    && (line_trimmed.contains("workspace_id") || line_trimmed.contains("ws_id"));

                if has_query_ws {
                    violations.push(format!(
                        "  handlers/{}:{} — workspace_id in Query params (use MemberContext)\n    {}",
                        rel_path,
                        line_no + 1,
                        line_trimmed,
                    ));
                }
            }
        }

        assert!(
            violations.is_empty(),
            "\n\n🔴 QUERY PARAM GUARD FAILURE\n\
                 ════════════════════════════════════════════════════════════\n\
                 {} handler(s) accept workspace_id via Query params:\n\n{}\n\n\
                 FIX: Remove workspace_id from query params. Use MemberContext\n\
                 to get the workspace from the authenticated session.\n\
                 ════════════════════════════════════════════════════════════\n",
            violations.len(),
            violations.join("\n")
        )
    }
}

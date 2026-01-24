
mod common;
use common::db::TestDatabase;

#[tokio::test]
async fn test_all_tables_have_workspace_id() {
    let db = TestDatabase::new().await;
    let pool = &db.pool;

    // Define tables that are global or system-related and do not require workspace isolation.
    let whitelist = vec![
        "_sqlx_migrations",
        "workspaces",
        "users",
        "sessions",
        // TODO: The following tables currently lack workspace_id and should be migrated to include it for strict isolation.
        // Once migrated, remove them from this whitelist.
        "attendance_adjustment_requests",
        "department_default_shifts",
        "department_payroll_configs",
        "employee_department_history",
        "employee_emergency_contacts",
        "employee_payroll_configs",
        "employee_profiles",
        "employee_salary_components",
        "employee_shifts",
        "employment_periods",
        "leave_balances",
        "leave_requests",
        "overtime_rules",
        "payroll_incentive_tiers",
        "payslip_incentives",
        "payslip_segments",
        "payslips",
        "shift_rosters",
        "working_conditions",
        "working_pattern_details",
    ];

    let rows = sqlx::query!(
        r#"
        SELECT t.table_name
        FROM information_schema.tables t
        WHERE t.table_schema = 'public'
          AND t.table_type = 'BASE TABLE'
          AND NOT EXISTS (
            SELECT 1
            FROM information_schema.columns c
            WHERE c.table_schema = t.table_schema
              AND c.table_name = t.table_name
              AND c.column_name = 'workspace_id'
          )
        "#
    )
    .fetch_all(pool)
    .await
    .expect("Failed to query database schema");

    let mut violating_tables = Vec::new();
    for row in rows {
        let table_name = row.table_name.unwrap_or_default();
        if !whitelist.contains(&table_name.as_str()) {
            violating_tables.push(table_name);
        }
    }

    if !violating_tables.is_empty() {
        // Sort for deterministic output
        violating_tables.sort();
        
        panic!(
            "VIOLATION: The following tables are missing the 'workspace_id' column, which is required for multi-tenancy isolation:\n\n{:#?}\n\nIf a table is intentionally global (e.g. system config), add it to the whitelist in 'apiserver/tests/schema_isolation_tests.rs'.",
            violating_tables
        );
    }
}

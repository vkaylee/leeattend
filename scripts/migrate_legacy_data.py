#!/usr/bin/env python3
"""
Legacy Database Migration Script
=================================
Migrates data from a legacy single-database PostgreSQL backup (restored into db_legacy)
to the new multi-tenant architecture with separate System DB and per-workspace Tenant DBs.

Usage:
    python3 scripts/migrate_legacy_data.py

Environment Variables (with defaults for local docker-compose):
    LEGACY_DB_URL  - postgres://postgres:postgres@localhost:5433/railway
    SYSTEM_DB_URL  - postgres://admin:change_this_password_immediately@localhost:5432/leeattend
"""

import os
import sys
import time
import json
import psycopg2
import psycopg2.extras
import uuid
from contextlib import contextmanager

# ─────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────

LEGACY_DB_URL = os.getenv("LEGACY_DB_URL", "postgres://postgres:postgres@localhost:5433/railway")
SYSTEM_DB_URL = os.getenv(
    "SYSTEM_DB_URL", "postgres://admin:change_this_password_immediately@localhost:5432/leeattend"
)

# Tables to SKIP entirely
SKIP_TABLES = {
    "_sqlx_migrations",
    "sessions",
    "system_roles",  # seed data from migrations
    "user_system_roles",  # will re-create after migration
    "workspace_roles",  # seed data from migrations
    "workspace_role_permissions",  # seed data from migrations
    "employment_periods",  # merged into employees.hire_date/termination_date
    "notifications",  # skip per user decision
}

# Tables that go into SYSTEM DB (not tenant)
SYSTEM_TABLES = {
    "users",
    "workspaces",
    "workspace_members",
    "authorized_agents",
    # audit_logs → SKIP per user decision
    # member_extra_roles → check if data exists
    # workspace_member_extra_permissions → check if data exists
}

# Partitioned tables in old DB that merge into single parent in new
PARTITIONED_PREFIXES = [
    "attendance_staging_logs_",
    "audit_logs_",
]


# ─────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────


def log(msg: str):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}", flush=True)


def log_ok(msg: str):
    print(f"[{time.strftime('%H:%M:%S')}] ✅ {msg}", flush=True)


def log_warn(msg: str):
    print(f"[{time.strftime('%H:%M:%S')}] ⚠️  {msg}", flush=True)


def log_err(msg: str):
    print(f"[{time.strftime('%H:%M:%S')}] ❌ {msg}", flush=True)


@contextmanager
def connect(url: str, autocommit: bool = False):
    """Context manager for database connections."""
    conn = psycopg2.connect(url)
    if autocommit:
        conn.autocommit = True
    try:
        yield conn
    finally:
        conn.close()


def get_table_columns(conn, table_name: str) -> list[str]:
    """Get column names for a table."""
    cur = conn.cursor()
    cur.execute(
        """
        SELECT column_name FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = %s
        ORDER BY ordinal_position
    """,
        (table_name,),
    )
    return [row[0] for row in cur.fetchall()]


def get_table_row_count(conn, table_name: str) -> int:
    """Get row count for a table."""
    cur = conn.cursor()
    try:
        cur.execute(f'SELECT COUNT(*) FROM public."{table_name}"')
        return cur.fetchone()[0]
    except psycopg2.Error:
        conn.rollback()
        return 0


def table_exists(conn, table_name: str) -> bool:
    """Check if a table exists."""
    cur = conn.cursor()
    cur.execute(
        """
        SELECT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = %s
        )
    """,
        (table_name,),
    )
    return cur.fetchone()[0]


def get_all_tables(conn) -> list[str]:
    """Get all user tables in public schema."""
    cur = conn.cursor()
    cur.execute("""
        SELECT tablename FROM pg_tables
        WHERE schemaname = 'public'
        ORDER BY tablename
    """)
    return [row[0] for row in cur.fetchall()]


def get_json_columns(conn, table_name: str) -> set[str]:
    """Get columns of JSON or JSONB type in a table."""
    cur = conn.cursor()
    cur.execute(
        """
        SELECT column_name FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = %s
          AND data_type IN ('json', 'jsonb')
    """,
        (table_name,),
    )
    return {row[0] for row in cur.fetchall()}


# ─────────────────────────────────────────────────────────────────────
# Column Mapping / Transformation
# ─────────────────────────────────────────────────────────────────────

# source_col → target_col renames
COLUMN_RENAMES = {
    "working_pattern_details": {
        "day_of_week": "schedule_day",
    },
    "employee_working_patterns": {
        "employee_id": "employment_id",
    },
    "raw_attendance_logs": {
        "employee_id": "employment_id",
    },
    "time_entries": {
        "employee_id": "employment_id",
    },
    "attendance_daily_summaries": {
        "employee_id": "employment_id",
    },
    "leave_balances": {
        "employee_id": "employment_id",
    },
    "leave_requests": {
        "employee_id": "employment_id",
    },
    "shift_rosters": {
        "employee_id": "employment_id",
    },
    "working_conditions": {
        "employee_id": "employment_id",
    },
    "employee_contracts": {
        "employee_id": "employment_id",
    },
    "employee_salary_components": {
        "employee_id": "employment_id",
    },
    "employee_shifts": {
        "employee_id": "employment_id",
    },
    "leave_accrual_logs": {
        "employee_id": "employment_id",
    },
    "overtime_request_employees": {
        "employee_id": "employment_id",
    },
    "payslips": {
        "employee_id": "employment_id",
    },
}

# Extra columns to add with default values when inserting
EXTRA_COLUMNS = {
    "employees": {
        "original_hire_date": "hire_date",  # copy from hire_date
    },
    "working_patterns": {
        "pattern_type": "'weekly'",
        "cycle_length_days": "NULL",
    },
    "employee_working_patterns": {
        "anchor_date": "NULL",
    },
}


def migrate_table_data(
    src_conn,
    dst_conn,
    table_name: str,
    src_table_name: str = None,
    batch_size: int = 1000,
):
    """
    Copy data from source table to destination table.
    Handles column renames and extra columns.
    """
    if src_table_name is None:
        src_table_name = table_name

    # Get source columns
    src_columns = get_table_columns(src_conn, src_table_name)
    if not src_columns:
        log_warn(f"  No columns found for source table {src_table_name}")
        return 0

    # Get destination columns
    dst_columns = get_table_columns(dst_conn, table_name)
    if not dst_columns:
        log_warn(f"  No columns found for destination table {table_name}")
        return 0

    # Build column mapping
    renames = COLUMN_RENAMES.get(table_name, {})
    extras = EXTRA_COLUMNS.get(table_name, {})

    # Map source columns to destination
    select_parts = []
    insert_cols = []

    for src_col in src_columns:
        dst_col = renames.get(src_col, src_col)
        if dst_col in dst_columns:
            select_parts.append(f'"{src_col}"')
            insert_cols.append(f'"{dst_col}"')

    # Add extra columns
    for extra_col, extra_val in extras.items():
        quoted_extra = f'"{extra_col}"'
        if extra_col in dst_columns and quoted_extra not in insert_cols:
            # Check if extra_val references an existing source column
            quoted_ref = f'"{extra_val}"'
            if quoted_ref in select_parts:
                # Reference to another column — copy its value
                select_parts.append(quoted_ref)
            else:
                # Literal value (e.g., "'weekly'", "NULL")
                select_parts.append(extra_val)
            insert_cols.append(quoted_extra)

    if not select_parts:
        log_warn(f"  No matching columns for {table_name}")
        return 0

    select_sql = f'SELECT {", ".join(select_parts)} FROM public."{src_table_name}"'

    # Read all data from source
    src_cur = src_conn.cursor()
    src_cur.execute(select_sql)
    rows = src_cur.fetchall()
    total = len(rows)

    if total == 0:
        log(f"  {table_name}: 0 rows (empty)")
        return 0

    # Detect JSON/JSONB columns in destination
    dst_json_cols = get_json_columns(dst_conn, table_name)
    json_indices = []
    for idx, col in enumerate(insert_cols):
        clean_col = col.strip('"')
        if clean_col in dst_json_cols:
            json_indices.append(idx)

    # Wrap non-None values in Json for JSON/JSONB columns
    if json_indices:
        processed_rows = []
        for row in rows:
            row_list = list(row)
            for idx in json_indices:
                val = row_list[idx]
                if val is not None:
                    if isinstance(val, (dict, list)):
                        row_list[idx] = psycopg2.extras.Json(val)
                    elif isinstance(val, str):
                        try:
                            parsed = json.loads(val)
                            row_list[idx] = psycopg2.extras.Json(parsed)
                        except (json.JSONDecodeError, TypeError):
                            row_list[idx] = psycopg2.extras.Json(val)
                    else:
                        row_list[idx] = psycopg2.extras.Json(val)
            processed_rows.append(tuple(row_list))
        rows = processed_rows

    # Insert into destination
    placeholders = ", ".join(["%s"] * len(insert_cols))
    insert_sql = f'INSERT INTO public."{table_name}" ({", ".join(insert_cols)}) VALUES ({placeholders}) ON CONFLICT DO NOTHING'

    dst_cur = dst_conn.cursor()
    inserted = 0
    for i in range(0, total, batch_size):
        batch = rows[i : i + batch_size]
        try:
            psycopg2.extras.execute_batch(dst_cur, insert_sql, batch, page_size=batch_size)
            inserted += len(batch)
        except psycopg2.Error as e:
            dst_conn.rollback()
            log_err(f"  Error inserting into {table_name}: {e}")
            raise

    dst_conn.commit()
    log_ok(f"  {table_name}: {inserted}/{total} rows migrated")
    return inserted


def migrate_partitioned_table(
    src_conn,
    dst_conn,
    parent_table: str,
    prefix: str,
):
    """Merge all partition tables into a single parent table in destination."""
    src_tables = get_all_tables(src_conn)
    partitions = [t for t in src_tables if t.startswith(prefix)]

    if not partitions:
        log(f"  No partitions found for {prefix}*")
        return 0

    total_rows = 0
    for part in sorted(partitions):
        count = get_table_row_count(src_conn, part)
        if count > 0:
            rows = migrate_table_data(src_conn, dst_conn, parent_table, src_table_name=part)
            total_rows += rows

    return total_rows


# ─────────────────────────────────────────────────────────────────────
# Phase 0: Run System DB Migrations
# ─────────────────────────────────────────────────────────────────────


def run_db_migrations(db_url: str, migration_dir: str, label: str):
    """Run SQL migrations from a directory on a database."""
    log(f"  Running {label} migrations...")

    if not os.path.isdir(migration_dir):
        log_err(f"  Migration directory not found: {migration_dir}")
        return

    conn = psycopg2.connect(db_url)
    conn.autocommit = False
    cur = conn.cursor()

    # Create migration tracking table
    cur.execute("""
        CREATE TABLE IF NOT EXISTS __diesel_schema_migrations (
            version VARCHAR(50) NOT NULL PRIMARY KEY,
            run_on TIMESTAMP NOT NULL DEFAULT NOW()
        )
    """)
    conn.commit()

    migration_folders = sorted(os.listdir(migration_dir))
    applied = 0

    for folder in migration_folders:
        up_sql_path = os.path.join(migration_dir, folder, "up.sql")
        if not os.path.exists(up_sql_path):
            continue

        version = folder.split("_")[0]

        cur.execute("SELECT 1 FROM __diesel_schema_migrations WHERE version = %s", (version,))
        if cur.fetchone():
            continue

        log(f"    Applying: {folder}")
        with open(up_sql_path, "r") as f:
            sql = f.read()
        try:
            cur.execute(sql)
            cur.execute("INSERT INTO __diesel_schema_migrations (version) VALUES (%s)", (version,))
            conn.commit()
            applied += 1
        except psycopg2.Error as e:
            conn.rollback()
            log_err(f"    Migration failed: {folder}: {e}")
            raise

    conn.close()
    if applied > 0:
        log_ok(f"  {label}: {applied} migration(s) applied")
    else:
        log_ok(f"  {label}: already up to date")


def ensure_system_db_ready():
    """Ensure system DB has migrations applied and fn_set_updated_at exists."""
    log("═══════════════════════════════════════")
    log("Phase 0: System DB Schema Setup")
    log("═══════════════════════════════════════")

    # Ensure fn_set_updated_at exists (needed by workspace_roles trigger)
    conn = psycopg2.connect(SYSTEM_DB_URL)
    cur = conn.cursor()
    cur.execute("""
        CREATE OR REPLACE FUNCTION public.fn_set_updated_at() RETURNS trigger
            LANGUAGE plpgsql AS $$
        BEGIN
            NEW.updated_at = NOW();
            RETURN NEW;
        END;
        $$;
    """)
    conn.commit()
    conn.close()

    project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    system_migration_dir = os.path.join(project_root, "apiserver", "migrations", "system")

    run_db_migrations(SYSTEM_DB_URL, system_migration_dir, "System DB")
    log_ok("Phase 0 Complete")


# ─────────────────────────────────────────────────────────────────────
# Phase 1: System DB Migration
# ─────────────────────────────────────────────────────────────────────


def migrate_system_data(src_conn, sys_conn):
    """Migrate system-level tables."""
    log("═══════════════════════════════════════")
    log("Phase 1: System DB Migration")
    log("═══════════════════════════════════════")

    # Order matters due to FK constraints
    system_table_order = [
        "users",
        "workspaces",
        "workspace_members",
        "authorized_agents",
    ]

    for table in system_table_order:
        if not table_exists(src_conn, table):
            log_warn(f"  Source table {table} not found, skipping")
            continue
        if not table_exists(sys_conn, table):
            log_warn(f"  Destination table {table} not found, skipping")
            continue
        migrate_table_data(src_conn, sys_conn, table)

    # user_system_roles: migrate the data
    if table_exists(src_conn, "user_system_roles") and table_exists(sys_conn, "user_system_roles"):
        migrate_table_data(src_conn, sys_conn, "user_system_roles")

    log_ok("Phase 1 Complete")


# ─────────────────────────────────────────────────────────────────────
# Phase 2: Tenant DB Provisioning
# ─────────────────────────────────────────────────────────────────────


def provision_tenant_db(sys_conn, workspace_id: str) -> str:
    """Create tenant database, run migrations, and return its URL."""
    log("═══════════════════════════════════════")
    log("Phase 2: Tenant DB Provisioning")
    log("═══════════════════════════════════════")

    db_name = f"tenant_{workspace_id.replace('-', '')}"

    # Parse system URL to build tenant URL
    import urllib.parse

    parsed = urllib.parse.urlparse(SYSTEM_DB_URL)
    tenant_url = f"{parsed.scheme}://{parsed.username}:{parsed.password}@{parsed.hostname}:{parsed.port or 5432}/{db_name}"

    # Create database (needs autocommit)
    sys_auto = psycopg2.connect(SYSTEM_DB_URL)
    sys_auto.autocommit = True
    cur = sys_auto.cursor()

    try:
        cur.execute("SELECT 1 FROM pg_database WHERE datname = %s", (db_name,))
        if cur.fetchone():
            log_warn(f"  Database {db_name} already exists, skipping CREATE")
        else:
            cur.execute(f'CREATE DATABASE "{db_name}"')
            log_ok(f"  Created database: {db_name}")
    finally:
        sys_auto.close()

    # Ensure fn_set_updated_at exists in tenant DB
    tenant_conn = psycopg2.connect(tenant_url)
    cur = tenant_conn.cursor()
    cur.execute("""
        CREATE OR REPLACE FUNCTION public.fn_set_updated_at() RETURNS trigger
            LANGUAGE plpgsql AS $$
        BEGIN
            NEW.updated_at = NOW();
            RETURN NEW;
        END;
        $$;
    """)
    tenant_conn.commit()
    tenant_conn.close()

    # Run tenant migrations using shared helper
    project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    tenant_migration_dir = os.path.join(project_root, "apiserver", "migrations", "tenant")
    run_db_migrations(tenant_url, tenant_migration_dir, f"Tenant ({db_name})")

    log_ok("Phase 2 Complete")
    return tenant_url


def populate_employments_from_employees(tenant_conn, workspace_id):
    """Create default employer and employment records for each employee in the tenant DB."""
    log("  Populating default employers and employments for legacy employees...")
    cur = tenant_conn.cursor()
    
    # 1. Ensure we have an employer
    cur.execute("SELECT id FROM public.employers WHERE workspace_id = %s LIMIT 1", (workspace_id,))
    employer_row = cur.fetchone()
    if employer_row:
        employer_id = employer_row[0]
    else:
        employer_id = str(uuid.uuid4())
        cur.execute(
            "INSERT INTO public.employers (id, workspace_id, name) VALUES (%s, %s, %s)",
            (employer_id, workspace_id, "Default Employer")
        )
        tenant_conn.commit()
        log(f"    Created default employer: {employer_id}")
        
    # 2. Get all employees
    cur.execute("SELECT id, hire_date, created_at FROM public.employees")
    employees = cur.fetchall()
    
    # 3. Create employment for each employee, using employee_id as employment_id
    inserted = 0
    for emp_id, hire_date, created_at in employees:
        start_date = hire_date if hire_date else (created_at.date() if created_at else time.strftime('%Y-%m-%d'))
        cur.execute(
            """
            INSERT INTO public.employments (
                id, workspace_id, employee_id, employer_id, status, start_date, created_at, updated_at
            ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
            ON CONFLICT DO NOTHING
            """,
            (emp_id, workspace_id, emp_id, employer_id, "active", start_date, created_at, created_at)
        )
        inserted += cur.rowcount
        
    tenant_conn.commit()
    log_ok(f"    Created {inserted} default employment record(s)")


# ─────────────────────────────────────────────────────────────────────
# Phase 3: Tenant DB Data Migration
# ─────────────────────────────────────────────────────────────────────


def migrate_tenant_data(src_conn, tenant_url: str, workspace_id: str):
    """Migrate tenant-level tables."""
    log("═══════════════════════════════════════")
    log("Phase 3: Tenant DB Data Migration")
    log("═══════════════════════════════════════")

    tenant_conn = psycopg2.connect(tenant_url)

    # Ordered by FK dependencies (parents first)
    tenant_table_order = [
        # Core config
        "feature_flags",
        "holidays",
        "policy_templates",
        # Shifts
        "shifts",
        # Working patterns
        "working_patterns",
        "working_pattern_details",
        # Departments (depends on shifts, working_patterns)
        "departments",
        "department_default_shifts",
        "department_payroll_configs",
        # Recruitment
        "recruitment_sources",
        # Employees (depends on departments, recruitment_sources)
        "employees",
        "employee_profiles",
        "employee_department_history",
        "employee_emergency_contacts",
        "employee_documents",
        "employee_contracts",
        "employee_shifts",
        "employee_working_patterns",
        "employee_payroll_configs",
        # Employment phases
        "employment_phase_definitions",
        "employee_phases",
        # Leave
        "leave_types",
        "leave_policy_definitions",
        "leave_policy_assignments",
        "leave_balances",
        "leave_requests",
        "leave_accrual_logs",
        # Attendance
        "attendance_policies",
        "attendance_policy_assignments",
        "attendance_policy_versions",
        "raw_attendance_logs",
        "time_entries",
        "attendance_daily_summaries",
        "attendance_adjustment_requests",
        "attendance_import_jobs",
        "attendance_log_errors",
        # Overtime
        "overtime_policy_definitions",
        "overtime_rules",
        "overtime_policy_assignments",
        # Payroll
        "payroll_settings",
        "payroll_rule_definitions",
        "payroll_rule_tiers",
        "payroll_rule_assignments",
        "payroll_incentive_policies",
        "payroll_incentive_tiers",
        "payroll_incentive_assignments",
        # Salary
        "salary_base_groups",
        "salary_component_definitions",
        "salary_component_base_members",
        "employee_salary_components",
        # Payslips
        "payslips",
        "payslip_segments",
        "payslip_incentives",
        "payslip_rule_evaluations",
        # Other
        "devices",
        "shift_rosters",
        "shift_swap_requests",
        "assignment_rules",
        "dashboard_preferences",
        "working_conditions",
    ]

    # Disable triggers during migration for performance
    cur = tenant_conn.cursor()
    cur.execute("SET session_replication_role = 'replica'")
    tenant_conn.commit()

    for table in tenant_table_order:
        if not table_exists(src_conn, table):
            log_warn(f"  Source table {table} not found, skipping")
            continue
        if not table_exists(tenant_conn, table):
            log_warn(f"  Destination table {table} not found, skipping")
            continue
        try:
            migrate_table_data(src_conn, tenant_conn, table)
            if table == "employees":
                populate_employments_from_employees(tenant_conn, workspace_id)
        except Exception as e:
            log_err(f"  Failed to migrate {table}: {e}")
            # Continue with other tables

    # Handle partitioned tables: attendance_staging_logs
    log("  Merging partitioned attendance_staging_logs...")
    if table_exists(tenant_conn, "attendance_staging_logs"):
        migrate_partitioned_table(
            src_conn, tenant_conn, "attendance_staging_logs", "attendance_staging_logs_"
        )

    # Re-enable triggers
    cur = tenant_conn.cursor()
    cur.execute("SET session_replication_role = 'origin'")
    tenant_conn.commit()

    tenant_conn.close()
    log_ok("Phase 3 Complete")


# ─────────────────────────────────────────────────────────────────────
# Phase 4: Verification
# ─────────────────────────────────────────────────────────────────────


def verify_migration(src_conn, sys_conn, tenant_url: str):
    """Verify migration by comparing row counts."""
    log("═══════════════════════════════════════")
    log("Phase 4: Verification")
    log("═══════════════════════════════════════")

    tenant_conn = psycopg2.connect(tenant_url)

    # Key tables to verify
    verify_tables = {
        "system": [
            ("users", sys_conn),
            ("workspaces", sys_conn),
            ("workspace_members", sys_conn),
        ],
        "tenant": [
            ("departments", tenant_conn),
            ("employees", tenant_conn),
            ("shifts", tenant_conn),
            ("time_entries", tenant_conn),
            ("raw_attendance_logs", tenant_conn),
            ("leave_types", tenant_conn),
            ("payroll_settings", tenant_conn),
        ],
    }

    all_ok = True
    for db_type, tables in verify_tables.items():
        log(f"  [{db_type.upper()}]")
        for table, dst in tables:
            src_count = get_table_row_count(src_conn, table)
            dst_count = get_table_row_count(dst, table)
            status = "✅" if dst_count >= src_count else "⚠️"
            if dst_count < src_count:
                all_ok = False
            log(f"    {status} {table}: source={src_count}, target={dst_count}")

    tenant_conn.close()

    if all_ok:
        log_ok("Verification PASSED — all row counts match or exceed source")
    else:
        log_warn("Verification completed with warnings — some tables have fewer rows")

    return all_ok


# ─────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────


def main():
    log("╔══════════════════════════════════════════════════╗")
    log("║  LeeAttend Legacy Data Migration                ║")
    log("║  Single-DB → Multi-Tenant Architecture          ║")
    log("╚══════════════════════════════════════════════════╝")
    log("")
    log(f"  Legacy DB: {LEGACY_DB_URL.split('@')[1] if '@' in LEGACY_DB_URL else LEGACY_DB_URL}")
    log(f"  System DB: {SYSTEM_DB_URL.split('@')[1] if '@' in SYSTEM_DB_URL else SYSTEM_DB_URL}")
    log("")

    # Connect to legacy DB
    log("Connecting to legacy database...")
    try:
        src_conn = psycopg2.connect(LEGACY_DB_URL)
        log_ok("Connected to legacy DB")
    except psycopg2.Error as e:
        log_err(f"Cannot connect to legacy DB: {e}")
        sys.exit(1)

    # Connect to system DB
    log("Connecting to system database...")
    try:
        sys_conn = psycopg2.connect(SYSTEM_DB_URL)
        log_ok("Connected to system DB")
    except psycopg2.Error as e:
        log_err(f"Cannot connect to system DB: {e}")
        sys.exit(1)

    # Get workspace IDs from source
    cur = src_conn.cursor()
    cur.execute("SELECT id FROM public.workspaces")
    workspaces = [row[0] for row in cur.fetchall()]
    log(f"Found {len(workspaces)} workspace(s) in legacy DB")

    if not workspaces:
        log_err("No workspaces found in legacy DB!")
        sys.exit(1)

    # Phase 0: Ensure system DB schema is ready
    ensure_system_db_ready()

    # Phase 1: System DB data
    migrate_system_data(src_conn, sys_conn)

    # Phase 2 + 3: For each workspace
    for ws_id in workspaces:
        ws_id_str = str(ws_id)
        log(f"\n  Processing workspace: {ws_id_str}")

        # Phase 2: Provision tenant DB
        tenant_url = provision_tenant_db(sys_conn, ws_id_str)

        # Phase 3: Migrate tenant data
        migrate_tenant_data(src_conn, tenant_url, ws_id_str)

    # Phase 4: Verify
    # Use first workspace for verification
    first_ws = str(workspaces[0])
    db_name = f"tenant_{first_ws.replace('-', '')}"
    import urllib.parse

    parsed = urllib.parse.urlparse(SYSTEM_DB_URL)
    tenant_url = f"{parsed.scheme}://{parsed.username}:{parsed.password}@{parsed.hostname}:{parsed.port or 5432}/{db_name}"
    verify_migration(src_conn, sys_conn, tenant_url)

    src_conn.close()
    sys_conn.close()

    log("")
    log_ok("Migration complete! 🎉")
    log("Next steps:")
    log("  1. Start apiserver: it will run any remaining tenant migrations")
    log("  2. Login with existing user credentials")
    log("  3. Verify data in the web dashboard")


if __name__ == "__main__":
    main()

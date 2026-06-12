# ruff: noqa
# mypy: ignore-errors
#!/usr/bin/env python3
"""
Tenant Workspace Migration Script
=================================
Migrates tenant data from the legacy railway DB to the new multi-tenant db_workspace,
replacing the old workspace ID and old user ID with the current ones.
"""

import os
import sys
import time
import json
import uuid
import psycopg2
import psycopg2.extras
from contextlib import contextmanager

# ─────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────

# Local dev DB urls
LEGACY_DB_URL = os.getenv("LEGACY_DB_URL", "postgres://tenant_user:tenant_password@localhost:5435/legacy_db")
WORKSPACE_DB_URL = os.getenv(
    "WORKSPACE_DB_URL", "postgres://tenant_user:tenant_password@localhost:5435/tenant_db"
)

# Workspace mapping
OLD_WORKSPACE_UUID = uuid.UUID("6e913c39-24f3-45b5-877d-e307c3e10c88")
NEW_WORKSPACE_UUID = uuid.UUID("019e632f-26b4-7660-956e-62b13870a145")

# User mapping
OLD_USER_UUID = uuid.UUID("019e07e0-9f95-7203-8191-ed3995e78495")
NEW_USER_UUID = uuid.UUID("019e632f-0444-7f21-b48a-c2a955950c8c")

# Tables to skip (managed by system DB or schema migrations)
SKIP_TABLES = {
    "_sqlx_migrations",
    "__diesel_schema_migrations",
    "sessions",
    "users",
    "workspaces",
    "workspace_members",
    "authorized_agents",
    "workspace_roles",
    "workspace_role_permissions",
    "user_system_roles",
    "system_roles",
}

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
def connect(url: str):
    conn = psycopg2.connect(url)
    psycopg2.extras.register_uuid()
    try:
        yield conn
    finally:
        conn.close()


def get_child_tables(conn) -> set[str]:
    """Get all child/partition tables that inherit from parent tables."""
    cur = conn.cursor()
    cur.execute("""
        SELECT inhrelid::regclass::text 
        FROM pg_inherits;
    """)
    return {row[0].strip('"') for row in cur.fetchall()}


def get_all_tables(conn) -> list[str]:
    """Get all base tables in the public schema."""
    cur = conn.cursor()
    cur.execute("""
        SELECT table_name 
        FROM information_schema.tables
        WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
        ORDER BY table_name;
    """)
    return [row[0] for row in cur.fetchall()]


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


def get_row_count(conn, table_name: str) -> int:
    """Get total row count of a table."""
    cur = conn.cursor()
    try:
        cur.execute(f'SELECT COUNT(*) FROM public."{table_name}"')
        return cur.fetchone()[0]
    except psycopg2.Error:
        conn.rollback()
        return 0


def replace_uuids_in_json(data):
    """Recursively replace old workspace/user UUIDs in dicts and lists."""
    if isinstance(data, dict):
        return {k: replace_uuids_in_json(v) for k, v in data.items()}
    if isinstance(data, list):
        return [replace_uuids_in_json(x) for x in data]
    if isinstance(data, str):
        try:
            val_uuid = uuid.UUID(data)
            if val_uuid == OLD_WORKSPACE_UUID:
                return str(NEW_WORKSPACE_UUID)
            if val_uuid == OLD_USER_UUID:
                return str(NEW_USER_UUID)
        except ValueError:
            pass
    return data


def process_value(val):
    """Replace old workspace/user UUIDs with current ones."""
    if val is None:
        return None
    if isinstance(val, uuid.UUID):
        if val == OLD_WORKSPACE_UUID:
            return NEW_WORKSPACE_UUID
        if val == OLD_USER_UUID:
            return NEW_USER_UUID
    elif isinstance(val, str):
        try:
            val_uuid = uuid.UUID(val)
            if val_uuid == OLD_WORKSPACE_UUID:
                return NEW_WORKSPACE_UUID
            if val_uuid == OLD_USER_UUID:
                return NEW_USER_UUID
        except ValueError:
            pass
    return val


# ─────────────────────────────────────────────────────────────────────
# Main Migration Logic
# ─────────────────────────────────────────────────────────────────────


def main():
    log("╔══════════════════════════════════════════════════╗")
    log("║  LeeAttend Tenant Workspace Data Migration       ║")
    log("║  Migrating data to db_workspace                  ║")
    log("╚══════════════════════════════════════════════════╝")
    log(f"  Legacy DB:  {LEGACY_DB_URL.split('@')[-1]}")
    log(f"  Target DB:  {WORKSPACE_DB_URL.split('@')[-1]}")
    log(f"  Workspace ID Mapping: {OLD_WORKSPACE_UUID} ➔ {NEW_WORKSPACE_UUID}")
    log(f"  User ID Mapping:      {OLD_USER_UUID} ➔ {NEW_USER_UUID}")
    log("")

    # Connect to databases
    try:
        src_conn = psycopg2.connect(LEGACY_DB_URL)
        psycopg2.extras.register_uuid(src_conn)
        log_ok("Connected to legacy DB")
    except psycopg2.Error as e:
        log_err(f"Cannot connect to legacy DB: {e}")
        sys.exit(1)

    try:
        dst_conn = psycopg2.connect(WORKSPACE_DB_URL)
        psycopg2.extras.register_uuid(dst_conn)
        log_ok("Connected to target workspace DB")
    except psycopg2.Error as e:
        log_err(f"Cannot connect to target workspace DB: {e}")
        src_conn.close()
        sys.exit(1)

    # Disable constraints & triggers for the target DB connection
    dst_cur = dst_conn.cursor()
    try:
        dst_cur.execute("SET session_replication_role = 'replica';")
        dst_conn.commit()
        log_ok("Disabled constraints & triggers in target DB (replica mode)")
    except psycopg2.Error as e:
        dst_conn.rollback()
        log_err(f"Failed to set session_replication_role: {e}")
        src_conn.close()
        dst_conn.close()
        sys.exit(1)

    # Find child/partition tables to skip
    child_tables = get_child_tables(src_conn)
    log(f"Detected {len(child_tables)} partition/child tables to skip")

    # Get all tables
    all_tables = get_all_tables(src_conn)
    tables_to_migrate = []
    for t in all_tables:
        if t in SKIP_TABLES:
            continue
        if t in child_tables:
            continue
        tables_to_migrate.append(t)

    log(f"Found {len(tables_to_migrate)} tables to migrate")

    # Truncate all tables first to avoid cascade deletions after insertions
    log("Truncating all target tables...")
    for table in tables_to_migrate:
        cur = dst_conn.cursor()
        cur.execute(
            """
            SELECT EXISTS (
                SELECT 1 FROM information_schema.tables 
                WHERE table_schema = 'public' AND table_name = %s
            );
        """,
            (table,),
        )
        if cur.fetchone()[0]:
            dst_cur = dst_conn.cursor()
            dst_cur.execute(f'TRUNCATE TABLE public."{table}" CASCADE;')
            dst_conn.commit()
    log_ok("All target tables truncated successfully")

    # Perform migration table by table
    batch_size = 2000
    migrated_stats = {}

    for table in tables_to_migrate:
        # Check if table exists in target DB
        cur = dst_conn.cursor()
        cur.execute(
            """
            SELECT EXISTS (
                SELECT 1 FROM information_schema.tables 
                WHERE table_schema = 'public' AND table_name = %s
            );
        """,
            (table,),
        )
        exists = cur.fetchone()[0]
        if not exists:
            log_warn(f"Table '{table}' does not exist in target DB, skipping")
            continue

        # Get columns and JSON columns
        cols = get_table_columns(src_conn, table)
        json_cols = get_json_columns(dst_conn, table)
        json_indices = [i for i, col in enumerate(cols) if col in json_cols]

        # Fetch data from source
        src_cur = src_conn.cursor()
        src_cur.execute(f'SELECT {", ".join(f'"{c}"' for c in cols)} FROM public."{table}";')
        rows = src_cur.fetchall()
        total_rows = len(rows)

        if total_rows == 0:
            log(f"  - {table}: 0 rows (empty)")
            migrated_stats[table] = (0, 0)

            continue

        # Process and insert rows in batches
        processed_rows = []
        for row in rows:
            row_list = list(row)
            # 1. Map UUIDs
            for idx in range(len(row_list)):
                row_list[idx] = process_value(row_list[idx])

            # 2. Map UUIDs inside JSON/JSONB columns
            for idx in json_indices:
                val = row_list[idx]
                if val is not None:
                    if isinstance(val, (dict, list)):
                        val = replace_uuids_in_json(val)
                        row_list[idx] = psycopg2.extras.Json(val)
                    elif isinstance(val, str):
                        try:
                            parsed = json.loads(val)
                            parsed = replace_uuids_in_json(parsed)
                            row_list[idx] = psycopg2.extras.Json(parsed)
                        except (json.JSONDecodeError, TypeError):
                            row_list[idx] = val
                    else:
                        row_list[idx] = val

            processed_rows.append(tuple(row_list))

        # Handle duplicate department names to avoid unique constraint violations
        if table == "departments":
            name_idx = cols.index("name")
            seen_names = set()
            renamed_rows = []
            for r in processed_rows:
                r_list = list(r)
                dept_name = r_list[name_idx]
                if dept_name:
                    dept_name_lower = dept_name.lower()
                    if dept_name_lower in seen_names:
                        new_name = f"{dept_name} (2)"
                        r_list[name_idx] = new_name
                        log_warn(
                            f"Renamed duplicate department '{dept_name}' to '{new_name}' to satisfy unique constraint"
                        )
                    else:
                        seen_names.add(dept_name_lower)
                renamed_rows.append(tuple(r_list))
            processed_rows = renamed_rows

        # Insert into target DB
        placeholders = ", ".join(["%s"] * len(cols))
        insert_sql = f'INSERT INTO public."{table}" ({", ".join(f'"{c}"' for c in cols)}) VALUES ({placeholders}) ON CONFLICT DO NOTHING;'

        inserted_count = 0
        dst_cur = dst_conn.cursor()
        for i in range(0, total_rows, batch_size):
            batch = processed_rows[i : i + batch_size]
            try:
                psycopg2.extras.execute_batch(dst_cur, insert_sql, batch, page_size=batch_size)
                dst_conn.commit()
                inserted_count += len(batch)
            except psycopg2.Error as e:
                dst_conn.rollback()
                log_err(f"Error inserting into {table}: {e}")
                raise

        log(f"  - {table}: migrated {inserted_count}/{total_rows} rows")
        migrated_stats[table] = (total_rows, inserted_count)

    # Re-enable triggers and constraints
    dst_cur = dst_conn.cursor()
    dst_cur.execute("SET session_replication_role = 'origin';")
    dst_conn.commit()
    log_ok("Re-enabled triggers and constraints in target DB")

    # Verification phase
    log("\n═══════════════════════════════════════")
    log("Verification Phase")
    log("═══════════════════════════════════════")
    all_passed = True
    for table, (src_cnt, dst_cnt) in migrated_stats.items():
        # Get actual target count (including partitions if parent table)
        actual_dst_cnt = get_row_count(dst_conn, table)
        if actual_dst_cnt != src_cnt:
            log_warn(
                f"  Mismatch in '{table}': source={src_cnt}, target_actual={actual_dst_cnt} (inserted={dst_cnt})"
            )
            all_passed = False
        else:
            log_ok(f"  '{table}': {actual_dst_cnt} rows match")

    src_conn.close()
    dst_conn.close()

    if all_passed:
        log_ok("All tables migrated and verified successfully! 🎉")
    else:
        log_warn("Migration completed, but some row counts did not match target.")


if __name__ == "__main__":
    main()

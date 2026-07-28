#!/usr/bin/env python3
"""
ADR-005 Point-in-Time Rollback Script
======================================
Reverts daily_attendance from cascade-primary back to legacy output.
Used within the 30-day dual-run window if cascade engine has a serious bug.

Per ADR-005 §4:
1. For each attendance_daily_summaries row with effective_rules_snapshot
   written after the cutover date, restore legacy_rules_snapshot values.
2. Export attendance_exceptions and employee_policy_modifiers created during
   the dual-run window to CSV (for HR to re-apply manually after rollback).
3. Disable cascade_as_primary feature flag (reverting to legacy-primary mode).

Usage:
    python3 scripts/adr005_rollback.py --cutover-date 2026-07-29 [--dry-run]
    python3 scripts/adr005_rollback.py --cutover-date 2026-07-29 --workspace-id <uuid>

Environment:
    WORKSPACE_DB_URL  — PostgreSQL connection string for the workspace/tenant DB.
"""

import argparse
import csv
import json
import os
import sys
from datetime import date, datetime
from typing import Optional

import psycopg2
import psycopg2.extras

# ── Configuration ───────────────────────────────────────────────────────

DB_URL = os.getenv(
    "WORKSPACE_DB_URL",
    "postgres://admin:change_this_password_immediately@localhost:5432/leeattend",
)

BATCH_SIZE = 100  # rows per transaction (ADR-005 §6: checkpoint-friendly)


# ── DB Helpers ───────────────────────────────────────────────────────────


def get_connection():
    try:
        return psycopg2.connect(DB_URL)
    except psycopg2.OperationalError as e:
        print(f"ERROR: Cannot connect to database: {e}", file=sys.stderr)
        sys.exit(1)


# ── Queries ──────────────────────────────────────────────────────────────


def fetch_rollback_candidates(conn, cutover_date: str, workspace_id: Optional[str]) -> list[dict]:
    """Find rows with cascade snapshot written after cutover that have legacy data."""
    query = """
    SELECT
        id,
        workspace_id,
        employment_id,
        date,
        actual_work_minutes,
        system_detected_ot_minutes,
        effective_rules_snapshot,
        legacy_rules_snapshot,
        policy_version_id
    FROM public.attendance_daily_summaries
    WHERE effective_rules_snapshot IS NOT NULL
      AND legacy_rules_snapshot IS NOT NULL
      AND is_superseded = false
      AND updated_at >= %s::timestamptz
    """
    params = [cutover_date]
    if workspace_id:
        query += " AND workspace_id = %s"
        params.append(workspace_id)
    query += " ORDER BY date, employment_id"

    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute(query, params)
        return cur.fetchall()


def fetch_exceptions_since(conn, cutover_date: str, workspace_id: Optional[str]) -> list[dict]:
    """Fetch attendance_exceptions created during the dual-run window."""
    query = """
    SELECT id, workspace_id, employee_id, policy_version_id,
           exception_from, exception_to, contributions, reason,
           approved_by, created_by, created_at, updated_at
    FROM public.attendance_exceptions
    WHERE created_at >= %s::timestamptz
    """
    params = [cutover_date]
    if workspace_id:
        query += " AND workspace_id = %s"
        params.append(workspace_id)
    query += " ORDER BY created_at"

    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute(query, params)
        return cur.fetchall()


def fetch_modifiers_since(conn, cutover_date: str, workspace_id: Optional[str]) -> list[dict]:
    """Fetch employee_policy_modifiers created during the dual-run window."""
    query = """
    SELECT id, workspace_id, employee_id, policy_version_id,
           priority_rank, effective_from, effective_to,
           contributions, reason, created_by, created_at, updated_at
    FROM public.employee_policy_modifiers
    WHERE created_at >= %s::timestamptz
    """
    params = [cutover_date]
    if workspace_id:
        query += " AND workspace_id = %s"
        params.append(workspace_id)
    query += " ORDER BY created_at"

    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute(query, params)
        return cur.fetchall()


def disable_cascade_flag(conn, dry_run: bool) -> str:
    """Disable cascade_as_primary feature flag."""
    query = """
    UPDATE public.feature_flags
    SET enabled = false,
        description = description || ' [ROLLBACK: ADR-005 point-in-time rollback at '
            || to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS TZ') || ']',
        updated_at = NOW()
    WHERE flag_key = 'cascade_as_primary'
      AND enabled = true
    RETURNING workspace_id, description
    """
    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        if dry_run:
            # Peek what would be affected
            cur.execute(
                "SELECT workspace_id, description FROM public.feature_flags WHERE flag_key = 'cascade_as_primary' AND enabled = true"
            )
            return f"DRY-RUN: would disable cascade_as_primary for {cur.rowcount} workspace(s)"
        cur.execute(query)
        rows = cur.fetchall()
        conn.commit()
        workspace_ids = [str(r["workspace_id"]) for r in rows]
        return f"Disabled cascade_as_primary for workspaces: {workspace_ids}"


# ── Rollback Logic ───────────────────────────────────────────────────────


def restore_legacy_rules(conn, row: dict, dry_run: bool) -> dict:
    """
    Restore legacy_rules_snapshot values into the cascade-computed fields.
    Since the cascade rules are already in effective_rules_snapshot->final_rules,
    and the effective_work/OT minutes were computed from those rules,
    a full rollback would require re-computation.  For a point-in-time safety
    measure, we store the delta metadata so HR can manually re-process affected rows.
    """
    legacy = row["legacy_rules_snapshot"]
    cascade = row["effective_rules_snapshot"]

    if not isinstance(legacy, dict) or not isinstance(cascade, dict):
        return {"status": "skip", "reason": "invalid snapshot JSON"}

    # Extract the cascade final_rules (what was actually applied)
    cascade_rules = cascade.get("final_rules", {})
    legacy_grace = legacy.get("grace_period_minutes", 0)
    legacy_ael = legacy.get("authorized_early_leave_minutes", 0)
    cascade_grace = cascade_rules.get("grace_period_minutes", 0)
    cascade_ael = cascade_rules.get("authorized_early_leave_minutes", 0)

    has_diff = (
        legacy_grace != cascade_grace
        or legacy_ael != cascade_ael
    )

    if not has_diff:
        return {"status": "skip", "reason": "no diff — legacy and cascade rules match"}

    if not dry_run:
        # Update the row: set effective_rules_snapshot.final_rules back to legacy values.
        # This effectively reverts the cascade engine output to legacy.
        # The cascade_trace and clamp_warnings are preserved for audit.
        legacy_final = {
            "grace_period_minutes": legacy_grace,
            "authorized_early_leave_minutes": legacy_ael,
            "is_early_leave_paid": legacy.get("is_early_leave_paid", cascade_rules.get("is_early_leave_paid", False)),
            "early_arrival_strategy": legacy.get("early_arrival_strategy", cascade_rules.get("early_arrival_strategy", "Allow")),
            "is_break_auto_deduct": legacy.get("is_break_auto_deduct", cascade_rules.get("is_break_auto_deduct", False)),
            "is_break_overtime": legacy.get("is_break_overtime", cascade_rules.get("is_break_overtime", False)),
            "allow_early_ot": legacy.get("allow_early_ot", cascade_rules.get("allow_early_ot", False)),
        }
        updated_snapshot = dict(cascade)
        updated_snapshot["final_rules"] = legacy_final
        updated_snapshot["_rollback"] = {
            "rolled_back_at": datetime.utcnow().isoformat(),
            "reason": "ADR-005 point-in-time rollback — cascade engine reverted to legacy output",
            "pre_rollback_final_rules": cascade_rules,
        }

        update_query = """
        UPDATE public.attendance_daily_summaries
        SET effective_rules_snapshot = %s::jsonb,
            updated_at = NOW()
        WHERE id = %s
        """
        with conn.cursor() as cur:
            cur.execute(update_query, (json.dumps(updated_snapshot), row["id"]))

    return {
        "status": "dry_run" if dry_run else "restored",
        "cascade_grace": cascade_grace,
        "legacy_grace": legacy_grace,
        "cascade_ael": cascade_ael,
        "legacy_ael": legacy_ael,
    }


def export_csv(rows: list[dict], filename: str, columns: list[str]) -> str:
    """Export rows to CSV for HR manual re-apply."""
    if not rows:
        return f"No data to export to {filename}"

    with open(filename, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=columns, extrasaction="ignore")
        writer.writeheader()
        for row in rows:
            # Serialize JSONB fields
            row_copy = dict(row)
            for key in row_copy:
                if isinstance(row_copy[key], (dict, list)):
                    row_copy[key] = json.dumps(row_copy[key])
                elif hasattr(row_copy[key], "isoformat"):
                    row_copy[key] = row_copy[key].isoformat()
            writer.writerow(row_copy)

    return f"Exported {len(rows)} rows to {filename}"


# ── Main ─────────────────────────────────────────────────────────────────


def main() -> None:
    parser = argparse.ArgumentParser(description="ADR-005 Point-in-Time Rollback")
    parser.add_argument("--cutover-date", required=True, help="Cutover date (YYYY-MM-DD)")
    parser.add_argument("--workspace-id", help="Limit to a specific workspace")
    parser.add_argument("--dry-run", action="store_true", help="Preview changes without applying")
    parser.add_argument("--export-dir", default="./logs", help="Directory for CSV exports (default: ./logs)")
    args = parser.parse_args()

    os.makedirs(args.export_dir, exist_ok=True)
    conn = get_connection()

    print("=" * 72)
    print("  ADR-005 Point-in-Time Rollback")
    print(f"  Cutover Date: {args.cutover_date}")
    print(f"  Dry Run:      {args.dry_run}")
    print("=" * 72)
    print()

    try:
        # 1. Fetch rollback candidates
        candidates = fetch_rollback_candidates(conn, args.cutover_date, args.workspace_id)
        print(f"  Rollback candidates: {len(candidates)} rows with cascade snapshots")
        print()

        # 2. Executorestore (or preview)
        restored_count = 0
        skipped_count = 0
        failed_items: list[dict] = []

        for i, row in enumerate(candidates):
            if i % BATCH_SIZE == 0 and i > 0 and not args.dry_run:
                conn.commit()
                print(f"  ... committed batch at row {i}/{len(candidates)}")

            result = restore_legacy_rules(conn, row, args.dry_run)
            if result["status"] in ("restored", "dry_run"):
                restored_count += 1
            else:
                skipped_count += 1
                failed_items.append({
                    "id": str(row["id"]),
                    "date": str(row["date"]),
                    "reason": result.get("reason", "unknown"),
                })

        if not args.dry_run:
            conn.commit()

        print(f"  Rows to restore:   {restored_count:>6}")
        print(f"  Rows skipped:      {skipped_count:>6}")
        print()

        # 3. Export exceptions created during dual-run window
        exceptions = fetch_exceptions_since(conn, args.cutover_date, args.workspace_id)
        exc_file = os.path.join(args.export_dir, "rollback_attendance_exceptions.csv")
        result = export_csv(exceptions, exc_file, [
            "id", "workspace_id", "employee_id", "policy_version_id",
            "exception_from", "exception_to", "contributions", "reason",
            "approved_by", "created_by", "created_at",
        ])
        print(f"  {result}")

        # 4. Export modifiers created during dual-run window
        modifiers = fetch_modifiers_since(conn, args.cutover_date, args.workspace_id)
        mod_file = os.path.join(args.export_dir, "rollback_employee_policy_modifiers.csv")
        result = export_csv(modifiers, mod_file, [
            "id", "workspace_id", "employee_id", "policy_version_id",
            "priority_rank", "effective_from", "effective_to",
            "contributions", "reason", "created_by", "created_at",
        ])
        print(f"  {result}")
        print()

        # 5. Disable cascade_as_primary flag
        flag_result = disable_cascade_flag(conn, args.dry_run)
        print(f"  Feature Flag: {flag_result}")
        print()

        # 6. Summary
        print("=" * 72)
        if args.dry_run:
            print("  DRY RUN COMPLETE — No changes applied.")
            print("  Remove --dry-run to execute the rollback.")
        else:
            print("  ROLLBACK COMPLETE")
            print(f"  {restored_count} rows restored from legacy_rules_snapshot")
            print(f"  {len(exceptions)} exceptions exported to {exc_file}")
            print(f"  {len(modifiers)} modifiers exported to {mod_file}")
            print(f"  cascade_as_primary flag disabled")
        print()
        print("  Next steps for HR:")
        print("  1. Review the exported CSV files")
        print("  2. Re-apply exceptions and modifiers as needed")
        print("  3. Notify engineering to investigate the root cause")
        print("=" * 72)

        if failed_items:
            print()
            print(f"  WARNING: {len(failed_items)} rows could not be restored:")
            for item in failed_items[:10]:
                print(f"    [{item['date']}] id={item['id']}: {item['reason']}")

    except Exception as e:
        conn.rollback()
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
    finally:
        conn.close()


if __name__ == "__main__":
    main()

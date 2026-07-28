#!/usr/bin/env python3
"""
ADR-005 §4: Legacy Manual Override Migration
=============================================
Classifies HR-edited attendance rows (is_manual_override=true) into three
categories and migrates them to the cascade policy system.

Categories (ADR-005 §4):
    Cat 1 — Late approval / single-day absence   → attendance_exceptions
    Cat 2 — Long-term adjustment (maternity,     → employee_policy_modifiers
            probation, special zone)
    Cat 3 — Unclassifiable                       → requires_hr_review=true

Usage:
    # Dry-run — preview classification without applying
    python3 scripts/adr005_migrate_manual_overrides.py --dry-run

    # With CSV import of legacy manual override IDs
    python3 scripts/adr005_migrate_manual_overrides.py --input legacy_overrides.csv

    # Only process Cat 3 review queue (for HR re-review after initial pass)
    python3 scripts/adr005_migrate_manual_overrides.py --review-only

Environment:
    WORKSPACE_DB_URL  — PostgreSQL connection string.
"""

import argparse
import csv
import os
import re
import sys
import uuid
from datetime import date, datetime, timedelta
from typing import Optional

import psycopg2
import psycopg2.extras

# ── Configuration ───────────────────────────────────────────────────────

DB_URL = os.getenv(
    "WORKSPACE_DB_URL",
    "postgres://admin:change_this_password_immediately@localhost:5432/leeattend",
)

EXPORT_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "logs")

# Vietnamese / English patterns for known adjustment types
CAT1_PATTERNS = [
    r"duyệt\s*muộn",              # "approved late"
    r"nghỉ\s*1\s*ngày",           # "absent 1 day"
    r"nghỉ\s*một\s*ngày",         # "absent one day"
    r"đi\s*muộn",                  # "arrived late"
    r"về\s*sớm",                  # "left early"
    r"approved\s*late",
    r"absent\s*1\s*day",
    r"late\s*approval",
    r"single\s*day",
    r"nghỉ\s*phép\s*đột\s*xuất", # "sudden leave"
    r"xin\s*nghỉ\s*đột\s*xuất",  # "request sudden leave"
]

CAT2_PATTERNS = [
    r"thai\s*sản",                 # "maternity"
    r"thử\s*việc",                 # "probation"
    r"khu\s*vực\s*đặc\s*biệt",    # "special zone"
    r"maternity",
    r"probation",
    r"dài\s*hạn",                  # "long term"
    r"điều\s*chỉnh\s*dài",        # "long term adjustment"
    r"long\s*term",
    r"hợp\s*đồng\s*mới",          # "new contract"
    r"chuyển\s*vùng",             # "zone transfer"
    r"chế\s*độ",                  # "policy/regime"
    r"ốm\s*dài",                  # "long illness"
    r"nghỉ\s*không\s*lương\s*dài",  # "long unpaid leave"
]


# ── DB Helpers ───────────────────────────────────────────────────────────


def get_connection():
    try:
        return psycopg2.connect(DB_URL)
    except psycopg2.OperationalError as e:
        print(f"ERROR: Cannot connect to database: {e}", file=sys.stderr)
        sys.exit(1)


def flag_manual_override_rows(conn, override_ids: set[str], dry_run: bool) -> int:
    """Mark rows as is_manual_override from a list of IDs."""
    if not override_ids:
        return 0
    query = """
    UPDATE public.attendance_daily_summaries
    SET is_manual_override = true,
        updated_at = NOW()
    WHERE id::text = ANY(%s)
      AND is_manual_override = false
    """
    if dry_run:
        with conn.cursor() as cur:
            cur.execute(
                "SELECT COUNT(*) FROM public.attendance_daily_summaries WHERE id::text = ANY(%s) AND is_manual_override = false",
                (list(override_ids),),
            )
            return cur.fetchone()[0]
    with conn.cursor() as cur:
        cur.execute(query, (list(override_ids),))
        return cur.rowcount


def fetch_manual_overrides(conn, workspace_id: Optional[str]) -> list[dict]:
    """Fetch all rows flagged as manual override that haven't been classified yet."""
    query = """
    SELECT id, workspace_id, employment_id, date,
           actual_work_minutes, system_detected_ot_minutes,
           ot_reason, manager_note, manual_override_notes,
           is_manual_override, manual_override_category, requires_hr_review
    FROM public.attendance_daily_summaries
    WHERE is_manual_override = true
      AND manual_override_category IS NULL
      AND is_superseded = false
    """
    if workspace_id:
        query += " AND workspace_id = %s"
        params = (workspace_id,)
    else:
        params = None

    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute(query, params)
        return cur.fetchall()


def fetch_employee_policy_version(conn, employee_id: str) -> Optional[str]:
    """Get the current active policy version for an employee."""
    query = """
    SELECT id FROM public.attendance_policy_versions
    WHERE status IN ('ACTIVE', 'FROZEN')
    ORDER BY CASE WHEN status = 'ACTIVE' THEN 0 ELSE 1 END, created_at DESC
    LIMIT 1
    """
    with conn.cursor() as cur:
        cur.execute(query)
        row = cur.fetchone()
        return str(row[0]) if row else None


# ── Classification Logic ─────────────────────────────────────────────────


def classify_row(row: dict) -> tuple[int, str]:
    """
    Classify a manual override row into Cat 1, 2, or 3.

    Returns (category, notes).
    """
    notes = (row.get("manager_note") or "") + " " + (row.get("ot_reason") or "") + " " + (row.get("manual_override_notes") or "")
    notes_lower = notes.lower()

    # Check Cat 2 first — long-term patterns take priority
    for pattern in CAT2_PATTERNS:
        if re.search(pattern, notes_lower):
            return (2, f"Cat 2 auto-classified: matched pattern '{pattern}'")

    # Check Cat 1 — single-day / late approval patterns
    for pattern in CAT1_PATTERNS:
        if re.search(pattern, notes_lower):
            return (1, f"Cat 1 auto-classified: matched pattern '{pattern}'")

    # Default: Cat 3 — unclassifiable
    return (3, "Cat 3: unable to auto-classify — requires HR review")


def create_exception_for_row(conn, row: dict, policy_version_id: str, dry_run: bool) -> Optional[str]:
    """Create an attendance_exception for a Cat 1 row."""
    exc_id = str(uuid.uuid4())
    ws_id = row["workspace_id"]
    emp_id = row["employment_id"]
    date_val = row["date"]

    # Default exception: authorized late arrival or early leave on that specific date
    contributions = {
        "grace_period_minutes": 120,  # generous grace for late approval
    }
    reason = f"ADR-005 Cat 1 migration: HR manual edit on {date_val}. Original notes: {row.get('manager_note', 'N/A')}"

    if dry_run:
        return exc_id

    query = """
    INSERT INTO public.attendance_exceptions
        (id, workspace_id, employee_id, policy_version_id, exception_from, exception_to,
         contributions, reason, created_by, created_at, updated_at)
    VALUES (%s, %s, %s, %s, %s, %s, %s::jsonb, %s, %s, NOW(), NOW())
    """
    try:
        with conn.cursor() as cur:
            cur.execute(
                query,
                (
                    exc_id, ws_id, emp_id, policy_version_id,
                    date_val, date_val,  # single-day exception
                    psycopg2.extras.Json(contributions),
                    reason,
                    row.get("created_by"),
                ),
            )
        return exc_id
    except Exception as e:
        print(f"  WARNING: Failed to create exception for row {row['id']}: {e}", file=sys.stderr)
        return None


def create_modifier_for_row(conn, row: dict, policy_version_id: str, dry_run: bool) -> Optional[str]:
    """Create an employee_policy_modifier for a Cat 2 row."""
    mod_id = str(uuid.uuid4())
    ws_id = row["workspace_id"]
    emp_id = row["employment_id"]
    date_val = row["date"]

    contributions = {
        "grace_period_minutes": 60,  # lenient for special cases
        "authorized_early_leave_minutes": 60,
    }
    reason = f"ADR-005 Cat 2 migration: HR long-term adjustment from {date_val}. Original notes: {row.get('manager_note', 'N/A')}"

    # Effective from the row's date, no end date (ongoing)
    effective_from = f"{date_val}T00:00:00Z"

    if dry_run:
        return mod_id

    query = """
    INSERT INTO public.employee_policy_modifiers
        (id, workspace_id, employee_id, policy_version_id,
         priority_rank, effective_from, effective_to,
         contributions, reason, created_by, created_at, updated_at)
    VALUES (%s, %s, %s, %s, %s, %s::timestamptz, %s::timestamptz, %s::jsonb, %s, %s, NOW(), NOW())
    """
    try:
        with conn.cursor() as cur:
            cur.execute(
                query,
                (
                    mod_id, ws_id, emp_id, policy_version_id,
                    50,  # mid-priority — HR can adjust
                    effective_from,
                    None,  # no end date
                    psycopg2.extras.Json(contributions),
                    reason,
                    row.get("created_by"),
                ),
            )
        return mod_id
    except Exception as e:
        print(f"  WARNING: Failed to create modifier for row {row['id']}: {e}", file=sys.stderr)
        return None


def update_classification(conn, row_id: str, category: int, notes: str, dry_run: bool) -> None:
    """Update the classification on the daily summary row."""
    if dry_run:
        return
    query = """
    UPDATE public.attendance_daily_summaries
    SET manual_override_category = %s,
        requires_hr_review = %s,
        manual_override_notes = %s,
        updated_at = NOW()
    WHERE id = %s
    """
    with conn.cursor() as cur:
        cur.execute(query, (category, category == 3, notes, row_id))


# ── Export ───────────────────────────────────────────────────────────────


def export_cat3_csv(rows: list[dict], filename: str) -> str:
    """Export Cat 3 rows to CSV for HR review."""
    if not rows:
        return "No Cat 3 rows to export"

    os.makedirs(EXPORT_DIR, exist_ok=True)
    filepath = os.path.join(EXPORT_DIR, filename)

    with open(filepath, "w", newline="") as f:
        writer = csv.DictWriter(
            f,
            fieldnames=[
                "id", "workspace_id", "employment_id", "date",
                "actual_work_minutes", "system_detected_ot_minutes",
                "ot_reason", "manager_note", "manual_override_notes",
            ],
            extrasaction="ignore",
        )
        writer.writeheader()
        for row in rows:
            row_copy = {k: str(v) if v is not None else "" for k, v in row.items()}
            writer.writerow(row_copy)

    return f"Exported {len(rows)} Cat 3 rows to {filepath}"


# ── Main ─────────────────────────────────────────────────────────────────


def main() -> None:
    parser = argparse.ArgumentParser(description="ADR-005 Manual Override Migration")
    parser.add_argument("--dry-run", action="store_true", help="Preview classification without persisting")
    parser.add_argument("--input", help="CSV file with legacy manual override IDs (column: 'id')")
    parser.add_argument("--workspace-id", help="Limit to a specific workspace")
    parser.add_argument("--review-only", action="store_true", help="Only export Cat 3 review queue, don't migrate")
    args = parser.parse_args()

    conn = get_connection()

    print("=" * 72)
    print("  ADR-005 §4: Legacy Manual Override Migration")
    print(f"  Dry Run: {args.dry_run}")
    print("=" * 72)
    print()

    try:
        # 1. If input CSV provided, flag those rows as manual overrides
        if args.input:
            override_ids = set()
            with open(args.input) as f:
                reader = csv.DictReader(f)
                for row in reader:
                    override_ids.add(row["id"])
            count = flag_manual_override_rows(conn, override_ids, args.dry_run)
            print(f"  Flagged {count} rows as manual overrides from {args.input}")
            print()

        # 2. Fetch unclassified manual overrides
        rows = fetch_manual_overrides(conn, args.workspace_id)
        print(f"  Unclassified manual overrides: {len(rows)}")
        if not rows:
            print("  Nothing to migrate.")
            return
        print()

        if args.review_only:
            # Just export Cat 3 review queue
            cat3_rows = [r for r in rows if r.get("requires_hr_review")]
            if cat3_rows:
                result = export_cat3_csv(cat3_rows, "hr_review_cat3_overrides.csv")
                print(f"  {result}")
            else:
                print("  No Cat 3 rows requiring review.")
            return

        # 3. Classify and migrate
        cat1_count = 0
        cat2_count = 0
        cat3_count = 0
        cat3_details: list[dict] = []
        failed: list[dict] = []

        for row in rows:
            category, notes = classify_row(row)

            if category == 1:
                policy_id = fetch_employee_policy_version(conn, str(row["employment_id"]))
                if policy_id:
                    exc_id = create_exception_for_row(conn, row, policy_id, args.dry_run)
                    if exc_id:
                        cat1_count += 1
                        update_classification(conn, str(row["id"]), 1, notes, args.dry_run)
                    else:
                        failed.append({"row_id": str(row["id"]), "category": 1, "reason": "policy_version not found"})
                else:
                    failed.append({"row_id": str(row["id"]), "category": 1, "reason": "no active policy version"})

            elif category == 2:
                policy_id = fetch_employee_policy_version(conn, str(row["employment_id"]))
                if policy_id:
                    mod_id = create_modifier_for_row(conn, row, policy_id, args.dry_run)
                    if mod_id:
                        cat2_count += 1
                        update_classification(conn, str(row["id"]), 2, notes, args.dry_run)
                    else:
                        failed.append({"row_id": str(row["id"]), "category": 2, "reason": "policy_version not found"})
                else:
                    failed.append({"row_id": str(row["id"]), "category": 2, "reason": "no active policy version"})

            else:
                cat3_count += 1
                update_classification(conn, str(row["id"]), 3, notes, args.dry_run)
                cat3_details.append(dict(row))

        if not args.dry_run:
            conn.commit()

        # 4. Export Cat 3 rows
        if cat3_details:
            result = export_cat3_csv(cat3_details, "hr_review_cat3_overrides.csv")
        else:
            result = "No Cat 3 rows"

        # 5. Summary
        print("=" * 72)
        if args.dry_run:
            print("  DRY RUN — Classification preview:")
        else:
            print("  MIGRATION COMPLETE")
        print(f"  Cat 1 (late approval → exceptions):     {cat1_count:>6}")
        print(f"  Cat 2 (long-term adjustment → modifiers): {cat2_count:>6}")
        print(f"  Cat 3 (unclassifiable → HR review):      {cat3_count:>6}")
        print(f"  {result}")
        if failed:
            print(f"  Failed: {len(failed)} rows")
            for f_item in failed[:10]:
                print(f"    [{f_item['category']}] id={f_item['row_id']}: {f_item['reason']}")
        print("=" * 72)

        if cat3_count > 0:
            print()
            print("  Next steps:")
            print("  1. Review Cat 3 CSV export with HR")
            print("  2. HR manually classifies or creates exceptions/modifiers")
            print("  3. Mark rows as resolved (clear requires_hr_review)")
            print("  4. SLA: 90 days from cutover (ADR-005 §4)")

    except Exception as e:
        conn.rollback()
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
    finally:
        conn.close()


if __name__ == "__main__":
    main()

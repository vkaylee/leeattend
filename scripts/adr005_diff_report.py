#!/usr/bin/env python3
"""
ADR-005 Diff Rate Monitoring Report
====================================
Queries `cascade_diff_report_v` to compute Group A/B/C diff classification
and emits a daily report with threshold alerts.

Usage:
    python3 scripts/adr005_diff_report.py [--workspace-id <uuid>] [--since YYYY-MM-DD] [--json]

The script reads `.attendagent/migration_expected_diffs.md` to classify Group A
(expected) diffs.  Remaining diffs are classified as Group B/C (unexpected)
which blocks cutover per ADR-005 §2.

Thresholds (ADR-005 §2):
    - Group B/C > 0       → BLOCK CUTOVER
    - Group A > 5% total  → HOLD — HR review required
"""

import argparse
import json
import os
import sys
from datetime import date, datetime, timedelta
from typing import Optional

import psycopg2
import psycopg2.extras

# ── Configuration ───────────────────────────────────────────────────────

DB_URL = os.getenv(
    "WORKSPACE_DB_URL",
    "postgres://admin:change_this_password_immediately@localhost:5432/leeattend",
)

EXPECTED_DIFFS_PATH = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    ".attendagent",
    "migration_expected_diffs.md",
)

# ── Helpers ─────────────────────────────────────────────────────────────


def load_expected_diffs() -> list[dict]:
    """Parse migration_expected_diffs.md for pre-documented Group A diffs."""
    if not os.path.exists(EXPECTED_DIFFS_PATH):
        return []
    try:
        with open(EXPECTED_DIFFS_PATH) as f:
            content = f.read()
    except OSError:
        return []

    expected = []
    in_table = False
    for line in content.splitlines():
        line = line.strip()
        if line.startswith("|") and "field" in line.lower():
            in_table = True
            continue
        if in_table and line.startswith("|"):
            parts = [p.strip() for p in line.split("|") if p.strip()]
            if len(parts) >= 3 and parts[0] != "---":
                expected.append({
                    "field": parts[0],
                    "legacy_value": parts[1] if len(parts) > 1 else None,
                    "cascade_value": parts[2] if len(parts) > 2 else None,
                    "delta": int(parts[3]) if len(parts) > 3 and parts[3].lstrip("-").isdigit() else None,
                    "justification": parts[4] if len(parts) > 4 else "Unknown",
                })
    return expected


def magnitude_bucket(delta: int) -> str:
    ad = abs(delta)
    if ad <= 5:
        return "0-5m"
    if ad <= 15:
        return "5-15m"
    return "15m+"


def match_expected(field: str, cascade_val: int, legacy_val: int, expected_diffs: list[dict]) -> bool:
    """Check if a diff matches a documented Group A expected diff."""
    for ed in expected_diffs:
        if ed["field"] != field:
            continue
        # Match cascade/legacy values if specified, otherwise match on field alone
        legacy_match = ed["legacy_value"] is None or int(ed["legacy_value"]) == legacy_val
        cascade_match = ed["cascade_value"] is None or int(ed["cascade_value"]) == cascade_val
        if legacy_match and cascade_match:
            return True
    return False


# ── Queries ──────────────────────────────────────────────────────────────


def fetch_diff_rows(conn, workspace_id: Optional[str], since: Optional[str]) -> list[dict]:
    """Query cascade_diff_report_v for rows with any diff."""
    query = """
    SELECT
        summary_id,
        workspace_id,
        employment_id,
        date,
        policy_version_id,
        cascade_grace_minutes,
        legacy_grace_minutes,
        grace_delta_minutes,
        cascade_ael_minutes,
        legacy_ael_minutes,
        ael_delta_minutes,
        has_any_diff,
        cascade_trace_len,
        created_at
    FROM public.cascade_diff_report_v
    WHERE has_any_diff = true
    """
    params = []
    if workspace_id:
        query += " AND workspace_id = %s"
        params.append(workspace_id)
    if since:
        query += " AND date >= %s"
        params.append(since)

    query += " ORDER BY date DESC, employment_id"

    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute(query, params)
        return cur.fetchall()


def fetch_totals(conn, workspace_id: Optional[str]) -> dict:
    """Fetch total summary rows and diff row counts."""
    where_clause = "WHERE is_superseded = false"
    if workspace_id:
        where_clause += " AND workspace_id = %s"

    query = f"""
    SELECT
        COUNT(*) AS total_rows,
        COUNT(*) FILTER (
            WHERE effective_rules_snapshot IS NOT NULL AND legacy_rules_snapshot IS NOT NULL
        ) AS dual_run_rows,
        COUNT(*) FILTER (
            WHERE effective_rules_snapshot IS NOT NULL
              AND legacy_rules_snapshot IS NOT NULL
              AND effective_rules_snapshot->'final_rules' IS DISTINCT FROM legacy_rules_snapshot
        ) AS diff_rows
    FROM public.attendance_daily_summaries
    {where_clause}
    """
    params = (workspace_id,) if workspace_id else None
    with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
        cur.execute(query, params)
        return cur.fetchone()


# ── Report ───────────────────────────────────────────────────────────────


def generate_report(
    conn, workspace_id: Optional[str], since: Optional[str], output_json: bool
) -> dict:
    """Generate the diff report and classify Group A/B/C."""
    expected_diffs = load_expected_diffs()
    rows = fetch_diff_rows(conn, workspace_id, since)
    totals = fetch_totals(conn, workspace_id)

    group_a_counts = {"grace": 0, "ael": 0}
    group_bc_counts = {"grace": 0, "ael": 0}
    magnitude_distribution = {"0-5m": 0, "5-15m": 0, "15m+": 0}
    bc_details: list[dict] = []

    for row in rows:
        # Grace delta
        gd = row["grace_delta_minutes"]
        if gd != 0:
            bucket = magnitude_bucket(gd)
            magnitude_distribution[bucket] += 1
            if match_expected("grace_period_minutes", row["cascade_grace_minutes"], row["legacy_grace_minutes"], expected_diffs):
                group_a_counts["grace"] += 1
            else:
                group_bc_counts["grace"] += 1
                bc_details.append({
                    "summary_id": str(row["summary_id"]),
                    "employment_id": str(row["employment_id"]),
                    "date": str(row["date"]),
                    "field": "grace_period_minutes",
                    "cascade": row["cascade_grace_minutes"],
                    "legacy": row["legacy_grace_minutes"],
                    "delta": gd,
                    "magnitude": bucket,
                })

        # AEL delta
        ad = row["ael_delta_minutes"]
        if ad != 0:
            bucket = magnitude_bucket(ad)
            magnitude_distribution[bucket] += 1
            if match_expected("authorized_early_leave_minutes", row["cascade_ael_minutes"], row["legacy_ael_minutes"], expected_diffs):
                group_a_counts["ael"] += 1
            else:
                group_bc_counts["ael"] += 1
                bc_details.append({
                    "summary_id": str(row["summary_id"]),
                    "employment_id": str(row["employment_id"]),
                    "date": str(row["date"]),
                    "field": "authorized_early_leave_minutes",
                    "cascade": row["cascade_ael_minutes"],
                    "legacy": row["legacy_ael_minutes"],
                    "delta": ad,
                    "magnitude": bucket,
                })

    total_diff_rows = len(rows)
    total_bc = group_bc_counts["grace"] + group_bc_counts["ael"]
    total_group_a = group_a_counts["grace"] + group_a_counts["ael"]

    # Threshold checks (ADR-005 §2)
    threshold_alerts = []
    if total_bc > 0:
        threshold_alerts.append("BLOCK CUTOVER: Group B/C diffs detected (unexpected new-engine bug or regression)")
    total_all = totals["total_rows"]
    if total_all > 0 and (total_group_a / total_all) > 0.05:
        threshold_alerts.append(f"HOLD: Group A diffs exceed 5% ({total_group_a}/{total_all} = {total_group_a/total_all:.1%}) — HR review required before cutover")

    report = {
        "generated_at": datetime.utcnow().isoformat(),
        "period": {"since": since or "(all time)"},
        "totals": {
            "total_rows": totals["total_rows"],
            "dual_run_rows": totals["dual_run_rows"],
            "diff_rows": totals["diff_rows"],
        },
        "classification": {
            "group_a_expected": {
                "grace": group_a_counts["grace"],
                "ael": group_a_counts["ael"],
                "total": total_group_a,
                "note": "Documented in .attendagent/migration_expected_diffs.md",
            },
            "group_bc_unexpected": {
                "grace": group_bc_counts["grace"],
                "ael": group_bc_counts["ael"],
                "total": total_bc,
                "note": "BLOCK CUTOVER if > 0 (ADR-005 §2)",
            },
        },
        "magnitude_distribution": magnitude_distribution,
        "threshold_alerts": threshold_alerts,
    }

    if output_json:
        report["bc_details"] = bc_details[:100]  # Truncate for JSON output
        print(json.dumps(report, indent=2))
    else:
        _print_text_report(report, bc_details, len(rows))

    return report


def _print_text_report(report: dict, bc_details: list[dict], diff_row_count: int) -> None:
    """Print a human-readable text report."""
    t = report["totals"]
    c = report["classification"]
    md = report["magnitude_distribution"]

    print("=" * 72)
    print("  ADR-005 Dual-Run Diff Rate Report")
    print(f"  Generated: {report['generated_at']}")
    print(f"  Period:    {report['period']['since']}")
    print("=" * 72)
    print()
    print(f"  Total rows (live):       {t['total_rows']:>8,}")
    print(f"  Dual-run instrumented:   {t['dual_run_rows']:>8,}")
    print(f"  Rows with any diff:      {t['diff_rows']:>8,}")
    print()
    print(f"  ── Diff Classification ──")
    print(f"  Group A (expected fix):  {c['group_a_expected']['total']:>8}")
    print(f"    - grace_period:         {c['group_a_expected']['grace']:>8}")
    print(f"    - authorized_early_leave: {c['group_a_expected']['ael']:>8}")
    print(f"  Group B/C (unexpected):  {c['group_bc_unexpected']['total']:>8}")
    print()
    print(f"  ── Magnitude Distribution ──")
    print(f"  0-5 min:   {md['0-5m']:>8}")
    print(f"  5-15 min:  {md['5-15m']:>8}")
    print(f"  15+ min:   {md['15m+']:>8}")
    print()

    if report["threshold_alerts"]:
        print(f"  ── THRESHOLD ALERTS ──")
        for alert in report["threshold_alerts"]:
            print(f"  !! {alert}")
    else:
        print(f"  ✓ All thresholds pass — cutover may proceed")

    if bc_details:
        print()
        print(f"  ── Group B/C Details (first 20) ──")
        for d in bc_details[:20]:
            print(f"  [{d['date']}] {d['field']}: cascade={d['cascade']} legacy={d['legacy']} Δ={d['delta']} [{d['magnitude']}]")
        if len(bc_details) > 20:
            print(f"  ... and {len(bc_details) - 20} more")

    print()


# ── Main ─────────────────────────────────────────────────────────────────


def main() -> None:
    parser = argparse.ArgumentParser(description="ADR-005 Diff Rate Report")
    parser.add_argument("--workspace-id", help="Filter to a specific workspace")
    parser.add_argument("--since", help="Start date (YYYY-MM-DD), default: 30 days ago")
    parser.add_argument("--json", action="store_true", help="Output JSON instead of text")
    args = parser.parse_args()

    since = args.since
    if not since:
        since = (date.today() - timedelta(days=30)).isoformat()

    db_url = os.getenv("TENANT_DB_URL", DB_URL)
    try:
        conn = psycopg2.connect(db_url)
    except psycopg2.OperationalError as e:
        print(f"ERROR: Cannot connect to database: {e}", file=sys.stderr)
        print(f"       Set TENANT_DB_URL or WORKSPACE_DB_URL env var.", file=sys.stderr)
        sys.exit(1)

    try:
        generate_report(conn, args.workspace_id, since, args.json)
    finally:
        conn.close()


if __name__ == "__main__":
    main()

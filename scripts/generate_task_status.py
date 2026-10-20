#!/usr/bin/env python3
"""Regenerate docs/migration/rust-go/task-status.csv from the migration ledger.

The ledger is the single source of truth for task IDs and dependencies; the CSV
is the machine-readable ownership map required by ledger task P0-03.

Usage: ./scripts/generate_task_status.py
"""

from __future__ import annotations

import csv
import re
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
LEDGER = PROJECT_ROOT / "rust-go-migration-task-ledger.md"
OUTPUT = PROJECT_ROOT / "docs" / "migration" / "rust-go" / "task-status.csv"

TASK_RE = re.compile(r"^- \[([ x])\] ([A-Z][A-Z0-9]*-[0-9]+) (.+)$")
DEPENDS_MARKER = ". Depends: "
EVIDENCE_MARKER = ". Evidence: "
VERIFY_MARKER = ". Verify: "

EXPECTED_TASKS = 103


def parse_task(line: str) -> dict[str, str] | None:
    match = TASK_RE.match(line)
    if match is None:
        return None
    checked, task_id, body = match.groups()
    if DEPENDS_MARKER not in body or VERIFY_MARKER not in body:
        raise ValueError(f"unparseable ledger task: {line}")
    description, rest = body.split(DEPENDS_MARKER, 1)
    dependency, verify = rest.split(VERIFY_MARKER, 1)
    evidence = ""
    if EVIDENCE_MARKER in dependency:
        dependency, evidence = dependency.split(EVIDENCE_MARKER, 1)
    return {
        "task_id": task_id,
        "owner": "Backend Lead",
        "status": "verified" if checked == "x" else "not-started",
        "description": description,
        "dependency": dependency,
        "evidence": evidence or "pending",
        "rollback_note": (
            "Parallel Go implementation; Rust API remains untouched and operational."
        ),
    }


def main() -> int:
    rows = [
        row
        for row in (parse_task(line) for line in LEDGER.read_text(encoding="utf-8").splitlines())
        if row is not None
    ]
    if len(rows) != EXPECTED_TASKS:
        print(f"expected {EXPECTED_TASKS} ledger tasks, parsed {len(rows)}", file=sys.stderr)
        return 1
    ids = [row["task_id"] for row in rows]
    if len(set(ids)) != len(ids):
        print("duplicate task IDs in ledger", file=sys.stderr)
        return 1

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    verified = sum(1 for row in rows if row["status"] == "verified")
    print(f"wrote {len(rows)} tasks to {OUTPUT} ({verified} verified)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

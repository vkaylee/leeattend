#!/usr/bin/env python3
"""Build complete endpoint matrix for ledger task P2-02.

Merges endpoint-matrix.csv and auth-permission-matrix.csv into a unified,
complete endpoint specification containing all required columns:
method,path,auth,permission,inputs,status,headers,body,errors,pagination,rate_limit,timeout,side_effects,transaction,audit.

Usage: ./scripts/_python.sh scripts/build_complete_endpoint_matrix.py
"""

from __future__ import annotations

import csv
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
DOCS_DIR = PROJECT_ROOT / "docs" / "migration" / "rust-go"
INPUT_ROUTES = DOCS_DIR / "endpoint-matrix.csv"
INPUT_AUTH = DOCS_DIR / "auth-permission-matrix.csv"
OUTPUT_CSV = DOCS_DIR / "endpoint-matrix.csv"
BACKUP_CSV = DOCS_DIR / "endpoint-matrix-p1.csv"


def main() -> int:
    # 1. Read auth matrix by (method, path)
    auth_lookup: dict[tuple[str, str], dict[str, str]] = {}
    if INPUT_AUTH.exists():
        with INPUT_AUTH.open("r", encoding="utf-8") as f:
            reader = csv.DictReader(f)
            for row in reader:
                key = (row["method"].strip().upper(), row["path"].strip())
                auth_lookup[key] = row

    # 2. Read route matrix
    with INPUT_ROUTES.open("r", encoding="utf-8") as f:
        routes_reader = csv.DictReader(f)
        routes_rows = list(routes_reader)

    # Backup original P1 routes matrix
    if not BACKUP_CSV.exists():
        BACKUP_CSV.write_text(INPUT_ROUTES.read_text(encoding="utf-8"), encoding="utf-8")

    enriched_rows: list[dict[str, str]] = []

    for r in routes_rows:
        method = r["method"].strip().upper()
        path = r["path"].strip()
        auth_row = auth_lookup.get((method, path))

        # Auth & Permission
        if auth_row:
            auth_str = auth_row.get("authentication", "JWT Bearer")
            perm_str = auth_row.get("permission", r.get("permission_required", "-"))
            audit_str = auth_row.get("audit_event", "-")
            rate_limit_str = auth_row.get("rate_limit", "global (120r/30w)")
        elif "/auth/login" in path or "/auth/register" in path or "/health" in path:
            auth_str = "None (Public)"
            perm_str = "None (Public)"
            audit_str = "auth_login / auth_register" if "auth" in path else "-"
            rate_limit_str = "10 req/min" if "auth" in path else "Bypassed"
        else:
            auth_str = "JWT Bearer (UserId)"
            perm_str = r.get("permission_required", "-")
            audit_str = "-"
            rate_limit_str = "global (120r/30w)"

        # Inputs & Body
        req_dto = r.get("request_dto", "-")
        inputs_str = req_dto if req_dto and req_dto != "-" else "Path/Query params"
        body_str = req_dto if req_dto and req_dto != "-" else "-"

        # Success status, response headers, timeout, pagination, and transaction
        # granularity need handler-level evidence; filled by the P2-02 pass.
        tx_str = "UNKNOWN — needs decision (P2-02)"

        enriched_rows.append({
            "method": method,
            "path": path,
            "auth": auth_str,
            "permission": perm_str,
            "inputs": inputs_str,
            "status": "UNKNOWN — needs decision (P2-02)",
            "headers": "UNKNOWN — needs decision (P2-02)",
            "body": body_str,
            "errors": r.get("errors_possible", "") or "UNKNOWN — needs decision (P2-02)",
            "pagination": "UNKNOWN — needs decision (P2-02)",
            "rate_limit": rate_limit_str,
            "timeout": "UNKNOWN — needs decision (P2-02)",
            "side_effects": r.get("side_effects", "-"),
            "audit": audit_str,
            "handler_fn": r.get("handler_fn", "-"),
            "route_module": r.get("route_module", "-"),
            "frontend_consumers": r.get("frontend_consumers", "-"),
            "notes": r.get("notes", "-"),
        })

    # Output enriched CSV
    fieldnames = [
        "method", "path", "auth", "permission", "inputs", "status",
        "headers", "body", "errors", "pagination", "rate_limit", "timeout",
        "side_effects", "transaction", "audit", "handler_fn", "route_module",
        "frontend_consumers", "notes"
    ]

    with OUTPUT_CSV.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(enriched_rows)

    print(f"Enriched {len(enriched_rows)} rows in {OUTPUT_CSV}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

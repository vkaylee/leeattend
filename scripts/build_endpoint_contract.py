#!/usr/bin/env python3
"""Derive endpoint response contracts and regenerate endpoint-matrix.csv.

Every derived value is produced by a deterministic rule over Rust source under
`apiserver/src` plus the P1 route inventory (`endpoint-matrix-p1.csv`). Nothing
is invented: when no source evidence exists the value says so explicitly.

Usage:
    ./scripts/_python.sh scripts/build_endpoint_contract.py
"""
from __future__ import annotations

import csv
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs" / "migration" / "rust-go"
CSV_PATH = DOCS / "endpoint-matrix.csv"
P1_PATH = DOCS / "endpoint-matrix-p1.csv"
SRC = ROOT / "apiserver" / "src"
UNKNOWN = "UNKNOWN — needs decision (P2-02)"
NO_LAYER = "none (no layer)"
NO_EVIDENCE = "none (no evidence)"
NO_PAGINATION = "none"
FIELDS = [
    "method", "path", "auth", "permission", "inputs", "status", "headers",
    "body", "errors", "pagination", "rate_limit", "timeout", "side_effects",
    "transaction", "audit", "handler_fn", "route_module", "frontend_consumers",
    "notes",
]
COLUMNS = ("status", "headers", "pagination", "timeout", "transaction", "errors", "side_effects")

STATUS_CODES = {
    "OK": "200", "CREATED": "201", "ACCEPTED": "202", "NO_CONTENT": "204",
    "BAD_REQUEST": "400", "UNAUTHORIZED": "401", "FORBIDDEN": "403",
    "NOT_FOUND": "404", "CONFLICT": "409", "PAYLOAD_TOO_LARGE": "413",
    "TOO_MANY_REQUESTS": "429", "INTERNAL_SERVER_ERROR": "500",
    "SERVICE_UNAVAILABLE": "503",
}
FN_RE = re.compile(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")
HANDLER_NOTE_RE = re.compile(r"handler .*?([A-Za-z_][A-Za-z0-9_]*) \((apiserver/src/handlers/[^:]+):(\d+)\)")
STATUS_NOTE_RE = re.compile(r"statuses=([0-9; ,]+)")
CALL_NOTE_RE = re.compile(r"([A-Za-z_][A-Za-z0-9_]*)\s*\((apiserver/src/(?:services|repositories)/[^:]+):(\d+)\)")

DB_WRITE_RE = re.compile(
    r"insert_into|diesel::update|diesel::delete|delete_from|save_changes"
    r"|batch_execute\s*\(\s*(?!\"SELECT)"
    r"|\.execute\s*\(\s*(?:&?mut\s+)?(?:\*?conn|boxed_conn)"
)
QUEUE_RE = re.compile(r"job_queue|enqueue|create_job|queue_job|BackgroundJob|list_jobs_paginated", re.I)
CACHE_RE = re.compile(r"redis|cache|moka", re.I)
STORAGE_RE = re.compile(r"\bS3\b|s3_client|object_storage|storage\(\)|upload_file|put_object|Multipart|presign", re.I)
EMAIL_RE = re.compile(r"email_service|EmailService|smtp", re.I)
AUDIT_RE = re.compile(r"AuditHelper::log|audit_repo\(\)")
REQUIRE_RE = re.compile(r"require_(?:permission|scoped_permission|any_scoped_permission|role|any_role)")
EXTRACTOR_RE = re.compile(r"\b(?:Json|Query|Path|Multipart|HeaderMap|TypedHeader)\s*[<(]")
TIMEOUT_RE = re.compile(r"tokio::time::timeout\(\s*(?:Duration::from_secs\(\s*)?([A-Z_0-9]+)")
CONST_SECS_RE = re.compile(r"const\s+([A-Z0-9_]+)\s*:\s*u64\s*=\s*(\d+)")


def read_csv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as stream:
        return list(csv.DictReader(stream))


def brace_body(text: str, opening: int) -> str:
    """Return the Rust block starting at `opening`, ignoring braces in strings/comments."""
    depth = 0
    i = opening
    quote = ""
    line_comment = False
    block_comment = 0
    while i < len(text):
        char = text[i]
        nxt = text[i + 1] if i + 1 < len(text) else ""
        if line_comment:
            if char == "\n":
                line_comment = False
        elif block_comment:
            if char == "/" and nxt == "*":
                block_comment += 1
                i += 1
            elif char == "*" and nxt == "/":
                block_comment -= 1
                i += 1
        elif quote:
            if char == "\\":
                i += 1
            elif char == quote:
                quote = ""
        elif char == "/" and nxt == "/":
            line_comment = True
            i += 1
        elif char == "/" and nxt == "*":
            block_comment = 1
            i += 1
        elif char in "\"'":
            quote = char
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[opening : i + 1]
        i += 1
    return text[opening:]


def strip_tests(text: str) -> str:
    """Drop `#[cfg(test)]` modules so test-only writes never look like behavior."""
    out: list[str] = []
    cursor = 0
    while True:
        start = text.find("#[cfg(test)]", cursor)
        if start < 0:
            out.append(text[cursor:])
            return "".join(out)
        out.append(text[cursor:start])
        opening = text.find("{", start)
        if opening < 0:
            return "".join(out)
        cursor = opening + len(brace_body(text, opening))


def function_index() -> dict[str, list[tuple[str, str]]]:
    """Map function name to (relative path, signature + body) for every Rust function."""
    index: dict[str, list[tuple[str, str]]] = {}
    for path in SRC.rglob("*.rs"):
        text = path.read_text(encoding="utf-8", errors="replace")
        rel = str(path.relative_to(ROOT))
        for match in FN_RE.finditer(text):
            opening = text.find("{", match.end())
            semicolon = text.find(";", match.end())
            if opening < 0 or 0 <= semicolon < opening:
                continue
            index.setdefault(match.group(1), []).append((rel, text[match.start() : opening] + brace_body(text, opening)))
    return index


def handler_source(row: dict[str, str], index: dict[str, list[tuple[str, str]]]) -> tuple[str, str]:
    """Resolve the handler function body using the notes callsite, then the fn name."""
    name = row["handler_fn"].split("::")[-1]
    note = HANDLER_NOTE_RE.search(row.get("notes", ""))
    if note:
        path = note.group(2)
        for found, body in index.get(name, []):
            if found == path:
                return found, body
    candidates = index.get(name, [])
    for path, body in candidates:
        if path.startswith("apiserver/src/handlers/"):
            return path, body
    if len(candidates) == 1:
        return candidates[0]
    return "", ""


def downstream_sources(row: dict[str, str], handler_path: str, body: str, index: dict[str, list[tuple[str, str]]]) -> list[tuple[str, str]]:
    """Resolve P1-recorded service/repository calls to their implementation bodies.

    P1 sometimes records the callsite file (the handler) rather than the
    implementation file, so resolution falls back to the repository/service
    implementation registered under the same method name.
    """
    result: list[tuple[str, str]] = []
    seen: set[tuple[str, str]] = set()

    def add(path: str, entry_body: str) -> None:
        key = (path, entry_body)
        if path.endswith("_test.rs") or key in seen:
            return
        seen.add(key)
        result.append(key)

    names: list[str] = []
    for column in ("service_calls", "repository_calls"):
        for value in row["_p1"].get(column, "").split(";"):
            match = re.search(r"::([A-Za-z_][A-Za-z0-9_]*)\s*\(", value)
            if match:
                names.append(match.group(1))
    for name in dict.fromkeys(names):
        candidates = [
            (path, entry)
            for path, entry in index.get(name, [])
            if ("/repositories/" in path or "/services/" in path or "/queue/" in path)
            and not path.endswith("_test.rs")
        ]
        if not candidates:
            continue
        exact = [item for item in candidates if item[0] == handler_path]
        add(*(exact[0] if exact else candidates[0]))
    return result


def request_evidence(body: str, downstream: list[tuple[str, str]]) -> str:
    """Handler plus downstream source, test modules removed."""
    parts = [strip_tests(body)]
    parts.extend(strip_tests(entry) for _, entry in downstream)
    return "\n".join(parts)


def derive_status(row: dict[str, str], handler_body: str) -> str:
    if "WebSocketUpgrade" in handler_body or "ws.on_upgrade" in handler_body:
        return "101 (WebSocket upgrade)"
    if row["handler_fn"].endswith("health_check") or row["handler_fn"].endswith("readiness_check"):
        return "200"
    explicit: list[str] = []
    for match in re.finditer(r"(?:Ok|return\s+Ok)\s*\([^;]{0,240}?StatusCode::([A-Z_]+)", handler_body, re.S):
        code = STATUS_CODES.get(match.group(1))
        if code and code not in explicit:
            explicit.append(code)
    for match in re.finditer(r"StatusCode::([A-Z_]+)[^;]{0,180}?\.into_response\(\)", handler_body, re.S):
        code = STATUS_CODES.get(match.group(1))
        if code and code not in explicit:
            explicit.append(code)
    documented: list[str] = []
    note = STATUS_NOTE_RE.search(row.get("notes", ""))
    if note:
        documented = [value.strip() for value in note.group(1).replace(";", ",").split(",") if value.strip().isdigit()]
    success = [code for code in documented if code.startswith("2")]
    for code in explicit:
        if code.startswith("2") and code not in success:
            success.append(code)
    if success:
        return ",".join(dict.fromkeys(success))
    if explicit:
        return ",".join(dict.fromkeys(explicit))
    if documented:
        return documented[0]
    if re.search(r"Json\s*[<(]", handler_body):
        return "200"
    return NO_EVIDENCE


def derive_headers(handler_body: str) -> str:
    found = ["x-request-id (global mw/request_id middleware)"]
    if re.search(r"CONTENT_DISPOSITION|Content-Disposition", handler_body):
        found.append("Content-Disposition")
    if re.search(r"header::CONTENT_TYPE|Sse\s*<", handler_body):
        found.append("Content-Type (endpoint override)")
    elif re.search(r"Json\s*[<(]", handler_body):
        found.append("Content-Type: application/json")
    if re.search(r"create_auth_cookies|clear_auth_cookies|Set-Cookie|set_cookie", handler_body):
        found.append("Set-Cookie")
    if re.search(r"Redirect::|Redirect\s*\{", handler_body):
        found.append("Location")
    if re.search(r"StatusCode::TOO_MANY_REQUESTS", handler_body):
        found.append("Retry-After (429 handler response)")
    return "; ".join(dict.fromkeys(found))


def derive_pagination(row: dict[str, str], handler_body: str) -> str:
    evidence = handler_body + "\n" + row["_p1"].get("response_dto", "") + "\n" + row["_p1"].get("request_dto", "")
    limit_match = re.search(r"(?:query\.)?limit[^;]{0,60}?unwrap_or\((\d+)\)[^;]{0,40}?clamp\((\d+),\s*(\d+)\)", handler_body)
    if re.search(r"next_cursor|cursor_token|CursorParams|CursorRequest|\bdurable_cursor\b", evidence):
        return "cursor (opaque next_cursor in response)"
    if re.search(r"PaginatedResponse|SafePagination|PaginationParams", evidence):
        if "DEFAULT_REPORT_PAGE_LIMIT" in handler_body:
            return "page/limit (default 20, clamped 1..=3000 by SafePagination)"
        return "page/limit (default 50, clamped 1..=3000 by SafePagination)"
    if limit_match:
        return f"page/limit (default {limit_match.group(1)}, clamped {limit_match.group(2)}..={limit_match.group(3)})"
    return NO_PAGINATION


def derive_timeout(body: str, evidence: str, consts: dict[str, int]) -> str:
    """Request layer has no timeout layer; report handler-level bounds when present."""
    bounds: list[str] = []
    for match in TIMEOUT_RE.finditer(body):
        token = match.group(1)
        if token.isdigit():
            bounds.append(f"handler-level bound {token}s")
        elif token in consts:
            bounds.append(f"handler-level bound {consts[token]}s ({token})")
    for match in TIMEOUT_RE.finditer(evidence):
        token = match.group(1)
        if token in consts:
            entry = f"handler-level bound {consts[token]}s ({token})"
            if entry not in bounds:
                bounds.append(entry)
    if not bounds:
        return NO_LAYER
    return f"{NO_LAYER}; " + "; ".join(dict.fromkeys(bounds))


def derive_transaction(handler_path: str, handler_body: str, downstream: list[tuple[str, str]]) -> str:
    if re.search(r"with_tenant_tx|\.transaction\s*\(", handler_body):
        return "explicit transaction boundary in handler (with_tenant_tx/transaction)"
    layers: list[str] = []
    for path, entry in downstream:
        if not re.search(r"with_tenant_tx|\.transaction\s*\(", entry):
            continue
        if "/services/" in path:
            layers.append("transaction in called service (with_tenant_tx/transaction)")
        elif "/repositories/" in path:
            layers.append("transaction per repository operation (with_tenant_tx/transaction)")
        else:
            layers.append("transaction in called helper (with_tenant_tx/transaction)")
    if layers:
        return "; ".join(dict.fromkeys(layers))
    return "none (no transaction evidence)"


def derive_errors(row: dict[str, str], handler_body: str, evidence: str) -> str:
    prior = row["_p1"].get("errors_possible", "").strip()
    markers = [value for value in prior.split(";") if value and value != "handler error path only"]
    derived: list[str] = []
    if REQUIRE_RE.search(evidence) or "forbidden" in evidence:
        derived.append("403 (require_* guard)")
    if re.search(r"AppError::(?:not_found|NotFound)", evidence):
        derived.append("AppError::not_found")
    if re.search(r"AppError::(?:validation|Validation|validation_args)", evidence):
        derived.append("AppError::validation")
    if re.search(r"AppError::(?:Conflict|conflict)", evidence):
        derived.append("AppError::conflict")
    if re.search(r"AppError::(?:Internal|internal)", evidence):
        derived.append("AppError::Internal")
    if EXTRACTOR_RE.search(handler_body):
        derived.append("extractor rejection (path/query/JSON/multipart)")
    if "Result<" in handler_body or "AppResult" in handler_body:
        derived.append("propagated service/repository error")
    if not derived:
        return ";".join(markers) if markers else NO_EVIDENCE
    merged = list(markers)
    for marker in derived:
        if marker not in merged:
            merged.append(marker)
    return ";".join(merged)


def derive_side_effects(method: str, handler_body: str, downstream: list[tuple[str, str]], consts: dict[str, int]) -> str:
    evidence = request_evidence(handler_body, downstream)
    effects: list[str] = []
    if DB_WRITE_RE.search(evidence):
        effects.append("DB write via service/repository mutation")
    if QUEUE_RE.search(evidence):
        effects.append("background job/queue enqueue")
    if CACHE_RE.search(evidence):
        effects.append("cache/Redis")
    if STORAGE_RE.search(evidence):
        effects.append("object storage/file IO/virus scan")
    if EMAIL_RE.search(evidence):
        effects.append("email/SMTP")
    if AUDIT_RE.search(evidence):
        effects.append("audit log")
    if effects:
        return "; ".join(dict.fromkeys(effects))
    if method != "GET":
        return f"{NO_EVIDENCE} (no DB/cache/queue/storage/side-effect callsite in handler or called service/repository)"
    return NO_EVIDENCE


def main() -> int:
    rows = read_csv(CSV_PATH)
    index = function_index()
    consts: dict[str, int] = {}
    for path in SRC.rglob("*.rs"):
        for match in CONST_SECS_RE.finditer(path.read_text(encoding="utf-8", errors="replace")):
            consts[match.group(1)] = int(match.group(2))
    p1 = {(row["method"].strip().upper(), row["path"].strip()): row for row in read_csv(P1_PATH)}

    unknowns: dict[str, list[str]] = {column: [] for column in COLUMNS}
    for row in rows:
        row["_p1"] = p1.get((row["method"].upper(), row["path"]), {})
        handler_path, handler_body = handler_source(row, index)
        downstream = downstream_sources(row, handler_path, handler_body, index) if handler_body else []
        evidence = request_evidence(handler_body, downstream)
        row["status"] = derive_status(row, handler_body)
        row["headers"] = derive_headers(handler_body)
        row["pagination"] = derive_pagination(row, handler_body)
        row["timeout"] = derive_timeout(handler_body, evidence, consts)
        row["transaction"] = derive_transaction(handler_path, handler_body, downstream)
        row["errors"] = derive_errors(row, handler_body, evidence)
        row["side_effects"] = derive_side_effects(row["method"], handler_body, downstream, consts)
        for column in COLUMNS:
            if row[column].startswith("UNKNOWN"):
                unknowns[column].append(f"{row['method']} {row['path']}")
        row.pop("_p1", None)

    if len(rows) != 403:
        raise SystemExit(f"expected 403 rows, found {len(rows)}")
    keys = {(row["method"], row["path"]) for row in rows}
    if len(keys) != len(rows):
        raise SystemExit("duplicate (method, path) keys in endpoint matrix")
    missing_handler = [
        f"{row['method']} {row['path']}" for row in rows
        if not handler_source(row, index)[1]
    ]

    with CSV_PATH.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=FIELDS, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)

    print(f"regenerated {len(rows)} rows in {CSV_PATH}")
    print(f"handlers without source body: {len(missing_handler)}")
    for value in missing_handler:
        print(f"  {value}")
    for column in COLUMNS:
        counts = Counter(row[column] for row in rows)
        print(f"[{column}] {len(counts)} distinct values")
        for value, count in counts.most_common():
            print(f"  {count}\t{value}")
        print(f"[{column}] UNKNOWN rows: {len(unknowns[column])}")
        for value in unknowns[column]:
            print(f"  {value}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

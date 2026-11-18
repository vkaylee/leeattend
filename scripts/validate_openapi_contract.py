#!/usr/bin/env python3
"""Validate Go-owned OpenAPI source against runtime route and SaaS cutover rules."""
from __future__ import annotations

import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError as exc:  # pragma: no cover
    raise SystemExit(f"PyYAML required: {exc}")

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "apiserver/openapi.yaml"
REGISTER = re.compile(r'registerRoute\([^,]+,\s*"([A-Z]+)\s+([^\"]+)"')
DIRECT = re.compile(r'router\.Get\("([^\"]+)",')
FORBIDDEN = ("/workspaces", "/system/tenants", "/system/roles", "db-clusters")
FORBIDDEN_TERMS = ("workspace_id", "tenant_id", "database_url", "byod", "platformadmin")


def runtime_routes() -> set[tuple[str, str]]:
    routes: set[tuple[str, str]] = set()
    for path in sorted((ROOT / "apiserver").glob("routes*.go")):
        text = path.read_text(encoding="utf-8")
        routes.update(REGISTER.findall(text))
        routes.update(("GET", route) for route in DIRECT.findall(text))
    return routes


def spec_routes(document: dict) -> set[tuple[str, str]]:
    routes = set()
    for path, methods in document.get("paths", {}).items():
        for method in methods:
            if method.lower() in {"parameters", "summary", "description"}:
                continue
            routes.add((method.upper(), path))
    return routes


def main() -> int:
    if not SPEC.is_file():
        print(f"missing OpenAPI source: {SPEC}", file=sys.stderr)
        return 1
    document = yaml.safe_load(SPEC.read_text(encoding="utf-8"))
    if not isinstance(document, dict) or document.get("openapi") not in {"3.0.3", "3.1.0"}:
        print("invalid OpenAPI version", file=sys.stderr)
        return 1
    text = SPEC.read_text(encoding="utf-8").lower()
    violations = [term for term in FORBIDDEN_TERMS if term in text]
    violations += [path for path in document.get("paths", {}) if any(item in path for item in FORBIDDEN)]
    if violations:
        print(f"forbidden contract terms: {sorted(set(violations))}", file=sys.stderr)
        return 1
    runtime = runtime_routes()
    declared = spec_routes(document)
    missing = sorted(runtime - declared)
    extra = sorted(declared - runtime)
    if missing or extra:
        if missing:
            print(f"missing runtime routes ({len(missing)}): {missing[:12]}", file=sys.stderr)
        if extra:
            print(f"extra contract routes ({len(extra)}): {extra[:12]}", file=sys.stderr)
        return 1
    print(f"valid OpenAPI source: routes={len(declared)} paths={len(document['paths'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

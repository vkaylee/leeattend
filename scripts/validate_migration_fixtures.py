#!/usr/bin/env python3
"""Validate migration fixture schema, IDs, and source references.

This is the deterministic P3 fixture preflight. Runtime HTTP execution belongs
to P4-12; this check catches malformed or untraceable fixtures first.
"""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "docs" / "migration" / "rust-go" / "fixtures"
SOURCE_REF = re.compile(r"(?:apiserver|docs)/[^;\s]+:\d+(?:-\d+)?")
REQUIRED_SECTIONS = (
    "schema_version", "metadata", "request", "identity", "setup",
    "expected_response", "expected_side_effects", "normalization_rules",
)


def main() -> int:
    paths = sorted(
        path for path in FIXTURES.rglob("*.json") if path.name != "fixture.schema.json"
    )
    if not paths:
        raise SystemExit("no fixture files found")
    ids: set[str] = set()
    for path in paths:
        data = json.loads(path.read_text(encoding="utf-8"))
        for section in REQUIRED_SECTIONS:
            if section not in data:
                raise SystemExit(f"missing section {section!r} in {path}")
        meta = data["metadata"]
        for field in ("id", "domain", "name", "description", "source_ref"):
            if field not in meta or not str(meta[field]).strip():
                raise SystemExit(f"missing metadata field {field!r} in {path}")
        fixture_id = meta["id"]
        if fixture_id in ids:
            raise SystemExit(f"duplicate fixture id: {fixture_id}")
        ids.add(fixture_id)
        refs = SOURCE_REF.findall(meta["source_ref"])
        if not refs:
            raise SystemExit(f"missing source_ref citation: {path}")
    print(f"PASS: {len(paths)} fixtures validated; {len(ids)} unique IDs; all sections present")
    return 0
if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Fail-closed OpenAPI artifact validation for migration P4-11."""
from __future__ import annotations

import hashlib
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: validate-openapi-contract.py ARTIFACT EXPECTED_SHA256", file=sys.stderr)
        return 2
    artifact = Path(sys.argv[1])
    expected = sys.argv[2].lower()
    if not artifact.is_file():
        print(f"FAIL: missing OpenAPI artifact: {artifact}", file=sys.stderr)
        return 1
    actual = hashlib.sha256(artifact.read_bytes()).hexdigest()
    if actual != expected:
        print(f"FAIL: OpenAPI drift: expected {expected}, got {actual}", file=sys.stderr)
        return 1
    print(f"PASS: OpenAPI artifact matches frozen SHA-256: {actual}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

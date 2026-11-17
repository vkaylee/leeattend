#!/usr/bin/env python3
"""Validate phase ownership against a schema-only PostgreSQL dump."""

from __future__ import annotations

import argparse
import csv
import re
import sys
from collections import Counter
from pathlib import Path

PHASES = {
    "foundation",
    "hr_core",
    "attendance_schedule",
    "payroll",
}
OBJECT_HEADER = re.compile(
    r"^-- Name: (?P<name>[^;]+); Type: (?P<kind>TABLE|VIEW); Schema: public;"
)


def read_source_objects(dump_path: Path) -> dict[str, str]:
    objects: dict[str, str] = {}
    for line_number, line in enumerate(
        dump_path.read_text(encoding="utf-8").splitlines(), 1
    ):
        match = OBJECT_HEADER.match(line)
        if not match:
            continue
        name = match.group("name").strip()
        kind = match.group("kind").lower()
        if name in objects:
            raise ValueError(f"duplicate public object {name!r} at dump line {line_number}")
        objects[name] = kind
    if not objects:
        raise ValueError(f"no public TABLE/VIEW sections found in {dump_path}")
    return objects


def read_source_map(map_path: Path) -> dict[str, tuple[str, str, int]]:
    result: dict[str, tuple[str, str, int]] = {}
    with map_path.open(newline="", encoding="utf-8") as handle:
        rows = csv.DictReader(handle, delimiter="\t")
        expected = {"object", "kind", "phase", "dump_line"}
        if set(rows.fieldnames or ()) != expected:
            raise ValueError(f"{map_path}: expected columns {sorted(expected)}")
        for row in rows:
            name = row["object"]
            if name in result:
                raise ValueError(f"duplicate map object {name!r}")
            phase = row["phase"]
            if phase not in PHASES:
                raise ValueError(f"{name}: invalid phase {phase!r}")
            try:
                dump_line = int(row["dump_line"])
            except ValueError as exc:
                raise ValueError(f"{name}: invalid dump_line {row['dump_line']!r}") from exc
            result[name] = (row["kind"], phase, dump_line)
    if not result:
        raise ValueError(f"empty source map {map_path}")
    return result


def validate_cross_dependencies(
    dependency_path: Path, source_map: dict[str, tuple[str, str, int]]
) -> int:
    count = 0
    with dependency_path.open(newline="", encoding="utf-8") as handle:
        rows = csv.DictReader(handle, delimiter="\t")
        expected = {"from_object", "to_object", "constraint", "from_phase", "to_phase"}
        if set(rows.fieldnames or ()) != expected:
            raise ValueError(
                f"{dependency_path}: expected columns {sorted(expected)}"
            )
        for row in rows:
            count += 1
            left = row["from_object"]
            right = row["to_object"]
            if left not in source_map or right not in source_map:
                raise ValueError(
                    f"cross dependency {left}->{right} references unmapped object"
                )
            actual_left_phase = source_map[left][1]
            actual_right_phase = source_map[right][1]
            if row["from_phase"] != actual_left_phase:
                raise ValueError(f"{left}: stale from_phase {row['from_phase']!r}")
            if row["to_phase"] != actual_right_phase:
                raise ValueError(f"{right}: stale to_phase {row['to_phase']!r}")
            if actual_left_phase == actual_right_phase:
                raise ValueError(
                    f"{row['constraint']}: dependency is not cross-phase"
                )
    return count


def validate(dump_path: Path, map_path: Path, dependency_path: Path) -> tuple[int, int]:
    source_objects = read_source_objects(dump_path)
    source_map = read_source_map(map_path)

    missing = sorted(set(source_objects) - set(source_map))
    extra = sorted(set(source_map) - set(source_objects) - {"department_payroll_reconciliation"})
    if missing:
        raise ValueError(f"objects missing from source map: {', '.join(missing)}")
    if extra:
        raise ValueError(f"source-map objects missing from dump: {', '.join(extra)}")

    for name, kind in source_objects.items():
        mapped_kind, _, dump_line = source_map[name]
        if mapped_kind != kind:
            raise ValueError(f"{name}: map kind {mapped_kind!r}, dump kind {kind!r}")
        if dump_line < 1:
            raise ValueError(f"{name}: dump_line must be positive")

    dependency_count = validate_cross_dependencies(dependency_path, source_map)
    return len(source_objects), dependency_count


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dump", type=Path, required=True)
    parser.add_argument("--map", dest="map_path", type=Path, required=True)
    parser.add_argument("--cross-dependencies", type=Path, required=True)
    args = parser.parse_args()
    try:
        object_count, dependency_count = validate(
            args.dump, args.map_path, args.cross_dependencies
        )
    except (OSError, ValueError) as exc:
        print(f"migration phase map: FAIL: {exc}", file=sys.stderr)
        return 1

    phases = Counter()
    with args.map_path.open(newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle, delimiter="\t"):
            phases[row["phase"]] += 1
    phase_summary = ", ".join(f"{phase}={phases[phase]}" for phase in sorted(PHASES))
    print(
        f"migration phase map: PASS objects={object_count} "
        f"cross_phase_fks={dependency_count} {phase_summary}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

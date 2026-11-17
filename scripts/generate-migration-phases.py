#!/usr/bin/env python3
"""Generate fresh-install phase SQL from a schema-only PostgreSQL dump.

Output is deterministic and intentionally excludes seed data. Validate the source
map before running this generator. Generated files are review artifacts until
catalog comparison and integration verification pass.
"""

from __future__ import annotations

import argparse
import csv
import re
from collections import defaultdict
from pathlib import Path

PHASE_ORDER = {
    "foundation": 10,
    "hr_core": 20,
    "attendance_schedule": 30,
    "payroll": 40,
}
PHASE_MARKERS = {
    "foundation": "fresh_foundation_v1",
    "hr_core": "fresh_hr_core_v1",
    "attendance_schedule": "fresh_attendance_schedule_v1",
    "payroll": "fresh_payroll_v1",
}
HEADER = re.compile(
    r"^-- Name: (?P<name>[^;]+); Type: (?P<kind>[^;]+); Schema: (?P<schema>[^;]+);"
)

PREAMBLE = """SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
"""


def read_map(path: Path) -> dict[str, str]:
    with path.open(newline="", encoding="utf-8") as handle:
        rows = csv.DictReader(handle, delimiter="\t")
        result = {}
        for row in rows:
            result[row["object"]] = row["phase"]
    return result


def read_sections(path: Path) -> list[tuple[str, str, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    starts: list[tuple[int, str, str, str]] = []
    for number, line in enumerate(lines):
        match = HEADER.match(line)
        if match:
            starts.append(
                (
                    number,
                    match.group("name").strip(),
                    match.group("kind").strip(),
                    match.group("schema").strip(),
                )
            )
    sections = []
    for index, (start, name, kind, schema) in enumerate(starts):
        end = starts[index + 1][0] if index + 1 < len(starts) else len(lines)
        body_lines = [
            line
            for line in lines[start:end]
            if not line.startswith("\\restrict ")
            and not line.startswith("\\unrestrict")
        ]
        body = "\n".join(body_lines).strip()
        sections.append((name, kind, schema, body))
    return sections


def object_name(name: str, kind: str) -> str | None:
    if kind in {"TABLE", "VIEW", "INDEX", "INDEX ATTACH", "TABLE ATTACH", "TRIGGER", "CONSTRAINT", "FK CONSTRAINT"}:
        return name.split()[0]
    if kind == "COMMENT":
        parts = name.split()
        if len(parts) >= 2 and parts[0] in {"TABLE", "COLUMN"}:
            return parts[1].split(".")[0]
        if parts and parts[0] == "EXTENSION":
            return parts[1] if len(parts) > 1 else None
        return None
    return None


def section_phase(
    name: str, kind: str, body: str, source_map: dict[str, str]
) -> str:
    if kind in {"EXTENSION", "TYPE", "FUNCTION", "SCHEMA"}:
        return "foundation"
    associated = object_name(name, kind)
    if kind == "INDEX":
        table_match = re.search(r" ON (?:ONLY )?public\.([a-zA-Z0-9_]+)", body)
        if table_match:
            associated = table_match.group(1)
    elif kind == "INDEX ATTACH":
        partition_match = re.search(
            r"ATTACH PARTITION public\.([a-zA-Z0-9_]+)", body
        )
        if partition_match:
            partition_index = partition_match.group(1)
            candidates = [
                table
                for table in source_map
                if partition_index.startswith(table + "_")
            ]
            if candidates:
                associated = max(candidates, key=len)
    if kind == "FK CONSTRAINT":
        source = source_map[associated]
        target_match = re.search(r"REFERENCES public\.([a-zA-Z0-9_]+)", body)
        if target_match and target_match.group(1) in source_map:
            target = source_map[target_match.group(1)]
            return max((source, target), key=lambda phase: PHASE_ORDER[phase])
        return source
    if associated in source_map:
        return source_map[associated]
    return "foundation"


def generate_down_sql(
    sections: list[tuple[str, str, str, str]],
    source_map: dict[str, str],
    output: Path,
) -> None:
    phase_tables: dict[str, list[str]] = defaultdict(list)
    cross_constraints: dict[str, set[tuple[str, str]]] = defaultdict(set)
    for name, kind, _, body in sections:
        associated = object_name(name, kind)
        if kind == "TABLE" and associated in source_map:
            phase_tables[source_map[associated]].append(associated)
        if kind != "FK CONSTRAINT" or associated not in source_map:
            continue
        target_match = re.search(r"REFERENCES public\\.([a-zA-Z0-9_]+)", body)
        if not target_match or target_match.group(1) not in source_map:
            continue
        source_phase = source_map[associated]
        target_phase = source_map[target_match.group(1)]
        if source_phase == target_phase:
            continue
        constraint_name = name.split()[-1]
        cross_constraints[source_phase].add((associated, constraint_name))
        cross_constraints[target_phase].add((associated, constraint_name))

    for phase, marker in PHASE_MARKERS.items():
        if phase == "runtime_seeds":
            continue
        chunks = [
            "BEGIN;",
            f"-- Fresh-install-only rollback for {phase}; run phases in reverse order.",
            "-- This is destructive and must not run against production data.",
        ]
        if phase == "payroll":
            chunks.append("DROP VIEW IF EXISTS public.department_payroll_reconciliation;")
        for table, constraint in sorted(cross_constraints[phase]):
            chunks.append(
                f"ALTER TABLE IF EXISTS public.{table} "
                f"DROP CONSTRAINT IF EXISTS {constraint};"
            )
        for table in reversed(phase_tables[phase]):
            chunks.append(f"DROP TABLE IF EXISTS public.{table} CASCADE;")
        if phase != "foundation":
            chunks.append(
                "DELETE FROM public.schema_migrations "
                f"WHERE version = '{marker}';"
            )
        else:
            chunks.extend(["DROP SCHEMA public CASCADE;", "CREATE SCHEMA public;"])
        chunks.append("COMMIT;")
        (output / f"{phase}.down.sql").write_text(
            "\n\n".join(chunks) + "\n", encoding="utf-8"
        )


def generate(dump: Path, source_map_path: Path, output: Path) -> None:
    source_map = read_map(source_map_path)
    sections = read_sections(dump)
    grouped: dict[str, list[tuple[str, str, str]]] = defaultdict(list)
    for name, kind, schema, body in sections:
        if kind in {"TABLE DATA", "SCHEMA"} or schema not in {"public", "-"}:
            # Non-public retirement snapshots are upgrade-only recovery state.
            # Fresh install encodes retired salary tables as absent.
            continue
        phase = section_phase(name, kind, body, source_map)
        if phase not in PHASE_MARKERS:
            continue
        grouped[phase].append((kind, name, body))
    output.mkdir(parents=True, exist_ok=True)
    for phase, marker in PHASE_MARKERS.items():
        if phase == "runtime_seeds":
            continue
        sections_for_phase = grouped[phase]
        # Reorder dump objects into executable dependency classes while retaining
        # original order inside each class.
        rank = {
            "EXTENSION": 10,
            "TYPE": 20,
            "FUNCTION": 30,
            "TABLE": 40,
            "TABLE ATTACH": 50,
            "CONSTRAINT": 60,
            "INDEX": 70,
            "INDEX ATTACH": 80,
            "TRIGGER": 90,
            "FK CONSTRAINT": 100,
            "VIEW": 110,
            "COMMENT": 120,
        }
        sections_for_phase.sort(key=lambda item: (rank.get(item[0], 999), item[1]))
        chunks = ["BEGIN;", PREAMBLE, f"-- Fresh install phase: {phase}"]
        chunks.extend(body for _, _, body in sections_for_phase)
        chunks.append(
            f"INSERT INTO public.schema_migrations(version) VALUES ('{marker}');"
        )
        chunks.append("COMMIT;")
        (output / f"{phase}.up.sql").write_text(
            "\n\n".join(chunks) + "\n", encoding="utf-8"
        )
    generate_down_sql(sections, source_map, output)
    (output / "runtime_seeds.policy.md").write_text(
        "# Runtime seeds\n\n"
        "Generated DDL owns no seed rows. Run idempotent feature/config/metric "
        "seeds after schema phases; run KPI formula seed after standalone admin "
        "bootstrap.\n",
        encoding="utf-8",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dump", type=Path, required=True)
    parser.add_argument("--map", dest="source_map", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    generate(args.dump, args.source_map, args.output)
    print(f"generated migration phase preview: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

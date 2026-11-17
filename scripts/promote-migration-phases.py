#!/usr/bin/env python3
"""Promote a verified disposable phase preview into tracked migration files."""

from __future__ import annotations

import argparse
import shutil
from pathlib import Path

PHASES = ("foundation", "hr_core", "attendance_schedule", "payroll")


def promote(preview: Path, target: Path) -> None:
    target.mkdir(parents=True, exist_ok=False)
    for phase in PHASES:
        source_up = preview / f"{phase}.up.sql"
        source_down = preview / f"{phase}.down.sql"
        if not source_up.is_file() or not source_down.is_file():
            raise FileNotFoundError(f"missing preview files for {phase}")
        phase_dir = target / phase
        phase_dir.mkdir()
        shutil.copyfile(source_up, phase_dir / "up.sql")
        shutil.copyfile(source_down, phase_dir / "down.sql")
    shutil.copyfile(preview / "runtime_seeds.policy.md", target / "runtime_seeds.policy.md")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--preview", type=Path, required=True)
    parser.add_argument("--target", type=Path, required=True)
    args = parser.parse_args()
    promote(args.preview, args.target)
    print(f"promoted migration phase preview: {args.target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

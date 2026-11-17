"""Tests for migration phase source-map validation."""

from __future__ import annotations

import importlib.util
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/validate-migration-phase-map.py"


spec = importlib.util.spec_from_file_location("migration_phase_map", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def write_inputs(tmp_path: Path) -> tuple[Path, Path, Path]:
    dump = tmp_path / "oracle.sql"
    source_map = tmp_path / "source-map.tsv"
    dependencies = tmp_path / "dependencies.tsv"
    dump.write_text(
        "-- Name: users; Type: TABLE; Schema: public; Owner: -\n"
        "-- Name: report; Type: VIEW; Schema: public; Owner: -\n",
        encoding="utf-8",
    )
    source_map.write_text(
        "object\tkind\tphase\tdump_line\n"
        "users\ttable\tfoundation\t1\n"
        "report\tview\tpayroll\t2\n",
        encoding="utf-8",
    )
    dependencies.write_text(
        "from_object\tto_object\tconstraint\tfrom_phase\tto_phase\n"
        "report\tusers\treport_users_fkey\tpayroll\tfoundation\n",
        encoding="utf-8",
    )
    return dump, source_map, dependencies


def test_validate_accepts_complete_map(tmp_path: Path) -> None:
    dump, source_map, dependencies = write_inputs(tmp_path)

    assert module.validate(dump, source_map, dependencies) == (2, 1)


def test_validate_rejects_unmapped_object(tmp_path: Path) -> None:
    dump, source_map, dependencies = write_inputs(tmp_path)
    source_map.write_text(
        "object\tkind\tphase\tdump_line\nusers\ttable\tfoundation\t1\n",
        encoding="utf-8",
    )

    with pytest.raises(ValueError, match="missing from source map"):
        module.validate(dump, source_map, dependencies)


def test_validate_rejects_stale_cross_phase_metadata(tmp_path: Path) -> None:
    dump, source_map, dependencies = write_inputs(tmp_path)
    dependencies.write_text(
        "from_object\tto_object\tconstraint\tfrom_phase\tto_phase\n"
        "report\tusers\treport_users_fkey\tfoundation\tfoundation\n",
        encoding="utf-8",
    )

    with pytest.raises(ValueError, match="stale from_phase"):
        module.validate(dump, source_map, dependencies)


def test_validate_rejects_same_phase_dependency_marked_cross_phase(tmp_path: Path) -> None:
    dump, source_map, dependencies = write_inputs(tmp_path)
    dependencies.write_text(
        "from_object\tto_object\tconstraint\tfrom_phase\tto_phase\n"
        "report\treport\treport_self_fkey\tpayroll\tpayroll\n",
        encoding="utf-8",
    )

    with pytest.raises(ValueError, match="not cross-phase"):
        module.validate(dump, source_map, dependencies)

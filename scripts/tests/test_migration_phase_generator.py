"""Tests for deterministic fresh-install migration phase generation."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/generate-migration-phases.py"
spec = importlib.util.spec_from_file_location("migration_phase_generator", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def test_generate_assigns_cross_phase_fk_to_later_phase(tmp_path: Path) -> None:
    dump = tmp_path / "oracle.sql"
    source_map = tmp_path / "source-map.tsv"
    output = tmp_path / "output"
    dump.write_text(
        "-- Name: salary_base_retirement_backup; Type: SCHEMA; Schema: -; Owner: -\n"
        "CREATE SCHEMA salary_base_retirement_backup;\n\n"
        "-- Name: users; Type: TABLE; Schema: public; Owner: -\n"
        "CREATE TABLE public.users (id uuid NOT NULL);\n\n"
        "-- Name: employees; Type: TABLE; Schema: public; Owner: -\n"
        "CREATE TABLE public.employees (id uuid NOT NULL, user_id uuid);\n\n"
        "-- Name: employees employees_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -\n"
        "ALTER TABLE ONLY public.employees\n"
        "    ADD CONSTRAINT employees_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);\n",
        encoding="utf-8",
    )
    source_map.write_text(
        "object\tkind\tphase\tdump_line\n"
        "users\ttable\tfoundation\t1\n"
        "employees\ttable\thr_core\t3\n",
        encoding="utf-8",
    )

    module.generate(dump, source_map, output)

    foundation = (output / "foundation.up.sql").read_text(encoding="utf-8")
    hr_core = (output / "hr_core.up.sql").read_text(encoding="utf-8")
    assert "CREATE TABLE public.users" in foundation
    assert "salary_base_retirement_backup" not in foundation
    assert "employees_user_id_fkey" not in foundation
    assert "CREATE TABLE public.employees" in hr_core
    assert "employees_user_id_fkey" in hr_core
    assert "fresh_foundation_v1" in foundation
    assert "fresh_hr_core_v1" in hr_core


def test_generate_is_deterministic(tmp_path: Path) -> None:
    dump = tmp_path / "oracle.sql"
    source_map = tmp_path / "source-map.tsv"
    first = tmp_path / "first"
    second = tmp_path / "second"
    dump.write_text(
        "-- Name: users; Type: TABLE; Schema: public; Owner: -\n"
        "CREATE TABLE public.users (id uuid NOT NULL);\n",
        encoding="utf-8",
    )
    source_map.write_text(
        "object\tkind\tphase\tdump_line\nusers\ttable\tfoundation\t1\n",
        encoding="utf-8",
    )

    module.generate(dump, source_map, first)
    module.generate(dump, source_map, second)

    assert (first / "foundation.up.sql").read_bytes() == (
        second / "foundation.up.sql"
    ).read_bytes()


def test_generate_writes_transactional_down_files(tmp_path: Path) -> None:
    dump = tmp_path / "oracle.sql"
    source_map = tmp_path / "source-map.tsv"
    output = tmp_path / "output"
    dump.write_text(
        "-- Name: users; Type: TABLE; Schema: public; Owner: -\n"
        "CREATE TABLE public.users (id uuid NOT NULL);\n",
        encoding="utf-8",
    )
    source_map.write_text(
        "object\tkind\tphase\tdump_line\nusers\ttable\tfoundation\t1\n",
        encoding="utf-8",
    )

    module.generate(dump, source_map, output)

    foundation_down = (output / "foundation.down.sql").read_text(encoding="utf-8")
    assert foundation_down.startswith("BEGIN;")
    assert "DROP SCHEMA public CASCADE;" in foundation_down
    assert foundation_down.rstrip().endswith("COMMIT;")
    assert "fresh_foundation_v1" not in foundation_down

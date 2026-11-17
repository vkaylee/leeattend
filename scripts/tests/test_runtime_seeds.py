"""Contract tests for fresh-install runtime seed SQL."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SEEDS = ROOT / "docs/plans/runtime-seeds.sql"
ROLLBACK = ROOT / "docs/plans/runtime-seeds.down.sql"


def test_runtime_seeds_are_idempotent_and_not_user_dependent() -> None:
    sql = SEEDS.read_text(encoding="utf-8")
    assert "ON CONFLICT (flag_key) DO NOTHING" in sql
    assert "ON CONFLICT (code) DO NOTHING" in sql
    assert "ON CONFLICT (scope_type, scope_id, config_key, effective_from) DO NOTHING" in sql
    assert "SELECT id FROM users" not in sql
    assert "workspace" not in sql.lower()
    assert "CURRENT_DATE" not in sql
    assert "fresh_runtime_seeds_v1" in sql


def test_runtime_seed_rollback_is_bounded() -> None:
    sql = ROLLBACK.read_text(encoding="utf-8")
    assert "BEGIN;" in sql and sql.rstrip().endswith("COMMIT;")
    assert "DROP TABLE" not in sql
    assert "workspace" not in sql.lower()
    assert "fresh_runtime_seeds_v1" in sql

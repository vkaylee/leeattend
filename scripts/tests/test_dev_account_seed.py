"""Contract tests for development account role reconciliation."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SEED = ROOT / "scripts/seed_dev_accounts.sql"


def test_dev_seed_revokes_permissions_removed_from_role_catalog() -> None:
    sql = SEED.read_text(encoding="utf-8")
    assert "DELETE FROM role_permissions rp" in sql
    assert "USING roles r" in sql
    assert "r.name IN ('shift_manager', 'team_lead', 'employee')" in sql
    assert "rp.permission_name = 'employees:read'" in sql
    assert "r.name = 'employee'" in sql
    assert "rp.permission_name = 'attendance:adjustments:submit'" in sql


def test_dev_seed_keeps_role_upsert_non_destructive() -> None:
    sql = SEED.read_text(encoding="utf-8")
    assert "ON CONFLICT (name) DO UPDATE" in sql
    assert "DELETE FROM users" not in sql
    assert "DELETE FROM employees" not in sql
    assert "DELETE FROM role_permissions" in sql

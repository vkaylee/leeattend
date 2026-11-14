"""Contract tests for the shared UI viewport manifest."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "config/ui-viewports.json"


def load_manifest() -> dict:
    return json.loads(MANIFEST.read_text(encoding="utf-8"))


def test_manifest_has_unique_supported_viewports_and_valid_bounds() -> None:
    manifest = load_manifest()
    supported = manifest["supported"]
    names = [viewport["name"] for viewport in supported]
    sizes = [(viewport["width"], viewport["height"]) for viewport in supported]

    assert len(names) == len(set(names))
    assert len(sizes) == len(set(sizes))
    assert manifest["minWidth"] == min(viewport["width"] for viewport in supported)
    assert manifest["maxWidth"] == max(viewport["width"] for viewport in supported)
    assert manifest["baseline"] in names
    assert all(viewport["width"] >= manifest["minWidth"] for viewport in supported)
    assert all(viewport["width"] <= manifest["maxWidth"] for viewport in supported)

    tiers = manifest["tiers"]
    tier_names = [name for tier in tiers.values() for name in tier]
    assert len(tier_names) == len(set(tier_names))
    assert set(tier_names) == set(names)
    assert set(tiers["required"]) == {"desktop-large", "desktop", "laptop", "laptop-small"}
    assert set(tiers["secondary"]) == {"tablet-landscape", "tablet-portrait"}
    assert set(tiers["smoke"]) == {"mobile-large", "mobile", "mobile-small"}


def test_manifest_includes_required_responsive_boundaries() -> None:
    manifest = load_manifest()
    supported = {(viewport["width"], viewport["height"]) for viewport in manifest["supported"]}

    assert (1920, 1080) in supported
    assert (1024, 768) in supported
    assert (768, 1024) in supported
    assert (390, 844) in supported
    assert (360, 800) in supported

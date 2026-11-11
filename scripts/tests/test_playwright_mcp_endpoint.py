"""Behavior tests for the Playwright MCP endpoint boundary."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
WRAPPER = ROOT / "scripts/playwright-mcp-role.sh"


def test_mcp_config_exposes_one_least_privilege_playwright_server() -> None:
    config = json.loads((ROOT / ".mcp.json").read_text())
    servers = [key for key in config["mcpServers"] if key.startswith("playwright")]

    assert servers == ["playwright"]
    assert config["mcpServers"]["playwright"]["args"][0] == "employee"


@pytest.mark.parametrize("extra_args", [[], ["--idle-timeout=1500"]])
def test_wrapper_forwards_configured_endpoint_to_cli(tmp_path: Path, extra_args: list[str]) -> None:
    compose = tmp_path / "compose"
    docker = tmp_path / "docker"
    args_file = tmp_path / "compose-args"
    compose.write_text('#!/bin/sh\nprintf "%s\\n" "$@" > "$COMPOSE_ARGS_FILE"\n')
    docker.write_text('#!/bin/sh\nexit 0\n')
    compose.chmod(0o755)
    docker.chmod(0o755)

    result = subprocess.run(
        ["bash", str(WRAPPER), "--supervised", "employee", "--endpoint", "http://example.com", *extra_args],
        cwd=ROOT,
        env={
            **os.environ,
            "DOCKER_COMPOSE_CMD": str(compose),
            "COMPOSE_ARGS_FILE": str(args_file),
        },
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    args = args_file.read_text().splitlines()
    assert "--allowed-origins=http://example.com" in args
    assert "--idle-timeout=300000" in args
    if extra_args:
        assert args[-1] == extra_args[0]
        assert args.index("--idle-timeout=300000") < len(args) - 1

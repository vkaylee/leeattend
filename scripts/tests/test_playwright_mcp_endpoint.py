"""Behavior tests for the Playwright MCP endpoint boundary."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
WRAPPER = ROOT / "scripts/playwright-mcp-role.sh"


def test_mcp_config_exposes_isolated_server_per_role() -> None:
    config = json.loads((ROOT / ".mcp.json").read_text())
    servers = [key for key in config["mcpServers"] if key.startswith("playwright-")]
    roles = ["employee", "team-lead", "shift-manager", "hr-manager", "admin"]

    assert servers == [f"playwright-{role}" for role in roles]
    for role in roles:
        assert config["mcpServers"][f"playwright-{role}"]["args"] == [role, "--endpoint", "http://caddy"]


def test_wrapper_allows_disposable_compose_project_and_file(tmp_path: Path) -> None:
    compose = tmp_path / "compose"
    args_file = tmp_path / "compose-args"
    compose.write_text('#!/bin/sh\nprintf "%s\\n" "$@" > "$COMPOSE_ARGS_FILE"\n')
    compose.chmod(0o755)
    (tmp_path / "disposable-compose.yml").write_text("services: {}\n")

    result = subprocess.run(
        ["bash", str(WRAPPER), "--supervised", "employee", "--endpoint", "http://caddy"],
        cwd=ROOT,
        env={
            **os.environ,
            "DOCKER_COMPOSE_CMD": str(compose),
            "COMPOSE_ARGS_FILE": str(args_file),
            "PLAYWRIGHT_COMPOSE_PROJECT": "leeattend-tenure-ui",
            "PLAYWRIGHT_COMPOSE_FILE": str(tmp_path / "disposable-compose.yml"),
        },
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    args = args_file.read_text().splitlines()
    assert "leeattend-tenure-ui" in args
    assert str(tmp_path / "disposable-compose.yml") in args


def test_wrapper_selects_role_from_environment(tmp_path: Path) -> None:
    compose = tmp_path / "compose"
    args_file = tmp_path / "compose-args"
    compose.write_text('#!/bin/sh\nprintf "%s\\n" "$@" > "$COMPOSE_ARGS_FILE"\n')
    compose.chmod(0o755)

    result = subprocess.run(
        ["bash", str(WRAPPER), "--supervised", "--endpoint", "http://example.com"],
        cwd=ROOT,
        env={
            **os.environ,
            "DOCKER_COMPOSE_CMD": str(compose),
            "COMPOSE_ARGS_FILE": str(args_file),
            "PLAYWRIGHT_ROLE": "hr-manager",
        },
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    assert args_file.exists()


def test_wrapper_requires_explicit_build_for_missing_image(tmp_path: Path) -> None:
    compose = tmp_path / "compose"
    docker = tmp_path / "docker"
    compose_args = tmp_path / "compose-args"
    compose.write_text('#!/bin/sh\nprintf "%s\\n" "$@" > "$COMPOSE_ARGS_FILE"\n')
    docker.write_text('#!/bin/sh\nexit 1\n')
    compose.chmod(0o755)
    docker.chmod(0o755)

    result = subprocess.run(
        ["bash", str(WRAPPER), "--supervised", "employee", "--endpoint", "http://example.com"],
        cwd=ROOT,
        env={
            **os.environ,
            "PATH": f"{tmp_path}:{os.environ['PATH']}",
            "DOCKER_COMPOSE_CMD": str(compose),
            "COMPOSE_ARGS_FILE": str(compose_args),
        },
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 1
    assert "Playwright MCP image missing" in result.stderr
    assert not compose_args.exists()


def test_wrapper_builds_only_with_explicit_flag(tmp_path: Path) -> None:
    compose = tmp_path / "compose"
    docker = tmp_path / "docker"
    args_file = tmp_path / "compose-args"
    compose.write_text('#!/bin/sh\nprintf "%s\\n" "$@" >> "$COMPOSE_ARGS_FILE"\n')
    docker.write_text('#!/bin/sh\nexit 0\n')
    compose.chmod(0o755)
    docker.chmod(0o755)

    result = subprocess.run(
        ["bash", str(WRAPPER), "--build", "--supervised", "employee", "--endpoint", "http://example.com"],
        cwd=ROOT,
        env={
            **os.environ,
            "PATH": f"{tmp_path}:{os.environ['PATH']}",
            "DOCKER_COMPOSE_CMD": str(compose),
            "COMPOSE_ARGS_FILE": str(args_file),
        },
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0, result.stderr
    args = args_file.read_text().splitlines()
    assert args.count("build") == 1
    assert "playwright-mcp" in args


def test_wrapper_passes_session_idle_timeout_to_watchdog(tmp_path: Path) -> None:
    compose = tmp_path / "compose"
    compose.write_text('#!/bin/sh\nexit 0\n')
    compose.chmod(0o755)

    result = subprocess.run(
        ["bash", str(WRAPPER), "employee", "--endpoint", "http://example.com"],
        cwd=ROOT,
        env={
            **os.environ,
            "DOCKER_COMPOSE_CMD": str(compose),
            "PLAYWRIGHT_MCP_SESSION_IDLE_TIMEOUT_SECS": "0",
        },
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 2
    assert "--session-idle-seconds" in result.stderr


def test_wrapper_rejects_unknown_environment_role(tmp_path: Path) -> None:
    compose = tmp_path / "compose"
    compose.write_text('#!/bin/sh\nexit 0\n')
    compose.chmod(0o755)

    result = subprocess.run(
        ["bash", str(WRAPPER), "--supervised", "--endpoint", "http://example.com"],
        cwd=ROOT,
        env={
            **os.environ,
            "DOCKER_COMPOSE_CMD": str(compose),
            "PLAYWRIGHT_ROLE": "owner",
        },
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 2
    assert "Unsupported Playwright role: owner" in result.stderr


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
    assert "--allowed-hosts=example.com" in args
    assert "--allowed-origins=http://example.com" in args
    assert "--quiet" in args
    assert "--idle-timeout=0" in args
    if extra_args:
        assert args[-1] == extra_args[0]
        assert args.index("--idle-timeout=0") < len(args) - 1

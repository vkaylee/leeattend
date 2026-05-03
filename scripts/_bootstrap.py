"""LeeAttend Internal Bootstrap — Python equivalent of _bootstrap.sh.

Centralizes container engine detection, compose tool selection, profile
injection, and environment variable setup. Used by the Python Orchestrator
and all other infrastructure modules.
"""

import os
import shutil
from pathlib import Path
from typing import Optional

PROJECT_ROOT = Path(__file__).resolve().parent.parent
SCRIPTS_DIR = PROJECT_ROOT / "scripts"


def _which(cmd: str) -> Optional[str]:
    """Find executable in PATH (delegates to shutil.which)."""
    return shutil.which(cmd)


def detect_engine() -> str:
    """Detect container engine: podman preferred, fallback to docker."""
    if _which("podman"):
        return "podman"
    if _which("docker"):
        return "docker"
    return "podman"


def detect_compose_cmd() -> list[str]:
    """Detect compose tool and return as argument list.

    Returns list for safe subprocess usage (no shell=True).
    """
    if _which("podman"):
        return ["podman-compose"]
    if _which("docker"):
        return ["docker", "compose"]
    return ["podman-compose"]


PROFILES: dict[str, list[str]] = {
    "api": [
        "--profile",
        "lint-api",
        "--profile",
        "unit-api",
        "--profile",
        "infra-db-init",
        "--profile",
        "infra-db",
    ],
    "integration": [
        "--profile",
        "lint-api",
        "--profile",
        "unit-api",
        "--profile",
        "int-api",
        "--profile",
        "infra-db",
        "--profile",
        "infra-db-init",
        "--profile",
        "infra-redis",
    ],
    "web": [
        "--profile",
        "lint-web",
        "--profile",
        "unit-web",
        "--profile",
        "e2e-web",
    ],
    "all": [
        "--profile",
        "lint-api",
        "--profile",
        "unit-api",
        "--profile",
        "int-api",
        "--profile",
        "lint-web",
        "--profile",
        "unit-web",
        "--profile",
        "e2e-web",
        "--profile",
        "infra-db",
        "--profile",
        "infra-db-init",
        "--profile",
        "infra-redis",
    ],
}

LIFECYCLE_PROFILES: dict[str, list[str]] = {
    "api": ["--profile", "backend"],
    "integration": ["--profile", "backend"],
    "web": ["--profile", "frontend"],
    "all": ["--profile", "backend", "--profile", "frontend"],
}


def resolve_profiles(mode: str) -> list[str]:
    """Translate a mode string into docker-compose --profile flags."""
    custom = os.environ.get("PROFILES", "")
    if custom:
        return custom.split()
    return PROFILES.get(mode, PROFILES["all"])


def resolve_lifecycle_profiles(mode: str) -> list[str]:
    """Translate a mode into lifecycle (up/down) profile flags."""
    return LIFECYCLE_PROFILES.get(mode, LIFECYCLE_PROFILES["all"])


def bootstrap_env(mode: str = "all") -> dict[str, str]:
    """Build the full environment dict for the given mode.

    Returns a dict suitable for os.environ.copy().update().
    """
    engine = detect_engine()
    compose_cmd = detect_compose_cmd()
    project_name = os.environ.get("COMPOSE_PROJECT_NAME", "leeattend-test")

    profiles = resolve_profiles(mode)

    compose_base = [
        "-p",
        project_name,
        "-f",
        str(PROJECT_ROOT / ".compose" / "docker-compose.test.yml"),
    ]
    compose_full = compose_cmd + compose_base + profiles
    compose_str = " ".join(compose_full)

    return {
        "CONTAINER_ENGINE": engine,
        "COMPOSE_PROJECT_NAME": project_name,
        "PODMAN_COMPOSE_PROJECT_NAME": project_name,
        "DOCKER_COMPOSE_CMD": compose_str,
        "DOCKER_COMPOSE_BASE": " ".join(compose_cmd + compose_base),
        "DOCKER_COMPOSE_TOOL": " ".join(compose_cmd),
        "USE_DOCKER": "true",
        "PROJECT_ROOT": str(PROJECT_ROOT),
        "MODE": mode,
    }

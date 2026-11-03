"""Test task builders for Go and web services."""

from __future__ import annotations

import shlex

from _test_utils import build_compose_exec, run_parallel_ordered


def _load_services() -> dict:
    from _bootstrap import PROJECT_ROOT
    from _devkit_config import _load_toml

    config_path = PROJECT_ROOT / "leedevkit.toml"
    if not config_path.exists():
        return {}
    return _load_toml(config_path).get("services", {})


def _resolve_web_service() -> str:
    """Return configured frontend service."""
    for name, service in _load_services().items():
        if isinstance(service, dict) and service.get("lang") in ("typescript", "javascript", "web"):
            return str(name)
    return "webdashboard"


def _has_go_service() -> bool:
    """Return whether project declares a Go module or service."""
    from _bootstrap import PROJECT_ROOT

    if (PROJECT_ROOT / "go.mod").exists():
        return True
    return any(
        isinstance(service, dict) and service.get("lang") == "go"
        for service in _load_services().values()
    )


def _has_configured_language(*languages: str) -> bool:
    """Return whether project declares one of the requested languages."""
    from _bootstrap import PROJECT_ROOT

    if "go" in languages and (PROJECT_ROOT / "go.mod").exists():
        return True
    return any(
        isinstance(service, dict) and service.get("lang") in languages
        for service in _load_services().values()
    )


def _has_web_service() -> bool:
    return _has_configured_language("typescript", "javascript", "web")


def _resolve_go_service(component_filter: str = "") -> str:
    """Return configured Go service."""
    services = _load_services()
    go_services = [
        str(name)
        for name, service in services.items()
        if isinstance(service, dict) and service.get("lang") == "go"
    ]
    if component_filter in go_services:
        return component_filter
    return go_services[0] if go_services else "apiserver"


def _resolve_go_workdir() -> str:
    """Resolve Go module path inside the test container."""
    from _bootstrap import PROJECT_ROOT

    services = _load_services()
    from _devkit_config import _load_toml

    config_path = PROJECT_ROOT / "leedevkit.toml"
    config = _load_toml(config_path) if config_path.exists() else {}
    targets = config.get("targets", {})
    service_names = targets.get("api", []) if isinstance(targets, dict) else []
    if not isinstance(service_names, list) or not service_names:
        service_names = list(services)
    for service_name in service_names:
        service = services.get(service_name, {})
        workdir = service.get("workdir") if isinstance(service, dict) else None
        if isinstance(workdir, str) and workdir.strip():
            candidate = (PROJECT_ROOT / workdir).resolve()
            if candidate.is_dir() and (candidate / "go.mod").is_file():
                return f"/workspace/{candidate.relative_to(PROJECT_ROOT).as_posix()}"
    return "/workspace"


def _safe_pattern(pattern: str) -> str:
    return shlex.quote(pattern) if pattern else ""


def _go_command(service: str, command: str, workdir: str) -> tuple[str, str, list[str]]:
    return (service, service, build_compose_exec(service, command, workdir=workdir, mode="go"))


def leedevkit_run_lint(component_filter: str = "", mode: str = "all", fix: bool = False) -> bool:
    """Run Go and web lint gates."""
    tasks: list[tuple[str, str, list[str]]] = []
    if mode in ("go", "all") and _has_go_service():
        service = _resolve_go_service(component_filter)
        workdir = _resolve_go_workdir()
        format_cmd = "gofmt -w ." if fix else 'test -z "$(gofmt -l .)"'
        tasks.extend(
            [
                ("go-format", service, build_compose_exec(service, format_cmd, workdir=workdir, mode="go")),
                ("go-vet", service, build_compose_exec(service, "go vet ./...", workdir=workdir, mode="go")),
                ("go-staticcheck", service, build_compose_exec(service, "staticcheck ./...", workdir=workdir, mode="go")),
                ("go-govulncheck", service, build_compose_exec(service, "govulncheck ./...", workdir=workdir, mode="go")),
                ("go-mod-verify", service, build_compose_exec(service, "go mod verify", workdir=workdir, mode="go")),
                (
                    "go-generated-diff",
                    service,
                    build_compose_exec(
                        service,
                        "bash -c 'if [ -f sqlc.yaml ]; then sqlc diff; fi && if [ -f oapi-codegen.yaml ]; then git diff --exit-code; fi'",
                        workdir=workdir,
                        mode="go",
                    ),
                ),
            ]
        )
    if mode in ("web", "all") and _has_web_service():
        service = _resolve_web_service()
        tasks.extend(
            [
                (f"{service}-lint", service, build_compose_exec(service, "bun run lint:fix" if fix else "bun run lint", mode="web")),
                (f"{service}-typecheck", service, build_compose_exec(service, "bun run type-check", mode="web")),
                (f"{service}-i18n", service, build_compose_exec(service, "bun run check-i18n", mode="web")),
            ]
        )
    return run_parallel_ordered("Linting", component_filter, tasks)


def leedevkit_run_unit(component_filter: str = "", mode: str = "all", test_pattern: str = "", shard_n: str = "", shard_m: str = "") -> bool:
    """Run Go and web unit tests."""
    tasks: list[tuple[str, str, list[str]]] = []
    if mode in ("go", "all") and _has_go_service():
        service = _resolve_go_service(component_filter)
        workdir = _resolve_go_workdir()
        run_flag = f" -run {_safe_pattern(test_pattern)}" if test_pattern else ""
        tasks.append(("go-unit", service, build_compose_exec(service, f"go test -race ./...{run_flag}", workdir=workdir, mode="go")))
    if mode in ("web", "all") and _has_web_service():
        service = _resolve_web_service()
        shard_flag = f" {shard_n}/{shard_m}" if shard_n and shard_m else ""
        tasks.append((service, service, build_compose_exec(service, f"bun run test -- {_safe_pattern(test_pattern)}{shard_flag} --passWithNoTests", mode="web")))
    return run_parallel_ordered("Unit Tests", component_filter, tasks)


def leedevkit_run_integration(component_filter: str = "", mode: str = "all", test_pattern: str = "") -> bool:
    """Run Go integration coverage and web E2E tests."""
    tasks: list[tuple[str, str, list[str]]] = []
    if mode in ("go", "all") and _has_go_service():
        service = _resolve_go_service(component_filter)
        workdir = _resolve_go_workdir()
        run_flag = f" -run {_safe_pattern(test_pattern)}" if test_pattern else ""
        tasks.append(("go-integration", service, build_compose_exec(service, f"go test -race ./...{run_flag}", workdir=workdir, mode="go")))
    if mode in ("web", "all") and _has_web_service():
        service = _resolve_web_service()
        pw_args = f"-g {_safe_pattern(test_pattern)}" if test_pattern else ""
        tasks.append(("playwright-e2e", service, build_compose_exec(service, f"bunx playwright test {pw_args} --workers 2", mode="e2e-web")))
    return run_parallel_ordered("Integration & E2E", component_filter, tasks)


def leedevkit_run_coverage(component_filter: str = "", mode: str = "all", unit_only: bool = False, test_pattern: str = "") -> bool:
    """Run Go and web coverage tasks."""
    tasks: list[tuple[str, str, list[str]]] = []
    if mode in ("go", "all") and _has_go_service():
        service = _resolve_go_service(component_filter)
        workdir = _resolve_go_workdir()
        run_flag = f" -run {_safe_pattern(test_pattern)}" if test_pattern else ""
        command = (
            "mkdir -p /workspace/.test_logs && "
            f"go test -coverprofile=/workspace/.test_logs/coverage-go.out ./...{run_flag} && "
            "go tool cover -func=/workspace/.test_logs/coverage-go.out"
        )
        tasks.append(("go-coverage", service, build_compose_exec(service, command, workdir=workdir, mode="go")))
    if mode in ("web", "all") and _has_web_service():
        service = _resolve_web_service()
        tasks.append((f"{service}-coverage", service, build_compose_exec(service, "bun run test:coverage", mode="web")))
    return run_parallel_ordered("Coverage", component_filter, tasks)

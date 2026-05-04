#!/usr/bin/env python3
import argparse
import atexit
import contextlib
import os
import shutil
import signal
import socket
import subprocess
import sys
import time
import typing
from datetime import datetime
from pathlib import Path
from types import FrameType

from _arg_sanitizer import sanitize
from _bootstrap import (
    PROJECT_ROOT,
    SCRIPTS_DIR,
    bootstrap_env,
    detect_engine,
    detect_compose_cmd,
)
from _lifecycle import lifecycle_down, lifecycle_up as _lifecycle_up
from _test_modules import (
    leeattend_run_lint,
    leeattend_run_unit,
    leeattend_run_integration,
    leeattend_run_coverage,
)

# LeeAttend Enterprise Orchestrator Core
# ==============================================================================


class Colors:
    GREEN = "\033[0;32m"
    RED = "\033[0;31m"
    YELLOW = "\033[1;33m"
    BLUE = "\033[0;34m"
    BOLD = "\033[1m"
    NC = "\033[0m"


def log_info(msg: str) -> None:
    print(f"{Colors.BLUE}{Colors.BOLD}ℹ️ {msg}{Colors.NC}", flush=True)


def log_success(msg: str) -> None:
    print(f"{Colors.GREEN}{Colors.BOLD}✅ {msg}{Colors.NC}", flush=True)


def log_warn(msg: str) -> None:
    print(f"{Colors.YELLOW}{Colors.BOLD}⚠️ {msg}{Colors.NC}", flush=True)


def log_error(msg: str, file: typing.TextIO = sys.stderr) -> None:
    print(f"{Colors.RED}{Colors.BOLD}❌ {msg}{Colors.NC}", file=file, flush=True)


class Orchestrator:
    def __init__(self) -> None:
        prog_name = (
            "manage.sh"
            if "manage" in sys.argv
            else ("test.sh" if "test" in sys.argv else "orchestrator")
        )
        self.parser = argparse.ArgumentParser(
            prog=prog_name,
            description="LeeAttend Enterprise Orchestrator",
            formatter_class=argparse.RawDescriptionHelpFormatter,
            epilog="Examples:\n  ./test.sh --all\n  ./manage.sh up dev\n  ./manage.sh sync:api",
        )
        # Mapping tool names to their target service in docker-compose.test.yml
        self.tool_map = {
            "npm": "webdashboard",
            "cargo": "apiserver",
            "sqlx": "apiserver",
        }
        self.setup_parser()
        self.dry_run = False
        self.env_vars: dict[str, str] = {
            "USE_DOCKER": "true",
            "COMPOSE_PROJECT_NAME": "leeattend-test",
            "PODMAN_COMPOSE_PROJECT_NAME": "leeattend-test",
            "PROJECT_ROOT": str(PROJECT_ROOT),
            "CONTAINER_ENGINE": self.engine,
            "DOCKER_COMPOSE_CMD": " ".join(self.compose_engine),
        }
        self.needs_cleanup = False
        self.active_mode = "all"
        self.start_time = time.time()
        self.register_traps()

    def register_traps(self) -> None:
        """Register signal handlers and exit hooks for cleanup."""

        atexit.register(self.cleanup)

        def signal_handler(sig: int, frame: FrameType | None) -> None:
            log_warn(f"Interrupted by signal {sig}. Triggering cleanup...")
            sys.exit(128 + sig)

        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)

    def cleanup(self) -> None:
        """Final cleanup of infrastructure."""
        if not self.needs_cleanup or self.dry_run:
            return

        self.needs_cleanup = False
        os.environ["MODE"] = self.active_mode

        try:
            cleanup_log = PROJECT_ROOT / ".test_logs" / "cleanup.log"
            cleanup_log.parent.mkdir(exist_ok=True)
            with cleanup_log.open("a") as f:
                now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                f.write(f"\n--- Cleanup started at {now} ---\n")
                lifecycle_down(self.active_mode)
        except (OSError, ValueError):
            pass  # stderr may be closed during test teardown
        except Exception as e:
            log_error(f"❌ Cleanup failed with error: {e}")

    @property
    def engine(self) -> str:
        """Detect container engine (podman or docker)."""
        return detect_engine()

    @property
    def compose_engine(self) -> list[str]:
        """Detect compose tool."""
        return detect_compose_cmd()

    @property
    def compose_engine_cmd(self) -> str:
        """Get compose tool as a single string for shell scripts."""
        return " ".join(self.compose_engine)

    def setup_parser(self) -> None:
        """Setup command line argument parser."""
        parent_parser = argparse.ArgumentParser(add_help=False)
        parent_parser.add_argument("--dry-run", action="store_true")

        subparsers = self.parser.add_subparsers(dest="command", required=True)
        self._setup_test_parser(subparsers, parent_parser)
        self._setup_manage_parser(subparsers, parent_parser)
        self._setup_run_parser(subparsers, parent_parser)

    def _setup_test_parser(
        self,
        subparsers: typing.Any,  # noqa: ANN401
        parent_parser: argparse.ArgumentParser,
    ) -> None:
        test_parser = subparsers.add_parser("test", parents=[parent_parser])
        test_parser.add_argument(
            "target",
            choices=["all", "api", "web", "apiserver", "agent-main", "webdashboard", "infra"],
            help="Target domain or component to test",
        )
        test_parser.add_argument("--lint-only", action="store_true", help="Run only linting")
        test_parser.add_argument("--unit-only", action="store_true", help="Run only unit tests")
        test_parser.add_argument("--e2e-only", action="store_true", help="Run only integration/e2e tests")
        test_parser.add_argument("--skip-lint", action="store_true", help="Skip linting phase")
        test_parser.add_argument("--pattern", dest="pattern", help="Regex pattern for tests")
        test_parser.add_argument("--coverage", action="store_true", help="Run tests with coverage reporting")
        test_parser.add_argument("--timeout", type=int, default=1800)

    def _setup_manage_parser(
        self,
        subparsers: typing.Any,  # noqa: ANN401
        parent_parser: argparse.ArgumentParser,
    ) -> None:
        manage_parser = subparsers.add_parser("manage", parents=[parent_parser])
        manage_cmd_sub = manage_parser.add_subparsers(dest="subcommand", required=True)

        for cmd in ["up", "down", "ps"]:
            p = manage_cmd_sub.add_parser(cmd, parents=[parent_parser])
            p.add_argument("env", choices=["dev", "test", "prod"], default="dev", nargs="?")

        logs_p = manage_cmd_sub.add_parser("logs", parents=[parent_parser])
        logs_p.add_argument("env", choices=["dev", "test", "prod"], default="dev", nargs="?")
        logs_p.add_argument("service", nargs="?")

        manage_cmds = [
            "sync:api",
            "test:infra",
            "migrate:run",
            "migrate:revert",
            "migrate:status",
            "db:setup",
            "prebuild",
            "fmt:infra",
            "clean",
            "doctor",
            "verify:infra",
        ]
        for cmd in manage_cmds:
            manage_cmd_sub.add_parser(cmd, parents=[parent_parser])

        exec_p = manage_cmd_sub.add_parser("exec", parents=[parent_parser])
        exec_p.add_argument("service")
        exec_p.add_argument("args", nargs=argparse.REMAINDER)

        db_query_p = manage_cmd_sub.add_parser("db:query", parents=[parent_parser])
        db_query_p.add_argument("sql", help="SQL query to execute")
        db_query_p.add_argument("--json", action="store_true", help="Output in JSON format")

    def _setup_run_parser(
        self,
        subparsers: typing.Any,  # noqa: ANN401
        parent_parser: argparse.ArgumentParser,
    ) -> None:
        run_p = subparsers.add_parser("run", help="Run toolbox command", parents=[parent_parser])
        run_p.add_argument("tool", choices=list(self.tool_map.keys()), help="Tool to run")
        run_p.add_argument("args", nargs=argparse.REMAINDER, help="Arguments for the tool")

    def run(self) -> None:
        args = self.parser.parse_args()
        self.dry_run = args.dry_run
        if not args.command:
            self.parser.print_help()
            return
        if args.command == "test":
            self.handle_test(args)
        elif args.command == "manage":
            self.handle_manage(args)
        elif args.command == "run":
            self.handle_run(args)

    def handle_test(self, args: argparse.Namespace) -> None:
        target = getattr(args, "target", None)

        if target == "infra":
            if args.lint_only:
                self.handle_lint_infra()
            else:
                self.handle_verify_infra()
            return

        mode_map = {
            "all": "all",
            "api": "api",
            "web": "web",
            "apiserver": "api",
            "agent-main": "api",
            "webdashboard": "web",
        }

        mode = mode_map.get(target, "all")
        component_filter = target if target in ["apiserver", "agent-main", "webdashboard"] else ""
        args.component = component_filter

        self.active_mode = mode
        log_info(f"🚀 Starting LeeAttend Test Suite for [{target}] in mode [{mode}]")

        if target == "all" and not (args.lint_only or args.unit_only or args.e2e_only):
            self.handle_verify_infra()

        startup_mode = "integration" if mode == "api" else mode
        self.run_phase("Startup", startup_mode, args)
        self.needs_cleanup = True

        run_lint = not args.skip_lint and not (args.unit_only or args.e2e_only)
        run_unit = not (args.lint_only or args.e2e_only)
        run_e2e = not (args.lint_only or args.unit_only)

        if args.lint_only:
            run_lint = True

        if run_lint:
            self.run_phase("Linting", mode, args)

        if run_unit:
            self.run_phase("Unit Tests", mode, args)

        if run_e2e:
            self.run_phase("Integration Tests", mode, args)

        if args.coverage:
            self.run_phase("Coverage", mode, args)

        self.print_test_summary(target)

    def run_phase(self, phase_name: str, mode: str, args: argparse.Namespace) -> None:
        if self.dry_run:
            log_info(f"🔍 Dry-run: Phase [{phase_name}] for [{mode}]")
            return

        log_info(f"🔹 Running {phase_name}...")
        func_map: dict[str, typing.Callable[..., typing.Any]] = {
            "Startup": lambda: _lifecycle_up(mode),
            "Linting": lambda: leeattend_run_lint(getattr(args, "component", "") or "", mode),
            "Unit Tests": lambda: leeattend_run_unit(
                getattr(args, "component", "") or "",
                mode,
                test_pattern=getattr(args, "pattern", "") or "",
            ),
            "Integration Tests": lambda: leeattend_run_integration(
                getattr(args, "component", "") or "",
                mode,
                test_pattern=getattr(args, "pattern", "") or "",
            ),
            "Coverage": lambda: leeattend_run_coverage(getattr(args, "component", "") or "", mode),
        }

        func = func_map.get(phase_name)
        if func is None:
            log_error(f"Unknown phase: {phase_name}")
            sys.exit(1)
        res = func()
        if res is False:
            sys.exit(1)

    def handle_manage(self, args: argparse.Namespace) -> None:
        """Handle 'manage' commands using a dispatch map for simplicity."""
        sub = args.subcommand

        simple_dispatch: dict[str, typing.Callable[[], typing.Any]] = {
            "sync:api": lambda: self.execute_safe(
                ["/bin/bash", "-c", f'source "{SCRIPTS_DIR}/_sync-api.sh"']
            ),
            "test:infra": self.handle_test_infra,
            "migrate:run": lambda: self.handle_sqlx(["migrate", "run"]),
            "migrate:revert": lambda: self.handle_sqlx(["migrate", "revert"]),
            "migrate:status": lambda: self.handle_sqlx(["migrate", "status"]),
            "db:setup": lambda: self.run_phase("Database Setup", "all", args),
            "prebuild": lambda: self.run_phase("Prebuild", "all", args),
            "fmt:infra": self.handle_fmt_infra,
            "doctor": self.handle_doctor,
            "verify:infra": self.handle_verify_infra,
            "clean": lambda: log_info("Clean not implemented yet"),
        }

        if sub in simple_dispatch:
            simple_dispatch[sub]()
            return

        if sub == "db:query":
            self.handle_db_query(args)
        elif sub in ["up", "down", "ps", "logs"]:
            target_env: str = getattr(args, "env", "dev")
            files = self.get_compose_files(target_env)
            cmd = self.compose_engine + files + [sub]
            if sub == "up":
                cmd.append("-d")
            if sub == "logs":
                cmd.append("-f")
                if args.service:
                    cmd.append(args.service)
            self.execute_safe(cmd)
        elif sub == "exec":
            files = self.get_compose_files("dev")
            cmd = (
                self.compose_engine
                + files
                + ["exec", args.service]
                + (args.args if args.args else ["bash"])
            )
            self.execute_safe(cmd)
        else:  # pragma: no cover
            self.parser.error(f"Unknown subcommand: {sub}")

    def handle_test_infra(self) -> None:
        """Enforce 100% coverage for infrastructure logic."""
        env = os.environ.copy()
        env["PYTHONPATH"] = str(SCRIPTS_DIR)
        test_files = [
            str(SCRIPTS_DIR / "tests" / "test_orchestrator.py"),
            str(SCRIPTS_DIR / "tests" / "test_safe_run.py"),
            str(SCRIPTS_DIR / "tests" / "test_usage.py"),
            str(SCRIPTS_DIR / "tests" / "test_git_wrapper.py"),
            str(SCRIPTS_DIR / "tests" / "test_apply_guards.py"),
            str(SCRIPTS_DIR / "tests" / "test_kill_hung_processes.py"),
        ]
        venv_pytest = PROJECT_ROOT / ".venv" / "bin" / "pytest"
        cmd = [
            str(venv_pytest),
            "--cov=scripts",
            "--cov-report=term-missing",
            "--cov-fail-under=100",
        ] + test_files
        self.execute_safe(cmd, env=env)

    def handle_run(self, args: argparse.Namespace) -> None:
        tool = args.tool
        tool_args = args.args
        service = self.tool_map[tool]

        if tool_args and tool_args[0] == "--":
            tool_args = tool_args[1:]

        # Sanitize AI-provided arguments BEFORE any processing
        tool_args = sanitize(tool_args, die_on_error=True)

        log_info(f"🛠️ Running {tool} inside container environment...")

        compose_cmd = self.compose_engine + [
            "-p",
            self.env_vars.get("COMPOSE_PROJECT_NAME", "leeattend-test"),
        ]

        if tool in ["cargo", "sqlx"]:
            compose_cmd.extend(
                [
                    "--profile",
                    "int-api",
                    "--profile",
                    "infra-db",
                    "--profile",
                    "infra-db-init",
                    "--profile",
                    "infra-redis",
                ]
            )
        else:
            compose_cmd.extend(["--profile", "frontend"])

        compose_cmd.extend(
            [
                "-f",
                str(PROJECT_ROOT / ".compose" / "docker-compose.test.yml"),
            ]
        )

        if tool == "npm":
            first = tool_args[0] if tool_args else ""
            is_standalone = tool_args and first in ("--version", "--help", "-v", "-h")
            if is_standalone:
                # standalone: compose run --no-deps --entrypoint bun (no prefix needed)
                compose_cmd.extend(["run", "-T", "--rm", "--no-deps", "--entrypoint", "bun", service])
                if tool_args:
                    compose_cmd.extend(tool_args)
            else:
                # compose exec into running container, prefix with bun
                compose_cmd.extend(["exec", "-T", service])
                self._handle_run_npm(compose_cmd, tool_args, service)
        elif tool == "cargo":
            compose_cmd.extend(["run", "-T", "--rm"])
            self._handle_run_cargo(compose_cmd, tool_args)
        else:  # sqlx
            compose_cmd.extend(["run", "-T", "--rm", "--entrypoint", tool, service])
            if tool_args:
                compose_cmd.extend(tool_args)

        self.execute_safe(compose_cmd)

    def _handle_run_npm(self, compose_cmd: list[str], tool_args: list[str], service: str) -> None:
        """Run npm/bun commands via compose exec into running container.

        Requires webdashboard container to be running (started by lifecycle_up
        or compose up -d). Uses direct entrypoint — no bash -c, no shell.
        """
        npm_min_args = 2
        is_typecheck = (
            len(tool_args) >= npm_min_args and tool_args[0] == "run" and tool_args[1] == "typecheck"
        )
        if is_typecheck:
            tool_args[1] = "type-check"

        first_arg = tool_args[0] if tool_args else ""

        if first_arg in ("bun", "npm", "bash", "sh"):
            compose_cmd.extend([first_arg])
            if len(tool_args) > 1:
                compose_cmd.extend(tool_args[1:])
        else:
            compose_cmd.extend(["bun"])
            if tool_args:
                compose_cmd.extend(tool_args)

    def _handle_run_cargo(self, compose_cmd: list[str], tool_args: list[str]) -> None:
        caller_dir = Path.cwd()
        rel_dir = ""
        with contextlib.suppress(ValueError):
            rel_dir = str(caller_dir.relative_to(PROJECT_ROOT))
            if rel_dir == ".":
                rel_dir = ""

        target_service = "apiserver"
        if "agent-main" in rel_dir or any("agent-main" in arg for arg in tool_args):
            target_service = "agent-main"

        workdir = f"/workspace/{rel_dir}" if rel_dir else "/workspace"
        compose_cmd.extend(
            [
                "-e",
                "SQLX_OFFLINE=true",
                "--workdir",
                workdir,
                "--entrypoint",
                "cargo",
                target_service,
            ]
        )
        if tool_args:
            compose_cmd.extend(tool_args)

    def handle_lint_infra(self) -> None:
        log_info("🎨 Linting infrastructure scripts...")
        venv_bin = PROJECT_ROOT / ".venv" / "bin"
        self.execute_safe([str(venv_bin / "ruff"), "check", str(SCRIPTS_DIR)])
        self.execute_safe([str(venv_bin / "mypy"), str(SCRIPTS_DIR)])

        sh_files = [str(f) for f in PROJECT_ROOT.glob("*.sh")]
        sh_files += [str(f) for f in SCRIPTS_DIR.glob("*.sh")]
        if shutil.which("shellcheck"):
            self.execute_safe(["shellcheck"] + sh_files)

        log_success("✨ Infrastructure linting passed!")

    def handle_fmt_infra(self) -> None:
        log_info("💅 Formatting infrastructure scripts...")
        venv_bin = PROJECT_ROOT / ".venv" / "bin"
        self.execute_safe([str(venv_bin / "ruff"), "format", str(SCRIPTS_DIR)])
        log_success("✨ Infrastructure formatting completed!")

    def handle_verify_infra(self) -> None:
        log_info("🚀 Starting full infrastructure verification...")
        self.handle_fmt_infra()
        self.handle_lint_infra()
        self.handle_test_infra()
        log_success("✨ Infrastructure is Premium Grade!")

    def handle_doctor(self) -> None:
        log_info("🩺 Running LeeAttend System Doctor...")
        engine = self.engine
        log_success(f"✅ Container Engine: {engine}")

        default_ports = [3000, 8000, 5432]
        connection_timeout = 0.5
        for port in default_ports:
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.settimeout(connection_timeout)
            if s.connect_ex(("127.0.0.1", port)) == 0:
                log_info(f"⚠️  Port {port} is occupied")
            s.close()

        if (PROJECT_ROOT / ".venv").exists():
            log_success("✅ Virtual Environment: Found")
        else:
            log_info("💡 Virtual Environment: Missing (will be created on next run)")

        if engine:
            res = subprocess.run(
                [engine, "ps", "--format", "{{.Names}}"],
                capture_output=True,
                text=True,
                stdin=subprocess.DEVNULL,
                check=False,
            )
            names = res.stdout.splitlines()
            for r in ["leeattend-dev-db", "leeattend-dev-api"]:
                if any(r in n for n in names):
                    log_success(f"✅ Container {r}: Running")

    def handle_db_query(self, args: argparse.Namespace) -> None:
        sql = args.sql
        log_info(f"🗄️ Executing SQL: {sql}")
        cmd = [
            self.engine,
            "exec",
            "-t",
            "leeattend-dev-db",
            "psql",
            "-U",
            "lee",
            "-d",
            "leeattend",
            "-c",
            sql,
        ]
        if args.json:
            cmd += ["-A", "-t", "--json"]
        self.execute_safe(cmd)

    def handle_sqlx(self, args: list[str]) -> None:
        """Wrapper for sqlx commands."""
        caller_dir = Path.cwd()
        rel_dir = ""
        with contextlib.suppress(ValueError):
            rel_dir = str(caller_dir.relative_to(PROJECT_ROOT))

        workdir = f"/workspace/{rel_dir}" if rel_dir else "/workspace"
        cmd = (
            self.compose_engine
            + [
                "-p",
                "leeattend-test",
                "-f",
                str(PROJECT_ROOT / ".compose" / "docker-compose.test.yml"),
                "run",
                "-T",
                "--rm",
                "--workdir",
                workdir,
                "--entrypoint",
                "sqlx",
                "apiserver",
            ]
            + args
        )
        self.execute_safe(cmd)

    def get_compose_files(self, env: str) -> list[str]:
        if env == "dev":
            return [
                "-p",
                "leeattend-dev",
                "-f",
                "docker-compose.yml",
                "-f",
                ".compose/docker-compose.dev.yml",
            ]
        if env == "test":
            project_name = self.env_vars.get("COMPOSE_PROJECT_NAME", "leeattend-test")
            return ["-p", project_name, "-f", ".compose/docker-compose.test.yml"]
        return ["-p", "leeattend", "-f", "docker-compose.yml"]

    def execute_safe(
        self, cmd: list[str], env: dict[str, str] | None = None, timeout: int = 1800
    ) -> None:
        if self.dry_run:
            log_info(f"🔍 Dry-run: Executing {' '.join(cmd)}")
            return

        safe_run = SCRIPTS_DIR / "_safe_run.py"
        full_cmd = ["python3", str(safe_run), str(timeout)] + cmd
        current_env = env if env else os.environ.copy()
        current_env.update(self.env_vars)

        result = subprocess.run(full_cmd, cwd=PROJECT_ROOT, env=current_env, check=False)
        if result.returncode != 0:
            log_error(f"❌ Command failed: {' '.join(cmd)}")
            sys.exit(result.returncode)

    def print_test_summary(self, target: str) -> None:
        import re
        total_tests = 0
        passed_tests = 0

        nextest_pattern = re.compile(r"Summary \[.*?\] (\d+) tests? run: (\d+) passed")
        vitest_pattern = re.compile(r"Tests\s+(\d+)\s+passed\s+\((\d+)\)")
        pw_pattern = re.compile(r"^\s*(\d+)\s+passed\s+\(.*?\)", re.MULTILINE)

        log_dir = PROJECT_ROOT / ".test_logs"
        if log_dir.exists():
            for log_file in log_dir.glob("*.log"):
                try:
                    if log_file.stat().st_mtime < self.start_time:
                        continue

                    content = log_file.read_text(errors="replace")
                    ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
                    clean_content = ansi_escape.sub('', content)

                    # nextest
                    matches = list(nextest_pattern.finditer(clean_content))
                    if matches:
                        last_match = matches[-1]
                        total_tests += int(last_match.group(1))
                        passed_tests += int(last_match.group(2))
                        continue

                    # vitest
                    matches = list(vitest_pattern.finditer(clean_content))
                    if matches:
                        last_match = matches[-1]
                        passed_tests += int(last_match.group(1))
                        total_tests += int(last_match.group(2))
                        continue

                    # playwright
                    if "playwright" in log_file.name.lower():
                        matches = list(pw_pattern.finditer(clean_content))
                        if matches:
                            last_match = matches[-1]
                            passed = int(last_match.group(1))
                            passed_tests += passed
                            total_tests += passed
                except Exception:
                    pass

        if total_tests > 0:
            log_success(f"All selected tests for [{target}] passed successfully! ({passed_tests}/{total_tests} tests)")
        else:
            log_success(f"All selected tests for [{target}] passed successfully!")


if __name__ == "__main__":  # pragma: no cover
    Orchestrator().run()

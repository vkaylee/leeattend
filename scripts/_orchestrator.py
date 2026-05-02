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
import typing
from datetime import datetime
from pathlib import Path
from types import FrameType

# LeeAttend Enterprise Orchestrator Core
# ==============================================================================

PROJECT_ROOT = Path(__file__).resolve().parent.parent
SCRIPTS_DIR = PROJECT_ROOT / "scripts"


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
            "PROJECT_ROOT": str(PROJECT_ROOT),
            "CONTAINER_ENGINE": self.engine,
            "DOCKER_COMPOSE_CMD": " ".join(self.compose_engine),
        }
        self.needs_cleanup = False
        self.active_mode = "all"
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
        log_info("🧹 Orchestrator: Triggering final infrastructure cleanup...")
        bootstrap = SCRIPTS_DIR / "_bootstrap.sh"
        env = os.environ.copy()
        env.update(self.env_vars)
        env["MODE"] = self.active_mode

        cleanup_timeout = 60
        try:
            cleanup_log = PROJECT_ROOT / ".test_logs" / "cleanup.log"
            cleanup_log.parent.mkdir(exist_ok=True)
            with cleanup_log.open("a") as f:
                now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                f.write(f"\n--- Cleanup started at {now} ---\n")
                cmd = [
                    "/bin/bash",
                    "-c",
                    f'source "{bootstrap}" && leeattend_bootstrap_env && lifecycle_down',
                ]
                subprocess.run(
                    cmd,
                    env=env,
                    cwd=PROJECT_ROOT,
                    stdout=f,
                    stderr=f,
                    timeout=cleanup_timeout,
                    check=False,
                )
        except subprocess.TimeoutExpired:
            log_error("⏰ Cleanup timed out after 60s.")
        except Exception as e:
            log_error(f"❌ Cleanup failed with error: {e}")

        log_success("✨ Infrastructure cleanup completed.")

    @property
    def engine(self) -> str:
        """Detect container engine (podman or docker)."""
        if shutil.which("podman"):
            return "podman"
        if shutil.which("docker"):
            return "docker"
        return "podman"

    @property
    def compose_engine(self) -> list[str]:
        """Detect compose tool."""
        if shutil.which("podman"):
            return ["podman-compose"]
        if shutil.which("docker"):
            return ["docker", "compose"]
        return ["podman-compose"]

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
        test_parser.add_argument("--all", action="store_true")
        test_parser.add_argument("--api", action="store_true")
        test_parser.add_argument("--web", action="store_true")
        test_parser.add_argument("--integration", action="store_true")
        test_parser.add_argument("--pattern", dest="pattern", help="Regex pattern for tests")
        test_parser.add_argument(
            "--component",
            choices=["apiserver", "agent-main", "webdashboard"],
            help="Specific component to test",
        )
        test_parser.add_argument("--test-only", action="store_true")
        test_parser.add_argument("--coverage", action="store_true")
        test_parser.add_argument("--timeout", type=int, default=1800)

        # Allow 'test lint:web' style commands
        test_sub = test_parser.add_subparsers(dest="test_subcommand")
        for cmd in ["lint:api", "lint:web", "lint:all", "lint:infra"]:
            test_sub.add_parser(cmd, parents=[parent_parser])

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
        sub = getattr(args, "test_subcommand", None)
        if sub:
            if sub == "lint:api":
                self.run_phase("Linting", "api", args)
            elif sub == "lint:web":
                self.run_phase("Linting", "web", args)
            elif sub == "lint:infra":
                self.handle_lint_infra()
            elif sub == "lint:all":
                self.run_phase("Linting", "all", args)
            return

        if not any([args.all, args.api, args.web, args.integration]):
            self.parser.error("No test mode specified. Use --all, --api, --web, or --integration.")

        mode = "all" if args.all else ("api" if args.api else ("web" if args.web else "api"))
        if args.integration:
            mode = "integration"

        self.active_mode = mode
        log_info(f"🚀 Starting LeeAttend Test Suite in [{mode}] mode")

        if mode == "all" and not args.test_only:
            self.handle_verify_infra()

        if not args.test_only:
            startup_mode = mode
            if mode == "api":
                startup_mode = "integration"
            self.run_phase("Startup", startup_mode, args)
            self.needs_cleanup = True

        if not args.test_only:
            self.run_phase("Linting", mode, args)
            self.run_phase("Unit Tests", mode, args)

        self.run_phase("Integration Tests", mode, args)

        if args.coverage:
            self.run_phase("Coverage", mode, args)

        log_success(f"All tests for [{mode}] passed successfully!")

    def run_phase(self, phase_name: str, mode: str, args: argparse.Namespace) -> None:
        if self.dry_run:
            log_info(f"🔍 Dry-run: Phase [{phase_name}] for [{mode}]")
            return

        log_info(f"🔹 Running {phase_name}...")
        func_map = {
            "Startup": "lifecycle_up",
            "Linting": "leeattend_run_lint",
            "Unit Tests": "leeattend_run_unit",
            "Integration Tests": "leeattend_run_integration",
            "Coverage": "leeattend_run_coverage",
        }

        bootstrap = SCRIPTS_DIR / "_bootstrap.sh"
        cmd = [
            "/bin/bash",
            "-c",
            f'source "{bootstrap}" && leeattend_bootstrap_env && '
            f'{func_map[phase_name]} "{getattr(args, "component", "")}"',
        ]

        env = os.environ.copy()
        env.update(self.env_vars)
        env["MODE"] = mode
        env["PHASE"] = phase_name
        pattern = getattr(args, "pattern", None)
        if pattern:
            env["PATTERN"] = pattern
        component = getattr(args, "component", None)
        if component:
            env["COMPONENT"] = component

        timeout = getattr(args, "timeout", 1800)
        self.execute_safe(cmd, env=env, timeout=timeout)

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

        log_info(f"🛠️ Running {tool} inside container environment...")

        compose_cmd = self.compose_engine + [
            "-p",
            self.env_vars.get("COMPOSE_PROJECT_NAME", "leeattend-test"),
            "--profile",
            "backend",
            "--profile",
            "frontend",
            "-f",
            str(PROJECT_ROOT / ".compose" / "docker-compose.test.yml"),
            "run",
            "-T",
            "--rm",
        ]

        if tool == "npm":
            self._handle_run_npm(compose_cmd, tool_args, service)
        elif tool == "cargo":
            self._handle_run_cargo(compose_cmd, tool_args)
        else:
            compose_cmd.extend(["--entrypoint", tool, service])
            if tool_args:
                compose_cmd.extend(tool_args)

        self.needs_cleanup = True
        self.execute_safe(compose_cmd)

    def _handle_run_npm(self, compose_cmd: list[str], tool_args: list[str], service: str) -> None:
        npm_min_args = 2
        is_typecheck = (
            len(tool_args) >= npm_min_args and tool_args[0] == "run" and tool_args[1] == "typecheck"
        )
        if is_typecheck:
            tool_args[1] = "type-check"

        first_arg = tool_args[0] if tool_args else ""
        if first_arg in ["bun", "npm", "bash", "sh"]:
            npm_cmd = " ".join(tool_args)
        else:
            npm_cmd = f"bun {' '.join(tool_args)}"

        compose_cmd.extend(["--entrypoint", "bash", service, "-c", npm_cmd])

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


if __name__ == "__main__":  # pragma: no cover
    Orchestrator().run()

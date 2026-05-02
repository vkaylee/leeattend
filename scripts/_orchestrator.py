#!/usr/bin/env python3
import argparse
import os
import shutil
import socket
import subprocess
import sys
import typing
from pathlib import Path

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
        self.parser = argparse.ArgumentParser(
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
        import atexit
        import signal

        atexit.register(self.cleanup)

        def signal_handler(sig, frame):
            log_warn(f"Interrupted by signal {sig}. Triggering cleanup...")
            # atexit will handle the actual call to self.cleanup()
            sys.exit(128 + sig)

        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)

    def cleanup(self) -> None:
        """Final cleanup of infrastructure."""
        if not self.needs_cleanup or self.dry_run:
            return

        # Avoid recursion or multiple cleanups
        self.needs_cleanup = False

        log_info("🧹 Orchestrator: Triggering final infrastructure cleanup...")
        bootstrap = SCRIPTS_DIR / "_bootstrap.sh"
        env = os.environ.copy()
        env.update(self.env_vars)
        env["MODE"] = self.active_mode

        # Run lifecycle_down via bootstrap with a safety timeout
        try:
            subprocess.run(
                ["/bin/bash", str(bootstrap), "lifecycle_down"],
                env=env,
                cwd=PROJECT_ROOT,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=60,  # Safety timeout for cleanup
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
        return "podman"  # Fallback

    @property
    def compose_engine(self) -> list[str]:
        """Detect compose tool."""
        if shutil.which("podman"):
            return ["podman-compose"]
        if shutil.which("docker"):
            # Check for 'docker compose' (v2) or 'docker-compose' (v1)
            # Standard is 'docker compose' now
            return ["docker", "compose"]
        return ["podman-compose"]  # Fallback

    @property
    def compose_engine_cmd(self) -> str:
        """Get compose tool as a single string for shell scripts."""
        return " ".join(self.compose_engine)

    def setup_parser(self) -> None:
        # Shared arguments for all commands
        parent_parser = argparse.ArgumentParser(add_help=False)
        parent_parser.add_argument("--dry-run", action="store_true")

        subparsers = self.parser.add_subparsers(dest="command", required=True)

        # Command: test
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

        manage_parser = subparsers.add_parser("manage", parents=[parent_parser])
        manage_cmd_sub = manage_parser.add_subparsers(dest="subcommand", required=True)

        # Sub-commands for manage also need to inherit dry-run if we want it at the end
        for cmd in ["up", "down", "ps"]:
            p = manage_cmd_sub.add_parser(cmd, parents=[parent_parser])
            p.add_argument("env", choices=["dev", "test", "prod"], default="dev", nargs="?")

        logs_p = manage_cmd_sub.add_parser("logs", parents=[parent_parser])
        logs_p.add_argument("env", choices=["dev", "test", "prod"], default="dev", nargs="?")
        logs_p.add_argument("service", nargs="?")

        manage_cmd_sub.add_parser("sync:api", parents=[parent_parser])
        manage_cmd_sub.add_parser("test:infra", parents=[parent_parser])
        manage_cmd_sub.add_parser("lint:infra", parents=[parent_parser])
        manage_cmd_sub.add_parser("fmt:infra", parents=[parent_parser])
        manage_cmd_sub.add_parser("clean", parents=[parent_parser])
        manage_cmd_sub.add_parser("doctor", parents=[parent_parser])
        manage_cmd_sub.add_parser("verify:infra", parents=[parent_parser])

        exec_p = manage_cmd_sub.add_parser("exec", parents=[parent_parser])
        exec_p.add_argument("service")
        exec_p.add_argument("args", nargs=argparse.REMAINDER)

        db_query_p = manage_cmd_sub.add_parser("db:query", parents=[parent_parser])
        db_query_p.add_argument("sql", help="SQL query to execute")
        db_query_p.add_argument("--json", action="store_true", help="Output in JSON format")

        # Command: run (toolbox)
        run_p = subparsers.add_parser("run", help="Run toolbox command", parents=[parent_parser])
        run_p.add_argument("tool", choices=list(self.tool_map.keys()), help="Tool to run")
        run_p.add_argument("args", nargs=argparse.REMAINDER, help="Arguments for the tool")

    def run(self) -> None:
        args = self.parser.parse_args()
        self.dry_run = args.dry_run
        if not args.command:  # pragma: no cover
            self.parser.print_help()
            return
        if args.command == "test":
            self.handle_test(args)
        elif args.command == "manage":
            self.handle_manage(args)
        elif args.command == "run":
            self.handle_run(args)

    def handle_test(self, args: argparse.Namespace) -> None:
        # If no mode is specified, show help instead of defaulting to 'api'
        if not any([args.all, args.api, args.web, args.integration]):
            self.parser.error("No test mode specified. Use --all, --api, --web, or --integration.")

        mode = "all" if args.all else ("api" if args.api else ("web" if args.web else "api"))
        if args.integration:
            mode = "integration"

        self.active_mode = mode
        log_info(f"🚀 Starting LeeAttend Test Suite in [{mode}] mode")

        # Phase 0: Infrastructure Verification (Only in 'all' mode)
        if mode == "all" and not args.test_only:
            self.handle_verify_infra()

        # Phase 0.5: Infrastructure Startup
        if not args.test_only:
            # Determine if we need full integration infrastructure
            startup_mode = mode
            if mode == "api" and not args.test_only:
                # In standard --api run, we eventually run integration
                startup_mode = "integration"
            
            self.run_phase("Startup", startup_mode, args)
            self.needs_cleanup = True

        # Phase 1: Lint
        if not args.test_only:
            self.run_phase("Linting", mode, args)

        # Phase 2: Unit
        if not args.test_only:
            self.run_phase("Unit Tests", mode, args)

        # Phase 3: Integration
        self.run_phase("Integration Tests", mode, args)

        if args.coverage:
            self.run_phase("Coverage", mode, args)

        log_success(f"All tests for [{mode}] passed successfully!")

    def run_phase(self, phase_name: str, mode: str, args: argparse.Namespace) -> None:
        if self.dry_run:
            log_info(f"🔍 Dry-run: Phase [{phase_name}] for [{mode}]")
            return

        log_info(f"🔹 Running {phase_name}...")

        # Map phase to internal functions in Shell modules
        func_map = {
            "Startup": "lifecycle_up",
            "Linting": "leeattend_run_lint",
            "Unit Tests": "leeattend_run_unit",
            "Integration Tests": "leeattend_run_integration",
            "Coverage": "leeattend_run_coverage",
        }

        bootstrap = SCRIPTS_DIR / "_bootstrap.sh"
        # Use 'source && bootstrap_env && command' pattern to satisfy the BASH_SOURCE guard
        cmd = [
            "/bin/bash",
            "-c",
            f'source "{bootstrap}" && leeattend_bootstrap_env && {func_map[phase_name]} "{args.component or ""}"',
        ]

        # Pass context to the script via environment
        env = os.environ.copy()
        env.update(self.env_vars)
        env["MODE"] = mode
        env["PHASE"] = phase_name
        if args.pattern:
            env["PATTERN"] = args.pattern
        if args.component:
            env["COMPONENT"] = args.component

        # Use timeout if provided in args
        timeout = getattr(args, "timeout", 1800)
        self.execute_safe(cmd, env=env, timeout=timeout)

    def handle_manage(self, args: argparse.Namespace) -> None:
        sub = args.subcommand
        if sub == "sync:api":
            # Use 'source' pattern for internal scripts too
            self.execute_safe(["/bin/bash", "-c", f'source "{SCRIPTS_DIR}/_sync-api.sh"'])
        elif sub == "test:infra":
            # Enforce 100% coverage for infrastructure logic
            env = os.environ.copy()
            env["PYTHONPATH"] = str(SCRIPTS_DIR)
            test_files = [
                str(SCRIPTS_DIR / "tests" / "test_orchestrator.py"),
                str(SCRIPTS_DIR / "tests" / "test_safe_run.py"),
                str(SCRIPTS_DIR / "tests" / "test_usage.py"),
                str(SCRIPTS_DIR / "tests" / "test_git_wrapper.py"),
            ]
            venv_pytest = PROJECT_ROOT / ".venv" / "bin" / "pytest"
            cmd = [
                str(venv_pytest),
                "--cov=scripts",
                "--cov-report=term-missing",
                "--cov-fail-under=100",
            ] + test_files
            self.execute_safe(cmd, env=env)
        elif sub == "lint:infra":
            self.handle_lint_infra()
        elif sub == "fmt:infra":
            self.handle_fmt_infra()
        elif sub == "doctor":
            self.handle_doctor()
        elif sub == "verify:infra":
            self.handle_verify_infra()
        elif sub == "db:query":
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
                ["podman-compose"]
                + files
                + ["exec", args.service]
                + (args.args if args.args else ["bash"])
            )
            if self.dry_run:
                log_info(f"🔍 Dry-run: Executing {' '.join(cmd)}")
                return
            subprocess.run(cmd, cwd=PROJECT_ROOT)  # nosec

    def handle_run(self, args: argparse.Namespace) -> None:
        tool = args.tool
        tool_args = args.args
        service = self.tool_map[tool]

        # Intercept REMAINDER args shift
        if tool_args and tool_args[0] == "--":  # pragma: no cover
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
            # Special logic for npm (using bun proxy)
            if len(tool_args) >= 2 and tool_args[0] == "run" and tool_args[1] == "typecheck":
                tool_args[1] = "type-check"

            npm_cmd = f"npm install -g bun && bun {' '.join(tool_args)}"
            compose_cmd.extend(["--entrypoint", "bash", service, "-c", npm_cmd])
        elif tool == "cargo":
            # Special logic for cargo
            caller_dir = Path.cwd()
            rel_dir = ""
            try:
                rel_dir = str(caller_dir.relative_to(PROJECT_ROOT))
                if rel_dir == ".":
                    rel_dir = ""
            except ValueError:
                pass

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
        else:
            compose_cmd.extend(["--entrypoint", tool, service])
            if tool_args:
                compose_cmd.extend(tool_args)

        self.execute_safe(compose_cmd)

    def handle_lint_infra(self) -> None:
        log_info("🎨 Linting infrastructure scripts...")
        venv_bin = PROJECT_ROOT / ".venv" / "bin"

        # 1. Ruff (Python Lint)
        log_info("🔹 Running Ruff...")
        self.execute_safe([str(venv_bin / "ruff"), "check", str(SCRIPTS_DIR)])

        # 2. Mypy (Python Types)
        log_info("🔹 Running Mypy...")
        self.execute_safe([str(venv_bin / "mypy"), str(SCRIPTS_DIR)])

        # 3. ShellCheck
        sh_files = [str(f) for f in PROJECT_ROOT.glob("*.sh")]
        sh_files += [str(f) for f in SCRIPTS_DIR.glob("*.sh")]

        if shutil.which("shellcheck"):
            log_info("🔹 Running ShellCheck...")
            self.execute_safe(["shellcheck"] + sh_files)
        else:
            log_info("⚠️  ShellCheck not found on host, skipping shell lint.")

        log_success("✨ Infrastructure linting passed!")

    def handle_fmt_infra(self) -> None:
        log_info("💅 Formatting infrastructure scripts...")
        venv_bin = PROJECT_ROOT / ".venv" / "bin"

        # Ruff Format
        self.execute_safe([str(venv_bin / "ruff"), "format", str(SCRIPTS_DIR)])
        log_success("✨ Infrastructure formatting completed!")

    def handle_verify_infra(self) -> None:
        log_info("🚀 Starting full infrastructure verification...")

        log_info("Phase 1: Formatting")
        self.handle_fmt_infra()

        log_info("Phase 2: Linting")
        self.handle_lint_infra()

        log_info("Phase 3: Testing with Coverage Enforcement")
        # Reuse handle_manage logic for test:infra
        mock_args = argparse.Namespace(subcommand="test:infra")
        self.handle_manage(mock_args)

        log_success("✨ Infrastructure is Premium Grade! (All checks passed)")

    def handle_doctor(self) -> None:
        log_info("🩺 Running LeeAttend System Doctor...")

        # 1. Check Container Engine
        engine = (
            "podman" if shutil.which("podman") else ("docker" if shutil.which("docker") else None)
        )
        if engine:
            log_success(f"✅ Container Engine: {engine}")
        else:
            log_error("❌ No container engine found (podman or docker)")

        # 2. Check Ports
        for port in [3000, 8000, 5432]:
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.settimeout(0.5)
            if s.connect_ex(("127.0.0.1", port)) == 0:  # pragma: no cover
                log_info(f"⚠️  Port {port} is occupied")
            s.close()

        # 3. Check Venv
        if (PROJECT_ROOT / ".venv").exists():
            log_success("✅ Virtual Environment: Found")
        else:
            log_info("💡 Virtual Environment: Missing (will be created on next run)")

        # 4. Check Containers
        if engine:
            res = subprocess.run(
                [engine, "ps", "--format", "{{.Names}}"], capture_output=True, text=True
            )  # nosec
            names = res.stdout.splitlines()
            required = ["leeattend-dev-db", "leeattend-dev-api"]
            for r in required:
                if any(r in n for n in names):
                    log_success(f"✅ Container {r}: Running")
                else:
                    log_info(f"⏳ Container {r}: Not running")

    def handle_db_query(self, args: argparse.Namespace) -> None:
        sql = args.sql
        log_info(f"🗄️ Executing SQL: {sql}")

        # Determine container engine
        engine = self.engine
        container = "leeattend-dev-db"

        cmd = [
            engine,
            "exec",
            "-t",
            container,
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
        self,
        cmd: list[str],
        env: dict[str, str] | None = None,
        timeout: int = 1800,
    ) -> None:
        if self.dry_run:
            log_info(f"🔍 Dry-run: Executing {' '.join(cmd)}")
            return

        safe_run = SCRIPTS_DIR / "_safe_run.py"
        full_cmd = ["python3", str(safe_run), str(timeout)] + cmd

        # Merge our env vars
        current_env = env if env else os.environ.copy()
        current_env.update(self.env_vars)

        result = subprocess.run(full_cmd, cwd=PROJECT_ROOT, env=current_env)  # nosec
        if result.returncode != 0:
            log_error(f"❌ Command failed: {' '.join(cmd)}")
            log_error(f"   Exit code: {result.returncode}")
            sys.exit(result.returncode)


if __name__ == "__main__":  # pragma: no cover
    Orchestrator().run()

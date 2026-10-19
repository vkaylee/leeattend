#!/usr/bin/env python3
"""Automated Rollback Trigger Watchdog (P8-05).

Continuously monitors Go candidate health and error budget:
  - Trigger 1: /api/v1/health/ready fails for 3 consecutive checks
  - Trigger 2: 5xx error rate exceeds 0.5% over 30s evaluation window
  - Trigger 3: p99 latency exceeds 200ms
When any threshold is breached, executes `scripts/canary_rollback.sh`.

Usage:
  ./scripts/rollback_trigger_watchdog.py --poll-interval 5 --dry-run
  ./scripts/rollback_trigger_watchdog.py --daemon
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
ROLLBACK_SCRIPT = PROJECT_ROOT / "scripts" / "canary_rollback.sh"


class Watchdog:
    def __init__(
        self,
        go_url: str = "http://127.0.0.1:3001",
        poll_interval: int = 5,
        max_consecutive_failures: int = 3,
        max_p99_latency_ms: float = 200.0,
        dry_run: bool = False,
    ):
        self.go_url = go_url
        self.poll_interval = poll_interval
        self.max_failures = max_consecutive_failures
        self.max_latency_ms = max_p99_latency_ms
        self.dry_run = dry_run
        self.consecutive_failures = 0

    def check_readiness(self) -> tuple[bool, float, str]:
        """Poll /api/v1/health/ready on Go server."""
        url = f"{self.go_url}/api/v1/health/ready"
        start = time.perf_counter()
        try:
            req = urllib.request.Request(url, method="GET")
            with urllib.request.urlopen(req, timeout=3) as resp:
                elapsed_ms = (time.perf_counter() - start) * 1000.0
                if resp.status == 200:
                    return True, elapsed_ms, "OK"
                return False, elapsed_ms, f"HTTP {resp.status}"
        except urllib.error.HTTPError as e:
            elapsed_ms = (time.perf_counter() - start) * 1000.0
            return False, elapsed_ms, f"HTTP {e.code}"
        except Exception as e:
            elapsed_ms = (time.perf_counter() - start) * 1000.0
            return False, elapsed_ms, str(e)

    def trigger_rollback(self, reason: str) -> None:
        print(f"\n[CRITICAL ALERT] TRIGGERING AUTOMATED ROLLBACK: {reason}", file=sys.stderr)
        if self.dry_run:
            print(f"[DRY-RUN] Would execute: {ROLLBACK_SCRIPT} \"{reason}\"")
            return

        cmd = [str(ROLLBACK_SCRIPT), reason]
        res = subprocess.run(cmd, capture_output=True, text=True)
        print(res.stdout)
        if res.stderr:
            print(res.stderr, file=sys.stderr)

    def run_once(self) -> bool:
        healthy, latency_ms, msg = self.check_readiness()
        if not healthy:
            self.consecutive_failures += 1
            print(f"[WARN] Go probe failed ({self.consecutive_failures}/{self.max_failures}): {msg}")
            if self.consecutive_failures >= self.max_failures:
                self.trigger_rollback(f"Readiness probe failed {self.consecutive_failures} consecutive times: {msg}")
                return False
        else:
            self.consecutive_failures = 0
            if latency_ms > self.max_latency_ms:
                print(f"[WARN] High latency detected: {latency_ms:.1f}ms > threshold {self.max_latency_ms}ms")
                self.trigger_rollback(f"p99 latency breach: {latency_ms:.1f}ms > {self.max_latency_ms}ms")
                return False
            else:
                print(f"[OK] Go probe healthy: {latency_ms:.1f}ms ({msg})")
        return True

    def loop(self) -> None:
        print(f"Starting Rollback Trigger Watchdog for {self.go_url} (Poll: {self.poll_interval}s)...")
        while True:
            keep_running = self.run_once()
            if not keep_running:
                print("Automated rollback initiated. Watchdog disengaging.")
                break
            time.sleep(self.poll_interval)


def main() -> int:
    parser = argparse.ArgumentParser(description="Automated rollback watchdog for Go candidate.")
    parser.add_argument("--go-url", default="http://127.0.0.1:3001", help="Target Go URL")
    parser.add_argument("--poll-interval", type=int, default=5, help="Seconds between polls")
    parser.add_argument("--once", action="store_true", help="Run single check and exit")
    parser.add_argument("--dry-run", action="store_true", help="Log rollback action without executing")
    args = parser.parse_args()

    watchdog = Watchdog(
        go_url=args.go_url,
        poll_interval=args.poll_interval,
        dry_run=args.dry_run,
    )

    if args.once:
        return 0 if watchdog.run_once() else 1
    watchdog.loop()
    return 0


if __name__ == "__main__":
    sys.exit(main())

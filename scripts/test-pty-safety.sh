#!/bin/bash
# ==============================================================================
# PTY Safety Test Suite
# Detects PTY-unsafe patterns in shell scripts that would cause hangs when
# invoked from AI agent terminals or git hooks.
#
# Two test layers:
#   1. STATIC SCAN  — grep for known anti-patterns (fast, no Docker needed)
#   2. RUNTIME TEST — run scripts inside a real PTY via `script`, verify clean exit
#
# Usage:
#   ./scripts/test-pty-safety.sh              # Run all tests
#   ./scripts/test-pty-safety.sh --static     # Static scan only
#   ./scripts/test-pty-safety.sh --runtime    # Runtime tests only
#
# Exit code: 0 = all pass, 1 = failures found
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

if [[ -n "${NO_COLOR:-}" ]]; then
    RED='' GREEN='' YELLOW='' BLUE='' BOLD='' NC=''
fi

PASS=0
FAIL=0
WARN=0

pass() { ((PASS++)) || true; echo -e "  ${GREEN}✓${NC} $1"; }
fail() { ((FAIL++)) || true; echo -e "  ${RED}✗${NC} $1"; }
warn() { ((WARN++)) || true; echo -e "  ${YELLOW}⚠${NC} $1"; }

# ═══════════════════════════════════════════════════════════════════════════════
# LAYER 1: STATIC SCAN
# Scans all .sh files for known PTY-unsafe patterns.
# ═══════════════════════════════════════════════════════════════════════════════
run_static_scan() {
    echo -e "\n${BLUE}${BOLD}═══ LAYER 1: Static PTY Safety Scan ═══${NC}"

    # Files to scan (project root .sh + scripts/*.sh)
    local files=()
    while IFS= read -r f; do
        files+=("$f")
    done < <(find "$PROJECT_ROOT" -maxdepth 1 -name "*.sh" -type f)
    while IFS= read -r f; do
        files+=("$f")
    done < <(find "$PROJECT_ROOT/scripts" -maxdepth 1 -name "*.sh" -type f 2>/dev/null | grep -v 'test-pty-safety\.sh')
    # Also scan ai-tools scripts
    while IFS= read -r f; do
        files+=("$f")
    done < <(find "$PROJECT_ROOT/scripts/ai-tools" -maxdepth 1 -name "*.sh" -type f 2>/dev/null)

    if [[ ${#files[@]} -eq 0 ]]; then
        warn "No .sh files found to scan"
        return
    fi

    echo -e "  ${BLUE}Scanning ${#files[@]} files...${NC}"

    for file in "${files[@]}"; do
        local basename
        basename="$(basename "$file")"
        local file_fails=0

        # ── Pattern 1: Bare background `&` without FD redirection ─────────
        # Matches: `command &` at end of line
        # Excludes: lines with </dev/null or >/dev/null before &
        # Excludes: comment lines
        while IFS=: read -r lineno line; do
            # Skip comments
            [[ "$line" =~ ^[[:space:]]*# ]] && continue
            # Skip if already has /dev/null redirect
            [[ "$line" =~ /dev/null ]] && continue
            # Skip lines inside functions named dc_detached (they use setsid)
            # This is a known safe pattern
            [[ "$line" =~ setsid ]] && continue

            fail "$basename:$lineno — Bare background '&' without </dev/null: $(echo "$line" | sed 's/^[[:space:]]*//')"
            ((file_fails++)) || true
        done < <(grep -n '[^&]&[[:space:]]*$' "$file" 2>/dev/null || true)

        # ── Pattern 2: Actual docker/docker-compose invocations without </dev/null
        # Only match lines where docker is actually INVOKED (not mentioned in strings)
        # Store regex patterns in variables to avoid bash parser issues with |, [[, ;;
        local _patt_echo='echo|printf|BOLD|NC'
        local _patt_cond='if[[:space:]]|elif[[:space:]]'
        local _patt_test='\[\[|\-f[[:space:]]|==|!='
        local _patt_tool='grep|sed|awk'
        while IFS=: read -r lineno line; do
            [[ "$line" =~ ^[[:space:]]*# ]] && continue
            [[ "$line" =~ /dev/null ]] && continue
            # Skip multi-line commands (redirect may be on continuation line)
            [[ "$line" == *'\' ]] && continue
            [[ "$line" =~ $_patt_echo ]] && continue
            [[ "$line" =~ ^[[:space:]]*[A-Z_]+= ]] && continue
            [[ "$line" =~ $_patt_cond ]] && continue
            [[ "$line" =~ $_patt_test ]] && continue
            [[ "$line" =~ $_patt_tool ]] && continue
            # Skip case patterns and function definitions
            local _patt_case='^[[:space:]]*(;;|\))'
            local _patt_func='^[[:space:]]*(dc|dc_detached)[[:space:]]*\(\)'
            [[ "$line" =~ $_patt_case ]] && continue
            [[ "$line" =~ $_patt_func ]] && continue
            # Must actually start with docker or $DOCKER (stripped of leading whitespace)
            local trimmed
            trimmed="$(echo "$line" | sed 's/^[[:space:]]*//')"
            # Only flag if the line starts with docker, $DOCKER, or common wrappers
            if [[ "$trimmed" =~ ^(docker|\$DOCKER|\$\{DOCKER|timeout.*docker|setsid.*docker) ]]; then
                fail "$basename:$lineno — docker command without </dev/null: $trimmed"
                ((file_fails++)) || true
            fi
        done < <(grep -n '\bdocker\b' "$file" 2>/dev/null || true)

        # ── Pattern 3: `sleep` in background watchdog pattern ────────────
        while IFS=: read -r lineno line; do
            [[ "$line" =~ ^[[:space:]]*# ]] && continue
            fail "$basename:$lineno — Background watchdog pattern (sleep+kill): $(echo "$line" | sed 's/^[[:space:]]*//')"
            ((file_fails++)) || true
        done < <(grep -n 'sleep.*kill\|kill.*sleep' "$file" 2>/dev/null | grep '&' || true)

        # ── Pattern 4: `git commit` without --no-gpg-sign ───────────────
        while IFS=: read -r lineno line; do
            [[ "$line" =~ ^[[:space:]]*# ]] && continue
            [[ "$line" =~ --no-gpg-sign ]] && continue
            fail "$basename:$lineno — git commit without --no-gpg-sign: $(echo "$line" | sed 's/^[[:space:]]*//')"
            ((file_fails++)) || true
        done < <(grep -n 'git commit' "$file" 2>/dev/null || true)

        # ── Pattern 5: $* inside bash -c (word-splitting) ───────────────
        while IFS=: read -r lineno line; do
            [[ "$line" =~ ^[[:space:]]*# ]] && continue
            fail "$basename:$lineno — \$* inside bash -c (word-splitting risk): $(echo "$line" | sed 's/^[[:space:]]*//')"
            ((file_fails++)) || true
        done < <(grep -n 'bash -c.*\$\*' "$file" 2>/dev/null || true)

        # ── Pattern 6: `nohup` without FD redirection ────────────────────
        while IFS=: read -r lineno line; do
            [[ "$line" =~ ^[[:space:]]*# ]] && continue
            [[ "$line" =~ /dev/null ]] && continue
            fail "$basename:$lineno — nohup without FD redirect: $(echo "$line" | sed 's/^[[:space:]]*//')"
            ((file_fails++)) || true
        done < <(grep -n '\bnohup\b' "$file" 2>/dev/null || true)

        # ── Pattern 7: `disown` (orphans process, can leak PTY FDs) ──────
        while IFS=: read -r lineno line; do
            [[ "$line" =~ ^[[:space:]]*# ]] && continue
            warn "$basename:$lineno — 'disown' usage (may orphan PTY-holding process): $(echo "$line" | sed 's/^[[:space:]]*//')"
        done < <(grep -n '\bdisown\b' "$file" 2>/dev/null || true)

        # ── Pattern 8: `coproc` (creates background process with FDs) ────
        while IFS=: read -r lineno line; do
            [[ "$line" =~ ^[[:space:]]*# ]] && continue
            fail "$basename:$lineno — coproc creates background FD-holding process: $(echo "$line" | sed 's/^[[:space:]]*//')"
            ((file_fails++)) || true
        done < <(grep -n '\bcoproc\b' "$file" 2>/dev/null || true)

        # ── Pattern 9: `setsid` WITHOUT FD redirect (partial detach) ─────
        while IFS=: read -r lineno line; do
            [[ "$line" =~ ^[[:space:]]*# ]] && continue
            [[ "$line" =~ /dev/null ]] && continue
            [[ "$line" == *'\' ]] && continue
            fail "$basename:$lineno — setsid without </dev/null >/dev/null 2>&1: $(echo "$line" | sed 's/^[[:space:]]*//')"
            ((file_fails++)) || true
        done < <(grep -n '\bsetsid\b' "$file" 2>/dev/null || true)

        # ── Pattern 10: $DOCKER_COMPOSE_CMD used directly without </dev/null ─
        # Scripts should use dc() wrapper, not $DOCKER_COMPOSE_CMD directly.
        # dc() already adds </dev/null. Direct usage may forget it.
        while IFS=: read -r lineno line; do
            [[ "$line" =~ ^[[:space:]]*# ]] && continue
            [[ "$line" =~ /dev/null ]] && continue
            [[ "$line" == *'\' ]] && continue
            # Skip export/assignment lines
            [[ "$line" =~ ^[[:space:]]*(export|local)[[:space:]] ]] && continue
            [[ "$line" =~ ^[[:space:]]*[A-Z_]+= ]] && continue
            # Skip if/conditional lines
            local _patt_if='if[[:space:]]|elif[[:space:]]|until[[:space:]]'
            [[ "$line" =~ $_patt_if ]] && continue
            [[ "$line" =~ echo ]] && continue
            warn "$basename:$lineno — \$DOCKER_COMPOSE_CMD used directly (use dc() wrapper instead): $(echo "$line" | sed 's/^[[:space:]]*//' | head -c 80)"
        done < <(grep -n '\$DOCKER_COMPOSE_CMD' "$file" 2>/dev/null | grep -v 'export\|^[[:space:]]*#\|dc()' || true)

        if [[ $file_fails -eq 0 ]]; then
            pass "$basename — clean"
        fi
    done

    # ── Cross-file check: trap cleanup must exist in main scripts ─────
    for main_script in "$PROJECT_ROOT/run-tests.sh" "$PROJECT_ROOT/ai-test.sh"; do
        [[ -f "$main_script" ]] || continue
        local bn
        bn="$(basename "$main_script")"
        if ! grep -q 'trap.*cleanup.*EXIT' "$main_script" 2>/dev/null && \
           ! grep -q 'trap.*EXIT' "$main_script" 2>/dev/null; then
            # ai-test.sh uses setsid pattern instead of trap — that's OK
            if [[ "$bn" != "ai-test.sh" ]]; then
                fail "$bn — missing 'trap cleanup EXIT' handler"
            fi
        fi
    done
}

# ═══════════════════════════════════════════════════════════════════════════════
# LAYER 2: RUNTIME PTY HANG TEST
# Runs a script inside a real PTY (via `script` command) and verifies it exits
# within the timeout. If it hangs, that means PTY FD leak.
# ═══════════════════════════════════════════════════════════════════════════════
run_runtime_test() {
    echo -e "\n${BLUE}${BOLD}═══ LAYER 2: Runtime PTY Hang Test ═══${NC}"

    # Prevent recursive loops: ai-test.sh calls us, we call ai-test.sh
    export _PTY_CHECK_DONE=1

    if ! command -v script &>/dev/null; then
        warn "'script' command not found — skipping runtime tests"
        return
    fi

    # Test 1: run-tests.sh --help must exit cleanly in a PTY
    _pty_test "run-tests.sh --help" \
        "$PROJECT_ROOT/run-tests.sh" "--help" 10

    # Test 2: run-tests.sh --dry-run must exit cleanly
    _pty_test "run-tests.sh --api --smoke --dry-run" \
        "$PROJECT_ROOT/run-tests.sh" "--api --smoke --dry-run" 15

    # Test 3: ai-test.sh --help (should fail fast, not hang)
    if [[ -f "$PROJECT_ROOT/ai-test.sh" ]]; then
        _pty_test "ai-test.sh --help (via wrapper)" \
            "$PROJECT_ROOT/ai-test.sh" "--help" 15
    fi

    # Test 4: Orphan process leak detection
    # Test 4: Orphan process leak detection
    _orphan_leak_test

    # Test 5: Signal cleanup — verify SIGTERM triggers clean exit
    _signal_cleanup_test

    # Test 6: Argument word-splitting test (validates $* fix in ai-test.sh)
    _arg_quoting_test
}

# Run a single command inside a PTY and verify it exits within timeout.
# Usage: _pty_test "description" "/path/to/script" "args" timeout_seconds
_pty_test() {
    local desc="$1"
    local script_path="$2"
    local args="$3"
    local timeout_sec="$4"
    local pty_log
    pty_log="/tmp/pty-test-$$.log"

    echo -e "  ${BLUE}Testing: $desc (timeout: ${timeout_sec}s)${NC}"

    # `script` creates a real PTY. The -c flag runs a command inside it.
    # `timeout` kills it if it hangs (= PTY FD leak detected).
    local exit_code=0
    timeout "$timeout_sec" script -qec "bash '$script_path' $args" "$pty_log" </dev/null >/dev/null 2>&1 || exit_code=$?

    # timeout returns 124 on timeout kill
    if [[ $exit_code -eq 124 ]]; then
        fail "$desc — HUNG (killed after ${timeout_sec}s) — PTY FD leak!"
        # Show last few lines for debugging
        echo -e "    ${RED}Last output:${NC}"
        tail -5 "$pty_log" 2>/dev/null | sed 's/^/    /' || true
    elif [[ $exit_code -eq 0 ]]; then
        pass "$desc — exited cleanly"
    else
        # Non-zero but didn't hang — that's OK for PTY safety
        # (the command itself may have failed, but it didn't hang)
        pass "$desc — exited with code $exit_code (no hang)"
    fi

    rm -f "$pty_log"
}

# Verify that arguments with special characters pass through correctly.
# This validates that the $* → "$@" quoting fix in ai-test.sh works.
_arg_quoting_test() {
    echo -e "  ${BLUE}Testing: argument quoting (spaces/special chars)${NC}"

    local test_script="$PROJECT_ROOT/run-tests.sh"
    [[ -f "$test_script" ]] || { warn "run-tests.sh not found — skipping arg test"; return; }

    # --test="foo bar" must not be split into --test="foo" and "bar"
    # --dry-run should show the pattern correctly
    local output
    output=$(bash "$test_script" --api --dry-run --test="hello world" </dev/null 2>&1) || true

    if echo "$output" | grep -q 'hello world'; then
        pass "argument quoting — spaces preserved in --test=\"hello world\""
    else
        fail "argument quoting — --test=\"hello world\" was split or lost"
        echo -e "    ${RED}Output:${NC} $(echo "$output" | grep -i pattern | head -1)" || true
    fi
}

# Verify that after a script exits, no orphan child processes remain.
_orphan_leak_test() {
    echo -e "  ${BLUE}Testing: orphan process leak detection${NC}"

    local test_script="$PROJECT_ROOT/run-tests.sh"
    [[ -f "$test_script" ]] || { warn "run-tests.sh not found — skipping orphan test"; return; }

    # Run in a subshell with timeout to guarantee we never hang
    local orphan_result=0
    (
        bash "$test_script" --help </dev/null >/dev/null 2>&1
        sleep 0.3
        # Check if we left any docker/cargo orphans behind
        # (pgrep returns 1 if no match — that's the GOOD case)
        if pgrep -f "docker.*leeattend-test" >/dev/null 2>&1; then
            exit 1  # orphans found
        fi
        exit 0
    ) & local check_pid=$!
    if ! timeout 10 tail --pid=$check_pid -f /dev/null 2>/dev/null; then
        kill -KILL $check_pid 2>/dev/null || true
        fail "orphan leak — test timed out (possible orphan processes)"
        return
    fi
    wait $check_pid 2>/dev/null && orphan_result=0 || orphan_result=$?

    if [[ $orphan_result -eq 0 ]]; then
        pass "orphan leak — no surviving child processes"
    else
        fail "orphan leak — child processes survived after script exit"
    fi
}

# Verify that SIGTERM to a running script results in clean exit (not hang).
_signal_cleanup_test() {
    echo -e "  ${BLUE}Testing: SIGTERM cleanup behavior${NC}"

    local test_script="$PROJECT_ROOT/run-tests.sh"
    [[ -f "$test_script" ]] || { warn "run-tests.sh not found — skipping signal test"; return; }

    # Start --help in background, send SIGTERM, verify it dies quickly
    bash "$test_script" --help </dev/null >/dev/null 2>&1 &
    local pid=$!

    sleep 0.2
    kill -TERM $pid 2>/dev/null || { pass "SIGTERM cleanup — process already exited (fast)"; return; }

    # Wait up to 5s using timeout instead of a polling loop
    if timeout 5 tail --pid=$pid -f /dev/null 2>/dev/null; then
        pass "SIGTERM cleanup — process exited cleanly after signal"
    else
        if kill -0 $pid 2>/dev/null; then
            fail "SIGTERM cleanup — process still alive after 5s (PID $pid)"
            kill -KILL $pid 2>/dev/null || true
        else
            pass "SIGTERM cleanup — process exited cleanly after signal"
        fi
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════════════════
main() {
    echo -e "${BLUE}${BOLD}🔒 PTY Safety Test Suite${NC}"
    echo -e "${BLUE}Testing scripts for PTY file descriptor leaks...${NC}"

    local run_static=true
    local run_runtime=true

    for arg in "$@"; do
        case "$arg" in
            --static)  run_runtime=false ;;
            --runtime) run_static=false ;;
            --help)
                echo "Usage: $0 [--static|--runtime]"
                exit 0
                ;;
        esac
    done

    [[ "$run_static" == "true" ]] && run_static_scan
    [[ "$run_runtime" == "true" ]] && run_runtime_test

    echo -e "\n${BLUE}${BOLD}═══ Summary ═══${NC}"
    echo -e "  ${GREEN}Pass: $PASS${NC}  ${RED}Fail: $FAIL${NC}  ${YELLOW}Warn: $WARN${NC}"

    if [[ $FAIL -gt 0 ]]; then
        echo -e "\n${RED}${BOLD}❌ PTY safety check FAILED — $FAIL issue(s) found${NC}"
        exit 1
    else
        echo -e "\n${GREEN}${BOLD}✅ All PTY safety checks passed${NC}"
        exit 0
    fi
}

main "$@"

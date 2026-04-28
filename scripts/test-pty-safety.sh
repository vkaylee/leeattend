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
#   ./scripts/test-pty-safety.sh --command "podman exec foo"   # Check single command
#   ./scripts/test-pty-safety.sh --check-file ./script.sh      # Check single file
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
# CORE: Scan a single file for PTY-unsafe patterns
# Usage: scan_single_file "/path/to/file"
# Sets global FAIL/WARN/PASS counters
# ═══════════════════════════════════════════════════════════════════════════════
scan_single_file() {
    local file="$1"
    local basename
    basename="$(basename "$file")"
    local file_fails=0

    # ── Pattern 1: Bare background `&` without FD redirection ─────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ "$line" =~ /dev/null ]] && continue
        [[ "$line" =~ setsid ]] && continue
        fail "$basename:$lineno — Bare background '&' without </dev/null: $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: add \`</dev/null\` before \`&\` to detach stdin from PTY" >&2
        ((file_fails++)) || true
    done < <(grep -n '[^&]&[[:space:]]*$' "$file" 2>/dev/null || true)

    # ── Pattern 16: Background inside subshell `(cmd &)` ──────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ "$line" =~ /dev/null ]] && continue
        fail "$basename:$lineno — Background '&' inside subshell without </dev/null: $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: add \`</dev/null\` before \`&\` inside the subshell" >&2
        ((file_fails++)) || true
    done < <(grep -nE '&\s*\)' "$file" 2>/dev/null || true)

    # ── Pattern 2: podman/docker invocations without </dev/null ────────
    local _patt_echo='echo|printf|BOLD|NC'
    local _patt_cond='if[[:space:]]|elif[[:space:]]'
    local _patt_test='\[\[|\-f[[:space:]]|==|!='
    local _patt_tool='grep|sed|awk'
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ "$line" =~ /dev/null ]] && continue
        [[ "$line" == *'\' ]] && continue
        [[ "$line" =~ $_patt_echo ]] && continue
        [[ "$line" =~ ^[[:space:]]*[A-Z_]+= ]] && continue
        [[ "$line" =~ $_patt_cond ]] && continue
        [[ "$line" =~ $_patt_test ]] && continue
        [[ "$line" =~ $_patt_tool ]] && continue
        local _patt_case='^[[:space:]]*(;;|\))'
        local _patt_func='^[[:space:]]*(dc|dc_detached)[[:space:]]*\(\)'
        [[ "$line" =~ $_patt_case ]] && continue
        [[ "$line" =~ $_patt_func ]] && continue
        local trimmed
        trimmed="$(echo "$line" | sed 's/^[[:space:]]*//')"
        if [[ "$trimmed" =~ ^(podman|docker|\$DOCKER|\$\{DOCKER|timeout.*podman|timeout.*docker|setsid.*podman|setsid.*docker) ]]; then
            fail "$basename:$lineno — container command without </dev/null: $trimmed"
            echo -e "    → fix: append \`</dev/null\` at end of command to prevent stdin inheritance" >&2
            ((file_fails++)) || true
        fi
    done < <(grep -nE '\bpodman\b|\bdocker\b|\$DOCKER' "$file" 2>/dev/null || true)

    # ── Pattern 3: Background watchdog (sleep+kill) ───────────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        fail "$basename:$lineno — Background watchdog pattern (sleep+kill): $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: use \`timeout\` command instead of sleep+kill watchdog" >&2
        ((file_fails++)) || true
    done < <(grep -n 'sleep.*kill\|kill.*sleep' "$file" 2>/dev/null | grep '&' || true)

    # ── Pattern 4: git commit without --no-gpg-sign ──────────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ "$line" =~ --no-gpg-sign ]] && continue
        fail "$basename:$lineno — git commit without --no-gpg-sign: $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: add \`--no-gpg-sign\` to skip GPG agent (holds PTY). Also append \`</dev/null\`" >&2
        ((file_fails++)) || true
    done < <(grep -n 'git commit' "$file" 2>/dev/null || true)

    # ── Pattern 5: $* inside bash -c (word-splitting) ────────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        fail "$basename:$lineno — \$* inside bash -c (word-splitting risk): $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: replace \`\$*\` with \`\$@\` (preserves quoting) or use \`\"\$@\"\`" >&2
        ((file_fails++)) || true
    done < <(grep -n 'bash -c.*\$\*' "$file" 2>/dev/null || true)

    # ── Pattern 6: nohup ──────────────────────────────────────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        fail "$basename:$lineno — nohup masks exit codes and detaches stdout/stderr: $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: remove nohup. Use \`cmd </dev/null >log 2>&1 &\` for background execution instead" >&2
        ((file_fails++)) || true
    done < <(grep -n '\bnohup\b' "$file" 2>/dev/null || true)

    # ── Pattern 7: disown ─────────────────────────────────────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        warn "$basename:$lineno — 'disown' may orphan PTY-holding process: $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: ensure process has all FDs redirected before disown, or use explicit \`wait \$PID\`" >&2
    done < <(grep -n '\bdisown\b' "$file" 2>/dev/null || true)

    # ── Pattern 8: coproc ─────────────────────────────────────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        fail "$basename:$lineno — coproc creates background FD-holding process: $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: replace coproc with explicit background process + named pipe or temp file" >&2
        ((file_fails++)) || true
    done < <(grep -n '\bcoproc\b' "$file" 2>/dev/null || true)

    # ── Pattern 9: setsid without FD redirect ─────────────────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ "$line" =~ /dev/null ]] && continue
        [[ "$line" == *'\' ]] && continue
        fail "$basename:$lineno — setsid without FD redirect: $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: add \`</dev/null >/dev/null 2>&1\` to fully detach all FDs" >&2
        ((file_fails++)) || true
    done < <(grep -n '\bsetsid\b' "$file" 2>/dev/null || true)

    # ── Pattern 10: $DOCKER_COMPOSE_CMD without dc() wrapper ──────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ "$line" =~ /dev/null ]] && continue
        [[ "$line" == *'\' ]] && continue
        [[ "$line" =~ ^[[:space:]]*(export|local)[[:space:]] ]] && continue
        [[ "$line" =~ ^[[:space:]]*[A-Z_]+= ]] && continue
        local _patt_if='if[[:space:]]|elif[[:space:]]|until[[:space:]]'
        [[ "$line" =~ $_patt_if ]] && continue
        [[ "$line" =~ echo ]] && continue
        warn "$basename:$lineno — \$DOCKER_COMPOSE_CMD used directly: $(echo "$line" | sed 's/^[[:space:]]*//' | head -c 80)"
        echo -e "    → fix: use the dc() wrapper function which includes </dev/null, or append </dev/null manually" >&2
    done < <(grep -n '\$DOCKER_COMPOSE_CMD' "$file" 2>/dev/null | grep -v 'export\|^[[:space:]]*#\|dc()' || true)

    # ── Pattern 12: compose exec without -T ───────────────────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        if echo "$line" | grep -qE 'compose[[:space:]]+exec'; then
            if ! echo "$line" | grep -qE 'exec[[:space:]]+-[[:alnum:]]*T'; then
                fail "$basename:$lineno — compose exec without TTY disable flag: $(echo "$line" | sed 's/^[[:space:]]*//')"
                echo -e "    → fix: add TTY disable flag after exec (e.g. \`-T\`). Run \`<tool> exec --help\` to verify flag" >&2
                ((file_fails++)) || true
            fi
        fi
    done < <(grep -nE 'compose[[:space:]]+exec' "$file" 2>/dev/null || true)

    # ── Pattern 13: global 'wait' without exact PID ───────────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        if echo "$line" | grep -qE '^[[:space:]]*wait[[:space:]]*$'; then
            warn "$basename:$lineno — global 'wait' without PID (hangs if background daemons exist): $(echo "$line" | sed 's/^[[:space:]]*//')"
            echo -e "    → fix: use \`wait \$PID\` with explicit PID from \`\$!\` after backgrounding" >&2
        fi
    done < <(grep -n 'wait' "$file" 2>/dev/null || true)

    # ── Pattern 14: EOF-buffering pipe targets ─────────────────────────
    # Commands that wait for stdin EOF before producing output. If a child
    # process inherits the pipe write-end FD, the pipe never closes → hang.
    local _pipe_targets='tee|tail|wc|sort|head|less|more'
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        # Skip lines annotated with pty-safe or not-a-pipe comments
        [[ "$line" =~ \#.*pty-safe ]] && continue
        [[ "$line" =~ \#.*not-a-pipe ]] && continue
        # Skip pipes inside bash -c "..." strings (container-internal, no host PTY risk)
        [[ "$line" =~ bash[[:space:]]+-c[[:space:]]+\" ]] && continue
        # Skip printf|head for glob array indexing (local, no child process FD leak)
        [[ "$line" =~ printf.*\|.*head ]] && continue
        local _matched
        _matched=$(echo "$line" | grep -oE "\|\s*($_pipe_targets)\b" | head -1 | sed 's/.*|\s*//' || true)
        [[ -z "$_matched" ]] && continue
        warn "$basename:$lineno — '| $_matched' pipe (FD leak risk — $_matched buffers until EOF): $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: replace \`cmd | $_matched ...\` with \`cmd >file 2>&1\` then \`$_matched file\`" >&2
    done < <(grep -nE "\|\s*($_pipe_targets)\b" "$file" 2>/dev/null || true)

    # ── Pattern 15: Interactive commands ───────────────────────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        fail "$basename:$lineno — Interactive command will hang in agentic terminal: $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: remove interactive command. Ask user to run it manually if needed" >&2
        ((file_fails++)) || true
    done < <(grep -nE '^[[:space:]]*(fzf|vim|nvim|nano|less|more|emacs)\b' "$file" 2>/dev/null || true)

    # ── Pattern 17: `read` / `select` (blocks on stdin) ───────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ "$line" =~ read.*-t ]] && continue
        [[ "$line" =~ \<\<\< ]] && continue
        [[ "$line" =~ read.*\< ]] && continue
        [[ "$line" =~ while.*read ]] && continue
        [[ "$line" =~ done.*\< ]] && continue
        fail "$basename:$lineno — 'read'/'select' blocks on stdin (AI agent cannot provide input): $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: use a default value or timeout flag (e.g. \`read -t 5\`). Remove if user input not needed" >&2
        ((file_fails++)) || true
    done < <(grep -nE '^\s*(read|select)\b' "$file" 2>/dev/null || true)

    # ── Pattern 18: `sudo` without -n (non-interactive) ───────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ "$line" =~ sudo.*-n ]] && continue
        [[ "$line" =~ sudo.*--non-interactive ]] && continue
        fail "$basename:$lineno — 'sudo' without non-interactive flag (will prompt for password): $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: add non-interactive flag. Run \`sudo --help\` to find correct flag for your platform" >&2
        ((file_fails++)) || true
    done < <(grep -nE '\bsudo\b' "$file" 2>/dev/null || true)

    # ── Pattern 19: `ssh` without BatchMode (may prompt) ──────────────
    while IFS=: read -r lineno line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ "$line" =~ BatchMode ]] && continue
        [[ "$line" =~ StrictHostKeyChecking ]] && continue
        [[ "$line" =~ ssh-(keygen|add|agent|copy-id) ]] && continue
        fail "$basename:$lineno — 'ssh' without non-interactive mode (may prompt for password/fingerprint): $(echo "$line" | sed 's/^[[:space:]]*//')"
        echo -e "    → fix: add batch/non-interactive option. Run \`ssh --help\` or check \`man ssh\` for flag on your platform" >&2
        ((file_fails++)) || true
    done < <(grep -nE '^\s*ssh\b' "$file" 2>/dev/null || true)

    # ── Pattern 20: cleanup() must have /proc FD sweep ─────────────
    local cleanup_body
    cleanup_body=$(sed -n '/^cleanup()/,/^}/p' "$file" 2>/dev/null || true)
    if [[ -n "$cleanup_body" ]]; then
        if echo "$cleanup_body" | grep -qE 'pkill.*-P'; then
            if ! echo "$cleanup_body" | grep -qE '/proc/.*/fd'; then
                fail "$basename — cleanup() has pkill but missing /proc FD sweep (Layer 3)"
                echo -e "    → fix: add /proc/PID/fd scan after pkill to catch reparented orphans" >&2
                ((file_fails++)) || true
            fi
        fi
    fi

    if [[ $file_fails -eq 0 ]]; then
        pass "$basename — clean"
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# LAYER 1: STATIC SCAN — scans all project .sh files
# ═══════════════════════════════════════════════════════════════════════════════
run_static_scan() {
    echo -e "\n${BLUE}${BOLD}═══ LAYER 1: Static PTY Safety Scan ═══${NC}"

    local files=()
    while IFS= read -r f; do
        files+=("$f")
    done < <(find "$PROJECT_ROOT" -maxdepth 1 -name "*.sh" -type f)
    while IFS= read -r f; do
        files+=("$f")
    done < <(find "$PROJECT_ROOT/scripts" -maxdepth 1 -name "*.sh" -type f 2>/dev/null | grep -v 'test-pty-safety\.sh')
    while IFS= read -r f; do
        files+=("$f")
    done < <(find "$PROJECT_ROOT/scripts/ai-tools" -maxdepth 1 -name "*.sh" -type f 2>/dev/null)
    while IFS= read -r f; do
        files+=("$f")
    done < <(find "$PROJECT_ROOT/.git-hooks" -type f 2>/dev/null | grep -v 'sample')

    if [[ ${#files[@]} -eq 0 ]]; then
        warn "No .sh files found to scan"
        return
    fi

    echo -e "  ${BLUE}Scanning ${#files[@]} files...${NC}"

    for file in "${files[@]}"; do
        scan_single_file "$file"
    done

    # ── Cross-file check: trap cleanup must exist in main scripts ─────
    for main_script in "$PROJECT_ROOT/run-tests.sh"; do
        [[ -f "$main_script" ]] || continue
        local bn
        bn="$(basename "$main_script")"
        if ! grep -q 'trap.*cleanup.*EXIT' "$main_script" 2>/dev/null && \
           ! grep -q 'trap.*EXIT' "$main_script" 2>/dev/null; then
                fail "$bn — missing 'trap cleanup EXIT' handler"
        fi
    done

    # ── Pattern 11: cleanup() must redirect FDs before podman ─────────
    for file in "${files[@]}"; do
        local basename
        basename="$(basename "$file")"
        local cleanup_body
        cleanup_body=$(sed -n '/^cleanup()/,/^}/p' "$file" 2>/dev/null || true)
        [[ -z "$cleanup_body" ]] && continue

        if echo "$cleanup_body" | grep -qE 'podman|docker|dc_detached'; then
            if ! echo "$cleanup_body" | grep -qE 'exec [0-9]*>|exec [0-9]*>>'; then
                fail "$basename — cleanup() spawns podman/docker but missing 'exec' FD redirect (PTY FD leak risk)"
            else
                pass "$basename — cleanup() has FD isolation before podman"
            fi
        fi
    done
}

# ═══════════════════════════════════════════════════════════════════════════════
# QUICK CHECK: Check a single command string for PTY-unsafe patterns
# Usage: check_command_string "podman-compose exec apiserver cargo test"
# ═══════════════════════════════════════════════════════════════════════════════
check_command_string() {
    local cmd="$1"
    local tmpfile
    tmpfile="$(mktemp /tmp/pty-check-XXXXXX.sh)"
    echo "$cmd" > "$tmpfile"
    scan_single_file "$tmpfile"
    rm -f "$tmpfile"
}

# ═══════════════════════════════════════════════════════════════════════════════
# LAYER 2: RUNTIME PTY HANG TEST
# Runs a script inside a real PTY (via `script` command) and verifies it exits
# within the timeout. If it hangs, that means PTY FD leak.
# ═══════════════════════════════════════════════════════════════════════════════
run_runtime_test() {
    echo -e "\n${BLUE}${BOLD}═══ LAYER 2: Runtime PTY Hang Test ═══${NC}"

    # Prevent recursive loops: run-tests.sh calls us
    export _PTY_CHECK_DONE=1

    if ! command -v script &>/dev/null; then
        warn "'script' command not found — skipping runtime tests"
        return
    fi

    # Test 1: run-tests.sh --help must exit cleanly in a PTY
    _pty_test "run-tests.sh --help" \
        "$PROJECT_ROOT/run-tests.sh" "--help" 10

    # Test 2: run-tests.sh --dry-run must exit cleanly
    _pty_test "run-tests.sh --api --dry-run" \
        "$PROJECT_ROOT/run-tests.sh" "--api --dry-run" 15

    # Test 3: Argument word-splitting test
    _arg_quoting_test

    # Test 4: Orphan process leak detection
    _orphan_leak_test

    # Test 5: Signal cleanup — verify SIGTERM triggers clean exit
    _signal_cleanup_test

    # Test 6: PTY FD sweep — verify orphans holding PTY FDs get killed
    _pty_fd_sweep_test
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
# This validates that argument quoting works correctly in run-tests.sh.
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
        if pgrep -f "podman.*leeattend-test" >/dev/null 2>&1; then
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

# Verify that the PTY FD sweep (Layer 3b) can detect and kill orphan processes
# that hold PTY file descriptors.
_pty_fd_sweep_test() {
    echo -e "  ${BLUE}Testing: PTY FD sweep kills orphan holders${NC}"

    # Check for PTY BEFORE spawning anything — avoid leaving orphans
    local our_tty
    our_tty=$(ps -o tty= -p $$ 2>/dev/null | tr -d ' ')
    if [[ -z "$our_tty" || "$our_tty" == "?" ]]; then
        pass "PTY FD sweep — skipped (no PTY detected, likely CI/agent)"
        return
    fi

    # If stdout is redirected to a file (not a terminal), child processes
    # inherit file FDs instead of PTY FDs — detection is meaningless.
    if [[ ! -t 1 ]]; then
        pass "PTY FD sweep — skipped (stdout redirected, FDs not PTY-attached)"
        return
    fi

    # Spawn a sleep that INTENTIONALLY inherits our PTY FDs (simulates leak).
    # Do NOT use </dev/null here — we need the process to hold PTY FDs.
    sleep 300 &
    local orphan_pid=$!

    # Verify it's alive
    if ! kill -0 $orphan_pid 2>/dev/null; then
        warn "PTY FD sweep — could not spawn test orphan"
        return
    fi

    # Test the DETECTION mechanism: verify /proc/PID/fd shows PTY FDs
    local detected=false
    if ls -la /proc/"$orphan_pid"/fd 2>/dev/null | grep -q "/dev/pts/"; then
        detected=true
    fi

    # Test the KILL mechanism: target ONLY the specific orphan PID
    # (A blanket sweep on the PTY would kill the test script itself)
    kill -KILL "$orphan_pid" 2>/dev/null || true
    wait "$orphan_pid" 2>/dev/null || true
    sleep 0.1

    if [[ "$detected" == "true" ]] && ! kill -0 $orphan_pid 2>/dev/null; then
        pass "PTY FD sweep — orphan detected via /proc/$orphan_pid/fd and killed"
    elif [[ "$detected" != "true" ]]; then
        fail "PTY FD sweep — /proc/$orphan_pid/fd did not detect PTY FDs"
        kill -KILL $orphan_pid 2>/dev/null || true
    else
        fail "PTY FD sweep — orphan PID $orphan_pid survived kill"
        kill -KILL $orphan_pid 2>/dev/null || true
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# SELF-TEST: Verify all patterns detect correctly
# Usage: test-pty-safety.sh --self-test
# ═══════════════════════════════════════════════════════════════════════════════
run_self_test() {
    local SELF="$0"
    local ST_PASS=0 ST_FAIL=0 ST_TOTAL=0

    _st_expect_unsafe() {
        local desc="$1" cmd="$2"
        ((ST_TOTAL++)) || true
        if "$SELF" --command="$cmd" >/dev/null 2>&1; then
            echo -e "  ${RED}✗ FAIL${NC} $desc — expected UNSAFE but got SAFE"
            ((ST_FAIL++)) || true
        else
            echo -e "  ${GREEN}✓${NC} $desc"; ((ST_PASS++)) || true
        fi
    }
    _st_expect_safe() {
        local desc="$1" cmd="$2"
        ((ST_TOTAL++)) || true
        if "$SELF" --command="$cmd" >/dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $desc"; ((ST_PASS++)) || true
        else
            echo -e "  ${RED}✗ FAIL${NC} $desc — expected SAFE but got UNSAFE"
            ((ST_FAIL++)) || true
        fi
    }
    _st_expect_fix() {
        local desc="$1" cmd="$2" expected="$3"
        ((ST_TOTAL++)) || true
        local out; out=$("$SELF" --command="$cmd" 2>&1 || true)
        if echo "$out" | grep -q "$expected"; then
            echo -e "  ${GREEN}✓${NC} $desc — fix message ok"; ((ST_PASS++)) || true
        else
            echo -e "  ${RED}✗ FAIL${NC} $desc — missing: '$expected'"
            ((ST_FAIL++)) || true
        fi
    }
    _st_file_unsafe() {
        local desc="$1" content="$2"
        local tmp; tmp="$(mktemp /tmp/pty-st-XXXXXX.sh)"; echo "$content" > "$tmp"
        ((ST_TOTAL++)) || true
        if "$SELF" --check-file="$tmp" >/dev/null 2>&1; then
            echo -e "  ${RED}✗ FAIL${NC} $desc — expected UNSAFE"; ((ST_FAIL++)) || true
        else
            echo -e "  ${GREEN}✓${NC} $desc"; ((ST_PASS++)) || true
        fi
        rm -f "$tmp"
    }
    _st_file_safe() {
        local desc="$1" content="$2"
        local tmp; tmp="$(mktemp /tmp/pty-st-XXXXXX.sh)"; echo "$content" > "$tmp"
        ((ST_TOTAL++)) || true
        if "$SELF" --check-file="$tmp" >/dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $desc"; ((ST_PASS++)) || true
        else
            echo -e "  ${RED}✗ FAIL${NC} $desc — expected SAFE"; ((ST_FAIL++)) || true
        fi
        rm -f "$tmp"
    }

    echo -e "${BLUE}${BOLD}🧪 PTY Safety Scanner — Self-Test${NC}\n"

    echo -e "${BOLD}P1: bare \`&\`${NC}"
    _st_expect_unsafe "bare &"                  "./build.sh &"
    _st_expect_safe   "& with </dev/null"       "./build.sh </dev/null &"
    _st_expect_safe   "no background"           "./build.sh"

    echo -e "${BOLD}P16: subshell \`&\`${NC}"
    _st_expect_unsafe "subshell &"              "(cd /tmp && ./build.sh &)"
    _st_expect_safe   "subshell & safe"         "(cd /tmp && ./build.sh </dev/null &)"

    echo -e "${BOLD}P2: container </dev/null${NC}"
    _st_expect_unsafe "podman bare"             "podman run myimage"
    _st_expect_unsafe "docker bare"             "docker run myimage"
    _st_expect_safe   "podman safe"             "podman run myimage </dev/null"

    echo -e "${BOLD}P4: git commit${NC}"
    _st_expect_unsafe "git commit bare"         "git commit -m 'test'"
    _st_expect_safe   "git commit safe"         "git commit --no-gpg-sign -m 'test'"

    echo -e "${BOLD}P6: nohup${NC}"
    _st_expect_unsafe "nohup"                   "nohup ./server.sh &"
    _st_expect_safe   "no nohup"                "./server.sh </dev/null &"

    echo -e "${BOLD}P7: disown (warning)${NC}"
    _st_expect_safe   "disown warn-only"        "disown %1"

    echo -e "${BOLD}P8: coproc${NC}"
    _st_expect_unsafe "coproc"                  "coproc myproc { sleep 10; }"

    echo -e "${BOLD}P9: setsid${NC}"
    _st_expect_unsafe "setsid bare"             "setsid ./daemon.sh"
    _st_expect_safe   "setsid safe"             "setsid ./daemon.sh </dev/null >/dev/null 2>&1"

    echo -e "${BOLD}P12: compose exec -T${NC}"
    _st_expect_unsafe "exec no -T"              "podman-compose exec apiserver cargo test"
    _st_expect_safe   "exec with -T"            "podman-compose exec -T apiserver cargo test </dev/null"

    echo -e "${BOLD}P14: EOF-buffering pipes (warning)${NC}"
    _st_expect_safe   "tee warn-only"           "./run.sh | tee out.log"
    _st_expect_safe   "tail warn-only"          "./run.sh | tail -20"
    _st_expect_safe   "wc warn-only"            "./run.sh | wc -l"
    _st_expect_safe   "sort warn-only"          "./run.sh | sort"
    _st_expect_safe   "head warn-only"          "./run.sh | head -5"
    _st_expect_safe   "cat no-warn"             "./run.sh | cat"

    echo -e "${BOLD}P15: interactive${NC}"
    _st_expect_unsafe "vim"                     "vim file.txt"
    _st_expect_unsafe "nano"                    "nano config.yml"
    _st_expect_safe   "cat (ok)"                "cat file.txt"

    echo -e "${BOLD}P17: read/select${NC}"
    _st_expect_unsafe "read -p"                 "read -p 'Name: ' NAME"
    _st_expect_unsafe "select"                  "select opt in a b; do echo \$opt; done"
    _st_expect_safe   "read -t (timeout)"       "read -t 5 -p 'x' NAME"

    echo -e "${BOLD}P18: sudo${NC}"
    _st_expect_unsafe "sudo bare"               "sudo systemctl restart nginx"
    _st_expect_safe   "sudo -n"                 "sudo -n systemctl restart nginx"
    _st_expect_safe   "sudo --non-interactive"  "sudo --non-interactive apt update"

    echo -e "${BOLD}P19: ssh${NC}"
    _st_expect_unsafe "ssh bare"                "ssh user@host ls"
    _st_expect_safe   "ssh BatchMode"           "ssh -o BatchMode=yes user@host ls"
    _st_expect_safe   "ssh-keygen"              "ssh-keygen -t rsa"

    echo -e "${BOLD}Fix messages${NC}"
    _st_expect_fix "container" "podman run foo"  "fix:"
    _st_expect_fix "nohup"     "nohup ./x.sh &"  "fix:"
    _st_expect_fix "sudo"      "sudo reboot"     "fix:"
    _st_expect_fix "ssh"       "ssh root@host"    "fix:"
    _st_expect_fix "read"      "read -p 'x' Y"   "fix:"
    _st_expect_fix "vim"       "vim foo.txt"      "fix:"

    echo -e "${BOLD}--check-file mode${NC}"
    _st_file_unsafe "file bare &" '#!/bin/bash
./long-task.sh &
wait'
    _st_file_safe "file safe &" '#!/bin/bash
./long-task.sh </dev/null &
BG_PID=$!
wait $BG_PID'
    _st_file_safe "clean file" '#!/bin/bash
echo "hello"
cargo test >out.log 2>&1'

    echo -e "${BOLD}P20: cleanup /proc FD sweep${NC}"
    _st_file_unsafe "cleanup missing /proc sweep" 'cleanup() {
    pkill -TERM -P $$ 2>/dev/null || true
    pkill -KILL -P $$ 2>/dev/null || true
    exit 0
}'
    _st_file_safe "cleanup with /proc sweep" 'cleanup() {
    pkill -TERM -P $$ 2>/dev/null || true
    if ls -la /proc/$_pid/fd 2>/dev/null | grep -q "/dev/pts/"; then
        kill -KILL $_pid 2>/dev/null || true
    fi
    exit 0
}'

    echo -e "${BOLD}Edge cases${NC}"
    _st_file_safe "continuation line" 'podman exec mycontainer \
  cargo test </dev/null'
    _st_file_safe "comments skipped" '#!/bin/bash
# nohup ./server.sh &
# podman exec foo
echo "safe"'

    echo -e "\n${BLUE}${BOLD}═══ Self-Test Summary ═══${NC}"
    echo -e "  Total: $ST_TOTAL  ${GREEN}Pass: $ST_PASS${NC}  ${RED}Fail: $ST_FAIL${NC}"
    if [[ $ST_FAIL -gt 0 ]]; then
        echo -e "${RED}${BOLD}❌ Self-test FAILED${NC}"; exit 1
    else
        echo -e "${GREEN}${BOLD}✅ All $ST_TOTAL self-tests passed${NC}"; exit 0
    fi
}

# ═══════════════════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════════════════
main() {
    local run_static=true
    local run_runtime=true
    local check_cmd=""
    local check_file=""

    for arg in "$@"; do
        case "$arg" in
            --static)    run_runtime=false ;;
            --runtime)   run_static=false ;;
            --command=*) check_cmd="${arg#--command=}" ;;
            --check-file=*) check_file="${arg#--check-file=}" ;;
            --self-test) run_self_test ;;
            --help)
                cat <<'HELP'
Usage: test-pty-safety.sh [OPTIONS]

Modes:
  (no flags)              Run full suite (static + runtime)
  --static                Static scan only (fast, no Docker needed)
  --runtime               Runtime PTY hang tests only
  --command="CMD"          Check a single command string for PTY-unsafe patterns
  --check-file=PATH       Check a single script file for PTY-unsafe patterns
  --self-test             Run self-test to verify all patterns work correctly
  --help                  Show this help

Examples:
  ./scripts/test-pty-safety.sh --command="podman exec apiserver cargo test"
  ./scripts/test-pty-safety.sh --check-file=./my-script.sh
  ./scripts/test-pty-safety.sh --static
  ./scripts/test-pty-safety.sh --self-test

Exit code: 0 = all pass, 1 = failures found
HELP
                exit 0
                ;;
        esac
    done

    # Handle --command= (also accept positional: --command "CMD")
    if [[ -z "$check_cmd" ]]; then
        local prev=""
        for arg in "$@"; do
            if [[ "$prev" == "--command" ]]; then
                check_cmd="$arg"
                break
            fi
            prev="$arg"
        done
    fi
    # Handle --check-file positional
    if [[ -z "$check_file" ]]; then
        local prev=""
        for arg in "$@"; do
            if [[ "$prev" == "--check-file" ]]; then
                check_file="$arg"
                break
            fi
            prev="$arg"
        done
    fi

    # ── Quick check modes (no banner, minimal output) ─────────────────
    if [[ -n "$check_cmd" ]]; then
        check_command_string "$check_cmd"
        if [[ $FAIL -gt 0 ]]; then
            echo -e "${RED}✗ UNSAFE — $FAIL violation(s) found${NC}" >&2
            exit 1
        else
            echo -e "${GREEN}✓ SAFE — no PTY violations detected${NC}"
            exit 0
        fi
    fi

    if [[ -n "$check_file" ]]; then
        if [[ ! -f "$check_file" ]]; then
            echo "Error: file not found: $check_file" >&2
            exit 2
        fi
        scan_single_file "$check_file"
        if [[ $FAIL -gt 0 ]]; then
            echo -e "${RED}✗ UNSAFE — $FAIL violation(s) found${NC}" >&2
            exit 1
        else
            echo -e "${GREEN}✓ SAFE — no PTY violations detected${NC}"
            exit 0
        fi
    fi

    # ── Full suite mode ───────────────────────────────────────────────
    echo -e "${BLUE}${BOLD}🔒 PTY Safety Test Suite${NC}"
    echo -e "${BLUE}Testing scripts for PTY file descriptor leaks...${NC}"

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


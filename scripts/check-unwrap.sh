#!/bin/bash
# check-unwrap.sh — CI gate against .unwrap() / .expect() in production code
#
# Uses count_unwrap.py for accurate detection that properly skips
# #[cfg(test)] mod tests { ... } blocks and test-only files.
#
# Strategy:
#   1. Run count_unwrap.py to get accurate production counts
#   2. Compare against the allow-list baseline
#   3. FAIL if any file has MORE violations than baseline
#   4. PASS if counts are equal or lower (progress!)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ALLOW_FILE="$PROJECT_ROOT/apiserver/clippy_unwrap_allow.toml"
COUNTER="$SCRIPT_DIR/count_unwrap.py"
APISERVER_SRC="$PROJECT_ROOT/apiserver/src"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[FAIL]${NC} $*"; }

# ---------------------------------------------------------------------------
# 1. Run Python counter for accurate production-only counts
# ---------------------------------------------------------------------------
if [[ ! -f "$COUNTER" ]]; then
    log_error "Counter script not found: $COUNTER"
    exit 1
fi

COUNTS_JSON=$("$COUNTER" "$APISERVER_SRC" --json 2>/dev/null)
if [[ $? -ne 0 ]]; then
    log_error "Counter script failed"
    exit 1
fi

# ---------------------------------------------------------------------------
# 2. Parse allow-list TOML
# ---------------------------------------------------------------------------
declare -A ALLOWED_UNWRAP
declare -A ALLOWED_EXPECT

parse_allow_list() {
    if [[ ! -f "$ALLOW_FILE" ]]; then
        log_error "Allow-list not found: $ALLOW_FILE"
        exit 1
    fi
    local section=""
    while IFS= read -r line; do
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ -z "$line" ]] && continue
        if [[ "$line" =~ ^\[(.*)\]$ ]]; then
            section="${BASH_REMATCH[1]}"
            continue
        fi
        if [[ "$line" =~ ^[[:space:]]*\"([^\"]+)\"[[:space:]]*=[[:space:]]*([0-9]+) ]]; then
            local key="${BASH_REMATCH[1]}"
            local val="${BASH_REMATCH[2]}"
            case "$section" in
                unwrap) ALLOWED_UNWRAP["$key"]="$val" ;;
                expect) ALLOWED_EXPECT["$key"]="$val" ;;
            esac
        fi
    done < "$ALLOW_FILE"
}

# ---------------------------------------------------------------------------
# 3. Extract counts from JSON and compare
# ---------------------------------------------------------------------------
check_category() {
    local label="$1"
    local json_key="$2"
    local -n allowed_ref="$3"

    local violations=0
    local new_files=()
    local increased_files=()

    # Parse JSON: {"unwrap": {"file.rs": N, ...}, "expect": {...}}
    local entries
    entries=$(echo "$COUNTS_JSON" | python3 -c "
import json, sys
data = json.load(sys.stdin)
for path, count in data.get('$json_key', {}).items():
    print(f'{count} {path}')
" 2>/dev/null)

    local total=0
    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        local count="${line%% *}"
        local file="${line#* }"
        local allowed="${allowed_ref[$file]:-0}"
        total=$((total + count))

        if [[ "$allowed" -eq 0 ]] && [[ "$count" -gt 0 ]]; then
            new_files+=("$file ($count)")
            violations=1
        elif [[ "$count" -gt "$allowed" ]]; then
            increased_files+=("$file (was $allowed, now $count)")
            violations=1
        elif [[ "$count" -lt "$allowed" ]]; then
            log_info "$label: $file decreased $allowed → $count ✓"
        fi
    done <<< "$entries"

    echo "Total ${label}: $total"

    if [[ ${#new_files[@]} -gt 0 ]]; then
        log_error "$label: NEW files with violations (not in allow-list):"
        for f in "${new_files[@]}"; do
            echo "       $f"
        done
    fi

    if [[ ${#increased_files[@]} -gt 0 ]]; then
        log_error "$label: files with INCREASED violations:"
        for f in "${increased_files[@]}"; do
            echo "       $f"
        done
    fi

    return $violations
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
echo "=== Unwrap/Expect CI Gate ==="
echo "Using: $COUNTER (accurately skips #[cfg(test)] blocks)"
echo "Allow-list: $ALLOW_FILE"
echo ""

parse_allow_list

FAILED=0
check_category ".unwrap()" "unwrap" ALLOWED_UNWRAP || FAILED=1
check_category ".expect()"  "expect"  ALLOWED_EXPECT || FAILED=1

echo ""
if [[ "$FAILED" -eq 0 ]]; then
    log_info "All unwrap/expect counts are within allowed baseline. ✓"
    exit 0
else
    log_error "Unwrap/expect violations detected!"
    echo ""
    echo "   If you INTENTIONALLY added a new .unwrap()/.expect():"
    echo "   → DON'T. Use AppError with proper error handling instead."
    echo "   → See: .agent/rules/coding-standards.md Section 3"
    echo ""
    echo "   If this is a legitimate exception:"
    echo "   → Update apiserver/clippy_unwrap_allow.toml"
    echo "   → Justify in PR description"
    exit 1
fi

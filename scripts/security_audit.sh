#!/bin/bash
# Security Audit Suite: Single-organization isolation, RBAC, PII masking, and auth boundaries.
# Invariants are corrected against current Go/Pgx architecture and the approved
# single-organization cutover contract. This script is a static source audit; runtime
# authorization behavior is covered by handler tests and isolated API smoke.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

API_DIR="apiserver/internal/api"
REPO_DIR="apiserver/internal/repository"

echo "=========================================================="
echo "LeeAttend Single-Organization Security Audit"
echo "=========================================================="

FAILED=0
TOTAL_CHECKS=0
PASSED_CHECKS=0

pass_check() {
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
    echo "[PASS] $1"
}

fail_check() {
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    FAILED=$((FAILED + 1))
    echo "[FAIL] $1: $2" >&2
}

echo ""
echo "--- Static check 1: request-derived organization selectors ---"
# Bounded literal scan only; this does not prove routing or authorization behavior.
SCOPE_RELAYS=$(grep -rn -E 'PathValue\("(ws_id|workspace_id|organization_id)"\)|Header\.Get\("X-Workspace-Id"\)|Query\(\)\.Get\("workspace_id"\)' "$API_DIR" --include='*.go' | grep -v '_test.go' || true)
if [ -n "$SCOPE_RELAYS" ]; then
    fail_check "Request-scope selector pattern" "$SCOPE_RELAYS"
else
    pass_check "No scanned request-scope selector literals found"
fi

echo ""
echo "--- Static check 2: canonical middleware and route references ---"
ROUTES_SRC="apiserver/routes.go"
LEGACY_ROUTE_ALIASES=$(grep -n -E '"/(api/auth|api/me|api/users|v1/|ws/)' "$ROUTES_SRC" || true)
if grep -q 'AuthMiddlewareWithRevocation' "$ROUTES_SRC" && [ -z "$LEGACY_ROUTE_ALIASES" ]; then
    pass_check "JWT middleware present and legacy route aliases absent from $ROUTES_SRC"
else
    fail_check "Canonical middleware and routes" "Legacy aliases or authentication middleware mismatch in $ROUTES_SRC: ${LEGACY_ROUTE_ALIASES:-missing middleware}"
fi

echo ""
echo "--- Static check 3: sensitive-field log-call patterns ---"
LEAKED_LOG_PATTERNS=$(grep -rn -E 'log\.(Print|Info|Warn|Error).*(password_hash|bank_account|national_id)' apiserver --include='*.go' | grep -v '_test.go' || true)
if [ -n "$LEAKED_LOG_PATTERNS" ]; then
    fail_check "Sensitive-field log-call pattern" "$LEAKED_LOG_PATTERNS"
else
    pass_check "No scanned sensitive-field log-call patterns found"
fi

echo ""
echo "--- Static check 4: password-hasher implementation reference ---"
HASHER_SRC="apiserver/internal/auth/hasher.go"
if [ -f "$HASHER_SRC" ] && grep -q 'argon2.IDKey' "$HASHER_SRC"; then
    pass_check "argon2.IDKey reference found in $HASHER_SRC"
else
    fail_check "Password hasher reference" "argon2.IDKey reference missing from $HASHER_SRC"
fi

echo ""
echo "--- Static check 5: token-revocation implementation references ---"
REVOCATION_SRC="apiserver/internal/infra/redis/revocation.go"
if [ -f "$REVOCATION_SRC" ] && grep -q 'IsSignatureRevoked' "$REVOCATION_SRC" && grep -q 'UserRevokedAt' "$REVOCATION_SRC" && grep -q 'SetUserRevocation' "$REVOCATION_SRC"; then
    pass_check "Expected token-revocation references found in $REVOCATION_SRC"
else
    fail_check "Token-revocation references" "One or more expected references missing from $REVOCATION_SRC"
fi

echo ""
echo "--- Static check 6: forbidden JWT-field name references ---"
JWT_SRC="apiserver/internal/auth/jwt.go"
if [ -f "$JWT_SRC" ] && ! grep -Eq 'workspace_id|organization_id|Email|FullName' "$JWT_SRC"; then
    pass_check "No scanned organization-selector or PII field-name references in $JWT_SRC"
else
    fail_check "JWT field-name reference" "Forbidden field-name reference found or $JWT_SRC missing"
fi

echo ""
echo "--- Static check 7: legacy SQL predicate patterns ---"
LEGACY_SCOPE_SQL=$(grep -rn -E 'WHERE[^`]*workspace_id[[:space:]]*=[[:space:]]*\$|workspace_id[[:space:]]*=[[:space:]]*ANY' "$REPO_DIR" --include='*.go' | grep -v '_test.go' || true)
if [ -n "$LEGACY_SCOPE_SQL" ]; then
    fail_check "Legacy SQL predicate pattern" "$LEGACY_SCOPE_SQL"
else
    pass_check "No scanned legacy SQL predicate patterns found"
fi

echo ""
echo "NOTE: These bounded source-pattern checks do not prove fail-closed behavior, complete PII safety, middleware order, or runtime authorization. Verify those with code review and behavior tests on disposable infrastructure."

echo ""
echo "=========================================================="
echo "Static Source Audit Summary: $PASSED_CHECKS/$TOTAL_CHECKS checks passed."
if [ $FAILED -gt 0 ]; then
    echo "STATUS: REVIEW REQUIRED ($FAILED static pattern checks failed)"
    exit 1
else
    echo "STATUS: STATIC CHECKS PASSED (not a runtime security proof)"
    exit 0
fi

echo ""
echo "=========================================================="
echo "Security Audit Summary: $PASSED_CHECKS/$TOTAL_CHECKS checks passed."
if [ $FAILED -gt 0 ]; then
    echo "STATUS: FAILED ($FAILED errors found)"
    exit 1
else
    echo "STATUS: PASSED (Zero security violations detected)"
    exit 0
fi

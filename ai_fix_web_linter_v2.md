# Task: Fix Web Linter Errors (V2)

## 🛡️ Rule Compliance Check
- **Rule ID: 4.3 (Testing Standards)**: The task is NOT complete until `./test.sh web --lint-only` returns success (0 errors).
- **Rule ID: 4.8 (PTY Safety)**: All background commands must use `</dev/null`.
- **Rule ID: 4.11 (Hermetic Toolbox)**: Use `./scripts/_npm.sh` or `./test.sh` for all web-related operations.
- **Rule ID: 5.4 (Verification Pipeline)**: Verify changes using the official linting pipeline.

## 📋 Objectives
Systematically resolve all ESLint and TypeScript linting violations in the `webdashboard` module to achieve a zero-warning build state.

## 🏗️ Phase 1: Assessment & Auto-fix
- [ ] Run `./test.sh web --lint-only` to get the baseline report.
- [ ] Identify high-frequency errors that can be auto-fixed.
- [ ] Run `./scripts/_npm.sh run lint:fix --prefix webdashboard` to resolve styling and sorting issues.

## 🛠️ Phase 2: Manual Remediation
- [ ] **Type Safety**: Fix `no-explicit-any`, `no-non-null-assertion`, and type mismatches.
- [ ] **React Best Practices**: Fix `react-hooks/exhaustive-deps`, `react/jsx-no-leaked-render`, and component complexity.
- [ ] **Structural Refactoring**: Decompose components that exceed `max-lines-per-function` or `complexity` limits.

## 🧪 Phase 3: Final Verification
- [ ] Ensure `tsc --noEmit` (type-check) passes.
- [ ] Run a final `./test.sh web --lint-only` and confirm zero errors.
- [ ] Verify that no visual regressions were introduced during refactoring.

## 📅 Timeline & Progress
- **Start Date**: 2026-05-03
- **Status**: Assessment in progress

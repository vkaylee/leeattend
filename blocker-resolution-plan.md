# LeeAttend Blocker Resolution Plan

Status: Awaiting approval

## Goal
Make standalone cutover verifiable end-to-end by restoring a trustworthy API contract pipeline, fixing frontend quality gates, and removing stale Rust-only test orchestration without weakening assertions or hiding failures.

## Constraints and interfaces
- Go is the only active API runtime; Rust is archival and MUST NOT remain a required verification dependency.
- `webdashboard/src/types/api-generated.ts` is generated and MUST NOT be hand-edited.
- API contract removal is breaking. Contract source, generated clients, compatibility evidence, and consumer tests MUST move together under approved change control.
- Existing failing tests MUST be fixed at their root cause; no skips, retries, warning suppression, or weakened assertions.
- Use `./leedevkit`, `scripts/_go-gates.sh`, and repository wrappers only.
- Preserve security, migration, and standalone authorization gates already passing.

## Tasks

- [ ] **Recover or establish OpenAPI source of truth.** Search repository history and archived artifacts for the last usable OpenAPI source/generator; if recoverable, restore it as the contract input and remove tenant/system-role/DB-cluster paths and schemas there. If unrecoverable, stop and obtain maintainer approval for a minimal Go-owned OpenAPI source rather than deriving a contract from generated TypeScript. Verify source diff, removed-path absence, retained system-route presence, schema validation, and generated-file diff.

- [ ] **Regenerate and validate frontend API types.** Run the canonical generator from the restored source, confirm `api-generated.ts` removes obsolete operations/schemas, update only legitimate generated re-exports/consumers, and run contract compatibility checks. Record the breaking-change migration note and generated hash/source citation.

- [ ] **Fix the 13 blocking frontend lint errors only.** Group work by DSM launcher, table-density UI, approvals, i18n config, and ESS clock. Preserve behavior and accessibility; use native interactive elements or complete keyboard semantics where required. Do not bulk-fix the 1,138 warnings in this blocker pass. Verify `./leedevkit test web --lint-only` reaches zero errors; warnings remain separately tracked.

- [ ] **Repair frontend unit-test fixtures.** Fix the `Response` mock contract in `src/lib/__tests__/api.test.ts` so all methods consumed by `openapi-fetch` exist. Fix `employees-page.test.tsx` module mocking/import setup so the test uses the mocked API namespace. Verify focused files first, then the complete web unit phase; retain assertions and add no skips.

- [ ] **Diagnose and fix E2E failures by failure cluster.** First reproduce dashboard, responsive, branding, and overtime groups separately with traces/error contexts. Check shared bootstrap/API mocks, viewport breakpoints, route readiness, and state isolation before changing product code. Fix the shared root cause, then repair individual flow contracts. Address the test image’s missing `git` only if it affects version/build behavior. Verify all E2E projects, including retries disabled during diagnosis.

- [ ] **Remove stale Rust test orchestration.** Trace `rust-backend`, `cargo`, `db_system`, and integration healthcheck phases to their LeeDevKit/compose source. Update the active API test target to run Go lint/unit/integration gates with PostgreSQL/Redis migrations, not archival Rust jobs. Keep historical Rust evidence outside the active gate. Verify `./leedevkit test api` and `./leedevkit test all` execute the intended services and fail on real Go/API regressions.

- [ ] **Run final cross-surface verification and review.** Run security audit, migration fixture validation, canonical Go gates, frontend lint/typecheck/unit/build, E2E, contract generation/validation, compose validation, and standalone smoke checks. Review the complete diff for generated artifacts, secrets, stale routes, and accidental warning suppression. Obtain required code review before merge.

## Test impact matrix
- Changed behavior: generated API contract removes obsolete control-plane operations; frontend tests and E2E harness become reliable; active API gate switches from Rust compatibility jobs to Go.
- Existing behavior: retained SMTP, health, monitor, organization-admin RBAC, dashboard, responsive navigation, branding, and overtime flows remain unchanged.
- Boundaries: missing OpenAPI source, generated drift, missing mock response methods, absent mocked namespace, viewport-specific layout, stale auth/API state, missing database/Redis readiness, and unavailable Rust tooling.
- Failures: generator mismatch, contract incompatibility, lint accessibility violations, API mock failures, browser startup/build failure, migration/healthcheck timeout, and real Go integration regression.
- Security: no bypass, no raw secrets in logs or fixtures, retained authorization negative paths remain enforced.
- Compatibility: removed API paths are an approved breaking change; generated clients and migration documentation must agree.
- Operations: test containers must have required build tools, deterministic healthchecks, bounded timeouts, and useful failure artifacts.

## Done when
- [ ] Current OpenAPI source is identified and generated contract no longer exposes removed control-plane paths, or an explicit maintainer-approved source decision blocks implementation.
- [ ] Frontend lint has zero errors; typecheck, unit tests, build, and E2E pass.
- [ ] Go/API full gate runs without Rust-only jobs and passes unit/integration verification.
- [ ] Existing standalone security, migration, authorization, and Go gates remain green.
- [ ] No generated files are hand-edited, no tests are weakened/skipped, and final diff receives required review.

## Approval decision
Recommended path: recover the historical OpenAPI source first; only create a new Go-owned source if recovery fails and maintainers approve the contract ownership change. Approve this plan before implementation because it changes generated API contracts, CI/test orchestration, and multiple user-visible frontend surfaces.

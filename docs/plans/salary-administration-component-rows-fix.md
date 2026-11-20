# Salary Administration Component Rows Fix

Status: Approved
Updated: 2026-11-12

## Goal

Sửa `/salary-management` thành bảng các **khoản lương/phụ cấp đã cấu hình**, ví dụ lương cơ bản của nhân sự A là `5.000` có hiệu lực từ `2026-10-01`; không hiển thị nhân sự chưa có khoản trong bảng mặc định.

## Read

- `coding-standards.md`
- `testing-standards.md`
- `development-workflow.md`
- `api-governance.md`
- `access-control.md`
- `data-governance.md`
- `database-rules.md`
- `design-rules.md`
- `execution-wrappers.md`

## Rule IDs

- `data-governance.md §1–2`: salary amount là T2; collection response cần được phê duyệt ngoại lệ hẹp, role-gated, no-store, audit đọc.
- `access-control.md §1–2`: handler kiểm quyền trước business logic; server scope là authority.
- `api-governance.md §1–3, §5–6`: contract-first, pagination, stable ordering, compatibility, contract tests.
- `database-rules.md §1–4`: parameterized SQL, zero N+1, index/filter integrity, migration rollback nếu cần.
- `development-workflow.md §2–7`: Test Impact Matrix, regression/edge/auth/failure/integration evidence.
- `design-rules.md §1–4`: component dùng lại, loading/empty/error, responsive, accessibility.

## Contract decision

- **Default table unit:** one row per configured salary component, not one row per employee.
- **Default visibility:** only components effective on or before selected `as_of`; future and ended components remain available through explicit status/history filters.
- **Employment identity:** every row carries `employment_id`; never merge rehire, employer, or legal-entity boundaries.
- **Amount:** return `amount`, `currency`, `unit`, and `effective_from` only after confirming the narrow T2 collection exception. If policy exception is rejected, keep amount out of collection and add a permission-gated detail view; do not bypass policy with one request per row.
- **Unconfigured employees:** excluded from default component table. Keep a separate explicit `unconfigured=true` operational view/endpoint only if HR still needs a missing-configuration worklist; that view must not pretend to contain configured components.
- **Compatibility:** inspect current endpoint consumers. If no released external consumer exists, replace current metadata semantics in one coordinated backend/frontend change. Otherwise add a versioned/component-specific contract and deprecate metadata semantics without silently changing released behavior.

## Test Impact Matrix

| Dimension | Required protection |
|---|---|
| Changed behavior | Configured component A/amount/effective date renders; employee with zero components absent by default. |
| Existing behavior | Employment, employer, department, as-of, search, filters, pagination, route guard, i18n, loading/error states remain valid. |
| Boundaries | Empty result, multiple components per employment, future component, ended component, zero amount, duplicate effective date, invalid date/page/limit. |
| State | Current, future, ended, rehire, transfer, multi-employer, missing employment, payroll-period boundary. |
| Failures | Repository/database failure, malformed response, unauthorized/forbidden, stale query retry, no partial UI state. |
| Security | 401/403, scope enforcement before query, no amount in logs/URL/localStorage, `Cache-Control: no-store`, audit read without raw PII/amount. |
| Compatibility | Existing frontend API client/tests, old response consumers, additive/versioned contract decision. |
| Operations | Stable ordering, bounded page size, no N+1 query, query latency/index review, full phase logs. |

## Tasks

- [ ] **1. Baseline and policy gate** — inspect current salary component tables, assignments, endpoint consumers, permissions, audit conventions, and payroll resolver; document whether T2 collection exception is approved. Verify with contract/scope matrix before editing.
- [ ] **2. Define component-row API contract** — specify request filters (`as_of`, search, employer, department, status, page, limit), response fields, status semantics, stable ordering, empty response, error shapes, amount classification, and compatibility path. Add/update API contract tests.
- [ ] **3. Implement batched repository query** — return configured component rows with employment context and effective-date semantics in one paginated SQL path; exclude zero-component employees by default; preserve employment boundaries; avoid salary amount selection if policy gate is not approved; add indexes only if query evidence requires them, with rollback migration.
- [ ] **4. Implement handler authorization/privacy** — enforce `payroll:read` or approved narrower salary-read permission before repository work; apply scope, no-store, audit read convention, validation, and structured errors; test 401/403 and sensitive-field behavior.
- [ ] **5. Update frontend table** — render component name, amount, currency/unit, effective date, status, employee, employer, department; remove configured-count-only columns and default unconfigured rows; retain skeleton, empty, retry, URL filters, pagination, keyboard labels, mobile horizontal scroll, and dark mode.
- [ ] **6. Update detail/history path** — make row navigation use `employment_id` plus component context; show current/history/future records without overwriting employment boundaries; preserve return filters. Do not add editing in this fix unless existing write contract is already safe.
- [ ] **7. Update tests and locales** — backend repository/API tests for configured-only rows, multiple components, as-of/future/ended/zero, pagination, auth/privacy; frontend API/page/navigation tests for amount/effective date and exclusion; update en/vi/zh keys. Remove obsolete assertions and dead metadata-only UI code after call-site verification.
- [ ] **8. Verification** — run focused unit tests, then `leedevkit test api`, `leedevkit test web`, and `leedevkit test all`; run `git diff --check`; inspect every phase log. Browser-check HR/admin allowed behavior and employee/team-lead/shift-manager denied behavior using the documented role matrix, without recording credentials, tokens, raw PII, or salary amounts in artifacts.

## Done when

- `/salary-management` shows only configured component rows by default.
- Example configured amount and `effective_from=2026-10-01` render correctly for authorized HR.
- Employees without components do not appear in default results.
- Employment/rehire/legal-entity boundaries remain intact.
- Amount access follows approved T2 policy; no leak through logs, URL, cache, unauthorized response, or browser storage.
- Tests cover success, empty, temporal, pagination, auth, privacy, and failure paths.
- Full verification has explicit PASS/FAIL/SKIP evidence; no incomplete run is reported as green.

## Scope exclusions

- No payroll calculation/finalization/payment changes.
- No bulk edit, import/export, new dependency, or localStorage persistence.
- No migration unless existing schema cannot represent component rows; any migration needs forward/down scripts and explicit review.

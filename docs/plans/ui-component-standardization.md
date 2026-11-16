# UI component standardization

## Scope and decisions
- Reuse existing UI primitives; no new dependency or duplicate input/date picker.
- Preserve control heights: sm 40px, md 52px, lg 60px.
- Share control typography and explicit size variants across Button, FormField (including native date/time), Combobox and SearchInput.
- Standardize existing Tabs and Typography; preserve semantic HTML and existing public variants.
- Prevent new size/typography overrides with a repository-native automated check. Migrate affected existing usages with a bounded baseline where necessary; no blanket redesign.

## Rule compliance
- DESIGN-1: `.agent/rules/design-rules.md`, section 1 — reuse predefined UI components.
- DESIGN-2: `.agent/rules/design-rules.md`, section 2 — labels, keyboard interaction, focus, semantic HTML.
- CODE-2: `.agent/rules/coding-standards.md`, section 2 — DRY, minimal abstractions, remove replaced code.
- WORKFLOW-2/7: `.agent/rules/development-workflow.md`, sections 2 and 7 — test impact and verified evidence.
- TEST-1/2: `.agent/rules/testing-standards.md`, sections 1 and 2 — focused tests followed by full `./leedevkit test web`.

## Test impact
| Dimension | Applicable checks |
|---|---|
| Changed behavior | Shared sizes/type; tab variants; prevention of new overrides |
| Existing behavior | Form values/change/ref, disabled/loading; tab selection; public props |
| Inputs/boundaries | Each size; date/time/text/textarea; long labels; independent generated IDs |
| State | Error association; active/inactive/disabled tabs; controlled inputs |
| Failures | Invalid styling rejected with actionable lint/test output; no network changes |
| Security | No auth/data/API changes; avoid test PII and secrets |
| Compatibility | Keep exports/defaults where possible; existing overrides require explicit migration/baseline |
| Operations | No backend/logging/dependency changes; type/lint/unit/integration gates |

## Verification
- Focused component and guard tests via `./leedevkit test web --unit-only --pattern ...`.
- Required full target: `./leedevkit test web`.
- Read per-phase logs; report pre-existing failures and unavailable integration/browser checks explicitly.

## Progress
- [x] Existing rulebooks and controls inspected.
- [x] Shared control tokens and existing primitives updated.
- [x] Toolbar alignment pattern added with `ToolbarRow`; Attendance and Reports callers migrated.
- [x] ResponsiveToolbar compatibility preserved through legacy `searchSlot` support.
- [x] Regression and integration checks executed; outcomes recorded.
- [ ] Migrate future desktop toolbar callers as they need unified search/filter layout.

## Verified outcomes
- Focused: `./leedevkit test web --unit-only --pattern 'ReportFilters|ResponsiveToolbar'` — 3 files, 10 tests passed.
- Full: `./leedevkit test web` — 322 unit files, 3001 unit tests, 43 integration/E2E tests passed, 9 E2E tests skipped; typecheck, i18n, and lint passed.
- Lint emitted existing warnings but 0 errors; `git diff --check` passed.

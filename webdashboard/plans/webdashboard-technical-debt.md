# Plan: Resolving Top 3 Webdashboard Technical Debts

> Status: IN PROGRESS · Owner: main (+ subagents) · 2026-07-09

## Compliance Rule IDs (Layer 1)

The following rulebooks were consulted (Layer 0 — CORE PRINCIPLES + Layer 4):
- **PERFORMANCE_BUDGET** (`coding-standards.md` §7): JS bundle < 200KB gzipped, zero waste.
- **STRICT_TYPE_SAFETY / no `any`** (`coding-standards.md` §6 Enterprise-Grade + §9 Dependency Injection). Every `as any` / `as unknown as` is a known defect.
- **COVERAGE_80** (`testing-standards.md` §2): min 80% coverage for business logic; mandatory pass via `./test.sh`.
- **MANDATORY_PASS** (`testing-standards.md` §2): a task is NOT complete until `./test.sh web` passes.
- **BOY_SCOUT** (`coding-standards.md` §5): fix nearby bugs immediately, never "out of scope".
- **PTY_SAFETY** (Layer 0 §5): all commands redirect to `logs/run_{slug}.log 2>&1 </dev/null`.
- **HERMETIC_WRAPPERS** (Layer 0 §6): use `./leedevkit` + `scripts/` wrappers, never raw `npm`/`bun`.

## Audit Evidence (verified, 2026-07-09)

| Metric | Value | Location |
|---|---|---|
| `as any` | 194 | `src/` |
| `as unknown as` | 383 | `src/` |
| `: any` | 67 | `src/` |
| Coverage total | 65.5% Stmt / 58.3% Branch | `coverage/coverage-final.json` |
| Coverage `lib/` | 45% Stmt / 38% Branch | lowest area |
| Coverage `lib/api/payroll.ts` | 13% | payroll domain |
| Coverage `lib/api/documents.ts` | 0% | document domain |
| Total JS bundle | 4.9 MB | `dist/assets/` |
| `common-*.js` duplicated | 3 chunks (en/vi/zh locale data) | bundling defect |
| Largest hand-written API file | `attendance.ts` 815 lines | `src/lib/api/` |
| `untyped.ts` escape hatch consumers | 3 files | system/documents/work-segments |

## Work Items

### #1 — Type Safety (PI 1) [SUBAGENT A]
- Remove `src/lib/api/untyped.ts` (the `client as any` escape hatch).
- Migrate its 3 consumers to typed `client`: `system.ts`, `documents.ts`, `work-segments.ts`.
- For paths NOT in generated schema: either add to `openapi.raw.json` → regenerate, OR cast `client` generically without module-level `as any`.
- Merge manual `types.ts` (696 lines) into generated types; delete duplicate re-declares.
- Enable `@typescript-eslint/no-explicit-any: error` (warn-now → error-next wave).

### #2 — Coverage `lib/` (PI 1) [SUBAGENT B, after #1]
- Raise `lib/api` from 45% → ≥80% (payroll, attendance, leaves, documents, work-segments prioritized).
- Add `coverageThresholds` to `vitest.config.ts`: `lib/` ≥ 80%, global ≥ 75%.
- Run via `./test.sh web </dev/null` — NEVER `npm test` directly (Podman isolation).

### #3 — Bundle Bloat (PI 2) [MAIN]
- Fix locale data pre-bundling into `common-*.js` (ensure i18n lazy-load stays lazy, per-locale chunks).
- Fail-on-budget: `check-budget.mjs` must hard-fail (exit 1) in `build` script, not warn.
- Remove useless lazy wrappers (e.g. `dashboard.lazy` 692B).
- Verify `exceljs` (937KB) never lands in initial bundle.

## Definition of Done
- `./test.sh web </dev/null` passes.
- `tsc --noEmit` clean (zero new errors).
- No regression in coverage (global ≥ current 65.5%).
- No commit until user confirms.

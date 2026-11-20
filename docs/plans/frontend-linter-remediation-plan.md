# Plan: Frontend Linter Remediation (`webdashboard`)

## Rule Compliance Check
- `RULE-LINT-001` (Clean Code & Dead Code Policy): Zero unused imports, zero unused vars, zero unnecessary type assertions. (`.agent/rules/coding-standards.md` §2)
- `RULE-LINT-002` (Hermetic Wrappers & Test Completeness): All verification runs via `leedevkit test web --lint-only` and `leedevkit test web`. (`.agent/rules/execution-wrappers.md`)
- `RULE-LINT-003` (Design System Strictness): Eliminate raw HTML elements forbidden by policy (`<h1>-<h6>`, `<p>`, `<button>`) in favor of design system primitives (`Typography`, `Button`). (`.agent/rules/design-rules.md`)
- `RULE-LINT-004` (Query Key Architecture): Ban ad-hoc query key array literals; consolidate into query key factories (`src/lib/query-keys.ts`). (`webdashboard/eslint.config.mjs`)

---

## 1. Problem Breakdown (1,799 warnings, 0 errors)

| Category | Warnings | Strategy | Risk |
|---|---|---|---|
| **A. Formatting & Imports** (`perfectionist/*`, `unused-imports/no-unused-imports`) | 767 | Auto-fix via ESLint `--fix` | Zero |
| **B. Framework False Positives** (`@typescript-eslint/only-throw-error`) | 24 | Config override for TanStack Router `throw redirect()` in `routes/` | Zero |
| **C. Dead Code & Variables** (`unused-imports/no-unused-vars`) | 37 | Manual cleanup / prefix `_` for intentionally unused parameters | Low |
| **D. React & Leaked Render** (`react/jsx-no-leaked-render`, `no-confusing-void-expression`) | 196 | Convert `count && <Comp />` to `Boolean(count)` / ternary; wrap void arrow functions | Low |
| **E. TypeScript Strictness** (`@typescript-eslint/no-unnecessary-condition`, `prefer-nullish-coalescing`) | 139 | Remove redundant optional chaining (`?.`), normalize `??` vs `\|\|` | Low |
| **F. Query Key Factories** (`no-restricted-syntax`) | 274 | Centralize query keys in `src/lib/query-keys.ts`, update 81 call-site files | Medium |
| **G. Design System Primitives** (`react/forbid-elements`) | 67 | Replace raw `p`, `h1`-`h6`, `button` with `<Typography>`, `<Button>` | Low |
| **H. Complexity & File Limits** (`complexity`, `max-lines`, `max-lines-per-function`) | 117 | Legacy file exclusion in `eslint.config.mjs` or targeted hook extraction | Low |
| **I. Miscellaneous** (`@typescript-eslint/no-deprecated`, `unicorn/filename-case`, etc.) | ~178 | Address deprecated API calls (`z.string().uuid()` vs `z.uuid()`) & edge cases | Low |

---

## 2. Phased Execution Plan

### Phase 1: Automated Remediation (Auto-Fix)
- **Goal:** Clear ~1,009 warnings automatically without behavioral changes.
- **Action:** Run `leedevkit test web --lint-only --fix`.
- **Files touched:** Broad formatting sweep across `webdashboard/src`.
- **Verification:**
  1. `leedevkit test web --lint-only` (verify warnings drop to ~790).
  2. `leedevkit test web --unit-only` (verify zero test regressions).

### Phase 2: ESLint Configuration Fine-Tuning
- **Goal:** Clear false positives from framework contracts (~50 warnings).
- **Actions:**
  1. Add route-specific rule override in `webdashboard/eslint.config.mjs` for TanStack router redirects:
     ```js
     {
       files: ['src/routes/**/*.{ts,tsx}'],
       rules: {
         '@typescript-eslint/only-throw-error': 'off',
       },
     }
     ```
  2. Update `max-lines` / `complexity` ignore list for legacy mega-components already tracked for refactoring.
- **Verification:** `leedevkit test web --lint-only`.

### Phase 3: Runtime Defect Prevention & React Polish
- **Goal:** Clear ~230 warnings related to render crashes and callbacks.
- **Actions:**
  1. **`react/jsx-no-leaked-render`:**
     - Replace falsy number/string conditions: `{items.length > 0 && ...}` instead of `{items.length && ...}`.
  2. **`@typescript-eslint/no-confusing-void-expression`:**
     - Add curly braces or `void` operator to arrow function event handlers: `onClick={() => { mutate(); }}`.
  3. **`unused-imports/no-unused-vars`:**
     - Strip unused variables or prefix intentional unused args with `_`.
  4. **`@typescript-eslint/no-unnecessary-condition`:**
     - Strip dead optional chaining on non-nullable types.
- **Verification:** `leedevkit test web --unit-only`.

### Phase 4: Query Key Architecture Standardization
- **Goal:** Eliminate 274 warnings from `no-restricted-syntax`.
- **Actions:**
  1. Audit 81 files currently declaring array literal `queryKey` in `useQuery`, `useMutation`, `invalidateQueries`.
  2. Extend `webdashboard/src/lib/query-keys.ts` with missing domain key factories:
     - `shiftKeys`, `reportKeys`, `payrollKeys`, `auditLogKeys`, `holidayKeys`, `timeSettingsKeys`, `assignmentRuleKeys`.
  3. Replace literal arrays in target hooks/components with corresponding factory calls.
- **Verification:** `leedevkit test web --lint-only` & `leedevkit test web --unit-only`.

### Phase 5: Design System Compliance & Deprecations
- **Goal:** Clear remaining ~100 warnings (`react/forbid-elements`, `@typescript-eslint/no-deprecated`).
- **Actions:**
  1. Replace `<p>`, `<h2>`, `<button>` in widgets with `<Typography>` and `<Button>`.
  2. Upgrade `z.uuid()` to `z.string().uuid()` or appropriate Zod v4 syntax.
- **Verification:**
  1. Full web test suite: `leedevkit test web`.
  2. Zero warnings check: `leedevkit test web --lint-only`.

---

## 3. Definition of Done
1. `bun scripts/lint.mjs src` reports `0 errors, 0 warnings`.
2. `leedevkit test web` passes completely (lint, unit tests, E2E).
3. Zero regressions in UI rendering and data mutations.

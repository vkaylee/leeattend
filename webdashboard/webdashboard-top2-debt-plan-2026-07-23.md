# Top 2 WebDashboard Technical Debt Resolution Plan (2026-07-23)

**Scope:** `webdashboard/` — React/TypeScript frontend  
**Methodology:** Static analysis via codebase-memory graph (17K nodes, 1147 TS files) + manual audit of top-50 largest files + lint suppression census

---

## Executive Summary

After deep analysis of **~175K lines** of frontend TypeScript/TSX code across **1147 source files**, two systemic debts emerge as most critical — measured against the project's core standards: "Enterprise Premium Standard", "Structured Errors", "No MVP code".

| # | Debt | Severity | Primary Metric | Key Risk |
|---|------|----------|---------------|----------|
| **#1** | **Type Safety Erosion in API Layer** | 🔴 CRITICAL | **35** `Record<string, unknown>` + **321** type casts | Payroll financial data typed as opaque objects |
| **#2** | **eslint-disable Epidemic** | 🔴 CRITICAL | **326** suppressions across **156 files** | Normalized bypass of quality gates; accessibility violations hidden |

---

## 🔴 DEBT #1: Type Safety Erosion in API Layer

### What It Is

Despite having a **18,184-line auto-generated `api-generated.ts`** from OpenAPI spec, many API functions use `Record<string, unknown>` and unsafe type casts, making critical data paths (payroll, work segments, overtime) completely untyped.

### Evidence

**35 `Record<string, unknown>` usages across 14 API files:**

| File | Count | Worst Offender |
|------|-------|---------------|
| `types-work-segments.ts` | **10** | `generation_metadata`, `source_detail`, `break_config_override`, `ot_config_override` all untyped |
| `payroll.ts` | **6** | `createOvertimePolicy(data: Record<string, unknown>)`, `setEmployeePayrollConfig` returns `Record<string, unknown>` |
| `client.ts` | **3** | Path/query params typed as `Record<string, unknown>` instead of generated types |
| `errors.ts` | **3** | Error body casts: `body as Record<string, unknown>` |
| `overtime-requests.ts` | **2** | `quota_warning?: Record<string, unknown>`, `listRequests(params?: Record<string, unknown>)` |

**Critical examples:**

```typescript
// payroll.ts:184 — completely untyped overtime policy creation
export async function createOvertimePolicy(wsId: string, data: Record<string, unknown>): Promise<OvertimePolicy> {
  const { data: resData } = await fetch<OvertimePolicy>("POST", "/workspaces/{ws_id}/overtime/policies", {
    params: { path: { ws_id: wsId } },
    body: data,
  });
  return resData;
}

// payroll.ts:200 — employee payroll config returns opaque object
export async function setEmployeePayrollConfig(wsId: string, employeeId: string, 
  data: { effective_date: string; hours_per_day: string; std_working_days: string } & Record<string, unknown>
): Promise<Record<string, unknown>> {

// types-work-segments.ts:11 — core work segment metadata is untyped
generation_metadata: Record<string, unknown>;

// shifts.ts:100 — roster data is untyped
export async function upsertRoster(wsId: string, employeeId: string, data: Record<string, unknown>): Promise<ShiftRoster> {
```

**321 type casts across API files:**
- `as never` — passing untyped data to API calls (leaves.ts, payroll.ts)
- `as LeavePolicy` / `as LeavePolicyAssignmentDetail[]` — return type assertions instead of proper generics
- `as Record<string, unknown>` — cascading unsafe casts in errors.ts

### Root Cause

1. The auto-generated `api-generated.ts` types are comprehensive but **not used** in many API functions
2. Some API endpoints have config/metadata fields that vary by workspace — the team used `Record<string, unknown>` as a shortcut instead of defining discriminated unions
3. The `client.ts` helper doesn't enforce type-safe path/query params

### Proposed Solution

**Phase 1: Define missing types for work-segments & payroll configs (~4-6h)**

Create proper interfaces replacing `Record<string, unknown>`:

```typescript
// types-work-segments.ts — replace 10 opaque objects
export interface WorkSegmentGenerationMetadata {
  source: 'auto' | 'manual' | 'import';
  generated_at: string;
  rule_version?: string;
}

export interface BreakConfigOverride {
  enabled: boolean;
  duration_minutes?: number;
  start_time?: string;
}

export interface OtConfigOverride {
  multiplier: number;
  threshold_hours?: number;
}

export interface WorkSegmentSourceDetail {
  shift_id?: string;
  roster_id?: string;
  adjustment_id?: string;
  entered_via?: string;
}
```

```typescript
// types-payroll.ts — define payroll config types
export interface EmployeePayrollConfig {
  effective_date: string;
  hours_per_day: string;
  std_working_days: string;
  overtime_multiplier?: string;
  night_shift_allowance?: string;
  // ... other known fields
}

export interface DepartmentPayrollConfig {
  effective_date: string;
  hours_per_day: string;
  std_working_days: string;
  // ... other known fields
}

export interface CreateOvertimePolicyRequest {
  name: string;
  description?: string;
  rules: OvertimeRuleInput[];
  effective_from?: string;
}
```

**Phase 2: Update API functions to use proper types (~4-6h)**

- `payroll.ts` — replace all 6 `Record<string, unknown>` with specific interfaces
- `overtime-requests.ts` — replace 2 `Record<string, unknown>` 
- `shifts.ts` — define `UpsertRosterRequest` type
- `system.ts` — define `SystemHealthResponse` type
- `workspace-config.ts` — replace `Json?: Record<string, unknown>` with `Json?: unknown` (unknown is safer)
- `client.ts` — use `paths` generated types for path/query params

**Phase 3: Remove unsafe casts (~2-4h)**

- Replace `as never` with properly typed `body` parameters
- Replace `as LeavePolicy` / `as LeavePolicyAssignmentDetail[]` with proper generic type params
- Replace `body as Record<string, unknown>` in errors.ts with a typed error interface

### Files to Modify

| File | Changes |
|------|---------|
| `src/lib/api/types-work-segments.ts` | Replace 10 `Record<string, unknown>` → proper interfaces |
| `src/lib/api/types-payroll.ts` | Add payroll config & overtime policy types |
| `src/lib/api/payroll.ts` | Replace 6 `Record<string, unknown>` → typed |
| `src/lib/api/overtime-requests.ts` | Replace 2 `Record<string, unknown>` → typed |
| `src/lib/api/shifts.ts` | Add `UpsertRosterRequest` type |
| `src/lib/api/system.ts` | Add `SystemHealthResponse` type |
| `src/lib/api/workspace-config.ts` | `Json?: Record<string, unknown>` → `Json?: unknown` |
| `src/lib/api/errors.ts` | Typed error body interface |
| `src/lib/api/client.ts` | Use generated path types |
| `src/lib/api/leaves.ts` | Remove `as never` cast |
| `src/lib/api/approval-workflows.ts` | Replace `Record<string, unknown>` |
| `src/lib/api/types-ess.ts` | Replace `Record<string, unknown>` |

### Effort Estimate: ~14 hours

---

## 🔴 DEBT #2: eslint-disable Epidemic — Systemic Quality Gate Bypass

### What It Is

The codebase has **326 eslint-disable suppressions** across **156 source files**, systematically bypassing the project's quality gates. The most suppressed rules are:

| Rule | Count | Impact |
|------|-------|--------|
| `@typescript-eslint/*` | **156** | Type safety violations, null checks, unnecessary conditions |
| `complexity` | **52** | Functions with cyclomatic complexity > 10 |
| `max-lines-per-function` | **31** | Functions > 100 lines |
| `max-lines` | **12** | Files > 300 lines |
| `jsx-a11y` | **10** | Accessibility violations (missing labels, roles, ARIA) |
| `react-hooks/*` | **18** | Hook dependency violations (stale closures, missing deps) |

### Evidence — The Worst Offenders

**Full-file disables (entire file exempt from linting):**

| File | Rules Disabled |
|------|---------------|
| `lifecycle-tab.tsx` | 5 rules: `no-unnecessary-condition`, `no-unnecessary-type-conversion`, `prefer-nullish-coalescing`, `complexity`, `max-lines`, `max-lines-per-function`, `react/no-unescaped-entities` |
| `NoShowTable.tsx` | 6 rules: `no-unnecessary-type-conversion`, `complexity`, `max-lines-per-function`, `react/forbid-elements`, `unicorn/filename-case`, `unused-imports/no-unused-vars` |
| `chart-catalog-sidebar.tsx` | 5 rules: `no-unnecessary-condition`, `no-unnecessary-type-conversion`, `complexity`, `max-lines-per-function`, `react/forbid-elements` |
| `leave-type-manager.tsx` | 4 rules: `no-unnecessary-condition`, `complexity`, `max-lines-per-function`, `unused-imports/no-unused-vars` |
| `daily-report-table-row.tsx` | 3 rules: `no-unnecessary-type-conversion`, `complexity`, `max-lines` |

**Top 5 files with most `complexity` suppressions:**

| File | Lines | Complexity disables |
|------|-------|-------------------|
| `employee-form-modal.tsx` | ~400 | 3 `complexity` disables |
| `smtp-settings-page.tsx` | 537 | 3 `complexity` disables |
| `contract-form-modal.tsx` | ~400 | 2 `complexity` disables |
| `use-smart-table-clipboard.ts` | ~100 | 2 `complexity` disables |
| `smart-table.tsx` | ~450 | 2 `complexity` disables |

### Root Cause

This is a **cultural debt**: components grow too large due to feature pressure, and instead of extracting sub-components or custom hooks, developers disable the linter. The comment justifications ("genuine layout complexity", "React Table boilerplate", "form rendering inherently has conditional branches") reveal that decomposition was considered but rejected — the team chose to suppress rather than refactor.

### Proposed Solution

The approach must be **pragmatic** — some `eslint-disable` comments are legitimate (e.g., `client.ts` which has a genuine multi-overload function, `navigable-list.tsx` which is already well-decomposed). The focus should be on the **systemic abusers** — files with 4+ rule suppressions or full-file disables.

**Phase 1: Fix `@typescript-eslint/*` violations (~8-12h)**

The 156 `@typescript-eslint/*` suppressions are the most numerous and the easiest to fix systematically:

1. **`no-unnecessary-condition`** — Replace `?? null` with proper optional chaining; remove redundant `&&` checks
2. **`no-unnecessary-type-conversion`** — Remove `String(x)` wrapping on already-string values; remove `Number()` on numeric values  
3. **`prefer-nullish-coalescing`** — Replace `\|\|` with `??` where appropriate

These are mechanical fixes that can be done file-by-file. Target: eliminate **80%+** (125 of 156).

**Phase 2: Decompose worst complexity offenders (~12-16h)**

Target the files with the **most `complexity` disables** — these indicate genuine architectural problems:

1. **`employee-form-modal.tsx`** (3 complexity disables) → Extract: `useEmployeeFormState` hook, `EmployeeFormFields` component, `EmploymentMutationSection` component
2. **`smtp-settings-page.tsx`** (537 lines, 3 complexity disables) → Extract: `SmtpConfigForm`, `SmtpTestDialog`, `SmtpConfigLogs` components
3. **`contract-form-modal.tsx`** (2 complexity disables) → Extract: `useContractFormValidation` hook
4. **`leave-type-manager.tsx`** → Extract: `LeaveTypeForm` component, `LeaveTypeList` component
5. **`NoShowTable.tsx`** → Extract sub-components for each cell renderer

**Phase 3: Fix accessibility violations (~4-6h)**

The 10 `jsx-a11y` suppressions hide real accessibility problems:
- Add `aria-label` to icon-only buttons
- Add `role` attributes to interactive elements
- Ensure form inputs have associated labels

**Phase 4: Add lint ratchet (~2-3h)**

Prevent regression by adding a lint threshold to CI:
```json
// .eslintrc — ratchet config
{
  "maxWarnings": 100  // start high, decrease each sprint
}
```
Target: reduce from 326 to **≤50** over 3 sprints. Then enable `--max-warnings 0`.

### Files to Modify (Priority Order)

| Priority | File | Action |
|----------|------|--------|
| P0 | `lifecycle-tab.tsx` | Remove 7 rule disables → extract hooks |
| P0 | `NoShowTable.tsx` | Remove 6 rule disables → decompose |
| P0 | `chart-catalog-sidebar.tsx` | Remove 5 rule disables |
| P0 | `leave-type-manager.tsx` | Remove 4 rule disables |
| P0 | `daily-report-table-row.tsx` | Remove 3 rule disables |
| P1 | `employee-form-modal.tsx` | Extract form state hook (3 complexity fixes) |
| P1 | `smtp-settings-page.tsx` | Decompose into sub-components (3 complexity fixes) |
| P1 | `contract-form-modal.tsx` | Extract validation logic (2 complexity fixes) |
| P2 | All `jsx-a11y` files (10) | Add ARIA labels, roles |
| P2 | All `@typescript-eslint` files (~80) | Mechanical type-safety fixes |
| P3 | CI config | Add `maxWarnings` ratchet |

### Effort Estimate: ~30 hours

---

## Implementation Order & Rationale

| Order | Debt | Why This Order |
|-------|------|---------------|
| **1st** | #1 Type Safety Erosion | Fix type safety first — it protects against regressions when doing the refactoring in #2. Having proper types makes IDE autocompletion work, which accelerates #2. |
| **2nd** | #2 eslint-disable Epidemic | Structural refactoring — safer after types are solid. Focus on the worst 10-15 files in Phase 2; mechanical fixes in Phase 1 can parallelize. |

**Total estimated effort: ~44 hours**

---

## Metrics — Current vs Target

| Metric | Current | Target | Debt # |
|--------|---------|--------|--------|
| `Record<string, unknown>` in API | **35** | **≤5** (only in legitimate generic helpers) | #1 |
| Unsafe `as` casts in API | **321** | **≤50** (only in client.ts generic helpers) | #1 |
| `eslint-disable` total | **326** | **≤50** | #2 |
| `complexity` suppressions | **52** | **≤15** | #2 |
| `max-lines-per-function` suppressions | **31** | **≤8** | #2 |
| `@typescript-eslint/*` suppressions | **156** | **≤30** | #2 |
| `jsx-a11y` suppressions | **10** | **0** | #2 |
| Files with 4+ rule disables | **12** | **0** | #2 |
| `Record<string, unknown>` in types-work-segments | **10** | **0** | #1 |
| `Record<string, unknown>` in payroll.ts | **6** | **0** | #1 |

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Payroll type changes break calculation UI | High | Types are additive — old `Record<string, unknown>` usages become narrower, not wider. Existing code that accesses fields will still work. |
| Decomposition introduces prop-drilling | Medium | Use composition with children; extract hooks not HOCs |
| eslint fix causes behavior change | Low | `@typescript-eslint` fixes (null checks, type conversions) are mechanical; run full test suite after each batch |
| Accessibility fixes don't cover all cases | Low | Phase 3 only fixes explicitly suppressed violations; full a11y audit is out of scope |
| Type narrowing reveals existing bugs | Medium | **This is a feature** — bugs found during type tightening should be fixed, not hidden |

---

## Verification Checklist

After each phase:
- [ ] `npm run type-check` — zero new errors
- [ ] `npm run lint` — warning count decreasing
- [ ] `npm run test` — all tests pass, 0 regression
- [ ] `npm run build` — production build succeeds
- [ ] `npm run test:e2e` — Playwright tests pass (spot-check payroll + attendance pages)
- [ ] Manual smoke test: Payroll page → create overtime policy → verify types
- [ ] Manual smoke test: Work segments page → verify metadata displays
- [ ] Manual smoke test: Employee form modal → create/edit employee
- [ ] Manual smoke test: Leave type manager → create/edit/delete leave type

---

## Related Past Work

- apiserver Top-3 Debt Plan (2026-07-19) in `apiserver-top3-debt-plan-2026-07-19.md` — sister document for the Rust backend
- Round 1 (a78a137): Top-3 critical debts (webdashboard + apiserver cross-cutting)
- **Model to follow for decomposition:** `src/lib/api/` directory itself (59 files, well-split by domain, clear module boundaries)
- **Model to follow for types:** `types-attendance.ts` (3.2K, well-typed with proper interfaces)

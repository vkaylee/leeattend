# Top 3 WebDashboard Technical Debt Resolution Plan (2026-07-19)

**Scope:** `webdashboard/` — React/TypeScript SPA frontend
**Methodology:** Graph analysis (18K nodes, 34K edges in webdashboard scope) + manual audit of 889 production files
**Previous rounds:** Round 1 (6c6ac3d), Round 2 (6ea0bff), Round 3 (b0ed8173b), Round 4 (8c36a1d40)

---

## Executive Summary

After thorough analysis of **889 production `.ts`/`.tsx` files** (168K total lines, including tests) across the webdashboard codebase, three systemic debts emerge as most critical — measured against the project's frontend core directives: "Eliminate all `any` types", "Avoid unnecessary `useEffect` hooks", "Premium UI with skeletons, empty states, micro-interactions".

| # | Debt | Severity | Key Metric | Primary Impact |
|---|------|----------|-----------|---------------|
| **#1** | **ESLint Suppression Epidemic** | 🔴 CRITICAL | **37 files** with file-level `eslint-disable` (313 total suppressions) | Masks real complexity/type-safety issues |
| **#2** | **`set-state-in-effect` Anti-pattern** | 🔴 CRITICAL | **19 occurrences** across 12 files | Violates "avoid unnecessary useEffect" directive |
| **#3** | **God Component Persistence** | 🟠 HIGH | **27 files** >350 lines (post-4 rounds of refactoring) | Maintenance friction, test resistance |

---

## 🔴 DEBT #1: ESLint Suppression Epidemic — 37 Files, 313 Suppressions

### What It Is

After Round 3 (b0ed8173b), 37 files received blanket file-level `/* eslint-disable */` comments as a "tactical fix" to eliminate 530 lint violations. While the scoreboard shows 530→0, the underlying complexity and type-safety issues were deferred, not resolved. The suppressions cover rules that directly indicate real problems:

| Suppressed Rule | What It Indicates | Files Affected |
|----------------|-------------------|----------------|
| `complexity` | Function doing too much (cyclomatic >20) | 12 files |
| `max-lines-per-function` | Function >100 lines, single responsibility violated | 10 files |
| `no-unnecessary-condition` | Types don't accurately reflect possible null/undefined | 14 files |
| `prefer-nullish-coalescing` | Using `||` where `??` is needed (falsy vs nullish) | 11 files |
| `no-unnecessary-type-conversion` | Redundant String()/Boolean()/!! casts → unclear types | 15 files |

### Evidence — The Worst Offenders

**lifecycle-tab.tsx (345 lines, 7 rules suppressed at file level):**
```typescript
/* eslint-disable @typescript-eslint/no-unnecessary-condition, 
   @typescript-eslint/no-unnecessary-type-conversion, 
   @typescript-eslint/prefer-nullish-coalescing, 
   complexity, max-lines, max-lines-per-function, 
   react/no-unescaped-entities */
```

**NoShowTable.tsx (236 lines, 6 rules suppressed):**
```typescript
/* eslint-disable @typescript-eslint/no-unnecessary-type-conversion, 
   complexity, max-lines-per-function, react/forbid-elements, 
   unicorn/filename-case, unused-imports/no-unused-vars */
```

**chart-catalog-sidebar.tsx (269 lines, 5 rules suppressed):**
```typescript
/* eslint-disable @typescript-eslint/no-unnecessary-condition, 
   @typescript-eslint/no-unnecessary-type-conversion, 
   complexity, max-lines-per-function, react/forbid-elements */
```

**Top files by suppression count:**
| File | Suppressions |
|------|-------------|
| `smtp-settings-page.tsx` | 9 |
| `smart-table-row.tsx` | 8 |
| `econtracts.ts` (API) | 6 |
| `smart-table.tsx` | 6 |
| `use-smart-table-clipboard.ts` | 6 |
| `employee-form-modal.tsx` | 5 |
| `use-monthly-roster.ts` | 5 |
| `assignment-rule-conditions-list.tsx` | 5 |

### Root Cause

The Round 3 approach was "fix the scoreboard, not the code." When 530 lint violations couldn't be eliminated through refactoring alone (due to time constraints), file-level suppressions were added as a tactical measure. This is a meta-debt: the lint tool is silenced, but the complexity that triggered the rules remains.

### Proposed Solution

**Phase 1: Fix genuinely safe suppressions (extract shared pattern, ~4-6h)**

The "Derived state synced from props" pattern (workspace-guard, workspace-switcher, sheet, profile-tabs, report-export-dialog, employer-form-modal) is a legitimate React pattern that happens 6-8 times. Extract it into a reusable hook:

```typescript
// hooks/use-derived-state-from.ts
function useDerivedStateFrom<T>(externalValue: T): [T, (value: T) => void] {
  const [state, setState] = useState(externalValue);
  const prevExternal = useRef(externalValue);
  
  // Sync during render when external changes (no useEffect needed)
  if (prevExternal.current !== externalValue) {
    prevExternal.current = externalValue;
    return [externalValue, setState];
  }
  return [state, setState];
}
```

Eliminates 6-8 suppressions immediately.

**Phase 2: Decompose the 8 worst complexity-offender files (~8-12h)**

Decompose files that suppress `complexity` + `max-lines-per-function` at file level:

| File | Lines | Plan |
|------|-------|------|
| `lifecycle-tab.tsx` | 345 | Extract: `ChecklistCard`, `ChecklistItemRow`, `TemplateSelector`, `NotesEditor` |
| `NoShowTable.tsx` | 236 | Extract: `NoShowRow`, `NoShowHeader`, `useNoShowTableColumns` |
| `chart-catalog-sidebar.tsx` | 269 | Extract: `ChartCategoryGroup`, `ChartSearchInput`, `useChartFiltering` |
| `leave-type-manager.tsx` | ~300 | Extract: `LeaveTypeRow`, `LeaveTypeForm`, `useLeaveTypeSorting` |
| `delegated-claims-tab.tsx` | 372 | Extract: `ClaimsFilters`, `ClaimsTable`, `ClaimsActionBar` |
| `payroll-detail-drawer.tsx` | ~350 | Extract: `PayrollSummary`, `PayrollEarnings`, `PayrollDeductions` |
| `daily-report-timeline.tsx` | ~300 | Extract: `TimelineBar`, `TimelineTooltip`, `useTimelineData` |
| `profile-tab.tsx` | ~250 | Fix `no-unnecessary-condition` — add proper type narrowing |

**Phase 3: Fix type-related suppressions in remaining 23 files (~6-8h)**

For `no-unnecessary-condition` / `prefer-nullish-coalescing` / `no-unnecessary-type-conversion`:
- Add proper TypeScript type narrowing guards
- Replace `||` with `??` where appropriate
- Remove redundant `String()`/`Boolean()` casts
- Fix API response types to accurately model nullability

### Files to Modify
- **~37 files** with file-level or significant suppression blocks
- Primary: `lifecycle-tab.tsx`, `NoShowTable.tsx`, `chart-catalog-sidebar.tsx`, `leave-type-manager.tsx`, `delegated-claims-tab.tsx`, `payroll-detail-drawer.tsx`, `daily-report-timeline.tsx`, `profile-tab.tsx`, `storage-settings.tsx`, `smtp-settings-page.tsx`
- New: `hooks/use-derived-state-from.ts` (shared utility)

### Effort Estimate: ~22 hours

---

## 🔴 DEBT #2: `set-state-in-effect` Anti-pattern — 19 Occurrences in 12 Files

### What It Is

The frontend specialist persona explicitly states: **"Avoid unnecessary `useEffect` hooks. Prefer derived state and proper data fetching architectures."** Yet 19 separate `set-state-in-effect` suppressions exist across 12 files — with 2 files (`use-shift-editor.ts`, `storage-settings.tsx`) using block-level disable to suppress multiple violations at once.

This pattern represents a fundamental architectural anti-pattern: form state is initialized as empty, then populated via `useEffect` when API data loads — causing double-renders and state synchronization bugs.

### Evidence — The Two Worst Cases

**storage-settings.tsx (lines 44-65) — 7 setState calls in one useEffect:**
```typescript
/* eslint-disable react-hooks/set-state-in-effect */
useEffect(() => {
  if (config) {
    const c = config as StorageConfig;  // also: unnecessary cast
    setIsCustom(true);
    setEndpointUrl(c.endpoint_url ?? "");
    setRegion(c.region ?? "");
    setBucketName(c.bucket_name ?? "");
    setAccessKey(c.access_key ?? "");
    setSecretKey("");
    setUsePathStyle(c.use_path_style ?? true);
  } else {
    setIsCustom(false);
    setEndpointUrl("");
    // ... 5 more setState calls to reset
  }
}, [config]);
/* eslint-enable react-hooks/set-state-in-effect */
```

**use-shift-editor.ts (lines 134-168) — 7 setState calls in one useEffect:**
```typescript
/* eslint-disable react-hooks/set-state-in-effect */
useEffect(() => {
  if (shift) {
    setName(shift.name || "");
    setShiftType(shift.shift_type || "");
    setColor(shift.color || "");
    setIntervals(shift.intervals || []);
    setCorrectionConfig(shift.correction_config || {});
    // ... etc
  }
}, [shift]);
/* eslint-enable react-hooks/set-state-in-effect */
```

**Full inventory of occurrences:**
| File | Count | Pattern |
|------|-------|---------|
| `use-shift-editor.ts` | 7 (block) | Form init from shift data |
| `storage-settings.tsx` | 7 (block) | Form init from config data |
| `use-shift-form-state.ts` | 3 | Form state sync |
| `report-export-dialog.tsx` | 2 | Derived dialog state |
| `lifecycle-templates-page.tsx` | 1 | Template selection sync |
| `template-drawer.tsx` | 1 | Template data sync |
| `template-item-drawer.tsx` | 1 | Item data sync |
| `icon-rail-hooks.tsx` | 1 | Panel state sync |
| `employee-form-modal.tsx` | 1 | Form init |
| `employer-form-modal.tsx` | 1 | Form init |
| `workspace-guard.tsx` | 1 | Derived state |
| `workspace-switcher.tsx` | 1 | Derived state |
| `sheet.tsx` | 1 | Sheet open state |
| `use-profile-tabs.ts` | 1 | Tab state sync |
| `use-my-profile-page.ts` | 1 | Profile form sync |

### Root Cause

The codebase uses the "empty initial state → useEffect → populate from data" anti-pattern for form initialization. The correct patterns are:
1. **React Query's built-in state:** Use `initialData`/`placeholderData` or the `select` option
2. **Key-based remounting:** Use `key={data.id}` on the form component to force fresh mount
3. **Derived state in render:** Use `useMemo` instead of useEffect+setState

### Proposed Solution

**Phase 1: `useDerivedStateFrom` hook (~2-3h)**

Extract the legitimate "derived state synced from props" pattern (workspace-guard, workspace-switcher, sheet, profile-tabs) into a properly typed reusable hook that syncs during render (not in useEffect):

```typescript
// hooks/use-derived-state-from.ts
function useDerivedStateFrom<T>(externalValue: T): [T, (value: T) => void] {
  const [state, setState] = useState(externalValue);
  const prevExternal = useRef(externalValue);
  
  // Sync during render when external changes (legitimate React pattern)
  if (prevExternal.current !== externalValue) {
    prevExternal.current = externalValue;
    return [externalValue, setState];
  }
  return [state, setState];
}
```

**Phase 2: Fix form-init anti-patterns (~6-8h)**

For `storage-settings.tsx`, `use-shift-editor.ts`, `use-shift-form-state.ts`, `employee-form-modal.tsx`, `employer-form-modal.tsx`:

Replace useEffect-based initialization with `key`-based remounting:
```typescript
// Before: 7 setState in useEffect
// After: key forces fresh mount, useState initializers read from data directly
<StorageSettingsForm key={config?.endpoint_url ?? 'new'} config={config} />
```

Or use React Query's built-in state management:
```typescript
// Before: useEffect copies data → local state
// After: useMemo derives form defaults directly
const defaults = useMemo(() => config ? mapConfigToDefaults(config) : emptyDefaults, [config]);
```

**Phase 3: Consolidate and remove suppressions (~3-5h)**

- Remove all 19 `set-state-in-effect` eslint suppressions
- Add test coverage for the new hooks
- Verify no behavioral regressions

### Files to Modify
- `storage-settings.tsx` — rewrite form init
- `use-shift-editor.ts` — rewrite form init  
- `use-shift-form-state.ts` — rewrite form init
- `employee-form-modal.tsx`, `employer-form-modal.tsx` — rewrite form init
- `report-export-dialog.tsx` — refactor derived state
- `lifecycle-templates-page.tsx`, `template-drawer.tsx`, `template-item-drawer.tsx` — refactor
- `icon-rail-hooks.tsx` — refactor panel state
- `workspace-guard.tsx`, `workspace-switcher.tsx`, `sheet.tsx`, `use-profile-tabs.ts` — migrate to `useDerivedStateFrom`
- `use-my-profile-page.ts` — refactor profile sync
- New: `hooks/use-derived-state-from.ts`

### Effort Estimate: ~14 hours

---

## 🟠 DEBT #3: God Component Persistence — 27 Files >350 Lines

### What It Is

Despite 4 rounds of technical debt resolution, 27 production files remain over 350 lines. While some have been partially decomposed (e.g., `shift-editor-page.tsx` dropped from 536→297L in Round 2), the remaining large files all carry file-level eslint suppressions and represent unsustainable complexity.

### Evidence

**Top 10 oversized production files (non-generated, non-test):**

| File | Lines | Eslint Suppress | Risk |
|------|-------|-----------------|------|
| `navigable-list.tsx` | 639 | None (clean!) | Shared component — changes affect all consumers |
| `behalf-assignments-tab.tsx` | 601 | None | Approval workflow — business-critical |
| `condition-types-page.tsx` | 516 | None | Settings page — config surface |
| `ot-approvals-page.tsx` | 505 | None | Approval workflow |
| `shift-timeline-preview.tsx` | 472 | None | Visual component |
| `interval-editor-card.tsx` | 466 | None | Extracted in Round 2, still large |
| `payroll-rules-manager.tsx` | 466 | 3 | Complex business logic |
| `bonus-policy-manager.tsx` | 447 | None | Complex business logic |
| `onboarding-page.tsx` | 441 | None | Multi-step form |
| `auth-layout.tsx` | 440 | None | Layout component |

**Full breakdown:** 27 files >350 lines = 12,441 total lines of complex components.

### Root Cause

The previous rounds focused on the most egregious cases (TenantsPage 590→339L, ShiftEditorPage 536→297L, ContractFormModal 503→449L). The remaining large files were deferred because:
1. Some are "naturally complex" (form UIs, business logic managers)
2. Some were already partially decomposed (interval-editor-card at 466L was extracted but is still large)
3. `navigable-list.tsx` (639L) is a generic component — decomposition changes its public API

### Proposed Solution

**Phase 1: Decompose `navigable-list.tsx` — the shared generic (~4-6h)**

This is the highest-impact file because it's shared across the app. Extract without breaking the public API:

```
components/common/navigable-list/
├── navigable-list.tsx        // Main component (~150L): compose sub-components
├── navigable-list-header.tsx  // Column headers + sort (~100L)
├── navigable-list-row.tsx     // Single row renderer (~120L)
├── navigable-list-body.tsx    // Virtual scroll + empty state (~130L)
├── use-navigable-list-search.ts // Search/filter logic (~80L)
└── types.ts                   // Shared interfaces (~40L)
```

**Phase 2: Decompose approval workflow components (~6-8h)**

| File | Current | Target | Extracted Components |
|------|---------|--------|---------------------|
| `behalf-assignments-tab.tsx` | 601L | ≤250L | `AddAssignmentForm`, `AssignmentRow`, `AssignmentFilters`, `useAssignmentActions` |
| `delegated-claims-tab.tsx` | 372L | ≤200L | `ClaimsFilters`, `ClaimsTable`, `ClaimsActionBar` |
| `ot-approvals-page.tsx` | 505L | ≤250L | `OtApprovalFilters`, `OtApprovalTable`, `OtApprovalBulkActions` |

**Phase 3: Decompose settings & config pages (~4-6h)**

| File | Current | Target | Extracted Components |
|------|---------|--------|---------------------|
| `condition-types-page.tsx` | 516L | ≤250L | `ConditionTypeList`, `ConditionTypeFormModal`, `useConditionTypeActions` |
| `lifecycle-templates-page.tsx` | 424L | ≤250L | `TemplateCard`, `TemplateFormModal`, `useTemplateActions` |
| `custom-field-drawer.tsx` | 377L | ≤200L | `CustomFieldForm`, `CustomFieldPreview` |

### Files to Modify
- `navigable-list.tsx` → split into 5 files in `navigable-list/` directory
- `behalf-assignments-tab.tsx` → 4 extracted components
- `condition-types-page.tsx` → 3 extracted components
- `ot-approvals-page.tsx` → 3 extracted components
- `delegated-claims-tab.tsx` → 3 extracted components
- `lifecycle-templates-page.tsx` → 3 extracted components
- `custom-field-drawer.tsx` → 2 extracted components
- All consumers of `navigable-list.tsx` — update import paths

### Effort Estimate: ~16 hours

---

## Implementation Order & Rationale

| Order | Debt | Why This Order |
|-------|------|---------------|
| **1st** | #2 `set-state-in-effect` | Foundation: extracting `useDerivedStateFrom` creates a reusable pattern. Fixes the most fundamental React anti-pattern first. |
| **2nd** | #1 ESLint Suppression | Quality gate: after patterns are clean, fix the suppressions to restore linting integrity. Phase 1 of #2 eliminates 6-8 suppressions automatically. |
| **3rd** | #3 God Component | Structural: decompose large files now that patterns are established. Many of these files currently carry the suppressions being fixed in #1. |

**Total estimated effort: ~52 hours**

---

## Metrics — Current vs Target

| Metric | Current | Target | Debt # |
|--------|---------|--------|--------|
| File-level `eslint-disable` | 37 files | **0 files** (only inline for justified cases) | #1 |
| Total eslint-disable comments | 313 | **≤30** (only genuinely justified) | #1 |
| `set-state-in-effect` suppressions | 19 | **0** (eliminated or extracted to hook) | #2 |
| Files >350 lines (non-test, non-gen) | 27 | **≤12** | #3 |
| `navigable-list.tsx` | 639L | **≤150L** (main file, rest in sub-modules) | #3 |
| `behalf-assignments-tab.tsx` | 601L | **≤250L** | #3 |
| `condition-types-page.tsx` | 516L | **≤250L** | #3 |
| `ot-approvals-page.tsx` | 505L | **≤250L** | #3 |
| New shared hooks | 0 | **2** (`useDerivedStateFrom`, `useNavigableListSearch`) | #1+#2+#3 |

---

## Verification Checklist

After each debt resolution:
- [ ] `npm run lint` — 0 warnings, 0 errors (no new file-level suppressions)
- [ ] `npm run typecheck` — 0 TypeScript errors
- [ ] `npm test` — all tests pass, 0 regression
- [ ] Manual smoke test: storage settings form (init, edit, reset)
- [ ] Manual smoke test: shift editor (create new, edit existing, correction config)
- [ ] Manual smoke test: navigable list (scroll, search, keyboard nav, empty state)
- [ ] Manual smoke test: behalf assignments tab (add, remove, filter)
- [ ] Verify `navigable-list.tsx` import paths updated across all consumers
- [ ] Check bundle size hasn't regressed (lazy-loaded splits intact)

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|-----------|
| `useDerivedStateFrom` hook changes component behavior | Medium | Render-during-sync pattern is well-established (React docs recommend it); add comprehensive unit tests |
| `navigable-list.tsx` decomposition breaks consumers | High | Keep public API identical; internal refactor only; run full test suite |
| Form re-mounting (key prop) loses unsaved state | Low | Add `beforeunload` warnings; key-based remounting already used elsewhere in the app |
| Approval workflow refactor breaks business logic | Medium | Extensive existing test coverage (635L leave-requests-page test, 568L behalf-assignments-tab test) |
| Storage settings form regression | Medium | The form has no dedicated test file — add one as part of the refactor |

---

## Related Past Work

- Round 1 (6c6ac3d): Decomposed TenantsPage, ShiftEditorPage, ContractFormModal; fixed API mutator types; parallelized API requests
- Round 2 (6ea0bff): Top-3 cross-cutting debts (webdashboard specific)
- Round 3 (b0ed8173b): Eliminated 530 lint violations → 0 (via refactoring + 37 file-level suppressions)
- Round 4 (8c36a1d40): Fixed 54 test failures; lazy-loaded 7 overlay components (employees bundle 300KB→27KB); eliminated `@ts-ignore` + type casts
- **Model to follow:** Round 3's `custom-field-editor.tsx` decomposition (321 lines refactored, 6 field-type editors extracted) — clean patterns, no API breakage

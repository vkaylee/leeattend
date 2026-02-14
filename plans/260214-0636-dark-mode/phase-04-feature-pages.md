# Phase 4: Feature Pages

## Context Links
- [Phase 3: UI Primitives](phase-03-ui-primitives.md)
- [Scout: 215 TSX files](scout/scout-01-webdashboard-structure.md)
- [AppRoutes](../../webdashboard/src/AppRoutes.tsx)

## Overview
- **Priority**: P2
- **Status**: pending
- **Effort**: 2h
- **Description**: Add dark mode variants to all page-level components. Bulk approach: search-replace common patterns, then manual pass for page-specific styling.

## Key Insights
- After Phases 1-3, semantic tokens (`bg-background`, `text-foreground`, `bg-card`) auto-switch; pages using CSS vars already work
- Most pages share common patterns: white card containers, slate text, slate borders
- Bulk search-replace handles 70-80% of changes; remaining 20% needs manual review
- Pages with data tables, charts, or custom components need extra attention

## Requirements

### Functional
- F1: Every page renders correctly in dark mode
- F2: Data tables, forms, cards all have dark backgrounds
- F3: Empty states, loading spinners, error states dark-aware
- F4: Charts/graphs readable in dark mode (if applicable)

### Non-Functional
- NF1: WCAG AA contrast ratios maintained
- NF2: No white "holes" — every container has dark variant

## Architecture

### Bulk Search-Replace Patterns

Apply across all `src/pages/**/*.tsx` and `src/components/**/*.tsx`:

| Search Pattern | Replace With |
|---------------|-------------|
| `bg-white ` | `bg-white dark:bg-slate-900 ` |
| `bg-white"` | `bg-white dark:bg-slate-900"` |
| `bg-slate-50 ` | `bg-slate-50 dark:bg-slate-800 ` |
| `bg-slate-50"` | `bg-slate-50 dark:bg-slate-800"` |
| `bg-slate-100` | `bg-slate-100 dark:bg-slate-700` |
| `border-slate-100` | `border-slate-100 dark:border-slate-800` |
| `border-slate-200` | `border-slate-200 dark:border-slate-700` |
| `border-slate-50` | `border-slate-50 dark:border-slate-800` |
| `text-slate-700` | `text-slate-700 dark:text-slate-200` |
| `text-slate-800` | `text-slate-800 dark:text-slate-100` |
| `text-slate-600` | `text-slate-600 dark:text-slate-300` |
| `text-slate-500` | `text-slate-500 dark:text-slate-400` |
| `text-slate-400` | `text-slate-400 dark:text-slate-500` |
| `shadow-slate-` | `shadow-slate- dark:shadow-black/` (manual) |
| `divide-slate-100` | `divide-slate-100 dark:divide-slate-800` |

**IMPORTANT**: Run these carefully. Some may already have `dark:` variants from earlier phases. Check for duplicates. Use a script or IDE global search with preview before applying.

### Pages Requiring Manual Attention

These pages likely have custom/complex styling beyond simple patterns:

| Page | Reason |
|------|--------|
| `src/pages/Dashboard.tsx` | Charts, stat cards, gradients |
| `src/pages/payroll/*.tsx` | Tables, currency highlights |
| `src/pages/reports/*.tsx` | Data visualization |
| `src/pages/attendance/*.tsx` | Status badges (present/absent/late colors) |
| `src/pages/login/*.tsx` or auth pages | Full-page layouts with gradients |
| `src/pages/settings/*.tsx` | Complex forms (Phase 5 handles Settings separately) |

### Status Badge Colors
Status badges (present, absent, late, on-leave) typically use colored backgrounds. These should adapt:
- `bg-emerald-50 text-emerald-700` -> add `dark:bg-emerald-950 dark:text-emerald-300`
- `bg-red-50 text-red-700` -> add `dark:bg-red-950 dark:text-red-300`
- `bg-amber-50 text-amber-700` -> add `dark:bg-amber-950 dark:text-amber-300`
- `bg-blue-50 text-blue-700` -> add `dark:bg-blue-950 dark:text-blue-300`

## Related Code Files

### Files to Modify
All files in:
- `webdashboard/src/pages/**/*.tsx`
- `webdashboard/src/components/**/*.tsx` (non-layout, non-UI)

Excluding (already done in earlier phases):
- `src/components/layout/*` (Phase 2)
- `src/components/ui/*` (Phase 3)
- `src/pages/settings/GeneralSettings.tsx` (Phase 5)

## Implementation Steps

### Step 1: Audit Page Count

Run glob to get full list of TSX files needing updates:
```bash
find webdashboard/src/pages -name "*.tsx" | wc -l
find webdashboard/src/components -name "*.tsx" \
  --not -path "*/layout/*" --not -path "*/ui/*" | wc -l
```

### Step 2: Automated Bulk Replace

Use a script (or IDE multi-file replace) for the safe patterns listed above. Preview each replacement before applying. Only replace patterns that don't already have `dark:` immediately after.

**Regex to avoid double-adding**:
```regex
bg-white(?! dark:)
```
This matches `bg-white` only when NOT already followed by ` dark:`.

### Step 3: Manual Review - Dashboard

Dashboard likely has:
- Stat cards with gradient backgrounds
- Chart containers
- Welcome/hero section

Gradients: add `dark:from-*` `dark:to-*` variants.
Chart backgrounds: ensure container is `bg-card` or `dark:bg-slate-900`.

### Step 4: Manual Review - Tables

Data tables across attendance, payroll, employees:
- Table header: `bg-slate-50` -> `dark:bg-slate-800`
- Table row hover: `hover:bg-slate-50` -> `dark:hover:bg-slate-800`
- Table row borders: `border-slate-100` -> `dark:border-slate-800`
- Alternating rows (if any): adjust dark alternation

### Step 5: Manual Review - Auth Pages

Login/register pages (if they exist outside MainLayout):
- Full-page background
- Form card container
- Input fields

### Step 6: Manual Review - Status Badges

Search for colored badge patterns and add dark variants per mapping above.

### Step 7: Visual QA Pass

Toggle dark mode, navigate every page route:
1. Dashboard
2. Employees list
3. Attendance logs
4. Leave requests/balances
5. Payroll
6. Reports
7. Devices/Agents
8. Profile
9. ESS pages
10. All settings sub-pages

Document any remaining white patches.

## Todo List
- [ ] Audit: list all TSX files needing changes
- [ ] Run bulk search-replace for common patterns (with preview)
- [ ] Manual: Dashboard page
- [ ] Manual: Data table components
- [ ] Manual: Auth/login pages
- [ ] Manual: Status badges across all pages
- [ ] Manual: Any remaining gradient/custom styles
- [ ] Visual QA: navigate all routes in dark mode
- [ ] Fix any remaining white patches found in QA

## Success Criteria
- All page routes render without white patches in dark mode
- Status badges readable with proper contrast
- Tables, forms, cards all have appropriate dark backgrounds
- No `bg-white` without corresponding `dark:` variant (verified by grep)

## Risk Assessment
| Risk | Impact | Mitigation |
|------|--------|------------|
| Bulk replace introduces bugs | Broken class strings | Preview all replacements, test after |
| Missed pages | White patches | Visual QA covers all routes |
| Chart libraries ignore dark mode | Charts unreadable | May need chart-specific theme config |
| Too many files to handle at once | Overwhelm | Prioritize visible pages first, defer rarely-visited ones |

## Security Considerations
- No security impact; purely visual changes.

## Next Steps
- Phase 5: Settings page dark mode and toggle UI (can run in parallel with Phase 4)

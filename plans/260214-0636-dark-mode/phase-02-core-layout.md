# Phase 2: Core Layout

## Context Links
- [Phase 1: Foundation](phase-01-foundation.md)
- [Navbar](../../webdashboard/src/components/layout/Navbar.tsx) (83 lines)
- [Sidebar](../../webdashboard/src/components/layout/Sidebar.tsx) (137 lines)
- [MainLayout](../../webdashboard/src/components/layout/MainLayout.tsx) (149 lines)

## Overview
- **Priority**: P1 (core visual shell)
- **Status**: ✅ complete
- **Effort**: 1.5h (completed 2026-02-14)
- **Description**: Add dark mode variants to the three layout components that frame every page.

## Key Insights
- Navbar uses hardcoded `bg-white/80`, `border-slate-100`, `text-slate-*` — needs `dark:` variants
- Sidebar already uses CSS vars for some colors (`var(--sidebar-background)`) — these auto-switch after Phase 1
- MainLayout has hardcoded `bg-[#fcfcfd]` — replace with `bg-background` semantic token
- Mobile sidebar overlay backdrop needs dark-aware opacity

## Requirements

### Functional
- F1: Navbar background, borders, text adapt to dark mode
- F2: Sidebar background, borders, nav links, active states adapt
- F3: MainLayout content area background adapts
- F4: Mobile sidebar overlay adapts
- F5: All hover/focus states have dark variants

### Non-Functional
- NF1: No layout shift on theme change
- NF2: Glassmorphism effects look good in both modes

## Architecture

### Color Mapping Strategy

**Navbar** (`bg-white/80` light):
| Light | Dark |
|-------|------|
| `bg-white/80` | `dark:bg-slate-900/80` |
| `border-slate-100` | `dark:border-slate-800` |
| `text-slate-500` | `dark:text-slate-400` |
| `text-slate-700` | `dark:text-slate-200` |
| `bg-slate-50` | `dark:bg-slate-800` |
| `bg-slate-100` | `dark:bg-slate-700` |
| `hover:bg-slate-100` | `dark:hover:bg-slate-700` |
| `shadow-slate-900/5` | `dark:shadow-black/20` |

**Sidebar** (`bg-white` light):
| Light | Dark |
|-------|------|
| `bg-white` | `dark:bg-slate-950` |
| `border-slate-100` | `dark:border-slate-800` |
| `bg-slate-50` | `dark:bg-slate-800` |
| `text-slate-500` | `dark:text-slate-400` |
| `shadow-slate-200/50` | `dark:shadow-black/30` |

**MainLayout**:
| Light | Dark |
|-------|------|
| `bg-[#fcfcfd]` | Replace with `bg-background` (uses CSS var) |

## Related Code Files

### Files to Modify
| File | Changes |
|------|---------|
| `webdashboard/src/components/layout/Navbar.tsx` | Add `dark:` variants to all color classes |
| `webdashboard/src/components/layout/Sidebar.tsx` | Add `dark:` variants, update shadow colors |
| `webdashboard/src/components/layout/MainLayout.tsx` | Replace hardcoded bg, add dark overlay |

## Implementation Steps

### Step 1: MainLayout.tsx

1. Replace `bg-[#fcfcfd]` with `bg-background` (semantic Tailwind token from Phase 1)
2. Update mobile sidebar backdrop: add `dark:bg-slate-950/60`
3. Any `border-slate-*` gets corresponding `dark:border-slate-*` variant

### Step 2: Navbar.tsx

Line-by-line changes on the `<header>`:
```
Current:  "h-20 border-b border-slate-100 bg-white/80 backdrop-blur-xl ... shadow-2xl shadow-slate-900/5"
Updated:  "h-20 border-b border-slate-100 dark:border-slate-800 bg-white/80 dark:bg-slate-900/80 backdrop-blur-xl ... shadow-2xl shadow-slate-900/5 dark:shadow-black/20"
```

Menu button:
```
Current:  "bg-slate-50 text-slate-500 hover:bg-slate-100 ... border-slate-100"
Updated:  "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 ... border-slate-100 dark:border-slate-700"
```

Collapse toggle: same pattern.

Breadcrumb workspace name: `text-slate-500` -> add `dark:text-slate-400`
Breadcrumb page badge: `text-slate-700 bg-slate-100` -> add `dark:text-slate-200 dark:bg-slate-800`
Breadcrumb separator: `text-slate-300` -> add `dark:text-slate-600`

Bell icon area: `text-slate-500` -> add `dark:text-slate-400`, `hover:border-slate-100` -> add `dark:hover:border-slate-700`
Notification dot: `border-white` -> add `dark:border-slate-900`

Divider: `bg-slate-100` -> add `dark:bg-slate-800`

User name: `text-slate-700` -> add `dark:text-slate-200`
User subtitle: `text-slate-500` -> add `dark:text-slate-400`
Avatar: `bg-slate-900` stays (already dark), `shadow-slate-400` -> add `dark:shadow-black`

### Step 3: Sidebar.tsx

Outer container:
```
Current:  "bg-white border-r border-slate-100 shadow-xl shadow-slate-200/50"
Updated:  "bg-white dark:bg-slate-950 border-r border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-200/50 dark:shadow-black/30"
```

Logo gradient: keep as-is (primary colors work in both modes).

Nav section headers: `text-slate-400` -> add `dark:text-slate-500`

Inactive link: `text-slate-500 hover:bg-slate-50` -> add `dark:text-slate-400 dark:hover:bg-slate-800`
Active link: `bg-primary-50 text-primary-700` -> add `dark:bg-primary-950 dark:text-primary-300`

Logout button: `bg-slate-50 group-hover:bg-white` -> add `dark:bg-slate-800 dark:group-hover:bg-slate-700`
Logout text: `text-slate-500` -> add `dark:text-slate-400`

## Todo List
- [x] Update MainLayout.tsx: replace `bg-[#fcfcfd]` with `bg-background`, dark overlay ✅
- [x] Update Navbar.tsx: all `dark:` variants per mapping table ✅
- [x] Update Sidebar.tsx: all `dark:` variants per mapping table ✅
- [x] Visual test: toggle `.dark` class, verify all three components ✅
- [x] Check glassmorphism (Navbar backdrop-blur) in dark mode ✅
- [x] Verify mobile sidebar overlay in dark mode ✅

## Success Criteria
- Layout shell looks cohesive in dark mode (no white "holes")
- Glassmorphism/blur effects render properly on dark backgrounds
- All interactive states (hover, active, focus) have dark variants
- No text contrast issues (WCAG AA: 4.5:1 ratio minimum)

## Risk Assessment
| Risk | Impact | Mitigation |
|------|--------|------------|
| Missed hardcoded color | White patch in dark mode | Visual QA pass after implementation |
| Sidebar CSS var colors not switching | Sidebar stays light | Phase 1 `.dark` block covers sidebar vars |
| Shadow too harsh in dark | Poor aesthetics | Use lower opacity shadows in dark |

## Security Considerations
- No security impact; purely visual changes.

## Next Steps
- After layout shell is dark-ready, Phase 3 handles UI primitives (modals, toasts) that overlay on top

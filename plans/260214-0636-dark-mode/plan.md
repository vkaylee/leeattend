---
title: "Dark Mode Implementation"
description: "Full-app dark mode with system preference detection and manual toggle"
status: in-progress
priority: P2
effort: 8h
branch: main
tags: [frontend, ui, dark-mode, tailwind]
created: 2026-02-14
---

# Dark Mode Implementation Plan

## Architecture

**Strategy**: Hybrid CSS variables + Tailwind `dark:` prefix.
- Define light/dark values in `:root` / `.dark` selectors
- Register via `@theme` for Tailwind integration
- Add `@variant dark (&:where(.dark, .dark *))` for class-based toggling
- Separate `useDarkMode` hook (NOT merged into existing `useTheme` color hook)
- FOUC prevention via inline `<script>` in `index.html` `<head>`
- localStorage key: `theme_mode` (values: `light` | `dark` | `system`)

## Dependencies

- Tailwind CSS v4 (already installed, CSS-first config)
- Lucide React icons: `Sun`, `Moon`, `Monitor` (already available)
- i18next translations for 3 languages (en, vi, zh)

## Phases

| # | Phase | Status | Effort | File |
|---|-------|--------|--------|------|
| 1 | Foundation (CSS vars, dark variant, FOUC, hook) | ✅ complete | 1.5h | [phase-01](phase-01-foundation.md) |
| 2 | Core Layout (Navbar, Sidebar, MainLayout) | pending | 1.5h | [phase-02](phase-02-core-layout.md) |
| 3 | UI Primitives (Modal, Toast, Dropdown, etc) | pending | 1h | [phase-03](phase-03-ui-primitives.md) |
| 4 | Feature Pages (all page components) | pending | 2h | [phase-04](phase-04-feature-pages.md) |
| 5 | Settings & Toggle (Navbar toggle + Settings page) | pending | 1h | [phase-05](phase-05-settings-and-toggle.md) |
| 6 | Testing (unit + visual verification) | pending | 1h | [phase-06](phase-06-testing.md) |

## Key Decisions

1. **Separate hook** (`use-dark-mode.ts`) keeps color theme and dark mode concerns isolated
2. **CSS vars for semantic tokens** (background, foreground, card, sidebar) auto-switch; hardcoded Tailwind classes get `dark:` variants
3. **No third-party library** needed; Tailwind v4 + custom variant sufficient
4. **Incremental rollout**: Foundation + Layout first, then pages can be updated progressively
5. **Smooth transitions**: `transition-colors duration-200` on `<html>` element

## Risk Summary

- ~215 TSX files need `dark:` variants; bulk search-replace for common patterns reduces effort
- Glass/blur effects need careful dark-mode tuning (opacity, backdrop)
- Print styles unaffected (already scoped via `@media print`)

## Validation Summary

**Validated:** 2026-02-14
**Questions asked:** 4

### Confirmed Decisions
- **Bulk replace strategy**: All-at-once across 215 files, then visual QA (fastest approach)
- **Hook design**: Separate `use-dark-mode.ts` from `use-theme.ts` (separation of concerns)
- **Dark palette**: Slate-based (slate-900 `#0f172a`, slate-800 `#1e293b`) — neutral, professional
- **Toggle UX**: Cycle on click (light→dark→system) with icon change + tooltip

### Action Items
- None — all answers aligned with plan defaults. No revisions needed.

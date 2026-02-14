# Phase 3: UI Primitives

## Context Links
- [Phase 2: Core Layout](phase-02-core-layout.md)
- [Modal](../../webdashboard/src/components/ui/Modal.tsx)
- [Toast](../../webdashboard/src/components/ui/Toast.tsx)
- [Combobox](../../webdashboard/src/components/ui/Combobox.tsx)
- [Sheet](../../webdashboard/src/components/ui/Sheet.tsx)
- [dropdown-menu](../../webdashboard/src/components/ui/dropdown-menu.tsx)
- [Tabs](../../webdashboard/src/components/ui/Tabs.tsx)
- [LanguageSwitcher](../../webdashboard/src/components/ui/LanguageSwitcher.tsx)

## Overview
- **Priority**: P1 (overlays render on top of everything)
- **Status**: pending
- **Effort**: 1h
- **Description**: Add dark mode variants to all shared UI primitives: Modal, Toast, Combobox, Sheet, dropdown-menu, Tabs, LanguageSwitcher.

## Key Insights
- `modal-content` and `glass-premium` utilities already updated in Phase 1 CSS; this phase handles component-level classes
- Toast likely uses its own background colors for success/error/info variants
- Combobox dropdown panel needs dark background + borders
- Sheet (slide-in panel) similar to Modal treatment
- These 7 components are used across all pages; fixing them cascades to entire app

## Requirements

### Functional
- F1: All overlays (Modal, Sheet, dropdown) have dark backgrounds
- F2: Toast notifications readable in dark mode
- F3: Combobox dropdown, search input, option highlights dark-aware
- F4: Tab active/inactive states have dark variants
- F5: LanguageSwitcher dropdown dark-aware

### Non-Functional
- NF1: Consistent border/shadow treatment across all primitives
- NF2: Focus rings visible in dark mode

## Architecture

### Common Dark Pattern for Overlays
```
Light panel:   bg-white border-slate-100 shadow-xl text-slate-700
Dark panel:    dark:bg-slate-900 dark:border-slate-700 dark:shadow-black/30 dark:text-slate-200
```

### Common Dark Pattern for Inputs
```
Light input:   bg-slate-50 border-slate-100 text-slate-700 placeholder:text-slate-400
Dark input:    dark:bg-slate-800 dark:border-slate-700 dark:text-slate-200 dark:placeholder:text-slate-500
```

## Related Code Files

### Files to Modify
| File | Key Changes |
|------|-------------|
| `src/components/ui/Modal.tsx` | Content panel bg, header/footer borders, close button |
| `src/components/ui/Toast.tsx` | Toast container bg, variant colors (success/error/info) |
| `src/components/ui/Combobox.tsx` | Dropdown panel, search input, option rows, hover states |
| `src/components/ui/Sheet.tsx` | Slide-in panel bg, overlay backdrop |
| `src/components/ui/dropdown-menu.tsx` | Menu panel, item hover, separator |
| `src/components/ui/Tabs.tsx` | Tab bar bg, active/inactive tab states |
| `src/components/ui/LanguageSwitcher.tsx` | Dropdown panel, selected state |

## Implementation Steps

### Step 1: Modal.tsx

Read file, identify hardcoded colors. Apply pattern:
- Content wrapper: add `dark:bg-slate-900 dark:border-slate-700`
- Close button: add `dark:text-slate-400 dark:hover:text-slate-200 dark:hover:bg-slate-800`
- Header text: add `dark:text-slate-100`
- Dividers/borders: add `dark:border-slate-700`

Note: `.modal-content` utility already updated in Phase 1 CSS. If Modal.tsx uses that class, it auto-inherits. Only override if component uses inline Tailwind classes.

### Step 2: Toast.tsx

Read file. Toasts typically have variant-specific styling:
- Success: `bg-emerald-50 text-emerald-800` -> add `dark:bg-emerald-950 dark:text-emerald-200`
- Error: `bg-red-50 text-red-800` -> add `dark:bg-red-950 dark:text-red-200`
- Info/Default: `bg-white text-slate-700` -> add `dark:bg-slate-800 dark:text-slate-200`
- Toast container border: add `dark:border-slate-700`

### Step 3: Combobox.tsx

Dropdown panel:
- `bg-white border-slate-200 shadow-xl` -> add `dark:bg-slate-900 dark:border-slate-700 dark:shadow-black/30`

Search input inside dropdown:
- `bg-slate-50 border-slate-100` -> add `dark:bg-slate-800 dark:border-slate-700`

Option rows:
- `hover:bg-slate-50` -> add `dark:hover:bg-slate-800`
- Selected: `bg-primary-50 text-primary-700` -> add `dark:bg-primary-950 dark:text-primary-300`

### Step 4: Sheet.tsx

- Overlay: same as modal backdrop
- Panel: `bg-white` -> add `dark:bg-slate-900`
- Borders: add dark variants

### Step 5: dropdown-menu.tsx

Likely uses Radix UI primitives. Add dark variants to:
- Menu content: `bg-white` -> add `dark:bg-slate-900`
- Menu item hover: `hover:bg-slate-50` -> add `dark:hover:bg-slate-800`
- Separator: `bg-slate-100` -> add `dark:bg-slate-800`
- Disabled items: `text-slate-400` -> add `dark:text-slate-600`

### Step 6: Tabs.tsx

- Tab bar container: `bg-slate-100` -> add `dark:bg-slate-800`
- Active tab: `bg-white text-primary-600` -> add `dark:bg-slate-900 dark:text-primary-400`
- Inactive tab: `text-slate-500` -> add `dark:text-slate-400`

### Step 7: LanguageSwitcher.tsx

- Trigger button: follow Navbar button pattern from Phase 2
- Dropdown: follow dropdown-menu pattern

## Todo List
- [ ] Update Modal.tsx with dark variants
- [ ] Update Toast.tsx with dark variants per toast type
- [ ] Update Combobox.tsx (panel, input, options)
- [ ] Update Sheet.tsx
- [ ] Update dropdown-menu.tsx
- [ ] Update Tabs.tsx
- [ ] Update LanguageSwitcher.tsx
- [ ] Visual test: open each component in dark mode

## Success Criteria
- All overlays render on dark background without white flashes
- Toast notifications readable in all variant colors (success/error/info)
- Combobox dropdown usable (search, select, scroll) in dark mode
- Focus rings visible on dark backgrounds
- Consistent look between all overlay types

## Risk Assessment
| Risk | Impact | Mitigation |
|------|--------|------------|
| Radix UI injects inline styles | Overrides dark classes | Use `!important` or CSS specificity if needed |
| Toast variant colors hard to read | Accessibility issue | Test contrast ratios |
| Combobox dropdown z-index | Appears behind dark overlay | Verify stacking context |

## Security Considerations
- No security impact; purely visual changes.

## Next Steps
- Phase 4: Feature pages use these primitives; once primitives are dark-ready, pages inherit most styling

# Phase 1: Foundation

## Context Links
- [Research: Tailwind v4 Dark Mode](research/researcher-01-tailwind-v4-dark-mode.md)
- [Scout: Webdashboard Structure](scout/scout-01-webdashboard-structure.md)
- [Existing CSS](../../webdashboard/src/index.css)
- [Existing useTheme](../../webdashboard/src/hooks/use-theme.ts)
- [DebugContext pattern](../../webdashboard/src/context/DebugContext.tsx)

## Overview
- **Priority**: P1 (blocks all other phases)
- **Status**: ✅ completed
- **Effort**: 1.5h
- **Description**: Set up dark mode infrastructure: CSS variables, Tailwind dark variant, FOUC prevention script, and `useDarkMode` hook.

## Key Insights
- Tailwind v4 requires `@variant dark (&:where(.dark, .dark *))` in CSS for class-based toggle
- Existing `:root` CSS vars (`--background`, `--foreground`, `--card`, `--sidebar-*`) are perfect for dark overrides
- FOUC script must be inline in `<head>`, blocking, before any rendering
- `useTheme` hook manages color palette only; dark mode is orthogonal concern

## Requirements

### Functional
- F1: `.dark` class on `<html>` activates dark color scheme
- F2: System preference detection via `prefers-color-scheme` media query
- F3: Manual override stored in `localStorage.theme_mode`
- F4: Real-time response to OS theme changes when mode is "system"
- F5: Cross-tab sync via `storage` event listener
- F6: No FOUC on page load

### Non-Functional
- NF1: Zero additional npm dependencies
- NF2: Hook under 80 lines
- NF3: CSS additions under 50 lines

## Architecture

### Data Flow
```
index.html FOUC script
  -> reads localStorage / matchMedia
  -> sets .dark class on <html> synchronously

React app loads
  -> DarkModeProvider wraps app
  -> useDarkMode() reads state
  -> Components render with dark: variants
  -> Toggle calls setMode() -> updates localStorage + class + dispatches event
```

### State Model
```typescript
type ThemeMode = 'light' | 'dark' | 'system';
// localStorage key: 'theme_mode'
// Resolved: always 'light' | 'dark' (system resolves to one)
```

## Related Code Files

### Files to Modify
| File | Change |
|------|--------|
| `webdashboard/index.html` | Add FOUC prevention script in `<head>` |
| `webdashboard/src/index.css` | Add `@variant dark`, `.dark` CSS var overrides, dark scrollbar/glass/modal |
| `webdashboard/src/App.tsx` | Wrap with `DarkModeProvider` |

### Files to Create
| File | Purpose |
|------|---------|
| `webdashboard/src/hooks/use-dark-mode.ts` | Core dark mode hook with localStorage + matchMedia |
| `webdashboard/src/context/DarkModeContext.tsx` | React context provider |

## Implementation Steps

### Step 1: Add Dark Variant to CSS (`src/index.css`)

Insert after `@plugin "tailwindcss-animate"`:
```css
@variant dark (&:where(.dark, .dark *));
```

### Step 2: Add `.dark` CSS Variable Overrides (`src/index.css`)

Add after existing `:root` block:
```css
.dark {
  --background: #0f172a;
  --foreground: #e2e8f0;

  --card: #1e293b;
  --card-foreground: #e2e8f0;

  --sidebar-background: #0f172a;
  --sidebar-foreground: #94a3b8;
  --sidebar-primary: var(--primary-500);
  --sidebar-border: #1e293b;
}
```

### Step 3: Update `@theme` to Include Semantic Tokens

Add to `@theme` block:
```css
--color-background: var(--background);
--color-foreground: var(--foreground);
--color-card: var(--card);
--color-card-foreground: var(--card-foreground);
--color-sidebar-bg: var(--sidebar-background);
--color-sidebar-fg: var(--sidebar-foreground);
--color-sidebar-border: var(--sidebar-border);
```

This allows using `bg-background`, `text-foreground`, `bg-card` etc. as Tailwind utilities that auto-switch.

### Step 4: Update Dark-Aware Utility Classes (`src/index.css`)

Update existing utilities for dark mode:
```css
/* Scrollbar */
.dark ::-webkit-scrollbar-thumb {
  @apply bg-slate-700 hover:bg-primary-400;
}

/* Glass */
.glass-premium {
  @apply bg-white/70 dark:bg-slate-900/70 backdrop-blur-xl border border-white/40 dark:border-slate-700/40 shadow-premium-lg;
}

/* Modal */
.modal-content {
  @apply bg-white dark:bg-slate-900 w-full max-w-md rounded-[3rem] shadow-premium-2xl border border-white/60 dark:border-slate-700/60 animate-in zoom-in-95 slide-in-from-bottom-8 duration-500 overflow-hidden;
}
```

### Step 5: FOUC Prevention Script (`index.html`)

Add in `<head>`, before `</head>`:
```html
<script>
  (function() {
    var stored = localStorage.getItem('theme_mode');
    var isDark = stored === 'dark' ||
      (!stored || stored === 'system') && window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (isDark) document.documentElement.classList.add('dark');
  })();
</script>
```

Also add `class="transition-colors duration-200"` to `<html>` tag for smooth transitions.

### Step 6: Create `use-dark-mode.ts` Hook

Path: `webdashboard/src/hooks/use-dark-mode.ts`

```typescript
import { useState, useEffect, useCallback } from 'react';

export type ThemeMode = 'light' | 'dark' | 'system';
const STORAGE_KEY = 'theme_mode';

function getSystemPreference(): 'light' | 'dark' {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function resolveTheme(mode: ThemeMode): 'light' | 'dark' {
  return mode === 'system' ? getSystemPreference() : mode;
}

function applyTheme(resolved: 'light' | 'dark') {
  document.documentElement.classList.toggle('dark', resolved === 'dark');
}

export function useDarkMode() {
  const [mode, setModeState] = useState<ThemeMode>(() => {
    return (localStorage.getItem(STORAGE_KEY) as ThemeMode) || 'system';
  });

  const resolved = resolveTheme(mode);

  const setMode = useCallback((newMode: ThemeMode) => {
    localStorage.setItem(STORAGE_KEY, newMode);
    setModeState(newMode);
    applyTheme(resolveTheme(newMode));
    window.dispatchEvent(new Event('theme-mode-change'));
  }, []);

  // Apply on mount + listen for system preference changes
  useEffect(() => {
    applyTheme(resolved);
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = () => {
      if (mode === 'system') applyTheme(getSystemPreference());
    };
    mq.addEventListener('change', handleChange);
    return () => mq.removeEventListener('change', handleChange);
  }, [mode, resolved]);

  // Cross-tab sync
  useEffect(() => {
    const handleStorage = (e: StorageEvent) => {
      if (e.key === STORAGE_KEY) {
        const val = (e.newValue as ThemeMode) || 'system';
        setModeState(val);
        applyTheme(resolveTheme(val));
      }
    };
    window.addEventListener('storage', handleStorage);
    return () => window.removeEventListener('storage', handleStorage);
  }, []);

  return { mode, resolved, setMode, isDark: resolved === 'dark' };
}
```

### Step 7: Create `DarkModeContext.tsx`

Path: `webdashboard/src/context/DarkModeContext.tsx`

```typescript
import React, { createContext, useContext } from 'react';
import { useDarkMode, ThemeMode } from '@/hooks/use-dark-mode';

interface DarkModeContextType {
  mode: ThemeMode;
  resolved: 'light' | 'dark';
  isDark: boolean;
  setMode: (mode: ThemeMode) => void;
}

const DarkModeContext = createContext<DarkModeContextType | undefined>(undefined);

export const DarkModeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const darkMode = useDarkMode();
  return (
    <DarkModeContext.Provider value={darkMode}>
      {children}
    </DarkModeContext.Provider>
  );
};

export const useDarkModeContext = () => {
  const context = useContext(DarkModeContext);
  if (!context) throw new Error('useDarkModeContext must be used within DarkModeProvider');
  return context;
};
```

### Step 8: Wire Provider into App (`App.tsx`)

Wrap inside `DebugProvider`:
```tsx
import { DarkModeProvider } from "./context/DarkModeContext";

// In render:
<DebugProvider>
  <DarkModeProvider>
    <Router>
      <AppRoutes />
    </Router>
  </DarkModeProvider>
  <DebugToolbar />
</DebugProvider>
```

## Todo List
- [x] Add `@variant dark` to `index.css`
- [x] Add `.dark` CSS variable overrides
- [x] Register semantic tokens in `@theme`
- [x] Update glass-premium, modal-content, scrollbar for dark
- [x] Add FOUC script to `index.html`
- [x] Add `transition-colors duration-200` to `<html>`
- [x] Create `use-dark-mode.ts` hook
- [x] Create `DarkModeContext.tsx` provider (split into 3 files per ESLint)
- [x] Wire `DarkModeProvider` into `App.tsx`
- [x] Verify: toggle `.dark` class manually in DevTools, confirm CSS vars switch
- [x] Code review completed (score: 9.2/10)
- [x] All tests passing (465 unit, 2 E2E baseline)

## Success Criteria
- `.dark` class on `<html>` flips all CSS variable values
- `dark:` prefix works in any component
- No flash of wrong theme on page load (FOUC test: set to dark, hard refresh)
- OS theme change reflected when mode is "system"
- Theme persists across page refresh and tabs

## Risk Assessment
| Risk | Impact | Mitigation |
|------|--------|------------|
| `@variant` syntax wrong for v4 | Blocks all phases | Verify with `dark:bg-red-500` test class |
| FOUC script timing | UX flash | Script in `<head>` is blocking by design |
| CSS var naming collision | Broken styles | Use existing naming convention from `:root` |

## Security Considerations
- localStorage only stores theme preference string, no sensitive data
- No API calls needed for dark mode (client-side only)

## Next Steps
- Phase 2 depends on this phase completing
- After foundation, test by manually adding `dark:bg-slate-900` to `<body>` in DevTools

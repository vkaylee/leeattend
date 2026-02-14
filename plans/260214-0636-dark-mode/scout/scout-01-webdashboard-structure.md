# Scout Report: Dark Mode Implementation - Webdashboard Structure

**Date**: 2026-02-14
**Scout**: Codebase Scout Agent
**Mission**: Identify all files relevant to dark mode implementation in webdashboard

---

## Executive Summary

Webdashboard uses **Tailwind CSS v4** with CSS-first config (`@theme` directive). Existing theme system handles color palette (violet/indigo/rose etc) but NO dark mode. CSS variables in `:root` are ready for dark mode variants. Layout shell complete with existing `useTheme` hook for color switching.

**Total TypeScript files**: 215
**LocalStorage usage**: 15 files (91 occurrences) — theme persistence already established

---

## 1. Tailwind Config & CSS System

### 1.1 PostCSS Config
**File**: `/home/elt1541/lee/leeattend/webdashboard/postcss.config.mjs`
- Minimal config using `@tailwindcss/postcss` plugin
- No custom dark mode config yet

### 1.2 Main CSS File
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/index.css`
- **Lines**: 157
- **Tailwind v4**: Uses `@import "tailwindcss"` and `@theme` directive
- **CSS Variables**: Extensive use in `:root`
  - `--background: #f8fafc` (light gray)
  - `--foreground: #0f172a` (dark slate)
  - `--card: #ffffff`
  - `--sidebar-background: #ffffff`
  - `--sidebar-foreground: #64748b`
  - `--primary-50` through `--primary-950` (dynamic color palette)
- **Custom utilities**: `.glass-premium`, `.modal-backdrop`, `.scrollbar-hide`
- **Missing**: No `.dark` class variants for CSS variables

**Key Insight**: All colors use CSS variables — perfect for dark mode. Just need `.dark` variants.

---

## 2. HTML Entry Point

**File**: `/home/elt1541/lee/leeattend/webdashboard/index.html`
- **Lines**: 17
- Simple structure, `<div id="root"></div>`
- **No FOUC prevention script** — needs adding
- Ideal place for:
  ```js
  <script>
    // Prevent FOUC by reading localStorage before React loads
    if (localStorage.theme === 'dark' || (!localStorage.theme && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
      document.documentElement.classList.add('dark');
    }
  </script>
  ```

---

## 3. App Entry & Providers

### 3.1 Main Entry
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/main.tsx`
- **Lines**: 16
- ReactDOM renders `<App />` wrapped in `<ToastProvider>`
- Imports CSS: `./index.css`, `@fontsource/inter`
- **Good place for**: ThemeProvider wrapper

### 3.2 App Component
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/App.tsx`
- **Lines**: 32
- Provider stack: `QueryClientProvider > DebugProvider > Router > AppRoutes`
- **DebugToolbar** rendered globally
- **Opportunity**: Add ThemeProvider after DebugProvider

### 3.3 Routes
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/AppRoutes.tsx`
- **Lines**: 92
- All authenticated routes wrapped in `<MainLayout />`
- Settings routes at `/workspace/settings/*`

---

## 4. Layout Components

### 4.1 MainLayout (Shell Component)
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/components/layout/MainLayout.tsx`
- **Lines**: 149
- **Already uses `useTheme()` hook** (line 7, 17)
- **Structure**:
  - Sidebar (collapsible)
  - Navbar (header with breadcrumbs)
  - Main content area (`<Outlet />`)
- **Hardcoded colors**:
  - Line 90: `bg-[#fcfcfd]` (light background)
  - Line 92-95: Backdrop overlay for mobile sidebar
- **CSS classes**: Uses Tailwind utilities extensively
- **State**: Manages sidebar open/collapsed via `localStorage`

**Action needed**: 
- Replace hardcoded colors with CSS variable classes
- Add dark mode aware backdrop

### 4.2 Sidebar
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/components/layout/Sidebar.tsx`
- **Lines**: 137
- **Hardcoded colors**:
  - Line 100: `bg-white border-r border-slate-100 shadow-xl shadow-slate-200/50`
  - Line 102: Logo gradient `bg-gradient-to-tr from-primary-600 to-indigo-500`
  - Line 114-116: Active/inactive link states with `bg-primary-50`, `bg-slate-50`
  - Line 132: Logout button `bg-slate-50 group-hover:bg-white`
- **Icons**: Lucide React (48 imports!)
- **Navigation**: Sections for org, time, payroll, system, ESS
- **Test IDs**: Already present (`data-testid` attributes)

**Action needed**:
- Replace `bg-white` → `bg-white dark:bg-slate-900`
- Update border colors for dark mode
- Adjust shadow colors

### 4.3 Navbar
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/components/layout/Navbar.tsx`
- **Lines**: 83
- **Hardcoded colors**:
  - Line 36: `bg-white/80 backdrop-blur-xl border-b border-slate-100 shadow-2xl`
  - Line 38: Menu button `bg-slate-50 hover:bg-slate-100`
  - Line 44: Collapse toggle `bg-slate-50 hover:bg-primary-50`
  - Line 53: Breadcrumb badge `bg-slate-100`
  - Line 75: User avatar `bg-slate-900 text-white`
- **Components**: 
  - `<LanguageSwitcher />` (line 61)
  - Bell notification icon
  - User profile section
- **Responsiveness**: Mobile menu toggle, collapse toggle

**Action needed**:
- Add dark mode variants to all background colors
- Update glassmorphism effect for dark mode
- **Ideal place for dark mode toggle button** (next to LanguageSwitcher)

---

## 5. Settings Pages

### 5.1 Settings Layout
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/pages/settings/SettingsLayout.tsx`
- **Lines**: 119
- **Sidebar navigation** for settings categories:
  - General, People, Time, Payroll, Leave, Connectivity
- **Styling**: Premium rounded cards with shadows
- Active route highlighting with `bg-primary-50`

### 5.2 General Settings (Appearance Tab)
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/pages/settings/GeneralSettings.tsx`
- **Lines**: 140
- **Already has Appearance tab!** (line 82, 110-135)
- **Two tabs**: "Localization" and "Appearance"
- **Appearance tab features**:
  - Primary color picker (7 presets: violet, indigo, blue, emerald, amber, rose, slate)
  - Custom hex input
  - Live preview via `useTheme().setPreviewTheme()`
- **Settings stored**:
  - `workspace.theme_primary_color` (API)
  - Date/time format, timezone, language

**Perfect spot for dark mode toggle!** (line 115-131)

**Suggested addition**:
```tsx
<div className="space-y-6">
  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-widest ml-1">
    {t("common.workspace_settings.theme_mode")}
  </label>
  <div className="flex items-center gap-4">
    <button onClick={() => setThemeMode("light")} className={cn("...")}>
      <Sun size={20} /> Light
    </button>
    <button onClick={() => setThemeMode("dark")} className={cn("...")}>
      <Moon size={20} /> Dark
    </button>
    <button onClick={() => setThemeMode("system")} className={cn("...")}>
      <Monitor size={20} /> System
    </button>
  </div>
</div>
```

---

## 6. Existing Theme System

### 6.1 useTheme Hook
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-theme.ts`
- **Lines**: 90
- **Purpose**: Manages primary color theme (not dark mode)
- **Features**:
  - Reads from localStorage (`user_theme_primary_color`)
  - Reads from workspace API (`workspace.theme_primary_color`)
  - Generates color shades (50-950) from base hex
  - Sets CSS variables on `document.documentElement`
  - Live preview support
  - Cross-tab sync via storage events
- **Color algorithm**: `adjustBrightness(hex, percent)` helper

**Key insight**: Already manipulates `:root` CSS vars — can extend for dark mode.

**NOT dark mode aware** — needs separate hook or extension:
- `useDarkMode()` or extend to `useTheme({ darkMode: true })`

### 6.2 Tests
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/hooks/__tests__/use-theme.test.tsx`
- Tests exist for color theme switching
- No dark mode tests yet

---

## 7. Context Providers

### 7.1 DebugContext
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/context/DebugContext.tsx`
- **Lines**: 83
- **Purpose**: Dev mode impersonation and mock dates
- **Uses localStorage**: `debug_enabled`, `debug_impersonate_id`, `debug_mock_date`
- **Pattern**: Context + Provider + custom hook

**Reusable pattern for ThemeContext**:
```tsx
// src/context/ThemeContext.tsx
export const ThemeProvider = ({ children }) => {
  const [theme, setTheme] = useState(() => localStorage.getItem('theme') || 'system');
  // ... dark mode logic
};
export const useTheme = () => useContext(ThemeContext);
```

---

## 8. Utility Functions

### 8.1 Utils
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/lib/utils.ts`
- **Likely contains**: `cn()` helper (Tailwind class merger)
- Used extensively throughout components (e.g., `cn("...", isActive && "...")`)

### 8.2 API Helpers
**File**: `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/helpers.ts`
- LocalStorage usage for auth tokens and workspace ID

---

## 9. UI Components (Potential Dark Mode Needs)

**Found**: 9 UI components in `/home/elt1541/lee/leeattend/webdashboard/src/components/ui/`

Relevant for dark mode:
- **Modal.tsx**: Backdrop and content styling
- **Toast.tsx**: Notification background/text
- **Combobox.tsx**: Dropdown styling
- **Sheet.tsx**: Slide-in panel
- **dropdown-menu.tsx**: Radix UI dropdown
- **LanguageSwitcher.tsx**: Already in Navbar, good reference

**All will need**: Dark mode color variants

---

## 10. Package Dependencies

**File**: `/home/elt1541/lee/leeattend/webdashboard/package.json`

**Relevant**:
- `tailwindcss: ^4` — CSS-first config with `@theme` directive
- `@tailwindcss/postcss: ^4`
- `tailwindcss-animate: ^1.0.7` — Animation support
- `lucide-react: ^0.562.0` — Icons (Sun, Moon, Monitor available)
- `clsx: ^2.1.1` + `tailwind-merge: ^3.4.0` → `cn()` utility
- `react: 19.2.3` — Latest React (Context API stable)
- `i18next: ^25.7.4` — i18n ready (need translations for "Dark Mode", "Light Mode", etc)

**No dark mode package needed** — native Tailwind v4 support.

---

## 11. LocalStorage Usage Analysis

**Files using localStorage**: 15 files (91 occurrences)

**Existing keys**:
- `user_theme_primary_color` (color theme)
- `sidebar-collapsed` (UI state)
- `debug_*` (dev tools)
- `i18n` (language)
- Auth tokens, workspace ID

**Proposed new key**: `theme` (values: `"light"` | `"dark"` | `"system"`)

**Consistency**: All hooks use `localStorage.getItem()` / `setItem()` pattern

---

## 12. Missing Components

**Need to create**:
1. **Dark mode toggle button** (Sun/Moon/Monitor icons)
   - Location: Navbar (next to LanguageSwitcher)
   - Or: Settings > Appearance tab
2. **ThemeProvider context** (optional, or extend useTheme)
3. **FOUC prevention script** in index.html
4. **Dark mode CSS variables** in index.css

**Do NOT need**:
- Third-party dark mode libraries (Tailwind v4 native)
- Complex state management (localStorage + CSS class sufficient)

---

## 13. Testing Infrastructure

**Test files found**: `__tests__` directories in pages, hooks, components, lib
**Test framework**: Vitest + React Testing Library + Playwright (e2e)
**Existing pattern**: `use-theme.test.tsx`, `Modal.test.tsx`

**Will need**:
- Unit tests for dark mode hook/context
- Integration tests for theme switching
- E2E tests for persistence across page navigation
- Visual regression tests (optional)

---

## 14. Internationalization (i18n)

**Files**: `/home/elt1541/lee/leeattend/webdashboard/src/i18n/config.ts`
**Hook**: `useTranslation()` from `react-i18next`
**Languages**: Vietnamese (vi), English (en), Chinese (zh)

**Translations needed**:
```json
{
  "common": {
    "dark_mode": "Dark Mode",
    "light_mode": "Light Mode",
    "system_mode": "System",
    "theme_mode": "Theme Mode"
  },
  "settings": {
    "appearance": {
      "theme_mode_desc": "Choose your preferred color scheme"
    }
  }
}
```

---

## 15. Key Color Classes to Update

**High-frequency Tailwind classes** (need dark: variants):
- `bg-white` → `bg-white dark:bg-slate-900`
- `bg-slate-50` → `bg-slate-50 dark:bg-slate-800`
- `text-slate-700` → `text-slate-700 dark:text-slate-200`
- `border-slate-100` → `border-slate-100 dark:border-slate-700`
- `shadow-xl` → `shadow-xl dark:shadow-slate-900/50`

**CSS variables** (add to index.css):
```css
:root {
  --background: #f8fafc;
  --foreground: #0f172a;
  /* ... */
}

.dark {
  --background: #0f172a;
  --foreground: #f8fafc;
  --card: #1e293b;
  --sidebar-background: #0f172a;
  /* ... */
}
```

---

## 16. Implementation Priority

**Phase 1: Foundation** (Core files)
1. `/home/elt1541/lee/leeattend/webdashboard/src/index.css` — Add `.dark` CSS vars
2. `/home/elt1541/lee/leeattend/webdashboard/index.html` — FOUC script
3. `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-theme.ts` — Extend or create `use-dark-mode.ts`
4. `/home/elt1541/lee/leeattend/webdashboard/src/context/ThemeContext.tsx` — New context (optional)

**Phase 2: Layout Shell**
5. `/home/elt1541/lee/leeattend/webdashboard/src/components/layout/MainLayout.tsx`
6. `/home/elt1541/lee/leeattend/webdashboard/src/components/layout/Sidebar.tsx`
7. `/home/elt1541/lee/leeattend/webdashboard/src/components/layout/Navbar.tsx`

**Phase 3: Toggle UI**
8. `/home/elt1541/lee/leeattend/webdashboard/src/components/ui/ThemeToggle.tsx` — New component
9. `/home/elt1541/lee/leeattend/webdashboard/src/pages/settings/GeneralSettings.tsx` — Add toggle to Appearance tab

**Phase 4: Components** (50+ pages/components)
10. All UI components in `/home/elt1541/lee/leeattend/webdashboard/src/components/ui/`
11. All page components (215 TS files)

**Phase 5: Testing**
12. Unit tests, E2E tests, visual regression

---

## 17. Architecture Decision

**Option A: Extend useTheme hook**
```tsx
// src/hooks/use-theme.ts
export function useTheme() {
  // Existing color theme logic...
  const [darkMode, setDarkMode] = useState(...);
  return { primaryColor, darkMode, setDarkMode };
}
```
**Pros**: Single hook, less code duplication
**Cons**: Mixing color theme and dark mode concerns

**Option B: Separate useDarkMode hook**
```tsx
// src/hooks/use-dark-mode.ts
export function useDarkMode() {
  const [mode, setMode] = useState<'light' | 'dark' | 'system'>(...);
  // Dark mode specific logic
}
```
**Pros**: Separation of concerns, easier testing
**Cons**: Two hooks to import

**Option C: ThemeProvider context** (RECOMMENDED)
```tsx
// src/context/ThemeContext.tsx
export function ThemeProvider({ children }) {
  // Manage both color theme AND dark mode
  return <ThemeContext.Provider value={{ ... }}>{children}</ThemeContext.Provider>;
}
```
**Pros**: Centralized theme management, easy to extend
**Cons**: More boilerplate

---

## 18. Unresolved Questions

1. **Persistence**: Should dark mode preference be:
   - Per-user (localStorage) ✓
   - Per-workspace (API)
   - Both?

2. **System preference**: Support `prefers-color-scheme` media query?
   - Light / Dark / System (3 options) ✓

3. **Transition**: Smooth theme transitions?
   - `transition-colors duration-300` on root element

4. **Scope**: Dark mode for:
   - All 215 TypeScript files?
   - Or incremental rollout (start with MainLayout, Settings)?

5. **Backward compat**: What if user has old localStorage keys?
   - Migration strategy needed?

6. **Testing**: Visual regression tests?
   - Playwright screenshots for light/dark?

---

## 19. Files Summary

**Critical files** (must modify):
```
webdashboard/
├── index.html                                          # Add FOUC script
├── src/
│   ├── index.css                                       # Add .dark CSS vars
│   ├── main.tsx                                        # Wrap with ThemeProvider
│   ├── App.tsx                                         # Or wrap here
│   ├── hooks/
│   │   ├── use-theme.ts                                # Extend or keep as-is
│   │   └── use-dark-mode.ts                            # NEW: Dark mode hook
│   ├── context/
│   │   └── ThemeContext.tsx                            # NEW: Theme provider
│   ├── components/
│   │   ├── layout/
│   │   │   ├── MainLayout.tsx                          # Update colors
│   │   │   ├── Sidebar.tsx                             # Update colors
│   │   │   └── Navbar.tsx                              # Add toggle button
│   │   └── ui/
│   │       ├── ThemeToggle.tsx                         # NEW: Toggle component
│   │       ├── Modal.tsx                               # Update colors
│   │       ├── Toast.tsx                               # Update colors
│   │       └── [other UI components]                   # Update colors
│   └── pages/
│       └── settings/
│           └── GeneralSettings.tsx                     # Add toggle to Appearance
```

**Total estimated files to modify**: ~30-50 (depending on scope)
**Total files in codebase**: 215 TypeScript files

---

## 20. Recommended Implementation Plan

### Step 1: Setup (1-2 hours)
- Create `use-dark-mode.ts` hook
- Create `ThemeContext.tsx` provider
- Add `.dark` CSS variables to `index.css`
- Add FOUC script to `index.html`

### Step 2: Layout (2-3 hours)
- Update MainLayout, Sidebar, Navbar
- Create ThemeToggle component
- Add toggle to Navbar

### Step 3: Settings (1 hour)
- Add dark mode section to GeneralSettings Appearance tab
- Wire up toggle UI

### Step 4: Testing (1 hour)
- Write unit tests for dark mode hook
- Write integration test for toggle
- Manual QA light/dark switching

### Step 5: Components (5-10 hours)
- Bulk update all UI components with dark mode variants
- Use search/replace for common patterns:
  - `bg-white ` → `bg-white dark:bg-slate-900 `
  - `text-slate-700 ` → `text-slate-700 dark:text-slate-200 `

### Step 6: i18n (30 min)
- Add translations for "Dark Mode", "Light Mode", "System"

---

**Total estimated effort**: 10-17 hours
**Risk level**: LOW (Tailwind v4 native support, CSS vars already in place)
**Breaking changes**: NONE (additive feature)

---

## END OF SCOUT REPORT

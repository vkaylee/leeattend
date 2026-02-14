# Phase 5: Settings & Toggle

## Context Links
- [Phase 1: Foundation](phase-01-foundation.md) (DarkModeContext)
- [Navbar](../../webdashboard/src/components/layout/Navbar.tsx)
- [GeneralSettings](../../webdashboard/src/pages/settings/GeneralSettings.tsx)
- [LanguageSwitcher](../../webdashboard/src/components/ui/LanguageSwitcher.tsx) (pattern reference)
- [i18n en.json](../../webdashboard/src/i18n/locales/en.json)

## Overview
- **Priority**: P1 (user-facing control)
- **Status**: pending
- **Effort**: 1h
- **Description**: Create dark mode toggle component for Navbar and add theme mode selector to Settings > Appearance tab. Add i18n translations.

## Key Insights
- Navbar already has `LanguageSwitcher` component as pattern reference for icon-button dropdowns
- Settings > Appearance tab already has color picker; dark mode toggle fits naturally above/below it
- Three modes: Light / Dark / System — use Sun/Moon/Monitor icons from lucide-react
- Toggle in Navbar should be compact (icon-only button cycling modes or small dropdown)
- Settings page should show all three options explicitly

## Requirements

### Functional
- F1: Navbar toggle: single icon button that cycles light -> dark -> system -> light
- F2: Navbar toggle: icon reflects current resolved theme (Sun for light, Moon for dark; half-circle for system)
- F3: Settings page: three explicit buttons (Light/Dark/System) with active state
- F4: Both controls wired to same `useDarkModeContext().setMode()`
- F5: i18n translations for all labels in en, vi, zh

### Non-Functional
- NF1: Toggle component under 60 lines
- NF2: Icon transitions smooth (scale + opacity)
- NF3: Tooltip on Navbar toggle showing current mode name

## Architecture

### Navbar Toggle Component

Path: `webdashboard/src/components/ui/DarkModeToggle.tsx`

Simple cycle button:
```
Click: light -> dark -> system -> light -> ...
Icon: Sun (light) | Moon (dark) | Monitor (system)
```

Styling follows Navbar button pattern (same as collapse toggle):
```
p-3 bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400
hover:bg-primary-50 dark:hover:bg-slate-700 hover:text-primary-600 dark:hover:text-primary-400
rounded-xl border border-slate-100 dark:border-slate-700 shadow-sm
```

### Settings Page Integration

Add section to GeneralSettings Appearance tab between header and color picker:
```
Theme Mode label
[Sun Light] [Moon Dark] [Monitor System]  <- three toggle buttons, active highlighted
```

## Related Code Files

### Files to Create
| File | Purpose |
|------|---------|
| `src/components/ui/DarkModeToggle.tsx` | Navbar compact toggle (cycle button) |

### Files to Modify
| File | Changes |
|------|---------|
| `src/components/layout/Navbar.tsx` | Import + render `<DarkModeToggle />` next to `<LanguageSwitcher />` |
| `src/pages/settings/GeneralSettings.tsx` | Add theme mode section to Appearance tab |
| `src/i18n/locales/en.json` | Add theme mode translation keys |
| `src/i18n/locales/vi.json` | Add Vietnamese translations |
| `src/i18n/locales/zh.json` | Add Chinese translations |

## Implementation Steps

### Step 1: Create `DarkModeToggle.tsx`

Path: `webdashboard/src/components/ui/DarkModeToggle.tsx`

```typescript
import { Sun, Moon, Monitor } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useDarkModeContext, type ThemeMode } from "@/context/DarkModeContext";
import { cn } from "@/lib/utils";

const MODES: ThemeMode[] = ['light', 'dark', 'system'];

const MODE_ICONS = {
  light: Sun,
  dark: Moon,
  system: Monitor,
} as const;

export function DarkModeToggle() {
  const { t } = useTranslation();
  const { mode, setMode } = useDarkModeContext();

  const cycleMode = () => {
    const idx = MODES.indexOf(mode);
    setMode(MODES[(idx + 1) % MODES.length]);
  };

  const Icon = MODE_ICONS[mode];
  const label = t(`common.theme_mode.${mode}`);

  return (
    <button
      onClick={cycleMode}
      title={label}
      data-testid="dark-mode-toggle"
      className={cn(
        "p-3 rounded-xl border shadow-sm transition-all active:scale-90",
        "bg-slate-50 dark:bg-slate-800",
        "text-slate-500 dark:text-slate-400",
        "hover:bg-primary-50 dark:hover:bg-slate-700",
        "hover:text-primary-600 dark:hover:text-primary-400",
        "border-slate-100 dark:border-slate-700",
        "hover:shadow-md hover:border-primary-100 dark:hover:border-slate-600"
      )}
    >
      <Icon size={20} strokeWidth={2.5} />
    </button>
  );
}
```

### Step 2: Add Toggle to Navbar

In `Navbar.tsx`, import and place before `<LanguageSwitcher />`:

```tsx
import { DarkModeToggle } from "@/components/ui/DarkModeToggle";

// In the right section div:
<div className="flex items-center gap-6 ml-auto">
  <DarkModeToggle />
  <LanguageSwitcher />
  {/* ... bell, divider, user avatar ... */}
</div>
```

### Step 3: Add Theme Mode to GeneralSettings Appearance Tab

In `GeneralSettings.tsx`, add above the primary color section inside the Appearance tab:

```tsx
import { Sun, Moon, Monitor } from "lucide-react";
import { useDarkModeContext } from "@/context/DarkModeContext";

// Inside GeneralSettingsContent:
const { mode: themeMode, setMode: setThemeMode } = useDarkModeContext();

// In Appearance tab JSX, before the color picker section:
<div className="space-y-6">
  <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
    {t("common.workspace_settings.theme_mode")}
  </label>
  <div className="flex flex-wrap gap-4">
    {[
      { key: 'light' as const, icon: Sun, label: t('common.theme_mode.light') },
      { key: 'dark' as const, icon: Moon, label: t('common.theme_mode.dark') },
      { key: 'system' as const, icon: Monitor, label: t('common.theme_mode.system') },
    ].map(({ key, icon: Icon, label }) => (
      <button
        key={key}
        data-testid={`theme-mode-${key}`}
        onClick={() => setThemeMode(key)}
        className={cn(
          "flex items-center gap-3 px-6 py-3 rounded-2xl text-xs font-bold uppercase tracking-widest transition-all border active:scale-95",
          themeMode === key
            ? "bg-primary-50 dark:bg-primary-950 text-primary-700 dark:text-primary-300 border-primary-200 dark:border-primary-800 shadow-sm"
            : "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-700 hover:bg-slate-100 dark:hover:bg-slate-700"
        )}
      >
        <Icon size={16} /> {label}
      </button>
    ))}
  </div>
  <p className="text-[10px] text-slate-400 dark:text-slate-500 ml-1">
    {t("common.workspace_settings.theme_mode_desc")}
  </p>
</div>
```

### Step 4: Add i18n Translation Keys

**en.json** additions (under `common`):
```json
"theme_mode": {
  "light": "Light",
  "dark": "Dark",
  "system": "System"
},
"workspace_settings": {
  "theme_mode": "Theme Mode",
  "theme_mode_desc": "Choose your preferred color scheme. System follows your OS preference."
}
```

**vi.json** additions:
```json
"theme_mode": {
  "light": "Sang",
  "dark": "Toi",
  "system": "He thong"
},
"workspace_settings": {
  "theme_mode": "Che do giao dien",
  "theme_mode_desc": "Chon che do mau sac. He thong se theo cai dat cua he dieu hanh."
}
```

**zh.json** additions:
```json
"theme_mode": {
  "light": "浅色",
  "dark": "深色",
  "system": "跟随系统"
},
"workspace_settings": {
  "theme_mode": "主题模式",
  "theme_mode_desc": "选择您偏好的颜色方案。系统模式将跟随操作系统设置。"
}
```

Note: Merge these into the existing JSON structure; don't overwrite existing keys.

### Step 5: Dark Mode for Settings Page Itself

GeneralSettings.tsx also needs dark variants on its own elements:
- Card containers: `bg-white` -> add `dark:bg-slate-900`
- Tab bar: `bg-slate-100/50` -> add `dark:bg-slate-800/50`
- Active tab: `bg-white` -> add `dark:bg-slate-900`
- Section headers: `text-slate-800` -> add `dark:text-slate-100`
- Labels: `text-slate-500` -> add `dark:text-slate-400`
- Borders: `border-slate-100` -> add `dark:border-slate-800`
- Info box: `bg-slate-50 border-slate-100` -> add `dark:bg-slate-800 dark:border-slate-700`

## Todo List
- [ ] Create `DarkModeToggle.tsx` component
- [ ] Add `<DarkModeToggle />` to Navbar.tsx
- [ ] Add theme mode section to GeneralSettings Appearance tab
- [ ] Add dark mode variants to GeneralSettings itself
- [ ] Add en.json translations
- [ ] Add vi.json translations
- [ ] Add zh.json translations
- [ ] Test: Navbar toggle cycles through modes correctly
- [ ] Test: Settings toggle reflects and controls same state
- [ ] Test: Switching language shows translated labels
- [ ] Test: `data-testid` attributes present on all interactive elements

## Success Criteria
- Navbar shows theme toggle icon next to language switcher
- Clicking cycles: light (Sun) -> dark (Moon) -> system (Monitor)
- Settings > Appearance shows all three options with active highlight
- Both controls sync (change in Navbar reflected in Settings and vice versa)
- Translations display correctly in all 3 languages
- `data-testid="dark-mode-toggle"` and `data-testid="theme-mode-*"` present

## Risk Assessment
| Risk | Impact | Mitigation |
|------|--------|------------|
| Context not available (Provider missing) | Crash | Error boundary; Phase 1 ensures Provider wraps app |
| i18n key path wrong | Missing labels | Verify key paths match existing JSON structure |
| Icon import missing from lucide | Build error | `Sun`, `Moon`, `Monitor` all in lucide-react |

## Security Considerations
- No security impact; client-side preference stored in localStorage.

## Next Steps
- Phase 6: Testing to validate all toggle behavior and visual correctness

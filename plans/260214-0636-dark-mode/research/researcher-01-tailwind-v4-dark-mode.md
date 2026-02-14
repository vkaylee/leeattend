# Tailwind CSS v4 Dark Mode Research

**Date:** 2026-02-14
**Focus:** React/TypeScript dark mode implementation with Tailwind v4

## 1. Tailwind v4 Dark Mode Configuration

### CSS-First Architecture
- **Major shift:** v4 replaces JavaScript config (`tailwind.config.js`) with CSS-first approach
- Use `@theme` directive in CSS instead of JS config file
- Theme parameters declared via `@layer theme`
- Built-in dark variant detects system preference by default

### v3 → v4 Changes
- **Removed:** `darkMode: "class"` or `darkMode: "selector"` from config file
- **Default behavior:** Only media query (`prefers-color-scheme`) supported out-of-box
- **Class-based toggle:** Requires custom variant definition in CSS
- **Migration impact:** Config location moved from JS to CSS, breaking change for v3 users

### Enabling Class-Based Dark Mode
Define custom variant in global CSS:

```css
@layer theme {
  @variant dark (&:where(.dark, .dark *));
}
```

Or simplified:
```css
@variant dark (.dark &);
```

This makes `dark:` utilities activate when `.dark` class present on `html` or ancestor element.

### CSS Variables Integration
- v4 uses CSS custom properties for colors by default (not hardcoded values)
- Custom properties must be defined in `@layer theme` to work with dark mode
- Example:
```css
@layer theme {
  :root {
    --color-primary: oklch(0.5 0.2 250);
  }
  .dark {
    --color-primary: oklch(0.8 0.15 260);
  }
}
```

## 2. React ThemeProvider Pattern

### Core Implementation
```typescript
// useTheme hook pattern
const useTheme = () => {
  const [theme, setTheme] = useState<'light' | 'dark' | 'system'>('system');

  useEffect(() => {
    // Priority: localStorage > system preference
    const stored = localStorage.getItem('theme');
    const systemPrefers = window.matchMedia('(prefers-color-scheme: dark)').matches;

    const resolved = stored || (systemPrefers ? 'dark' : 'light');
    setTheme(resolved);
    document.documentElement.classList.toggle('dark', resolved === 'dark');
  }, []);
};
```

### Best Practices
- **Priority logic:** Manual user selection (localStorage) overrides system preference
- **System detection:** `window.matchMedia('(prefers-color-scheme: dark)').matches`
- **Persistence:** Store user choice in localStorage
- **Listen to changes:** Add event listener to media query for real-time system preference updates
- **Hydration warning:** Avoid rendering theme-dependent UI during SSR (values undefined on server)

### Popular Libraries
- `next-themes`: Zero-flash dark mode for Next.js (2 lines of code)
- Material-UI ThemeProvider: Built-in `colorSchemes` with system preference support

## 3. Preventing FOUC

### The Problem
- SSR/SSG renders HTML with default (light) theme
- React hydration happens after page load
- Brief flash of wrong theme before JS applies correct class

### Solution: Inline Script in HTML Head
Place **before** React app loads:

```html
<script>
  (function() {
    const stored = localStorage.getItem('theme');
    const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const theme = stored || (systemDark ? 'dark' : 'light');

    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
    }
  })();
</script>
```

### Why It Works
- Executes **before** page renders (blocking script)
- Applies `.dark` class synchronously before first paint
- No React dependency, pure DOM manipulation
- Should be inlined (not external file) for immediate execution

### Alternative: CSS-Only for System Preference
If only supporting system preference (no manual toggle):
```css
@media (prefers-color-scheme: dark) {
  /* Dark styles applied immediately via CSS */
}
```
No FOUC possible as CSS is available before render.

## 4. CSS Custom Properties vs Tailwind Dark Prefix

### Approach A: Tailwind `dark:` Utilities
```jsx
<div className="bg-white dark:bg-gray-900 text-black dark:text-white">
```

**Pros:**
- Native Tailwind integration
- Type-safe with IntelliSense
- No custom CSS needed
- Works with Tailwind utilities out-of-box

**Cons:**
- Verbose for complex themes
- Hard to swap multiple colors at once
- Requires rebuild to change theme values

### Approach B: CSS Custom Properties
```css
@layer theme {
  :root {
    --bg-primary: #ffffff;
    --text-primary: #000000;
  }
  .dark {
    --bg-primary: #1a1a1a;
    --text-primary: #ffffff;
  }
}
```

```jsx
<div className="bg-[var(--bg-primary)] text-[var(--text-primary)]">
```

**Pros:**
- Runtime theme switching without rebuild
- Centralized theme definition
- Easier to manage complex color schemes
- Can support 3+ themes (not just light/dark)

**Cons:**
- Loses Tailwind IntelliSense for custom properties
- More verbose className syntax
- Requires custom CSS definitions

### Hybrid Approach (Recommended for v4)
Combine both:

```css
@layer theme {
  --color-primary: oklch(0.5 0.2 250);
}

/* Register with Tailwind */
@theme {
  --color-primary: var(--color-primary);
}
```

Use in JSX:
```jsx
<div className="bg-primary dark:bg-primary-dark">
```

Tailwind v4's `@theme` makes custom properties first-class citizens.

## Recommendations

1. **Use class-based dark mode** (not just media query) for manual toggle support
2. **Inline script in HTML head** to prevent FOUC (critical for UX)
3. **Hybrid approach:** CSS custom properties for theme colors + Tailwind utilities for spacing/layout
4. **ThemeProvider context:** Centralize theme state, localStorage, and system preference logic
5. **v4 migration:** Define custom dark variant in CSS, use `@layer theme` for variables

## Sources
- [Tailwind CSS v4 Dark Mode Docs](https://tailwindcss.com/docs/dark-mode)
- [Implementing Dark Mode with Tailwind v4 and Next.js](https://www.thingsaboutweb.dev/en/posts/dark-mode-with-tailwind-v4-nextjs)
- [Tailwind v4 CSS Variables Discussion](https://github.com/tailwindlabs/tailwindcss/discussions/15083)
- [Fixing React Dark Mode Flickering (FOUC)](https://notanumber.in/blog/fixing-react-dark-mode-flickering)
- [next-themes Library](https://github.com/pacocoursey/next-themes)
- [Material-UI Dark Mode](https://mui.com/material-ui/customization/dark-mode/)
- [Theme Colors with Tailwind v4](https://medium.com/@kevstrosky/theme-colors-with-tailwind-css-v4-0-and-next-themes-dark-light-custom-mode-36dca1e20419)

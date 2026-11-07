# Theme Switch Component Compliance & Design Spec

## 1. Context & Objective
Implement a VitePress-style dark/light appearance switch component directly inspired by `https://nginxproxymanager.com/` (`VPSwitchAppearance`).
The component is placed in the primary header (`webdashboard/src/components/layout/topbar.tsx`), supporting a 2-way toggle (Light <-> Dark) with smooth knob slide and Sun/Moon transition, enhanced with View Transitions API (Circular Wipe) when supported.

## 2. Mandatory Rule Compliance
- **Rule ID 1: UI/UX Design Rule 1 - Design System**
  - Component created as a reusable UI building block in `webdashboard/src/components/ui/theme-switch.tsx`.
  - Seamlessly integrates with the existing Tailwind v4 color tokens and dark mode classes (`dark:`).
- **Rule ID 2: UI/UX Design Rule 2 - WCAG 2.1 AA Accessibility**
  - Semantic ARIA switch attributes: `role="switch"`, `aria-checked="true|false"`, `aria-label`, and `title`.
  - Accessible keyboard focus ring: `focus-visible:ring-2 focus-visible:ring-primary focus-visible:outline-none focus-visible:ring-offset-2`.
  - Distinct icons (`Sun` in amber/neutral, `Moon` in neutral/white) with sufficient contrast ratio against knob background.
- **Rule ID 3: UI/UX Design Rule 3 - Responsive & Motion**
  - Pill dimensions: 40px width, 22px height, 18px knob diameter with 18px translateX on dark mode.
  - Micro-interactions timed at 250ms with `ease-out`.
  - View Transitions API circular wipe handles smooth screen-wide transition, strictly disabled when `prefers-reduced-motion: reduce` is active.
- **Rule ID 4: Coding Standards Rule 2 - Clean Code & SOLID**
  - Replaces obsolete single-icon toggle button in Topbar without breaking existing unit tests (`data-testid="topbar-theme-toggle"` preserved).
  - Isolated unit tests verifying keyboard interaction, state toggle, and ARIA attributes.

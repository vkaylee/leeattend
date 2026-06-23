# Rule Compliance Check - Working Patterns UI/UX

This document tracks the compliance of the Working Patterns UI/UX List Card implementation with the project rules.

## Rule IDs to Comply With

1. **Rule UI2.B: Keyboard Accessibility & Focus** (from `design-rules.md`) - Ensure all list card controls (toggle buttons, action buttons) are fully focusable and operable via keyboard alone with visible focus rings (`focus-visible:ring-2`), and not hidden purely behind hover states.
2. **Rule UI2.C: Screen Reader Support (Semantic HTML & ARIA)** (from `design-rules.md`) - Ensure that all buttons, icons, and interactive elements (like the expand chevrons or list/grid toggles) have descriptive labels or `sr-only` text.
3. **Rule CS6: Enterprise-Grade Premium Standard** (from `coding-standards.md` / `GEMINI.md`) - Implement high-end styling, translations for new keys, beautiful empty states/skeletons, and smooth hover/active micro-interactions.
4. **Rule CS9: Frontend Architecture** (from `coding-standards.md`) - Decouple business logic (hooks) from "dumb" visual components. Preserve the usage of the existing `useWorkingPatterns` custom hook.

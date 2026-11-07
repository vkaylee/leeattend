# Compliance Checklist: Employees UI/UX Refactor (SOLID & DRY)

## Rule Compliance
- **Rule ID UI-01**: WCAG 2.1 AA Compliance (`.agent/rules/design-rules.md` Sec 2)
  - Color contrast ratio >= 4.5:1 for regular text.
  - Descriptive `aria-label` on all icon-only interactive controls (scroll buttons, close buttons).
  - Proper WAI-ARIA tab pattern (`role="tablist"`, `role="tab"`, `aria-selected`).
- **Rule ID CODE-01**: SOLID & DRY Architecture (`.agent/rules/coding-standards.md` Sec 2 & 9)
  - SRP: Decouple settings-level admin RBAC from entity-level read access.
  - DRY: Single Source of Truth for tab configuration (`EMPLOYEE_DRAWER_TABS` registry).
  - OCP: Open to new tab extensions without touching rendering or navigation internals.
- **Rule ID I18N-01**: Multi-locale Support (`.agent/rules/coding-standards.md` Sec 6)
  - Zero hardcoded user-facing strings; all tabs and alerts localized in `vi`, `en`, and `zh`.

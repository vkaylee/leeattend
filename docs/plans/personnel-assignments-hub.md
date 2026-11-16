# Task Compliance & Design: Personnel Assignments Hub (Registry View)

## 📋 Rule Compliance Check (MANDATORY - GEMINI.md Layer 1)
- **Rule ID 1: `UI-DES-001` (WCAG 2.1 AA Compliance & Design System)**
  - Source: `.agent/rules/design-rules.md`
  - Implementation: Minimum 4.5:1 text contrast, full keyboard accessibility (`Tab`, `Escape`, `Enter`), visible focus rings, ARIA roles for tables/modals/filters, skeleton loading states, empty states.
- **Rule ID 2: `CODE-STD-001` (SOLID & DRY Architecture)**
  - Source: `.agent/rules/coding-standards.md`
  - Implementation: Dual-mode modal for shared assignment creation/editing across Employee Detail and Central Registry; separation of concerns between data hooks, UI presentation, and filtering logic.
- **Rule ID 3: `TEST-STD-001` (Testing Standards & Completeness)**
  - Source: `.agent/rules/testing-standards.md`
  - Implementation: Unit tests for filter state and dual-mode modal, integration/E2E test verification via `./leedevkit test web`.
- **Rule ID 4: `EXEC-PTY-001` (PTY Safety & Hermetic Execution)**
  - Source: `.agent/rules/execution-wrappers.md`
  - Implementation: Output redirection to `logs/run_*.log`, non-blocking commands, hermetic wrapper invocation.

## 🎯 Architecture & Objectives
1. **Registry Route:** `/personnel/assignments` (or `/assignments`) for workspace-wide assignment management.
2. **Dual-Mode Form:** `EmployeeAssignmentFormModal` supports `fixedEmployeeId` (when invoked from employee detail) or employee selector combobox (when invoked from the registry).
3. **Multi-dimensional Filtering:** By status (`active`, `scheduled`, `completed`, `cancelled`), assignment type, date ranges, and search by employee name/code.
4. **Bi-directional Navigation:** Deep links between `/personnel/assignments` and `/employees/:id?tab=assignments`.

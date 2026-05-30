# Rule Compliance Check: Shift Break Overtime configuration

## Task Details
- **Task**: Restore the `break_is_overtime` configuration in Create and Edit Shift modals.
- **Task Slug**: `shift-break-overtime`

## Rule Verification Table

| Rule ID | Rule Source | Description | Planned Compliance |
| :--- | :--- | :--- | :--- |
| **CODING_STANDARDS_6_ENTERPRISE_PREMIUM** | `coding-standards.md` | Every feature must default to an Enterprise-Grade Premium Ready standard (i18n support, polished UI, micro-interactions). | We use a sleek radio group instead of raw checkboxes and support localized strings (vi, en, zh) for the options. |
| **CODING_STANDARDS_9_FRONTEND_ARCH** | `coding-standards.md` | Separate Business Logic (Custom Hooks) from UI Components. | The state and handler modifications are kept strictly inside the custom hooks (`useShiftFormState`, `useShiftModalHandlers`), keeping modals relatively "dumb". |
| **DESIGN_RULES_1_DESIGN_SYSTEM** | `design-rules.md` | Use predefined design system components / styles. Don't reinvent the wheel. | We will style the radio group following existing theme and spacing tokens of the dashboard. |
| **EXEC-PTY-07** | `execution-pty-safety.md` / `GEMINI.md` | Prevent PTY Hangs. Redirect outputs `> logs/run_{slug}.log 2>&1 </dev/null` and prefix with `mkdir -p logs &&`. | Any tests or dev runs we execute will use this wrapper/redirection pattern. |
| **EXEC-WRAP-01** | `execution-wrappers.md` | Hermetic execution: Use scripts in `scripts/` (e.g. `./scripts/_npm.sh`). | We will run all NPM commands using `./scripts/_npm.sh`. |

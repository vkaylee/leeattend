# Work-policy UI route × role acceptance matrix

Viewport baseline: `desktop-large` 1920×1080. Before each role scenario: resize, verify `window.innerWidth`/`window.innerHeight`, authenticate through `/login`, capture console and network.

| Role | Allowed route/actions | Expected denied route/actions | Console evidence | Network evidence |
|---|---|---|---|---|
| `employee` | `/settings/payroll` none | direct route, policy list/detail/history, validation, version/membership writes, impact aggregate | no unexpected errors | redirect `/`; no forbidden API call after denial |
| `team_lead` | `/settings/payroll` none | same policy/settings actions | no unexpected errors | redirect `/`; no forbidden API call |
| `shift_manager` | `/settings/payroll` none | payroll policy read/write and impact aggregate | no unexpected errors | redirect `/`; no forbidden API call |
| `hr_manager` | `/settings/payroll`; read policy groups/history/detail/validation; manage version/membership if backend permission allows; impact aggregate if sensitive-read allows | infra/system administration | no unexpected errors or repeated errors | expected 2xx for allowed calls; expected 403/redirect only for denied calls |
| `admin` | `/settings/payroll`; all policy read/detail/history/validation/version/membership/impact actions | only explicit backend denials | no unexpected errors | expected 2xx; no 404/5xx/crash |

Routes/actions to test:

- `/settings/payroll` initial loading, populated/empty/error states.
- Work-policy group selection, history/detail, validation, membership editor.
- Impact batch input and stale/expiry display.
- Direct denied navigation for lower roles.
- Console messages and network status for every role.

No credentials, tokens, raw IDs, or PII in evidence.

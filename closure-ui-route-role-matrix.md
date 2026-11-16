# Attendance Closure UI Route × Role Acceptance Matrix

Viewport: `desktop-large` 1920×1080. Browser evidence: console messages + network requests captured per scenario. Accounts: development-only accounts from `docs/dev-accounts.md`; mask all identifiers.

| Role | Route | Allowed actions | Expected denied actions | Console | Network |
|---|---|---|---|---|---|
| `employee` | `/reports` and `/attendance-logs` | Existing allowed attendance/report reads only | Closure status/actions hidden or API `403`; no close/reopen request | capture | capture |
| `team_lead` | `/reports` and `/attendance-logs` | Existing team attendance reads only | Closure status/actions hidden or API `403`; no close/reopen request | capture | capture |
| `shift_manager` | `/reports` and `/attendance-logs` | Existing schedule/attendance reads only | Closure status/actions hidden or API `403`; no close/reopen request | capture | capture |
| `hr_manager` | `/reports` and `/attendance-logs` | Closure status read; close/reopen when employer scope is allowed | Out-of-scope employer must return `403`; blocked close states show localized reason | capture | capture |
| `admin` | `/reports` and `/attendance-logs` | Closure status, close, reopen, reclose; immutable evidence metadata | Only malformed/stale/blocked business operations denied | capture | capture |

Acceptance states:

- `open` / `reopened`: close control and reason required.
- `closed`: reopen control and reason required; reclose is close after successful reopen.
- stale version: localized conflict; refresh status action remains available.
- missing coverage: localized blocker; no partial mutation.
- open punch: localized blocker; no partial mutation.
- locked payroll: localized blocker; no reopen/close mutation.
- immutable evidence: closure number, day count, and snapshot hash render read-only; no edit/delete affordance.
- loading: skeleton with `aria-busy`.
- empty: no employer selected message.
- error: alert with retry; no raw IDs, PII, or secrets.

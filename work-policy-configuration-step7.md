# Step 7 UI implementation record

Scope: complete payroll work-policy editor UX without changing payroll calculation semantics.

Rules applied:
- TEST-01: changed observable UI behavior requires focused and applicable LeeDevKit tests.
- API-01: preserve existing API contracts and surface structured error status safely.
- ACCESS-01: keep backend authorization as source of truth; UI must not imply unauthorized access.
- DESIGN-01: provide loading, empty, error, keyboard, and accessible form states.
- DATA-01: do not expose or log salary, PII, credentials, or tokens.

Test impact:
- Success: four policy sections render and submit existing immutable version payload.
- Boundaries: missing references, invalid dates/minutes, monthly-only fields, hourly/shift rate component.
- Failure: validation errors, stale data, 409 conflict, unsaved changes.
- Compatibility: existing payroll settings and API client behavior remain unchanged.
- Security: no new authorization path; simulation and impact APIs remain backend-gated.

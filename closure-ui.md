# Closure UI Phase

- AC-01: Apply `.agent/rules/access-control.md`; read authorization precedes payload and period lookup.
- API-01: Apply `.agent/rules/api-governance.md`; add backward-compatible attendance period read contract and typed client.
- UI-01: Apply `.agent/rules/design-rules.md`; reuse attendance page and provide accessible loading, empty, error, and mutation states.
- TEST-01: Apply `.agent/rules/development-workflow.md`; cover success, stale, missing coverage, open punch, payroll lock, and immutable evidence states.

Test impact:
- Success: open/reopened period can close; reopened period can reclose; closure number/hash render.
- Negative: employee/team lead/shift manager cannot see mutation controls; API 403 remains expected; stale and blocked closure errors render localized messages.
- Failure: GET load error and mutation error render retry/status feedback without leaking IDs or PII.
- Compatibility: existing attendance log tabs, filters, and APIs remain unchanged; new GET endpoint is additive.

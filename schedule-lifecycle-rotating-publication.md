# Rotating schedule publication

Scope: expose explicit rotating preview/publication through existing lifecycle transaction code.

Rules:
- API Governance: preserve office routes; additive route only.
- Access Control: rotating route checks `schedules:publish:rotating` before payload validation.
- Development Workflow: cover route authorization, no-approval rotating validation, and publication mode.
- Database Rules: reuse publication transaction and existing `schedule_publication_batches.mode` constraint.

Acceptance:
- Office routes remain office-only and require approved policy.
- Rotating preview/publication accept no approval instance.
- Rotating publication persists mode `rotating` and null approval.
- Unauthorized rotating caller is denied before payload parsing.

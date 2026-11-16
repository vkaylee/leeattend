# OT claim status in daily report API

- `TEST-API-01`: Additive API field uses existing frontend contract name `ot_claim_status`.
- `TEST-DATA-02`: Map canonical statuses and no-claim/null rows to `none`.
- `TEST-CONTRACT-03`: Preserve authorization, pagination, ordering, and backward-compatible response shape.
- `TEST-VERIFY-04`: Run focused API checks, then full applicable API verification through repository wrapper when available.

Test impact: repository mapping success for all canonical statuses; null/no-claim fallback; API JSON regression; existing filters and pagination unchanged. Generated OpenAPI source/generator must be discovered before any generated-file change; do not hand-edit generated artifacts.

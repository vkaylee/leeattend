# Dashboard Metrics Contract Migration

## Status

`executive-dashboard.v1` is a shared domain contract only. No existing dashboard endpoint changes in this phase.

## New contract

The backend owns typed Rust DTOs in `apiserver/src/models/dashboard_metrics.rs`. The frontend owns matching TypeScript/Zod runtime schemas in `webdashboard/src/hooks/dashboard-metric-contract.ts`.

Canonical rules:

- Metric identifier is `key`.
- `value`, `numerator`, and `denominator` are nullable. Null is meaningful, never an implicit zero.
- `status` describes data availability: `available`, `not_applicable`, `data_gap`, `restricted`.
- `health` describes decision state: `ok`, `warning`, `critical`, `not_applicable`, `data_gap`.
- Numerator and denominator are `{ value, unit }` objects.
- `delta` is absolute delta. `delta_percent` is relative percent delta. `delta_percentage_points` is rate-to-rate difference in percentage points.
- Restricted metrics return no value, numerator, or denominator.

## Backward compatibility

Existing paths keep current payloads and defaults:

- `/v1/dashboard/workspace-stats`
- `/v1/dashboard/attendance-trend`
- `/v1/dashboard/department-attendance`
- `/v1/dashboard/unexcused-absences`
- `/v1/dashboard/overtime-trend`
- `/v1/dashboard/peak-times`
- `/v1/dashboard/leave-types`
- `/v1/dashboard/lateness-by-day`
- `/v1/dashboard/recruitment-sources`
- `/v1/dashboard/turnover-analytics`
- `/v1/dashboard/preferences`

Consumers must not parse old responses with the new metric schema. Keep old Zod schemas until endpoint migration completes.

## Migration sequence

1. Add an executive endpoint with an explicit versioned response and server-normalized reporting context.
2. Populate only metrics with authoritative sources. Use `data_gap` for unavailable sources; never fabricate zero.
3. Add OpenAPI paths and regenerate frontend API types. Do not hand-edit generated files.
4. Introduce a separate React Query hook using `dashboardMetricsResponseSchema`.
5. Reconcile each metric against authoritative report fixtures.
6. Migrate one existing widget at a time behind a feature flag.
7. Deprecate an old endpoint only after client inventory, replacement coverage, rollout, and rollback plan are complete.

## Security and privacy

Handlers must authorize before service execution. Workspace and department scope must be applied before querying. Payroll, cost, budget, and employee-level absence remain separately gated. Do not log metric payloads containing confidential data.

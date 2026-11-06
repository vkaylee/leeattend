import { describe, expect, it } from "vitest";

import {
  dashboardMetricSchema,
  dashboardMetricsResponseSchema,
} from "../dashboard-metric-contract";

const freshness = {
  as_of: "2026-08-13T00:00:00Z",
  lag_seconds: 120,
};

const metric = {
  key: "attendance_rate",
  value: 94.2,
  unit: "percent",
  numerator: { value: 942, unit: "eligible_employee_days" },
  denominator: { value: 1000, unit: "eligible_employee_days" },
  status: "available",
  health: "ok",
  null_reason: null,
  definition_version: "attendance-rate.v1",
  direction: "higher_is_better",
  freshness,
  quality_flags: [],
  comparison: {
    basis: "previous_period",
    period_start: "2026-07-01",
    period_end: "2026-07-31",
    value: 93.1,
    delta: 1.1,
    delta_percent: 1.18,
    delta_percentage_points: 1.1,
    status: "available",
  },
  target: { value: 95, operator: "gte" },
};

const response = {
  contract_version: "executive-dashboard.v1",
  workspace: { timezone: "Asia/Ho_Chi_Minh", currency: "VND" },
  context: {
    period: {
      key: "this_month",
      start_date: "2026-08-01",
      end_date: "2026-08-13",
      timezone: "Asia/Ho_Chi_Minh",
      as_of: "2026-08-13T00:00:00Z",
    },
    comparison: "previous_period",
    filters: {},
    scope: {
      kind: "workspace",
      department_ids: [],
      location_ids: [],
      omitted_dimensions: [],
    },
  },
  metrics: [metric],
  insights: [],
  actions: { total: 0, overdue: 0, by_type: [] },
  data_quality: { status: "complete", sources: [] },
  generated_at: "2026-08-13T00:00:00Z",
};

describe("dashboard metric contract", () => {
  it("accepts complete metric response", () => {
    expect(dashboardMetricsResponseSchema.parse(response)).toEqual(response);
  });

  it("accepts explicit null values for data gaps", () => {
    const parsed = dashboardMetricSchema.parse({
      ...metric,
      value: null,
      numerator: null,
      denominator: null,
      status: "data_gap",
      health: "data_gap",
      null_reason: "missing_source",
    });
    expect(parsed.value).toBeNull();
  });

  it("rejects missing required contract fields", () => {
    const { definition_version: _definitionVersion, ...invalid } = metric;
    expect(dashboardMetricSchema.safeParse(invalid).success).toBe(false);
  });

  it("rejects invalid status and out-of-range coverage", () => {
    expect(dashboardMetricSchema.safeParse({ ...metric, status: "warning" }).success).toBe(false);
    expect(dashboardMetricsResponseSchema.safeParse({
      ...response,
      data_quality: {
        status: "complete",
        sources: [{ key: "attendance", status: "partial", coverage_ratio: 2, freshness, warnings: [] }],
      },
    }).success).toBe(false);
  });

  it("rejects unknown fields at contract boundary", () => {
    expect(dashboardMetricSchema.safeParse({ ...metric, extra: true }).success).toBe(false);
  });

  it("accepts valid health values including neutral and healthy", () => {
    for (const health of ["ok", "healthy", "warning", "critical", "neutral", "not_applicable", "data_gap"] as const) {
      const result = dashboardMetricSchema.safeParse({ ...metric, health });
      expect(result.success).toBe(true);
    }
  });
});

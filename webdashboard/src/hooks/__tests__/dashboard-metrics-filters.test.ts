import { describe, expect, it } from "vitest";

import {
  DEFAULT_DASHBOARD_METRICS_FILTERS,
  parseDashboardMetricsFilters,
  serializeDashboardMetricsFilters,
} from "../dashboard-metrics-filters";

const departmentA = "11111111-1111-4111-8111-111111111111";
const departmentB = "22222222-2222-4222-8222-222222222222";

 describe("dashboard metrics URL filters", () => {
  it("serializes canonical state in stable order and round trips", () => {
    const query = serializeDashboardMetricsFilters({
      ...DEFAULT_DASHBOARD_METRICS_FILTERS,
      comparison: "previous_period",
      departmentIds: [departmentB, departmentA, departmentA],
      period: "custom",
      startDate: "2026-08-01",
      endDate: "2026-08-13",
    });

    expect(query).toBe(`period=custom&comparison=previous_period&start_date=2026-08-01&end_date=2026-08-13&department_ids=${departmentA}&department_ids=${departmentB}`);
    expect(parseDashboardMetricsFilters(query)).toEqual({
      comparison: "previous_period",
      departmentIds: [departmentA, departmentB],
      endDate: "2026-08-13",
      locationIds: [],
      period: "custom",
      startDate: "2026-08-01",
    });
  });

  it("falls back to defaults for invalid values", () => {
    expect(parseDashboardMetricsFilters("period=invalid&comparison=invalid&start_date=nope")).toEqual(DEFAULT_DASHBOARD_METRICS_FILTERS);
  });
});

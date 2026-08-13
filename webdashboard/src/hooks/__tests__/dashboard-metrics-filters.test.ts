import { describe, expect, it } from "vitest";

import {
  DEFAULT_DASHBOARD_METRICS_FILTERS,
  parseDashboardMetricsFilters,
  serializeDashboardMetricsFilters,
} from "../dashboard-metrics-filters";

const departmentA = "11111111-1111-4111-8111-111111111111";
const departmentB = "22222222-2222-4222-8222-222222222222";
const businessUnit = "33333333-3333-4333-8333-333333333333";
const manager = "44444444-4444-4444-8444-444444444444";
const shift = "55555555-5555-4555-8555-555555555555";

describe("dashboard metrics URL filters", () => {
  it("serializes every dimension in stable order and round trips", () => {
    const query = serializeDashboardMetricsFilters({
      ...DEFAULT_DASHBOARD_METRICS_FILTERS,
      businessUnitIds: [businessUnit],
      comparison: "previous_period",
      departmentIds: [departmentB, departmentA, departmentA],
      employeeStatuses: ["inactive", "active", "active"],
      employmentTypes: ["contractor", "full_time"],
      period: "custom",
      startDate: "2026-08-01",
      endDate: "2026-08-13",
      locationIds: [departmentB],
      managerIds: [manager],
      riskThreshold: "high",
      shiftIds: [shift],
      topN: 25,
    });

    expect(query).toBe(
      `period=custom&comparison=previous_period&start_date=2026-08-01&end_date=2026-08-13&department_ids=${departmentA}&department_ids=${departmentB}&location_ids=${departmentB}&business_unit_ids=${businessUnit}&manager_ids=${manager}&employment_types=contractor&employment_types=full_time&employee_statuses=active&employee_statuses=inactive&shift_ids=${shift}&top_n=25&risk_threshold=high`,
    );
    expect(parseDashboardMetricsFilters(query)).toEqual({
      businessUnitIds: [businessUnit],
      comparison: "previous_period",
      departmentIds: [departmentA, departmentB],
      employeeStatuses: ["active", "inactive"],
      employmentTypes: ["contractor", "full_time"],
      endDate: "2026-08-13",
      locationIds: [departmentB],
      managerIds: [manager],
      period: "custom",
      riskThreshold: "high",
      shiftIds: [shift],
      startDate: "2026-08-01",
      topN: 25,
    });
  });

  it("falls back to defaults for invalid values", () => {
    expect(
      parseDashboardMetricsFilters(
        "period=invalid&comparison=invalid&start_date=nope&top_n=0&risk_threshold=invalid",
      ),
    ).toEqual(DEFAULT_DASHBOARD_METRICS_FILTERS);
  });

  it("drops invalid IDs, malformed dates, and non-custom date bounds", () => {
    expect(
      parseDashboardMetricsFilters(
        "period=today&start_date=2026-02-30&end_date=2026-08-01&department_ids=bad&employment_types=%20",
      ),
    ).toEqual({
      ...DEFAULT_DASHBOARD_METRICS_FILTERS,
      period: "today",
    });
  });

  it("uses defaults for an invalid custom range while preserving dimensions", () => {
    expect(
      parseDashboardMetricsFilters(
        `period=custom&start_date=2026-08-13&end_date=2026-08-01&department_ids=${departmentA}`,
      ),
    ).toEqual({
      ...DEFAULT_DASHBOARD_METRICS_FILTERS,
      departmentIds: [departmentA],
    });
  });
});

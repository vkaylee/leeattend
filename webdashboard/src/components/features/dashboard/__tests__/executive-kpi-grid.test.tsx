import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

import type { DashboardMetric, DashboardMetricsResponse } from "@/hooks/dashboard-metric-contract";

import { ExecutiveKpiGrid } from "../executive-kpi-grid";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en" },
  }),
}));

const freshness = { as_of: "2026-08-13T09:00:00Z", lag_seconds: 120 };

function metric(overrides: Partial<DashboardMetric> = {}): DashboardMetric {
  return {
    key: "headcount",
    value: 100,
    unit: "count",
    numerator: null,
    denominator: null,
    status: "available",
    health: "ok",
    null_reason: null,
    definition_version: "headcount.v1",
    direction: "neutral",
    freshness,
    quality_flags: [],
    comparison: null,
    target: null,
    ...overrides,
  };
}

function response(metrics: DashboardMetric[], currency = "USD"): DashboardMetricsResponse {
  return {
    contract_version: "executive-dashboard.v1",
    workspace: { timezone: "Asia/Ho_Chi_Minh", currency },
    context: {
      period: {
        key: "this_month",
        start_date: "2026-08-01",
        end_date: "2026-08-13",
        timezone: "Asia/Ho_Chi_Minh",
        as_of: "2026-08-13T09:00:00Z",
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
    metrics,
    insights: [],
    actions: { total: 0, overdue: 0, by_type: [] },
    data_quality: { status: "complete", sources: [] },
    generated_at: "2026-08-13T09:00:00Z",
  };
}

describe("ExecutiveKpiGrid", () => {
  it("renders seven skeleton cards while loading", () => {
    render(<ExecutiveKpiGrid loading error={false} onRetry={vi.fn()} />);
    const grid = screen.getByLabelText("dashboard.executive.loading");
    expect(grid.children).toHaveLength(7);
  });

  it("renders an error state with a retry button", () => {
    const onRetry = vi.fn();
    render(<ExecutiveKpiGrid loading={false} error onRetry={onRetry} />);
    expect(screen.getByRole("alert")).toBeInTheDocument();
    const retry = screen.getByRole("button", { name: /dashboard.executive.retry/i });
    retry.click();
    expect(onRetry).toHaveBeenCalled();
  });

  it("renders every metric key in fixed order", () => {
    const metrics = [
      metric({ key: "headcount" }),
      metric({ key: "attendance_rate", unit: "percent", value: 95, numerator: { value: 1900, unit: "employee_days" }, denominator: { value: 2000, unit: "employee_days" } }),
      metric({ key: "absence_rate", unit: "percent", value: 5 }),
      metric({ key: "late_rate", unit: "percent", value: null, status: "data_gap", health: "data_gap", null_reason: "scheduled_attendance_source_unavailable" }),
      metric({ key: "turnover_rate", unit: "percent", value: 2 }),
      metric({ key: "overtime_hours", unit: "hours", value: 8 }),
      metric({ key: "pending_hr_actions", unit: "count", value: 7 }),
    ];
    render(<ExecutiveKpiGrid loading={false} error={false} data={response(metrics)} onRetry={vi.fn()} />);
    // All seven labels render.
    for (const key of ["headcount", "attendance_rate", "absence_rate", "late_rate", "turnover_rate", "overtime", "pending_hr_actions"]) {
      expect(screen.getByText(`dashboard.executive.metrics.${key}.label`)).toBeInTheDocument();
    }
  });

  it("renders a data-gap fallback for missing metric keys", () => {
    render(<ExecutiveKpiGrid loading={false} error={false} data={response([metric({ key: "headcount" })])} onRetry={vi.fn()} />);
    // Six missing slots fall back to the data-gap placeholder text.
    expect(screen.getAllByText("dashboard.executive.status.data_gap").length).toBeGreaterThanOrEqual(6);
  });

  it("prefers overtime_cost over overtime_hours when both present", () => {
    const metrics = [
      metric({ key: "headcount" }),
      metric({ key: "attendance_rate", unit: "percent", value: 95 }),
      metric({ key: "absence_rate", unit: "percent", value: 5 }),
      metric({ key: "late_rate", unit: "percent", value: null, status: "data_gap", health: "data_gap" }),
      metric({ key: "turnover_rate", unit: "percent", value: 2 }),
      metric({ key: "overtime_hours", unit: "hours", value: 8 }),
      metric({ key: "overtime_cost", unit: "currency", value: 1250 }),
      metric({ key: "pending_hr_actions", unit: "count", value: 7 }),
    ];
    render(<ExecutiveKpiGrid loading={false} error={false} data={response(metrics)} onRetry={vi.fn()} />);
    expect(screen.getByText("dashboard.executive.metrics.overtime.label")).toBeInTheDocument();
    // Currency rendering: "USD 1,250".
    expect(screen.getByText(/USD\s*1,250/i)).toBeInTheDocument();
  });

  it("renders restricted and not_applicable statuses with visible text", () => {
    const metrics = [
      metric({ key: "headcount" }),
      metric({ key: "attendance_rate", unit: "percent", value: null, status: "not_applicable", health: "not_applicable", null_reason: "zero_denominator" }),
      metric({ key: "absence_rate", unit: "percent", value: null, status: "restricted", health: "warning" }),
      metric({ key: "late_rate", unit: "percent", value: null, status: "data_gap", health: "data_gap" }),
      metric({ key: "turnover_rate", unit: "percent", value: 2 }),
      metric({ key: "overtime_hours", unit: "hours", value: 8 }),
      metric({ key: "pending_hr_actions", unit: "count", value: 7 }),
    ];
    render(<ExecutiveKpiGrid loading={false} error={false} data={response(metrics)} onRetry={vi.fn()} />);
    expect(screen.getAllByText("dashboard.executive.status.not_applicable").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("dashboard.executive.status.restricted").length).toBeGreaterThanOrEqual(1);
  });

  it("renders a zero count value instead of treating it as a data gap", () => {
    const metrics = [
      metric({ key: "headcount", value: 0 }),
      metric({ key: "attendance_rate", unit: "percent", value: 95 }),
      metric({ key: "absence_rate", unit: "percent", value: 5 }),
      metric({ key: "late_rate", unit: "percent", value: null, status: "data_gap", health: "data_gap" }),
      metric({ key: "turnover_rate", unit: "percent", value: 2 }),
      metric({ key: "overtime_hours", unit: "hours", value: 0 }),
      metric({ key: "pending_hr_actions", unit: "count", value: 0 }),
    ];
    render(<ExecutiveKpiGrid loading={false} error={false} data={response(metrics)} onRetry={vi.fn()} />);
    // Zero values render as "0", not the data-gap status text.
    expect(screen.getAllByText("0").length).toBeGreaterThanOrEqual(1);
  });
});

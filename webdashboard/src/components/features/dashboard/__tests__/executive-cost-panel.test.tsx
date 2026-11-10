import { fireEvent, render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

import type { DashboardMetric, DashboardMetricsResponse } from "@/hooks/dashboard-metric-contract";

import { ExecutiveCostPanel } from "../executive-cost-panel";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en" },
  }),
}));

vi.mock("recharts", () => ({
  BarChart: ({ children }: { children: React.ReactNode }) => <div data-testid="bar-chart">{children}</div>,
  Bar: ({ children }: { children: React.ReactNode }) => <div data-testid="bar">{children}</div>,
  Cell: () => <div data-testid="cell" />,
  XAxis: () => <div data-testid="x-axis" />,
  YAxis: () => <div data-testid="y-axis" />,
  CartesianGrid: () => <div data-testid="cartesian-grid" />,
  Tooltip: () => <div data-testid="tooltip" />,
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="responsive-container">{children}</div>
  ),
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

function response(metrics: DashboardMetric[], currency = "VND"): DashboardMetricsResponse {
  return {
    contract_version: "executive-dashboard.v2",
    organization: { timezone: "Asia/Ho_Chi_Minh", currency },
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
      scope: { kind: "organization", department_ids: [], location_ids: [], omitted_dimensions: [] },
    },
    metrics,
    insights: [],
    actions: { total: 0, overdue: 0, by_type: [] },
    data_quality: { status: "complete", sources: [] },
    generated_at: "2026-08-13T09:00:00Z",
  };
}

describe("ExecutiveCostPanel", () => {
  it("shows the hours view by default with the three hour series and approval-rate chip", () => {
    const metrics = [
      metric({ key: "system_ot_hours", unit: "hours", value: 12 }),
      metric({ key: "approved_ot_hours", unit: "hours", value: 8 }),
      metric({ key: "unapproved_ot_hours", unit: "hours", value: 4 }),
      metric({ key: "ot_approval_rate", unit: "percent", value: 66.7 }),
    ];
    render(<ExecutiveCostPanel currency="VND" data={response(metrics)} />);
    // Hours view is the default view.
    expect(screen.getByRole("radio", { name: "dashboard.executive.view_selector.hours" })).toHaveAttribute("aria-checked", "true");
    // The approval rate is reported as a text chip (not a bar), so it never shares
    // the hour axis — the no-dual-axis guarantee. Exactly three hour bars plot.
    expect(screen.getByText("dashboard.executive.metrics.ot_approval_rate.label")).toBeInTheDocument();
    expect(screen.getByText("66.7%")).toBeInTheDocument();
    expect(screen.getAllByTestId("cell").length).toBe(3);
  });

  it("shows currency values with the workspace currency prefix in the cost view table", async () => {
    const metrics = [
      metric({ key: "overtime_cost", unit: "currency", value: 1_250 }),
      metric({ key: "ot_cost_per_employee", unit: "currency", value: 312.5 }),
      metric({ key: "labor_cost", unit: "currency", value: 5_000_000 }),
      metric({ key: "payroll_actual", unit: "currency", value: 4_800_000 }),
      metric({ key: "budget_variance", unit: "currency", value: -500_000 }),
    ];
    render(<ExecutiveCostPanel currency="VND" data={response(metrics)} />);
    // Switch to cost view.
    fireEvent.click(screen.getByRole("radio", { name: "dashboard.executive.view_selector.cost" }));
    // Open the accessible table twin (collapsed by default).
    fireEvent.click(screen.getByRole("button", { name: /dashboard.hr.table_view/i }));
    // Currency-prefixed values render in the table.
    expect(screen.getByText(/VND\s*5,000,000/)).toBeInTheDocument();
    expect(screen.getByText(/VND\s*1,250/)).toBeInTheDocument();
  });

  it("does not plot restricted or data-gap values (skips them)", () => {
    const metrics = [
      metric({ key: "system_ot_hours", unit: "hours", value: 12 }),
      metric({ key: "approved_ot_hours", unit: "hours", value: 8 }),
      metric({ key: "unapproved_ot_hours", unit: "hours", value: 4 }),
      metric({ key: "overtime_cost", unit: "currency", value: null, status: "restricted", health: "warning" }),
      metric({ key: "ot_cost_per_employee", unit: "currency", value: null, status: "data_gap", health: "data_gap", null_reason: "no_paid_payslips" }),
      metric({ key: "labor_cost", unit: "currency", value: null, status: "data_gap", health: "data_gap", null_reason: "no_paid_payslips" }),
      metric({ key: "payroll_actual", unit: "currency", value: null, status: "data_gap", health: "data_gap", null_reason: "no_paid_payslips" }),
      metric({ key: "budget_variance", unit: "currency", value: null, status: "data_gap", health: "data_gap", null_reason: "budget_config_missing" }),
    ];
    render(<ExecutiveCostPanel currency="VND" data={response(metrics)} />);
    // Cost view has no plottable values → falls back to empty state.
    fireEvent.click(screen.getByRole("radio", { name: "dashboard.executive.view_selector.cost" }));
    expect(screen.getByText("common.no_data")).toBeInTheDocument();
  });
});

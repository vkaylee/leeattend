import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as dashboardHooks from "@/hooks/use-dashboard";

import { ExecutiveOvertimeBreakdown } from "../executive-overtime-breakdown";

vi.mock("@/hooks/use-dashboard", () => ({
  useOvertimeBreakdown: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en" },
  }),
}));

const renderWithProviders = (component: React.ReactElement) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      {component}
    </QueryClientProvider>,
  );
};

describe("ExecutiveOvertimeBreakdown", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders loading spinner while fetching", () => {
    vi.mocked(dashboardHooks.useOvertimeBreakdown).mockReturnValue({
      data: undefined,
      isLoading: true,
    } as unknown as ReturnType<typeof dashboardHooks.useOvertimeBreakdown>);

    renderWithProviders(<ExecutiveOvertimeBreakdown range={{}} />);
    expect(screen.queryByText("common.no_data")).not.toBeInTheDocument();
    expect(screen.queryByText("dashboard.executive.breakdown.title")).toBeInTheDocument();
  });

  it("renders an empty state when no departments have overtime", () => {
    vi.mocked(dashboardHooks.useOvertimeBreakdown).mockReturnValue({
      data: { contract_version: "overtime-breakdown.v2", organization: { timezone: "Asia/Ho_Chi_Minh", currency: "VND" }, rows: [], generated_at: "2026-08-13T09:00:00Z" },
      isLoading: false,
    } as unknown as ReturnType<typeof dashboardHooks.useOvertimeBreakdown>);

    renderWithProviders(<ExecutiveOvertimeBreakdown range={{}} />);
    expect(screen.getByText("common.no_data")).toBeInTheDocument();
  });

  it("renders department rows with hours and currency cost values", () => {
    const rows = [
      { department_id: "d1", department_name: "Engineering", system_ot_hours: 12.5, approved_ot_hours: 8, unapproved_ot_hours: 4.5, overtime_cost: "1250000", cost_available: true },
      { department_id: "d2", department_name: "Sales", system_ot_hours: 6, approved_ot_hours: 6, unapproved_ot_hours: 0, overtime_cost: "600000", cost_available: true },
    ];
    vi.mocked(dashboardHooks.useOvertimeBreakdown).mockReturnValue({
      data: { contract_version: "overtime-breakdown.v2", organization: { timezone: "Asia/Ho_Chi_Minh", currency: "VND" }, rows, generated_at: "2026-08-13T09:00:00Z" },
      isLoading: false,
    } as unknown as ReturnType<typeof dashboardHooks.useOvertimeBreakdown>);

    renderWithProviders(<ExecutiveOvertimeBreakdown costStatus="available" range={{}} />);
    // Open the accessible table twin (collapsed by default).
    fireEvent.click(screen.getByRole("button", { name: /dashboard.hr.table_view/i }));
    expect(screen.getByText("Engineering")).toBeInTheDocument();
    expect(screen.getByText("Sales")).toBeInTheDocument();
    // Hours formatted with "h".
    expect(screen.getByText("12.5h")).toBeInTheDocument();
    // Currency formatted with organization currency.
    expect(screen.getByText(/VND\s*1,250,000/)).toBeInTheDocument();
  });

  it("shows the restricted shield when the caller lacks payroll permission", () => {
    const rows = [
      { department_id: "d1", department_name: "Engineering", system_ot_hours: 12.5, approved_ot_hours: 8, unapproved_ot_hours: 4.5, overtime_cost: null, cost_available: false },
    ];
    vi.mocked(dashboardHooks.useOvertimeBreakdown).mockReturnValue({
      data: { contract_version: "overtime-breakdown.v2", organization: { timezone: "Asia/Ho_Chi_Minh", currency: "VND" }, rows, generated_at: "2026-08-13T09:00:00Z" },
      isLoading: false,
    } as unknown as ReturnType<typeof dashboardHooks.useOvertimeBreakdown>);

    renderWithProviders(<ExecutiveOvertimeBreakdown costStatus="restricted" range={{}} />);
    fireEvent.click(screen.getByRole("button", { name: /dashboard.hr.table_view/i }));
    expect(screen.getByText("dashboard.executive.breakdown.not_available")).toBeInTheDocument();
  });
});

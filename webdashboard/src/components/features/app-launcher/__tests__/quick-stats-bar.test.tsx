import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import QuickStatsBar from "@/components/features/app-launcher/quick-stats-bar";

// ── Mocks ─────────────────────────────────────────────────────────

const mockStats = {
  total_employees: 150,
  present_today: 130,
  late_today: 12,
  absent_today: 5,
  on_leave_today: 3,
  pending_leave_requests: 8,
  estimated_monthly_payroll: 125000.50,
  avg_work_hours_today: 7.8,
  early_departure_today: 2,
};

let mockLoading = false;
let mockError = false;

vi.mock("@/hooks/use-dashboard", () => ({
  useDashboardStats: () => ({
    data: mockError ? null : mockStats,
    isLoading: mockLoading,
    isError: mockError,
  }),
}));

vi.mock("@/hooks/use-currency", () => ({
  useCurrency: () => ({
    formatCurrency: (val: number) => `$${val.toLocaleString()}`,
  }),
}));

// ── Wrapper ───────────────────────────────────────────────────────

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

function Wrapper({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
}

// ── Tests ─────────────────────────────────────────────────────────

describe("QuickStatsBar", () => {
  beforeEach(() => {
    mockLoading = false;
    mockError = false;
  });

  it("renders all 5 stat items when data is loaded", () => {
    render(<QuickStatsBar />, { wrapper: Wrapper });

    expect(screen.getByText("150")).toBeInTheDocument(); // total
    expect(screen.getByText("130")).toBeInTheDocument(); // present
    expect(screen.getByText("12")).toBeInTheDocument(); // late
    expect(screen.getByText("8")).toBeInTheDocument(); // pending
    expect(screen.getByText("$125,000.5")).toBeInTheDocument(); // payroll
  });

  it("renders 5 stat items with proper structure", () => {
    const { container } = render(<QuickStatsBar />, { wrapper: Wrapper });

    // Each stat item has an SVG icon
    const icons = container.querySelectorAll("svg");
    expect(icons.length).toBeGreaterThanOrEqual(5);

    // Values should be rendered as text
    const valueTexts = ["150", "130", "12", "8"];
    for (const text of valueTexts) {
      expect(screen.getByText(text)).toBeInTheDocument();
    }
  });

  it("renders 5 skeleton items when loading", () => {
    mockLoading = true;

    const { container } = render(<QuickStatsBar />, { wrapper: Wrapper });

    // Skeleton elements should be present
    const skeletons = container.querySelectorAll(".animate-pulse");
    expect(skeletons.length).toBeGreaterThanOrEqual(5);
  });

  it("renders nothing when error occurs", () => {
    mockError = true;

    const { container } = render(<QuickStatsBar />, { wrapper: Wrapper });

    // Should render nothing (empty container)
    expect(container.innerHTML).toBe("");
  });

  it("renders nothing when data is null", () => {
    mockStats.total_employees = undefined as unknown as number;
    // Override data to null for this test
    mockError = true;

    const { container } = render(<QuickStatsBar />, { wrapper: Wrapper });

    expect(container.innerHTML).toBe("");
  });

  it("shows dashes for null payroll value", () => {
    mockStats.estimated_monthly_payroll = null as unknown as number;
    mockStats.avg_work_hours_today = 7.8;
    mockError = false;

    render(<QuickStatsBar />, { wrapper: Wrapper });

    // The payroll stat should show a dash ("—")
    expect(screen.getByText("—")).toBeInTheDocument();
  });
});

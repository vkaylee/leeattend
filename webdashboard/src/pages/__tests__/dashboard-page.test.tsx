 
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ children, to }: { children: React.ReactNode; to: string }) => <a href={to}>{children}</a>,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en" },
  }),
}));

vi.mock("i18next", () => ({
  t: (key: string) => key,
  default: { t: (key: string) => key },
}));



vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatTime: (v: string | number | Date) => v,
  }),
}));

 
vi.mock("@/hooks/use-dashboard", () => ({
  useDashboardStats: vi.fn(),
  useDashboardMetrics: vi.fn(),
  useOvertimeBreakdown: vi.fn(() => ({ data: undefined, isLoading: false, error: null })),
}));

vi.mock("@/hooks/use-dashboard-layout", () => ({
  useDashboardLayout: vi.fn(),
}));

vi.mock("@/hooks/use-header-actions", () => ({
  useHeaderActions: () => ({ actions: null, setActions: vi.fn() }),
  HeaderActionsProvider: ({ children }: { children: React.ReactNode }) => children,
}));

vi.mock("@/hooks/use-currency", () => ({
  useCurrency: () => ({ formatCurrency: (v: number) => `$${v}` }),
}));

vi.mock("@/hooks/use-dashboard-charts", () => ({
  useDashboardCharts: () => ({ chartElements: [], chartErrorCount: 0 }),
}));

let shouldQuickStatsThrow = false;
vi.mock("@/components/features/dashboard/hr-quick-stats", () => ({
  HrQuickStats: () => {
    if (shouldQuickStatsThrow) {
      throw new Error("Simulated Quick Stats Render Error");
    }
    return <div data-testid="quick-stats-mock">Quick Stats</div>;
  }
}));

import { useDashboardMetrics, useDashboardStats } from "@/hooks/use-dashboard";
import { useDashboardLayout } from "@/hooks/use-dashboard-layout";

import DashboardPage from "../dashboard-page";

describe("DashboardPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    vi.clearAllMocks();
    shouldQuickStatsThrow = false;



    vi.mocked(useDashboardMetrics).mockReturnValue({
      data: null,
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useDashboardMetrics>);

    vi.mocked(useDashboardStats).mockReturnValue({
      data: {

        total_employees: 100,
        present_today: 95,
        late_today: 5,
        absent_today: 2,
        on_leave_today: 3,
        pending_leave_requests: 1,
        avg_work_hours_today: 7.5,
        early_departure_today: 1,
        estimated_monthly_payroll: 50000,
      },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useDashboardStats>);

    vi.mocked(useDashboardLayout).mockReturnValue({
      activeView: "overview",
 
      visibleWidgets: {
        kpi_cards: true,
        donut_chart: true,
        quick_stats: true,
        attendance_trend: true,
        department_chart: true,
        promo_cards: true,
      },
      widgetOrder: [
        "kpi_cards",
        "donut_chart",
        "quick_stats",
        "attendance_trend",
        "department_chart",
        "promo_cards",
      ],
      collapsedSections: {
        summary: false,
        charts: false,
        actions: false,
      },
      isSaving: false,
      setActiveView: vi.fn(),
      toggleWidget: vi.fn(),
      setWidgetOrder: vi.fn(),
      toggleSection: vi.fn(),
      resetToDefault: vi.fn(),
      allWidgets: [],
    } as unknown as ReturnType<typeof useDashboardLayout>);
  });

  const renderComponent = () => {
    return render(
      <ToastProvider>
        <QueryClientProvider client={queryClient}>
          <DashboardPage />
        </QueryClientProvider>
      </ToastProvider>
    );
  };

  it("renders correctly with data", () => {
    renderComponent();

    // Check KPI cards render stats (title is now in Navbar breadcrumbs)
    expect(screen.getByText("100")).toBeInTheDocument(); // total_employees
    expect(screen.getByText("95")).toBeInTheDocument(); // present_today
  });

  it("renders HR tab with KPI data by default", () => {
    renderComponent();

    // Tab buttons are now in Navbar header via useHeaderActions context.
    // In test, useHeaderActions is mocked — the HR tab renders by default.
    // Verify KPI stats are visible:
    expect(screen.getByText("100")).toBeInTheDocument();
    expect(screen.getByText("95")).toBeInTheDocument();
  });

  it("isolates render crash inside a single widget using ErrorBoundary", () => {
    shouldQuickStatsThrow = true;
    
    // Suppress console.error output during the throwing test to keep test runs clean
    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(vi.fn());
    
    renderComponent();
    
    // 1. Verify that the crashed widget renders the fallback UI
    expect(screen.getByText("common.error_boundary.widget_error")).toBeInTheDocument();
    expect(screen.getByText("Simulated Quick Stats Render Error")).toBeInTheDocument();
    
    // 2. Verify that sibling components (e.g., KPI card) still render correctly
    expect(screen.getByText("100")).toBeInTheDocument(); // total_employees
    
    // Clean up
    consoleErrorSpy.mockRestore();
    shouldQuickStatsThrow = false;
  });
});

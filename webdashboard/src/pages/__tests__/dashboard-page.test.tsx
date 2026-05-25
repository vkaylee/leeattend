 
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ children, to }: { children: React.ReactNode; to: string }) => <a href={to}>{children}</a>,
}));

vi.mock("react-i18next", () => ({
 
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en" },
  }),
}));

vi.mock("@/hooks/use-agents", () => ({
  useAgentsStatus: vi.fn(),
}));

vi.mock("@/hooks/use-devices", () => ({
  useDevices: vi.fn(),
}));

vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatTime: (v: string | number | Date) => v,
  }),
}));

 
vi.mock("@/hooks/use-dashboard", () => ({
  useDashboardStats: vi.fn(),
}));

vi.mock("@/hooks/use-dashboard-layout", () => ({
  useDashboardLayout: vi.fn(),
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

import { useAgentsStatus } from "@/hooks/use-agents";
import { useDashboardStats } from "@/hooks/use-dashboard";
import { useDashboardLayout } from "@/hooks/use-dashboard-layout";
import { useDevices } from "@/hooks/use-devices";

import DashboardPage from "../dashboard-page";

describe("DashboardPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    vi.clearAllMocks();
    shouldQuickStatsThrow = false;

    vi.mocked(useAgentsStatus).mockReturnValue({
      agents: [],
 
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useAgentsStatus>);

    vi.mocked(useDevices).mockReturnValue({
      devices: [],
 
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useDevices>);

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
      <QueryClientProvider client={queryClient}>
        <DashboardPage />
      </QueryClientProvider>
    );
  };

  it("renders correctly with data", () => {
    renderComponent();
    
    // Check main title
    expect(screen.getByTestId("dashboard-title")).toBeInTheDocument();
    
    // Check KPI cards render stats
    expect(screen.getByText("100")).toBeInTheDocument(); // total_employees
    expect(screen.getByText("95")).toBeInTheDocument(); // present_today
  });

  it("switches to system tab", () => {
    renderComponent();
    
    // System tab button is labeled via translation
    const systemTabButton = screen.getByText("dashboard.tabs.system");
    fireEvent.click(systemTabButton);
    
    // Check if system section titles appear
    expect(screen.getByText("dashboard.system.total_agents")).toBeInTheDocument();
    expect(screen.getByText("dashboard.system.online_agents")).toBeInTheDocument();
    expect(screen.getByText("dashboard.system.total_devices")).toBeInTheDocument();
  });

  it("isolates render crash inside a single widget using ErrorBoundary", () => {
    shouldQuickStatsThrow = true;
    
    // Suppress console.error output during the throwing test to keep test runs clean
    const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(vi.fn());
    
    renderComponent();
    
    // 1. Verify that the crashed widget renders the fallback UI
    expect(screen.getByText("Widget Error")).toBeInTheDocument();
    expect(screen.getByText("Simulated Quick Stats Render Error")).toBeInTheDocument();
    
    // 2. Verify that sibling components (e.g., dashboard title and KPI card) still render correctly
    expect(screen.getByTestId("dashboard-title")).toBeInTheDocument();
    expect(screen.getByText("100")).toBeInTheDocument(); // total_employees
    
    // Clean up
    consoleErrorSpy.mockRestore();
    shouldQuickStatsThrow = false;
  });
});

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import DesktopPage from "../desktop-page";

const renderDesktop = () => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={queryClient}>
      <DesktopPage />
    </QueryClientProvider>,
  );
};

const mockNavigate = vi.fn();

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
  Link: ({ children, to, ...props }: React.AnchorHTMLAttributes<HTMLAnchorElement> & { to?: string }) => (
    <a href={to} {...props}>
      {children}
    </a>
  ),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { defaultValue?: string }) => opts?.defaultValue ?? key,
  }),
}));
vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  }),
  ToastProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));


vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: () => true,
    isLoading: false,
  }),
}));

vi.mock("@/hooks/use-profile", () => ({
  useProfile: () => ({ user: { id: "test-user", full_name: "John Doe" } }),
}));
vi.mock("@/hooks/use-dashboard", () => ({
  useDashboardStats: () => ({
    data: {
      total_employees: 100,
      present_today: 85,
      on_leave_today: 10,
      pending_leave_requests: 5,
      late_today: 2,
      absent_today: 3,
      early_departure_today: 1,
      avg_work_hours_today: 8,
      estimated_monthly_payroll: "50000000",
      attendance_rate: { value: 85, formatted: "85%", raw: 0.85, unit: "%" },
      absence_rate: { value: 3, formatted: "3%", raw: 0.03, unit: "%" },
      late_rate: { value: 2, formatted: "2%", raw: 0.02, unit: "%" },
    },
    isLoading: false,
    isError: false,
  }),
}));

vi.mock("@/hooks/use-my-attendance", () => ({
  useMyClock: () => ({
    activeEntry: undefined,
    isPunching: false,
    punch: vi.fn(),
    isLoading: false,
  }),
}));

vi.mock("@/hooks/use-currency", () => ({
  useCurrency: () => ({
    formatCurrency: (val: number) => `${val.toLocaleString()} đ`,
  }),
}));

describe("DesktopPage", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });

  it("renders desktop header and core widgets", () => {
    renderDesktop();

    expect(screen.getByTestId("desktop-page")).toBeInTheDocument();
    expect(screen.getByTestId("desktop-search-trigger")).toBeInTheDocument();
    expect(screen.getByTestId("desktop-customize-btn")).toBeInTheDocument();

    // Verify presence of core widgets
    expect(screen.getByTestId("desktop-widget-quick-punch")).toBeInTheDocument();
    expect(screen.getByTestId("desktop-widget-system-health")).toBeInTheDocument();
    expect(screen.getByTestId("desktop-widget-today-overview")).toBeInTheDocument();
  });

  it("opens customization modal when 'Gắn Widget' button is clicked", () => {
    renderDesktop();

    const customizeBtn = screen.getByTestId("desktop-customize-btn");
    fireEvent.click(customizeBtn);

    expect(screen.getByTestId("desktop-customizer-modal")).toBeInTheDocument();
  });

  it("toggles widget visibility", () => {
    renderDesktop();

    // Open customizer
    fireEvent.click(screen.getByTestId("desktop-customize-btn"));

    // Toggle off quick-punch
    const toggleQuickPunch = screen.getByTestId("toggle-widget-quick-punch");
    fireEvent.click(toggleQuickPunch);

    // Verify it is removed from the desktop grid
    expect(screen.queryByTestId("desktop-widget-quick-punch")).not.toBeInTheDocument();
  });

  it("renders empty state when all widgets are hidden", () => {
    localStorage.setItem(
      "leeattend-desktop-widgets-v1",
      JSON.stringify({ enabledIds: [], order: [] }),
    );

    renderDesktop();

    expect(screen.getByTestId("desktop-empty-state")).toBeInTheDocument();
    expect(screen.getByTestId("desktop-empty-add-widget-btn")).toBeInTheDocument();
  });
});

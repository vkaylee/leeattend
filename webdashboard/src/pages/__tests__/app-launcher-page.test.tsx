import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React, { Suspense } from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import AppLauncherPage from "@/pages/app-launcher-page";

// ── Mocks ─────────────────────────────────────────────────────────

const activePermissions = new Set<string>();
let mockSystemRoles: string[] = [];
let mockPermsLoading = false;

const mockHasAnyPermission = vi.fn((...perms: string[]) =>
  perms.length === 0 || perms.some((p) => activePermissions.has(p))
);

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: mockHasAnyPermission,
    permissions: Array.from(activePermissions),
    roles: [],
    managedDepartmentIds: null,
    isLoading: mockPermsLoading,
    hasPermission: vi.fn(),
    hasRole: vi.fn(),
    hasScopedPermission: vi.fn(),
  }),
}));

vi.mock("@/hooks/use-profile", () => ({
  useProfile: () => ({
    user: {
      id: "x",
      email: "a@b.com",
      full_name: "Test",
      system_roles: mockSystemRoles,
    },
    isLoading: false,
    isError: false,
    error: null,
    refetch: vi.fn(),
    updateProfile: vi.fn(),
    isUpdating: false,
  }),
}));

const mockStats = {
  total_employees: 100,
  present_today: 85,
  late_today: 8,
  absent_today: 5,
  on_leave_today: 2,
  pending_leave_requests: 4,
  estimated_monthly_payroll: 80000,
  avg_work_hours_today: 8,
  early_departure_today: 1,
};

vi.mock("@/hooks/use-dashboard", () => ({
  useDashboardMetrics: () => ({ data: null, isLoading: false, error: null }),
  useDashboardStats: () => ({
    data: mockStats,
    isLoading: false,
    isError: false,
  }),
}));

vi.mock("@/hooks/use-currency", () => ({
  useCurrency: () => ({
    formatCurrency: (val: number) => `$${val}`,
  }),
}));

// ── Wrapper ───────────────────────────────────────────────────────

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

function Wrapper({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      <Suspense fallback={<div>Loading...</div>}>
        {children}
      </Suspense>
    </QueryClientProvider>
  );
}

// ── Helpers ───────────────────────────────────────────────────────

function grantAllPermissions() {
  const allPerms = [
    "employees:read", "members:read", "departments:read", "employees:update",
    "attendance:logs:read", "leave:requests:read", "leave:requests:submit_on_behalf",
    "leave:requests:approve", "leave:balances:read", "attendance:adjustments:approve",
    "attendance:adjustments:submit_on_behalf", "shift_swaps:manage", "reports:read",
    "overtime:approve", "overtime:claim_on_behalf", "shifts:read",
    "payroll:read", "workspace:settings:read", "members:roles:manage",
    "ess:profile", "ess:attendance", "ess:clock",
  ];
  for (const p of allPerms) activePermissions.add(p);
  mockSystemRoles = ["platform_admin"];
}

// ── Tests ─────────────────────────────────────────────────────────

describe("AppLauncherPage", () => {
  beforeEach(() => {
    activePermissions.clear();
    mockSystemRoles = [];
    mockPermsLoading = false;
    localStorage.clear();
    vi.clearAllMocks();
  });

  // ── Loading state ──────────────────────────────────────────

  it("shows skeleton when permissions are loading", () => {
    mockPermsLoading = true;
    render(<AppLauncherPage />, { wrapper: Wrapper });
    expect(screen.getByTestId("launcher-skeleton")).toBeInTheDocument();
  });

  // ── Empty / minimal state ───────────────────────────────────

  it("dashboard app cards always visible (no permission required)", () => {
    render(<AppLauncherPage />, { wrapper: Wrapper });
    expect(screen.getByTestId("app-card-dashboard")).toBeInTheDocument();
    expect(screen.getByTestId("app-card-dashboard-analytics")).toBeInTheDocument();

    // Only 2 dashboard cards — no other app cards
    const allCards = screen.getAllByTestId(/^app-card-/);
    expect(allCards.length).toBe(2);
  });

  // ── Full permissions ────────────────────────────────────────

  it("renders quick stats section", () => {
    grantAllPermissions();
    render(<AppLauncherPage />, { wrapper: Wrapper });
    expect(screen.getByTestId("launcher-quick-stats")).toBeInTheDocument();
  });

  it("renders hero search trigger and triggers open event", () => {
    grantAllPermissions();
    const dispatchSpy = vi.spyOn(window, "dispatchEvent");
    render(<AppLauncherPage />, { wrapper: Wrapper });
    const searchBtn = screen.getByTestId("launcher-hero-search").querySelector("button");
    expect(searchBtn).toBeInTheDocument();
    fireEvent.click(searchBtn!);
    expect(dispatchSpy).toHaveBeenCalled();
    dispatchSpy.mockRestore();
  });

  it("renders all apps section when user has permissions", () => {
    grantAllPermissions();
    render(<AppLauncherPage />, { wrapper: Wrapper });
    expect(screen.getByTestId("launcher-all-apps")).toBeInTheDocument();
  });

  it("renders apps from every group", () => {
    grantAllPermissions();
    render(<AppLauncherPage />, { wrapper: Wrapper });

    expect(screen.getByTestId("app-card-dashboard")).toBeInTheDocument();
    expect(screen.getByTestId("app-card-employees")).toBeInTheDocument();
    expect(screen.getByTestId("app-card-payroll")).toBeInTheDocument();
    expect(screen.getByTestId("app-card-settings")).toBeInTheDocument();
  });

  it("renders group sections with testIds", () => {
    grantAllPermissions();
    render(<AppLauncherPage />, { wrapper: Wrapper });

    expect(screen.getByTestId("launcher-group-dashboard")).toBeInTheDocument();
    expect(screen.getByTestId("launcher-group-org")).toBeInTheDocument();
    expect(screen.getByTestId("launcher-group-time")).toBeInTheDocument();
    expect(screen.getByTestId("launcher-group-payroll")).toBeInTheDocument();
    expect(screen.getByTestId("launcher-group-system")).toBeInTheDocument();
    expect(screen.getByTestId("launcher-group-ess")).toBeInTheDocument();
  });

  // ── Quick Access ────────────────────────────────────────────

  it("shows Quick Access section when recent apps exist", () => {
    localStorage.setItem(
      "leeattend-quick-access",
      JSON.stringify(["dashboard", "employees", "payroll"]),
    );
    grantAllPermissions();

    render(<AppLauncherPage />, { wrapper: Wrapper });
    expect(screen.getByTestId("launcher-quick-access")).toBeInTheDocument();
  });

  it("hides Quick Access when no recent apps", () => {
    grantAllPermissions();
    render(<AppLauncherPage />, { wrapper: Wrapper });
    expect(screen.queryByTestId("launcher-quick-access")).not.toBeInTheDocument();
  });

  // ── Role-based visibility ───────────────────────────────────

  it("hides system-role-gated items from non-platform_admin", () => {
    activePermissions.add("employees:read");
    activePermissions.add("members:read");
    activePermissions.add("workspace:settings:read");
    activePermissions.add("ess:profile");
    mockSystemRoles = [];

    render(<AppLauncherPage />, { wrapper: Wrapper });

    // Non-admin should NOT see system-role-gated items
    expect(screen.queryByTestId("app-card-system-roles")).not.toBeInTheDocument();
    expect(screen.queryByTestId("app-card-infra-monitor")).not.toBeInTheDocument();

    // But should see permission-gated settings
    expect(screen.getByTestId("app-card-settings")).toBeInTheDocument();
  });

  it("shows system items for platform_admin", () => {
    mockSystemRoles = ["platform_admin"];
    render(<AppLauncherPage />, { wrapper: Wrapper });

    expect(screen.getByTestId("app-card-system-roles")).toBeInTheDocument();
    expect(screen.getByTestId("app-card-infra-monitor")).toBeInTheDocument();
  });

  it("shows system items for unified admin", () => {
    mockSystemRoles = ["admin"];
    render(<AppLauncherPage />, { wrapper: Wrapper });

    expect(screen.getByTestId("app-card-system-roles")).toBeInTheDocument();
    expect(screen.getByTestId("app-card-infra-monitor")).toBeInTheDocument();
  });

  it("shows ESS apps only when user has corresponding permissions", () => {
    activePermissions.add("ess:profile");
    activePermissions.add("ess:attendance");

    render(<AppLauncherPage />, { wrapper: Wrapper });

    expect(screen.getByTestId("app-card-ess-profile")).toBeInTheDocument();
    expect(screen.getByTestId("app-card-ess-attendance")).toBeInTheDocument();
    expect(screen.queryByTestId("app-card-ess-clock")).not.toBeInTheDocument();
  });

  // ── Interaction ─────────────────────────────────────────────

  it("records access on card click", () => {
    activePermissions.add("employees:read");
    activePermissions.add("members:read");
    activePermissions.add("departments:read");
    activePermissions.add("ess:profile");

    render(<AppLauncherPage />, { wrapper: Wrapper });

    fireEvent.click(screen.getByTestId("app-card-ess-profile"));

    const stored = JSON.parse(localStorage.getItem("leeattend-quick-access")!);
    expect(stored).toContain("ess-profile");
  });
});

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React, { Suspense, lazy } from "react";
import { MemoryRouter } from "react-router-dom";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/Toast";
import i18n from "../../i18n/config";

// Mock all hooks needed by pages
vi.mock("@/hooks/use-permissions", () => ({
    useMyPermissions: () => ({
        roles: ["Admin"],
        permissions: ["employees:read", "employees:create", "employees:update", "employees:delete"],
        managedDepartmentIds: null,
        isLoading: false,
        hasPermission: () => true,
        hasAnyPermission: () => true,
        hasRole: () => true,
        hasScopedPermission: () => true,
    }),
}));

vi.mock("@/hooks/use-profile", () => ({
    useProfile: () => ({
        user: { id: "test-user", email: "test@example.com", full_name: "Test User", preferred_language: "en" },
        isLoading: false,
        isError: false,
    }),
}));

vi.mock("@/hooks/use-workspace", () => ({
    useWorkspaces: () => ({
        workspaces: [{ id: "test-ws-id", name: "Test Workspace", default_language: "en" }],
        isLoading: false,
        isError: false,
    }),
    useWorkspace: () => ({
        workspace: { id: "test-ws-id", name: "Test Workspace", date_format: "DD/MM/YYYY", default_language: "en" },
        isLoading: false,
        isError: false,
    }),
}));

vi.mock("@/hooks/use-theme", () => ({
    useTheme: () => ({
        primaryColor: "#10b981",
        setPreviewTheme: vi.fn(),
        setUserTheme: vi.fn(),
    }),
}));

vi.mock("@/hooks/use-i18n-sync", () => ({
    useI18nSync: vi.fn(),
}));

vi.mock("@/hooks/use-agents", () => ({
    useAgentsStatus: () => ({ agents: [], isLoading: false, error: null }),
}));

vi.mock("@/hooks/use-devices", () => ({
    useDevices: () => ({ devices: [], isLoading: false, error: null }),
}));

vi.mock("@/hooks/use-dashboard", () => ({
    useDashboardStats: () => ({
        data: { total_employees: 10, present_today: 8, pending_leave_requests: 2, estimated_monthly_payroll: "50000", late_today: 1, absent_today: 1, on_leave_today: 1, early_departure_today: 0, avg_work_hours_today: 7.5 },
        isLoading: false, error: null,
    }),
    useAttendanceTrend: () => ({ data: [], isLoading: false, error: null }),
    useDepartmentAttendance: () => ({ data: [], isLoading: false, error: null }),
    useOvertimeTrend: () => ({ data: [], isLoading: false, error: null }),
    usePeakTimes: () => ({ data: [], isLoading: false, error: null }),
    useLeaveTypesBreakdown: () => ({ data: [], isLoading: false, error: null }),
    useLatenessByDay: () => ({ data: [], isLoading: false, error: null }),
    useUnexcusedAbsences: () => ({ data: [], isLoading: false, error: null }),
    useRecruitmentSourceBreakdown: () => ({ data: [], isLoading: false, error: null }),
    useTurnoverAnalytics: () => ({ data: null, isLoading: false, error: null }),
}));

vi.mock("@/hooks/use-dashboard-layout", () => ({
    useDashboardLayout: () => ({
        activeView: "hr",
        visibleWidgets: {},
        widgetOrder: [],
        collapsedSections: {},
        isSaving: false,
        setActiveView: vi.fn(),
        toggleWidget: vi.fn(),
        setWidgetOrder: vi.fn(),
        toggleSection: vi.fn(),
        resetToDefault: vi.fn(),
        allWidgets: [],
    }),
}));

vi.mock("@/lib/api", async (importOriginal) => {
 
    const actual = await importOriginal<Record<string, unknown>>();
    return {
        ...actual,
        getCurrentUserId: vi.fn(() => "test-user"),
        getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
        fetchWorkspaceLeaveBalances: vi.fn(() => Promise.resolve([])),
        fetchEmployees: vi.fn(() => Promise.resolve([])),
        fetchWorkspace: vi.fn(() => Promise.resolve({ date_format: "DD/MM/YYYY", timezone: "UTC", default_language: "en" })),
        fetchDepartments: vi.fn(() => Promise.resolve([])),
        fetchRawAttendanceLogs: vi.fn(() => Promise.resolve({ data: [], total: 0 })),
        fetchMyEmployeeProfile: vi.fn(() => Promise.resolve({ id: "emp-1", full_name: "Test User" })),
    };
});

// Mock IntersectionObserver for jsdom
class MockIntersectionObserver {
    observe() { }
    unobserve() { }
    disconnect() { }
}
 
window.IntersectionObserver = MockIntersectionObserver as unknown as typeof IntersectionObserver;
vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);

vi.mock("recharts", () => ({
 
    ResponsiveContainer: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
    BarChart: () => <div />, Bar: () => <div />, XAxis: () => <div />, YAxis: () => <div />,
    CartesianGrid: () => <div />, Tooltip: () => <div />, Legend: () => <div />,
    LineChart: () => <div />, Line: () => <div />, AreaChart: () => <div />, Area: () => <div />,
    PieChart: () => <div />, Pie: () => <div />, Cell: () => <div />,
    ComposedChart: () => <div />,
}));

describe("Lazy Route Loading", () => {
    let queryClient: QueryClient;

    beforeEach(() => {
        vi.clearAllMocks();
        i18n.changeLanguage("en");
        queryClient = new QueryClient({
            defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
        });
    });

    const renderWithProviders = (ui: React.ReactElement) => {
        return render(
            <QueryClientProvider client={queryClient}>
                <ToastProvider>
                    <MemoryRouter>
                        {ui}
                    </MemoryRouter>
                </ToastProvider>
            </QueryClientProvider>
        );
    };

    it("should lazy load DashboardPage with Suspense", async () => {
 
        const LazyDashboard = lazy(() => import("../DashboardPage"));

        renderWithProviders(
            <Suspense fallback={<div data-testid="loading">Loading...</div>}>
                <LazyDashboard />
            </Suspense>
        );

        // Should eventually render the dashboard (may briefly show fallback)
        await waitFor(() => {
            expect(screen.getByTestId("dashboard-title")).toBeInTheDocument();
        }, { timeout: 5000 });
    });

    it("should lazy load LoginPage with Suspense", async () => {
 
        const LazyLogin = lazy(() => import("../LoginPage"));

        renderWithProviders(
            <Suspense fallback={<div data-testid="loading">Loading...</div>}>
                <LazyLogin />
            </Suspense>
        );

        await waitFor(() => {
            // LoginPage should render some form or heading
            expect(document.querySelector("form, h1, h2, [data-testid]")).toBeTruthy();
        }, { timeout: 5000 });
    });

    it("should lazy load EmployeesPage with Suspense", async () => {
 
        const LazyEmployees = lazy(() => import("../EmployeesPage"));

        renderWithProviders(
            <Suspense fallback={<div data-testid="loading">Loading...</div>}>
                <LazyEmployees />
            </Suspense>
        );

        await waitFor(() => {
            const headings = screen.getAllByRole("heading");
            expect(headings.some(h =>
                /Employees/i.test(h.textContent) || /Nhân viên/i.test(h.textContent)
            )).toBe(true);
        }, { timeout: 5000 });
    });

    it("should show fallback while page is loading", async () => {
        // Create a lazy component that simulates a slow import
 
        let resolveImport: (value: unknown) => void = () => {};
 
        const slowImport = new Promise<unknown>((resolve) => {
            resolveImport = resolve;
        });

 
        const LazyComponent = lazy(() => slowImport as Promise<{ default: React.ComponentType<unknown> }>);

        renderWithProviders(
            <Suspense fallback={<div data-testid="suspense-fallback">Loading...</div>}>
                <LazyComponent />
            </Suspense>
        );

        // Fallback should be shown immediately while component is loading
        expect(screen.getByTestId("suspense-fallback")).toBeInTheDocument();

        // Resolve the import with a simple component
        (resolveImport)({ default: () => <div data-testid="loaded-component">Loaded</div> });

        // After resolving, the component should render
        await waitFor(() => {
            expect(screen.getByTestId("loaded-component")).toBeInTheDocument();
        });
    });

    it("should handle multiple lazy pages without interference", async () => {
 
        const LazyDashboard = lazy(() => import("../DashboardPage"));

        const { unmount } = renderWithProviders(
            <Suspense fallback={<div>Loading...</div>}>
                <LazyDashboard />
            </Suspense>
        );

        await waitFor(() => {
            expect(screen.getByTestId("dashboard-title")).toBeInTheDocument();
        }, { timeout: 5000 });

        unmount();

        // Render a different lazy page after unmounting the first
 
        const LazyEmployees = lazy(() => import("../EmployeesPage"));

        renderWithProviders(
            <Suspense fallback={<div>Loading...</div>}>
                <LazyEmployees />
            </Suspense>
        );

        await waitFor(() => {
            const headings = screen.getAllByRole("heading");
            expect(headings.length).toBeGreaterThan(0);
        }, { timeout: 5000 });
    });
});

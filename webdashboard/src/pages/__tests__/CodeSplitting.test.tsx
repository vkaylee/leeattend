import React, { Suspense } from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/Toast";


// ── Mock hooks needed by pages ──────────────────────────────────────

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
    const actual = await importOriginal<any>();
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
        fetchMe: vi.fn(() => Promise.resolve({ id: "test-user", email: "test@example.com", full_name: "Test User" })),
    };
});

vi.mock("@/components/debug/DebugToolbar", () => ({
    DebugToolbar: () => null,
}));

class MockIntersectionObserver {
    observe() { }
    unobserve() { }
    disconnect() { }
}
window.IntersectionObserver = MockIntersectionObserver as any;
vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);

vi.mock("recharts", () => ({
    ResponsiveContainer: ({ children }: any) => <div>{children}</div>,
    BarChart: () => <div />, Bar: () => <div />, XAxis: () => <div />, YAxis: () => <div />,
    CartesianGrid: () => <div />, Tooltip: () => <div />, Legend: () => <div />,
    LineChart: () => <div />, Line: () => <div />, AreaChart: () => <div />, Area: () => <div />,
    PieChart: () => <div />, Pie: () => <div />, Cell: () => <div />,
    ComposedChart: () => <div />,
}));

// ── Route lazy loading — file structure verification ────────────────

describe("TanStack Router .lazy.tsx file structure", () => {
    it("lazy route for dashboard exports Route with createLazyFileRoute", async () => {
        const mod = await import("../../routes/_authenticated/_workspace/index.lazy");
        expect(mod.Route).toBeDefined();
        expect(mod.Route.options.component).toBeDefined();
    });

    it("lazy route for employees exports Route with EmployeesPage", async () => {
        const mod = await import(/* @vite-ignore */ "../../routes/_authenticated/_workspace/employees.lazy");
        expect(mod.Route).toBeDefined();
        expect(mod.Route.options.component).toBeDefined();
    });

    it("lazy route for login exports Route with LoginPage", async () => {
        const mod = await import(/* @vite-ignore */ "../../routes/login.lazy");
        expect(mod.Route).toBeDefined();
        expect(mod.Route.options.component).toBeDefined();
    });

    it("lazy route for reports exports Route with ReportsPage", async () => {
        const mod = await import(/* @vite-ignore */ "../../routes/_authenticated/_workspace/reports.lazy");
        expect(mod.Route).toBeDefined();
        expect(mod.Route.options.component).toBeDefined();
    });

    it("lazy route for settings/general exports Route with GeneralSettings", async () => {
        const mod = await import("../../routes/_authenticated/_workspace/workspace/settings/general.lazy");
        expect(mod.Route).toBeDefined();
        expect(mod.Route.options.component).toBeDefined();
    });

    it("skeleton route (employees) has no component", async () => {
        const mod = await import("../../routes/_authenticated/_workspace/employees");
        expect(mod.Route).toBeDefined();
        expect(mod.Route.options.component).toBeUndefined();
    });

    it("skeleton route (dashboard) has no component", async () => {
        const mod = await import("../../routes/_authenticated/_workspace/index");
        expect(mod.Route).toBeDefined();
        expect(mod.Route.options.component).toBeUndefined();
    });

    it("layout route _workspace retains MainLayout component", async () => {
        const mod = await import("../../routes/_authenticated/_workspace");
        expect(mod.Route.options.component).toBeDefined();
        expect(mod.Route.options.beforeLoad).toBeDefined();
    });

    it("layout route _infra retains InfraLayout component", async () => {
        const mod = await import("../../routes/_authenticated/_infra");
        expect(mod.Route.options.component).toBeDefined();
    });

    it("settings layout retains SettingsLayout component", async () => {
        const mod = await import("../../routes/_authenticated/_workspace/workspace/settings");
        expect(mod.Route.options.component).toBeDefined();
    });

    it("auth layout retains beforeLoad and component", async () => {
        const mod = await import("../../routes/_authenticated");
        expect(mod.Route.options.beforeLoad).toBeDefined();
        expect(mod.Route.options.component).toBeDefined();
    });
});

// ── Page component lazy loading via React.lazy ───────────────────────

describe("Page-level code splitting", () => {
    let queryClient: QueryClient;

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({
            defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
        });
    });

    const renderLazy = (component: React.LazyExoticComponent<React.ComponentType<any>>) => {
        return render(
            <QueryClientProvider client={queryClient}>
                <ToastProvider>
                    <Suspense fallback={<div data-testid="fallback">Loading...</div>}>
                        {React.createElement(component)}
                    </Suspense>
                </ToastProvider>
            </QueryClientProvider>
        );
    };

    it("DashboardPage renders via lazy import", async () => {
        const LazyDashboard = React.lazy(() => import("../DashboardPage"));
        renderLazy(LazyDashboard);

        await waitFor(() => {
            expect(screen.getByTestId("dashboard-title")).toBeInTheDocument();
        }, { timeout: 5000 });
    });

    it("LoginPage renders via lazy import", async () => {
        const LazyLogin = React.lazy(() => import("../LoginPage"));
        renderLazy(LazyLogin);

        await waitFor(() => {
            expect(document.querySelector("form, h1, h2, [data-testid]")).toBeTruthy();
        }, { timeout: 5000 });
    });

    it("EmployeesPage renders via lazy import", async () => {
        const LazyEmployees = React.lazy(() => import("../EmployeesPage"));
        renderLazy(LazyEmployees);

        await waitFor(() => {
            const headings = screen.getAllByRole("heading");
            expect(headings.some(h =>
                /Employees/i.test(h.textContent!) || /Nhân viên/i.test(h.textContent!)
            )).toBe(true);
        }, { timeout: 5000 });
    });

    it("ShiftsPage imports and becomes a defined component", async () => {
        const mod = await import("../ShiftsPage");
        expect(mod.default).toBeDefined();
    });

    it("ReportsPage imports and becomes a defined component", async () => {
        const mod = await import("../ReportsPage");
        expect(mod.default).toBeDefined();
    });

    it("shows suspense fallback while page is loading", async () => {
        let resolveImport: (value: any) => void;
        const slowImport = new Promise<any>((resolve) => {
            resolveImport = resolve;
        });

        const LazyComponent = React.lazy(() => slowImport);

        render(
            <QueryClientProvider client={queryClient}>
                <Suspense fallback={<div data-testid="suspense-fallback">Loading...</div>}>
                    <LazyComponent />
                </Suspense>
            </QueryClientProvider>
        );

        expect(screen.getByTestId("suspense-fallback")).toBeInTheDocument();

        resolveImport!({ default: () => <div data-testid="loaded-component">Loaded</div> });

        await waitFor(() => {
            expect(screen.getByTestId("loaded-component")).toBeInTheDocument();
        });
    });

    it("multiple lazy pages can be loaded sequentially", async () => {
        const LazyDashboard = React.lazy(() => import("../DashboardPage"));

        const { unmount } = renderLazy(LazyDashboard);

        await waitFor(() => {
            expect(screen.getByTestId("dashboard-title")).toBeInTheDocument();
        }, { timeout: 5000 });

        unmount();

        const LazyEmployees = React.lazy(() => import("../EmployeesPage"));
        renderLazy(LazyEmployees);

        await waitFor(() => {
            const headings = screen.getAllByRole("heading");
            expect(headings.length).toBeGreaterThan(0);
        }, { timeout: 5000 });
    });
});

// ── i18n lazy loading ───────────────────────────────────────────────

import i18n from "../../i18n/config";

describe("i18n lazy locale loading", () => {
    it("has fallback language set to vi", () => {
        const f = i18n.options.fallbackLng;
        const fallback = Array.isArray(f) ? f[0] : f;
        expect(fallback).toBe("vi");
    });

    it("loads detected language on init (has translations)", () => {
        expect(i18n.language).toBeTruthy();
        expect(["en", "vi", "zh", "en-US", "vi-VN", "zh-CN"]).toContain(i18n.language);
    });

    it("changeLanguage loads new locale on demand", async () => {
        await i18n.changeLanguage("vi");
        expect(i18n.language).toBe("vi");
        expect(i18n.t("auth.checking_security")).toBeTruthy();
    });

    it("loaded locale resources contain translation keys", async () => {
        await i18n.changeLanguage("en");

        await waitFor(() => {
            const resources = i18n.getResourceBundle("en", "translation");
            expect(resources).toBeDefined();
            expect(Object.keys(resources || {}).length).toBeGreaterThan(0);
        }, { timeout: 3000 });
    });
});

// ── Vendor chunk splitting (manualChunks in vite.config.ts) ─────────

describe("Vendor chunk splitting", () => {
    it("vendor-react chunk can be imported", async () => {
        const react = await import("react");
        expect(react).toBeDefined();
    });

    it("vendor-query chunk can be imported", async () => {
        const query = await import("@tanstack/react-query");
        expect(query).toBeDefined();
    });

    it("vendor-charts chunk can be imported", async () => {
        const recharts = await import("recharts");
        expect(recharts).toBeDefined();
    });

    it("vendor-i18n chunk can be imported", async () => {
        const i18next = await import("i18next");
        expect(i18next).toBeDefined();
    });

    it("vendor-dnd chunk can be imported", async () => {
        const dnd = await import("@dnd-kit/core");
        expect(dnd).toBeDefined();
    });
});

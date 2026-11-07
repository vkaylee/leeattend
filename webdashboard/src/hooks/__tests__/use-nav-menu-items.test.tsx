import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { useFilteredMenuItems, useNavMenuItems, getActiveNavGroup } from "@/hooks/use-nav-menu-items";
import { APP_REGISTRY } from "@/lib/app-registry";
import { isFeatureEnabled } from "@/lib/feature-flags";

// ── Mock dependencies ─────────────────────────────────────────────

const activePermissions = new Set<string>();
const mockHasAnyPermission = vi.fn((...perms: string[]) =>
  perms.length === 0 || perms.some((p) => activePermissions.has(p))
);


vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: mockHasAnyPermission,
    permissions: Array.from(activePermissions),
    roles: [],
    managedDepartmentIds: null,
    isLoading: false,
    hasPermission: vi.fn(),
    hasRole: vi.fn(),
    hasScopedPermission: vi.fn(),
  }),
}));


// ── Providers wrapper ─────────────────────────────────────────────

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

function wrapper({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
}

// ── Tests ─────────────────────────────────────────────────────────

describe("useNavMenuItems", () => {
  beforeEach(() => {
    activePermissions.clear();
    vi.clearAllMocks();
  });

  it("returns dashboard, settings, and 6 groups", () => {
    const { result } = renderHook(() => useNavMenuItems(), { wrapper });

    expect(result.current.dashboard).toBeDefined();
    expect(result.current.dashboard.to).toBe("/");
    expect(result.current.dashboard.testId).toBe("sidebar-link-dashboard");

    expect(result.current.settings).toBeDefined();
    expect(result.current.settings.to).toBe("/workspace/settings");
    expect(result.current.settings.testId).toBe("sidebar-link-settings");
    expect(result.current.settings.icon).toBeDefined();

    expect(result.current.groups).toHaveLength(6);
    const sections = result.current.groups.map((g) => g.section);
    expect(sections).toEqual(["org", "time", "shifts", "payroll", "system", "ess"]);
  });

  it("each group has a sectionIcon, label, testId, and items", () => {
    const { result } = renderHook(() => useNavMenuItems(), { wrapper });

    for (const group of result.current.groups) {
      expect(group.sectionIcon).toBeDefined();
      expect(group.label).toBeTruthy();
      expect(group.testId).toMatch(/^sidebar-section-/);
      expect(group.items.length).toBeGreaterThan(0);
      for (const item of group.items) {
        expect(item.to).toBeTruthy();
        expect(item.label).toBeTruthy();
        expect(item.testId).toMatch(/^sidebar-link-/);
      }
    }
  });

  it("includes all registered apps from APP_REGISTRY in sidebar menu (drift-detection)", () => {
    const { result } = renderHook(() => useNavMenuItems(), { wrapper });
    const sidebarRoutes = new Set<string>();
    sidebarRoutes.add(result.current.dashboard.to);
    sidebarRoutes.add(result.current.settings.to);
    for (const group of result.current.groups) {
      for (const item of group.items) {
        sidebarRoutes.add(item.to);
      }
    }

    // Explicit intentional exclusions:
    // - "/dashboard": Secondary analytics catalog route, opened via in-page customization
    const EXCLUDED_APP_ROUTES = new Set<string>(["/dashboard"]);

    const missingFromSidebar = APP_REGISTRY
      .filter((app) => !app.featureFlag || isFeatureEnabled(app.featureFlag))
      .map((app) => app.to)
      .filter((route) => !EXCLUDED_APP_ROUTES.has(route) && !sidebarRoutes.has(route));
    expect(
      missingFromSidebar,
      `Found apps in APP_REGISTRY missing from sidebar navigation: ${missingFromSidebar.join(", ")}`
    ).toEqual([]);
  });
});

// ── Filtered menu ────────────────────────────────────────────────

describe("useFilteredMenuItems", () => {
  beforeEach(() => {
    activePermissions.clear();
    vi.clearAllMocks();
  });

  it("hides all sections when user has no permissions", () => {
    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });
    const groupsWithItems = result.current.groups.filter((g) => g.items.length > 0);

    // Every navigation item is gated by a standalone workspace permission.
    // No permission means no visible groups; the dashboard remains separate.
    expect(groupsWithItems).toHaveLength(0);
  });

  it("shows retained system navigation only with SMTP permission", () => {
    activePermissions.add("security:settings:manage");

    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    const systemGroup = result.current.groups.find((g) => g.section === "system");
    expect(systemGroup?.items.map((item) => item.to)).toEqual(["/system/smtp"]);
  });

  it("shows org section items when user has org permissions", () => {
    activePermissions.add("employees:read");
    activePermissions.add("members:read");
    activePermissions.add("departments:read");

    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    const orgGroup = result.current.groups.find((g) => g.section === "org");
    expect(orgGroup).toBeDefined();
    expect(orgGroup!.items.length).toBeGreaterThanOrEqual(3);
  });

  it("shows approval inbox when user has any inbox permission", () => {
    activePermissions.add("attendance:adjustments:approve");

    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    const timeGroup = result.current.groups.find((g) => g.section === "time");
    const approvalItem = timeGroup?.items.find((i) => i.to === "/approvals");
    expect(approvalItem?.testId).toBe("sidebar-link-approvals");
  });

  it("hides approval inbox when user has none of its permissions", () => {
    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    const timeGroup = result.current.groups.find((g) => g.section === "time");
    const approvalItem = timeGroup?.items.find((i) => i.to === "/approvals");
    expect(approvalItem).toBeUndefined();
  });

  it("shows leave-requests when user has any of its permissions (anyOf)", () => {
    activePermissions.add("leave:requests:approve");

    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    const timeGroup = result.current.groups.find((g) => g.section === "time");
    expect(timeGroup).toBeDefined();
    const lrItem = timeGroup!.items.find((i) => i.to === "/leave-requests");
    expect(lrItem).toBeDefined();
  });

  it("hides leave-requests when user has none of the required permissions", () => {
    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    const timeGroup = result.current.groups.find((g) => g.section === "time");
    const lrItem = timeGroup?.items.find((i) => i.to === "/leave-requests");
    expect(lrItem).toBeUndefined();
  });

  it("shows worker monitor when user has infra:monitor permission", () => {
    activePermissions.add("infra:monitor");

    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    const systemGroup = result.current.groups.find((g) => g.section === "system");
    const monitorItem = systemGroup?.items.find((i) => i.to === "/infra/monitor");
    expect(monitorItem?.testId).toBe("sidebar-link-infra-monitor");
  });

  it("shows audit-logs when user has audit_logs:read permission", () => {
    activePermissions.add("audit_logs:read");

    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    const systemGroup = result.current.groups.find((g) => g.section === "system");
    const auditItem = systemGroup?.items.find((i) => i.to === "/audit-logs");
    expect(auditItem?.testId).toBe("sidebar-link-audit-logs");
  });

  it("shows documents hub when user has payroll:read permission", () => {
    activePermissions.add("payroll:read");

    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    const payrollGroup = result.current.groups.find((g) => g.section === "payroll");
    const docItem = payrollGroup?.items.find((i) => i.to === "/documents");
    expect(docItem?.testId).toBe("sidebar-link-documents");
  });

});

// ── Path detection ────────────────────────────────────────────────

describe("getActiveNavGroup", () => {
  it('returns "dashboard" for "/"', () => {
    expect(getActiveNavGroup("/")).toBe("dashboard");
  });

  it('returns "org" for /employees', () => {
    expect(getActiveNavGroup("/employees")).toBe("org");
  });

  it('returns "org" for /departments', () => {
    expect(getActiveNavGroup("/departments")).toBe("org");
  });

  it('returns "time" for /attendance-logs', () => {
    expect(getActiveNavGroup("/attendance-logs")).toBe("time");
  });

  it('returns "time" for /leave-requests', () => {
    expect(getActiveNavGroup("/leave-requests")).toBe("time");
  });

  it('returns "time" for /approvals', () => {
    expect(getActiveNavGroup("/approvals")).toBe("time");
  });

  it('returns "time" for /approvals/ot', () => {
    expect(getActiveNavGroup("/approvals/ot")).toBe("time");
  });

  it('returns "shifts" for /shifts', () => {
    expect(getActiveNavGroup("/shifts")).toBe("shifts");
  });

  it('returns "shifts" for /shifts/rules', () => {
    expect(getActiveNavGroup("/shifts/rules")).toBe("shifts");
  });

  it('returns "payroll" for /payroll', () => {
    expect(getActiveNavGroup("/payroll")).toBe("payroll");
  });

  it('returns "payroll" for /documents', () => {
    expect(getActiveNavGroup("/documents")).toBe("payroll");
  });

  it('returns "settings" for /workspace/settings', () => {
    expect(getActiveNavGroup("/workspace/settings")).toBe("settings");
  });

  it('returns "settings" for /workspace/settings sub-pages', () => {
    expect(getActiveNavGroup("/workspace/settings/general")).toBe("settings");
    expect(getActiveNavGroup("/workspace/settings/time")).toBe("settings");
  });


  it('returns "system" for /infra/monitor', () => {
    expect(getActiveNavGroup("/infra/monitor")).toBe("system");
  });

  it('returns "system" for /audit-logs', () => {
    expect(getActiveNavGroup("/audit-logs")).toBe("system");
  });

  it('returns "ess" for /ess/profile', () => {
    expect(getActiveNavGroup("/ess/profile")).toBe("ess");
  });

  it("returns null for unknown paths", () => {
    expect(getActiveNavGroup("/nonexistent")).toBeNull();
  });
});

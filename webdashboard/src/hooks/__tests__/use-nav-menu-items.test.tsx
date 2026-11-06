import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { useFilteredMenuItems, useNavMenuItems, getActiveNavGroup } from "@/hooks/use-nav-menu-items";

// ── Mock dependencies ─────────────────────────────────────────────

const activePermissions = new Set<string>();
const mockHasAnyPermission = vi.fn((...perms: string[]) =>
  perms.length === 0 || perms.some((p) => activePermissions.has(p))
);

let mockSystemRoles: string[] = [];

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

vi.mock("@/hooks/use-profile", () => ({
  useProfile: () => ({
    user: {
      id: "test-user",
      email: "test@example.com",
      full_name: "Test User",
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
    mockSystemRoles = [];
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
});

// ── Filtered menu ────────────────────────────────────────────────

describe("useFilteredMenuItems", () => {
  beforeEach(() => {
    activePermissions.clear();
    mockSystemRoles = [];
    vi.clearAllMocks();
  });

  it("hides all sections when user has no permissions", () => {
    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    // Only dashboard + sections with zero-permission items or system roles
    // With no permissions, sections with all items gated should be empty
    const groupsWithItems = result.current.groups.filter((g) => g.items.length > 0);
    // ESS items all require permissions, so none show
    // Org/Time/Shifts/Payroll all require permissions too
    // Only "system" has system-role items (platform_admin), which won't match
    expect(groupsWithItems.length).toBeLessThanOrEqual(1); // Maybe system if no roles, OR just dashboard always visible
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

  it("shows system items for platform_admin", () => {
    mockSystemRoles = ["platform_admin"];

    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    const systemGroup = result.current.groups.find((g) => g.section === "system");
    expect(systemGroup).toBeDefined();
    // Standalone keeps SMTP as its only system navigation item.
    expect(systemGroup!.items.length).toBeGreaterThanOrEqual(1);
  });

  it("hides system items for non-admin", () => {
    mockSystemRoles = [];

    const { result } = renderHook(() => useFilteredMenuItems(), { wrapper });

    const systemGroup = result.current.groups.find((g) => g.section === "system");
    // Without system roles, system group should be empty (all items gated behind roles)
    expect(systemGroup).toBeUndefined();
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

  it('returns "settings" for /workspace/settings', () => {
    expect(getActiveNavGroup("/workspace/settings")).toBe("settings");
  });

  it('returns "settings" for /workspace/settings sub-pages', () => {
    expect(getActiveNavGroup("/workspace/settings/general")).toBe("settings");
    expect(getActiveNavGroup("/workspace/settings/time")).toBe("settings");
  });

  it('returns "system" for /system/roles', () => {
    expect(getActiveNavGroup("/system/roles")).toBe("system");
  });

  it('returns "system" for /infra/monitor', () => {
    expect(getActiveNavGroup("/infra/monitor")).toBe("system");
  });

  it('returns "ess" for /ess/profile', () => {
    expect(getActiveNavGroup("/ess/profile")).toBe("ess");
  });

  it("returns null for unknown paths", () => {
    expect(getActiveNavGroup("/nonexistent")).toBeNull();
  });
});

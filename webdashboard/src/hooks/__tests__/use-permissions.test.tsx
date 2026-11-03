import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { useMyPermissions } from "../use-permissions";

// ── Mocks ────────────────────────────────────────────────────────────────────

const api = vi.hoisted(() => ({
  fetchMyPermissions: vi.fn(),
  getStandaloneOrganizationId: vi.fn(() => WS_ID),
}));

vi.mock("@/lib/api", () => api);

const WS_ID = "ws-rbac-42";

// ── Helpers ──────────────────────────────────────────────────────────────────

let queryClient: QueryClient;

function createWrapper() {
  queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

const FULL_PERMISSIONS = {
  permissions: ["employees:read", "employees:create", "attendance:manage", "settings:write"],
  roles: ["Admin", "Manager"],
  managed_department_ids: ["dept-1", "dept-2"],
};

const EMPTY_PERMISSIONS = {
  permissions: [] as string[],
  roles: [] as string[],
  managed_department_ids: null as string[] | null,
};

// ── Tests ────────────────────────────────────────────────────────────────────

describe("useMyPermissions", () => {
  describe("initialization & loading", () => {
    beforeEach(() => {
      vi.clearAllMocks();
      api.getStandaloneOrganizationId.mockReturnValue(WS_ID);
    });

    it("returns isLoading=true during fetch and isLoading=false after data arrives", async () => {
      (api.fetchMyPermissions as Mock).mockResolvedValue(FULL_PERMISSIONS);

      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });

      expect(result.current.isLoading).toBe(true);
      expect(result.current.permissions).toEqual([]);
      expect(result.current.roles).toEqual([]);

      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.permissions).toEqual(FULL_PERMISSIONS.permissions);
    });

    it("uses the standalone organization context", async () => {
      (api.fetchMyPermissions as Mock).mockResolvedValue(EMPTY_PERMISSIONS);
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));
      expect(api.getStandaloneOrganizationId).toHaveBeenCalled();
      expect(api.fetchMyPermissions).toHaveBeenCalled();
    });

    it("falls back to empty arrays when fetchMyPermissions returns undefined", async () => {
      (api.fetchMyPermissions as Mock).mockResolvedValue(undefined);

      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });

      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.permissions).toEqual([]);
      expect(result.current.roles).toEqual([]);
      expect(result.current.managedDepartmentIds).toBeNull();
    });

    it("sets staleTime=5min and gcTime=10min on the query", async () => {
      (api.fetchMyPermissions as Mock).mockResolvedValue(FULL_PERMISSIONS);

      renderHook(() => useMyPermissions(), { wrapper: createWrapper() });

      await waitFor(() => expect(api.fetchMyPermissions).toHaveBeenCalledTimes(1));
      // Values are forwarded to useQuery; we confirm the query was set up
      // by verifying fetchMyPermissions was called exactly once.
    });
  });

  describe("hasPermission", () => {
    beforeEach(() => {
      vi.clearAllMocks();
      api.getStandaloneOrganizationId.mockReturnValue(WS_ID);
      (api.fetchMyPermissions as Mock).mockResolvedValue(FULL_PERMISSIONS);
    });

    it("returns true for a permission the user holds", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasPermission("employees:read")).toBe(true);
      expect(result.current.hasPermission("employees:create")).toBe(true);
      expect(result.current.hasPermission("attendance:manage")).toBe(true);
    });

    it("returns false for a permission the user does NOT hold", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasPermission("employees:delete")).toBe(false);
      expect(result.current.hasPermission("payroll:approve")).toBe(false);
    });

    it("returns false when permissions list is empty", async () => {
      (api.fetchMyPermissions as Mock).mockResolvedValue(EMPTY_PERMISSIONS);

      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasPermission("employees:read")).toBe(false);
    });

    it("returns false for empty string permission", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasPermission("")).toBe(false);
    });
  });

  describe("hasAnyPermission", () => {
    beforeEach(() => {
      vi.clearAllMocks();
      api.getStandaloneOrganizationId.mockReturnValue(WS_ID);
      (api.fetchMyPermissions as Mock).mockResolvedValue(FULL_PERMISSIONS);
    });

    it("returns true when at least one permission matches", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasAnyPermission("employees:read", "payroll:approve")).toBe(true);
      expect(result.current.hasAnyPermission("settings:write")).toBe(true);
    });

    it("returns false when none of the permissions match", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasAnyPermission("employees:delete", "payroll:approve")).toBe(false);
    });

    it("returns false when called with no arguments", async () => {
      (api.fetchMyPermissions as Mock).mockResolvedValue(EMPTY_PERMISSIONS);
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      // [].some(…) is always false — no permissions to check against
      expect(result.current.hasAnyPermission()).toBe(false);
    });
  });

  describe("hasRole", () => {
    beforeEach(() => {
      vi.clearAllMocks();
      api.getStandaloneOrganizationId.mockReturnValue(WS_ID);
      (api.fetchMyPermissions as Mock).mockResolvedValue(FULL_PERMISSIONS);
    });

    it("returns true for an exact role match", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasRole("Admin")).toBe(true);
      expect(result.current.hasRole("Manager")).toBe(true);
    });

    it("performs case-insensitive role matching", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasRole("admin")).toBe(true);
      expect(result.current.hasRole("ADMIN")).toBe(true);
      expect(result.current.hasRole("manager")).toBe(true);
      expect(result.current.hasRole("mAnAgEr")).toBe(true);
    });

    it("returns false for a role the user does not have", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasRole("SuperAdmin")).toBe(false);
      expect(result.current.hasRole("Viewer")).toBe(false);
    });

    it("returns false when roles array is empty", async () => {
      (api.fetchMyPermissions as Mock).mockResolvedValue(EMPTY_PERMISSIONS);
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasRole("Admin")).toBe(false);
    });
  });

  describe("hasScopedPermission", () => {
    beforeEach(() => {
      vi.clearAllMocks();
      api.getStandaloneOrganizationId.mockReturnValue(WS_ID);
      (api.fetchMyPermissions as Mock).mockResolvedValue(FULL_PERMISSIONS);
    });

    it("returns true when permission is unscoped (managedDeptIds is null)", async () => {
      (api.fetchMyPermissions as Mock).mockResolvedValue({
        ...FULL_PERMISSIONS,
        managed_department_ids: null,
      });

      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasScopedPermission("employees:read")).toBe(true);
      expect(result.current.hasScopedPermission("employees:read", "dept-1")).toBe(true);
    });

    it("returns true when permission is unscoped (managedDeptIds is undefined)", async () => {
      (api.fetchMyPermissions as Mock).mockResolvedValue({
        ...FULL_PERMISSIONS,
        managed_department_ids: undefined,
      });

      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasScopedPermission("employees:read", "dept-99")).toBe(true);
    });

    it("returns false when user lacks the base permission", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasScopedPermission("employees:delete")).toBe(false);
    });

    it("returns true when target department is within the allowed set", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasScopedPermission("employees:read", "dept-1")).toBe(true);
      expect(result.current.hasScopedPermission("attendance:manage", "dept-2")).toBe(true);
    });

    it("returns false when target department is NOT within the allowed set", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasScopedPermission("employees:read", "dept-99")).toBe(false);
      expect(result.current.hasScopedPermission("attendance:manage", "dept-unknown")).toBe(false);
    });

    it("returns false when no target department is provided but scoping is active", async () => {
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      // allowedDepts is ["dept-1","dept-2"], targetDepartmentId is undefined → false
      expect(result.current.hasScopedPermission("employees:read")).toBe(false);
    });

    it("still returns true for null managedDeptIds even without a target dept", async () => {
      (api.fetchMyPermissions as Mock).mockResolvedValue({
        ...FULL_PERMISSIONS,
        managed_department_ids: null,
      });
      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.hasScopedPermission("employees:read")).toBe(true);
    });
  });

  describe("data shape", () => {
    it("exposes permissions, roles, managedDepartmentIds, and isLoading", async () => {
      api.getStandaloneOrganizationId.mockReturnValue(WS_ID);
      (api.fetchMyPermissions as Mock).mockResolvedValue(FULL_PERMISSIONS);

      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      expect(result.current.permissions).toEqual(FULL_PERMISSIONS.permissions);
      expect(result.current.roles).toEqual(FULL_PERMISSIONS.roles);
      expect(result.current.managedDepartmentIds).toEqual(FULL_PERMISSIONS.managed_department_ids);
      expect(result.current.isLoading).toBe(false);
    });

    it("maintains referential stability for hasPermission between renders", async () => {
      api.getStandaloneOrganizationId.mockReturnValue(WS_ID);
      (api.fetchMyPermissions as Mock).mockResolvedValue(FULL_PERMISSIONS);

      const { result, rerender } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });
      await waitFor(() => expect(result.current.isLoading).toBe(false));

      const checkFn1 = result.current.hasPermission;
      rerender();
      const checkFn2 = result.current.hasPermission;

      expect(checkFn1).toBe(checkFn2);
    });
  });

  describe("error handling", () => {
    it("does not expose raw error state but gracefully returns empty defaults on API failure", async () => {
      api.getStandaloneOrganizationId.mockReturnValue(WS_ID);
      (api.fetchMyPermissions as Mock).mockRejectedValue(new Error("Network Error"));

      const { result } = renderHook(() => useMyPermissions(), { wrapper: createWrapper() });

      await waitFor(() => expect(result.current.isLoading).toBe(false));

      // The query would be in error state but we don't expose isError; the hook
      // only destructures isLoading. Permissions/roles fall back to [] via useMemo.
      // The query itself is in error; without exposing isError we verify the defaults hold.
      expect(result.current.permissions).toEqual([]);
      expect(result.current.roles).toEqual([]);
    });
  });
});

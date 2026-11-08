import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { useOrganization } from "../use-organization";
import {
  usePermissionCatalog,
  useOrganizationRoles,
  useRolePermissions,
  useOrganizationPermissionsMatrix,
} from "../use-organization-roles";
import { organizationKeys } from "../use-organization-queries";

const api = vi.hoisted(() => ({
  fetchOrganization: vi.fn(),
  fetchOrganizationRoles: vi.fn(),
  createOrganizationRole: vi.fn(),
  updateOrganizationRole: vi.fn(),
  deleteOrganizationRole: vi.fn(),
  fetchRolePermissions: vi.fn(),
  updateRolePermissions: vi.fn(),
  fetchPermissionCatalog: vi.fn(),
}));

vi.mock("@/lib/api", () => api);

const MOCK_ORG = {
  name: "Acme Corp",
  date_format: "DD/MM/YYYY",
  time_format: "24h",
  timezone: "Asia/Ho_Chi_Minh",
  theme_primary_color: "#10b981",
  default_language: "vi",
  holiday_overnight_strategy: "split",
  default_holiday_ot_multiplier: "1.5",
  created_at: "2026-01-01T00:00:00Z",
};

const MOCK_ROLES = [
  {
    id: "role-1",
    name: "Admin",
    description: "Full access",
    color: "#ff0000",
    is_system: true,
    scope_type: "global" as const,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "role-2",
    name: "Manager",
    description: "Dept manager",
    color: "#00ff00",
    is_system: false,
    scope_type: "scoped" as const,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

let queryClient: QueryClient;

function createWrapper() {
  queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

describe("useOrganization", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchOrganization as Mock).mockResolvedValue(MOCK_ORG);
  });

  it("fetches singleton organization data without an ID parameter", async () => {
    const { result } = renderHook(() => useOrganization(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchOrganization).toHaveBeenCalled();
    expect(result.current.organization).toEqual(MOCK_ORG);
    expect(result.current.organizationName).toBe("Acme Corp");
    expect(result.current.isError).toBe(false);
  });

  it("handles null response when organization is not found", async () => {
    (api.fetchOrganization as Mock).mockResolvedValue(null);
    const { result } = renderHook(() => useOrganization(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.organization).toBeNull();
    expect(result.current.organizationName).toBe("");
  });
});

describe("useOrganizationRoles", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchOrganizationRoles as Mock).mockResolvedValue(MOCK_ROLES);
  });

  it("fetches roles without organization scope", async () => {
    const { result } = renderHook(() => useOrganizationRoles(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchOrganizationRoles).toHaveBeenCalled();
    expect(result.current.roles).toEqual(MOCK_ROLES);
  });

  it("creates a role and invalidates role cache", async () => {
    (api.createOrganizationRole as Mock).mockResolvedValue({ id: "role-3", name: "Staff" });
    const { result } = renderHook(() => useOrganizationRoles(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.createRole({ name: "Staff", color: "#333333" });
    });

    expect(api.createOrganizationRole).toHaveBeenCalledWith({ name: "Staff", color: "#333333" });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: organizationKeys.roles() });
  });

  it("updates a role and invalidates role cache", async () => {
    (api.updateOrganizationRole as Mock).mockResolvedValue({ id: "role-2", name: "Lead" });
    const { result } = renderHook(() => useOrganizationRoles(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.updateRole({ roleId: "role-2", data: { name: "Lead" } });
    });

    expect(api.updateOrganizationRole).toHaveBeenCalledWith("role-2", { name: "Lead" });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: organizationKeys.roles() });
  });

  it("deletes a role and invalidates role cache", async () => {
    (api.deleteOrganizationRole as Mock).mockResolvedValue(undefined);
    const { result } = renderHook(() => useOrganizationRoles(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.deleteRole("role-2");
    });

    expect(api.deleteOrganizationRole).toHaveBeenCalledWith("role-2");
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: organizationKeys.roles() });
  });
});

describe("useRolePermissions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchRolePermissions as Mock).mockResolvedValue(["employees:read", "employees:create"]);
  });

  it("fetches role permissions", async () => {
    const { result } = renderHook(() => useRolePermissions("role-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchRolePermissions).toHaveBeenCalledWith("role-1");
    expect(result.current.permissions).toEqual(["employees:read", "employees:create"]);
  });

  it("updates permissions and invalidates query cache", async () => {
    (api.updateRolePermissions as Mock).mockResolvedValue(undefined);
    const { result } = renderHook(() => useRolePermissions("role-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.updatePermissions(["employees:read"]);
    });

    expect(api.updateRolePermissions).toHaveBeenCalledWith("role-1", ["employees:read"]);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: organizationKeys.rolePermissions("role-1") });
  });
});

describe("useOrganizationPermissionsMatrix", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchOrganizationRoles as Mock).mockResolvedValue(MOCK_ROLES);
    (api.fetchRolePermissions as Mock).mockImplementation((roleId: string) => {
      if (roleId === "role-1") return Promise.resolve(["employees:read", "employees:create"]);
      if (roleId === "role-2") return Promise.resolve(["employees:read"]);
      return Promise.resolve([]);
    });
  });

  it("builds the matrix keyed by role IDs without tenant scope", async () => {
    const { result } = renderHook(() => useOrganizationPermissionsMatrix(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.matrix).toEqual({
      "role-1": ["employees:read", "employees:create"],
      "role-2": ["employees:read"],
    });
  });
});

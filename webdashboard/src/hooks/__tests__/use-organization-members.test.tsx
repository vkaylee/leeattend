import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import {
  useOrganizationMembers,
  useMemberRoles,
  useMemberPermissions,
} from "../use-organization-members";
import { organizationKeys } from "../use-organization-queries";

const api = vi.hoisted(() => ({
  fetchOrganizationMembers: vi.fn(),
  addOrganizationMember: vi.fn(),
  removeOrganizationMember: vi.fn(),
  updateOrganizationMemberRole: vi.fn(),
  fetchMemberRoles: vi.fn(),
  assignExtraRole: vi.fn(),
  revokeExtraRole: vi.fn(),
  fetchMemberPermissions: vi.fn(),
  assignExtraPermission: vi.fn(),
  revokeExtraPermission: vi.fn(),
  fetchPermissionCatalog: vi.fn(),
}));

vi.mock("@/lib/api", () => api);

const MOCK_MEMBER = {
  user_id: "user-1",
  email: "alice@example.com",
  full_name: "Alice",
  role_id: "role-1",
  avatar_url: null,
  extra_role_ids: [],
};

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

describe("useOrganizationMembers", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchOrganizationMembers as Mock).mockResolvedValue([MOCK_MEMBER]);
  });

  it("fetches organization members", async () => {
    const { result } = renderHook(() => useOrganizationMembers(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchOrganizationMembers).toHaveBeenCalled();
    expect(result.current.members).toHaveLength(1);
    expect(result.current.members[0].email).toBe("alice@example.com");
  });

  it("defaults to an empty list when no members are loaded", () => {
    (api.fetchOrganizationMembers as Mock).mockResolvedValue(undefined);

    const { result } = renderHook(() => useOrganizationMembers(), { wrapper: createWrapper() });

    expect(result.current.members).toEqual([]);
  });

  it("invites a member by email and role and invalidates the members cache", async () => {
    (api.addOrganizationMember as Mock).mockResolvedValue(undefined);
    const { result } = renderHook(() => useOrganizationMembers(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.addMember({ email: "bob@example.com", roleId: "role-1" });
    });

    expect(api.addOrganizationMember).toHaveBeenCalledWith("bob@example.com", "role-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: organizationKeys.members(),
      });
    });
  });

  it("toggles isAdding while the invite mutation is in flight", async () => {
    let resolveAdd: ((value: unknown) => void) | undefined;
    const deferred = new Promise((resolve) => { resolveAdd = resolve; });
    (api.addOrganizationMember as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useOrganizationMembers(), { wrapper: createWrapper() });

    act(() => {
      void result.current.addMember({ email: "bob@example.com", roleId: "role-1" });
    });
    await waitFor(() => expect(result.current.isAdding).toBe(true));

    await act(async () => {
      resolveAdd?.(undefined);
    });
    await waitFor(() => expect(result.current.isAdding).toBe(false));
  });

  it("removes a member and invalidates the members cache", async () => {
    (api.removeOrganizationMember as Mock).mockResolvedValue(undefined);
    const { result } = renderHook(() => useOrganizationMembers(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.removeMember("user-1");
    });

    expect(api.removeOrganizationMember).toHaveBeenCalledWith("user-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: organizationKeys.members(),
      });
    });
  });

  it("toggles isRemoving while the removal mutation is in flight", async () => {
    let resolveRemove: ((value: unknown) => void) | undefined;
    const deferred = new Promise((resolve) => { resolveRemove = resolve; });
    (api.removeOrganizationMember as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useOrganizationMembers(), { wrapper: createWrapper() });

    act(() => {
      void result.current.removeMember("user-1");
    });
    await waitFor(() => expect(result.current.isRemoving).toBe(true));

    await act(async () => {
      resolveRemove?.(undefined);
    });
    await waitFor(() => expect(result.current.isRemoving).toBe(false));
  });
});

describe("useMemberRoles", () => {
  const MOCK_ROLES = {
    primary: "role-1",
    extra: [{ id: "extra-1", role_id: "role-2", user_id: "user-1", expires_at: "2026-12-31" }],
  };

  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchMemberRoles as Mock).mockResolvedValue(MOCK_ROLES);
  });

  it("fetches a member's roles", async () => {
    const { result } = renderHook(() => useMemberRoles("user-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchMemberRoles).toHaveBeenCalledWith("user-1");
    expect(result.current.roles).toEqual(MOCK_ROLES);
  });

  it("leaves roles undefined when the query is disabled", () => {
    const { result } = renderHook(() => useMemberRoles(""), { wrapper: createWrapper() });

    expect(result.current.roles).toBeUndefined();
    expect(api.fetchMemberRoles).not.toHaveBeenCalled();
  });

  it("updates the primary role and invalidates roles plus members", async () => {
    (api.updateOrganizationMemberRole as Mock).mockResolvedValue(undefined);
    const { result } = renderHook(() => useMemberRoles("user-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.updatePrimaryRole("role-2");
    });

    expect(api.updateOrganizationMemberRole).toHaveBeenCalledWith("user-1", "role-2");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: organizationKeys.memberRoles("user-1"),
      });
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: organizationKeys.members(),
      });
    });
  });

  it("assigns an extra role with an expiry and invalidates roles", async () => {
    (api.assignExtraRole as Mock).mockResolvedValue({ id: "extra-new" });
    const { result } = renderHook(() => useMemberRoles("user-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.assignRole({ role: "role-2", expiresAt: "2026-12-31" });
    });

    expect(api.assignExtraRole).toHaveBeenCalledWith("user-1", "role-2", "2026-12-31");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: organizationKeys.memberRoles("user-1"),
      });
    });
  });

  it("revokes an extra role and invalidates roles", async () => {
    (api.revokeExtraRole as Mock).mockResolvedValue(undefined);
    const { result } = renderHook(() => useMemberRoles("user-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.revokeRole("extra-1");
    });

    expect(api.revokeExtraRole).toHaveBeenCalledWith("user-1", "extra-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: organizationKeys.memberRoles("user-1"),
      });
    });
  });
});

describe("useMemberPermissions", () => {
  const MOCK_PERMISSIONS = [
    { id: "perm-1", user_id: "user-1", permission: "employees:create", expires_at: "2026-12-31" },
  ];
  const MOCK_CATALOG = [
    { id: "employees:create", group: "employees", label_key: "permissions.employees.create" },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchMemberPermissions as Mock).mockResolvedValue(MOCK_PERMISSIONS);
    (api.fetchPermissionCatalog as Mock).mockResolvedValue(MOCK_CATALOG);
  });

  it("fetches permissions and catalog together", async () => {
    const { result } = renderHook(() => useMemberPermissions("user-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchMemberPermissions).toHaveBeenCalledWith("user-1");
    expect(api.fetchPermissionCatalog).toHaveBeenCalled();
    expect(result.current.permissions).toEqual(MOCK_PERMISSIONS);
    expect(result.current.catalog).toEqual(MOCK_CATALOG);
  });

  it("assigns an extra permission and invalidates permissions", async () => {
    (api.assignExtraPermission as Mock).mockResolvedValue({ id: "perm-new" });
    const { result } = renderHook(() => useMemberPermissions("user-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.assignPermission({ permission: "employees:delete", expiresAt: "2026-12-31" });
    });

    expect(api.assignExtraPermission).toHaveBeenCalledWith("user-1", "employees:delete", "2026-12-31");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: organizationKeys.memberPermissions("user-1"),
      });
    });
  });

  it("revokes an extra permission and invalidates permissions", async () => {
    (api.revokeExtraPermission as Mock).mockResolvedValue(undefined);
    const { result } = renderHook(() => useMemberPermissions("user-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.revokePermission("perm-1");
    });

    expect(api.revokeExtraPermission).toHaveBeenCalledWith("user-1", "perm-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: organizationKeys.memberPermissions("user-1"),
      });
    });
  });
});

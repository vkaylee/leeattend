import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import {
  useWorkspaceMembers,
  useMemberRoles,
  useMemberPermissions,
} from "../use-workspace-members";
import { workspaceKeys } from "../use-workspace-roles";

const api = vi.hoisted(() => ({
  fetchWorkspaceMembers: vi.fn(),
  addWorkspaceMember: vi.fn(),
  removeWorkspaceMember: vi.fn(),
  updateWorkspaceMemberRole: vi.fn(),
  fetchMemberRoles: vi.fn(),
  assignExtraRole: vi.fn(),
  revokeExtraRole: vi.fn(),
  fetchMemberPermissions: vi.fn(),
  assignExtraPermission: vi.fn(),
  revokeExtraPermission: vi.fn(),
  fetchAvailablePermissions: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "ws-test-123"),
}));

vi.mock("@/lib/api", () => api);

const WS_ID = "ws-test-123";

const MOCK_MEMBER = {
  id: "user-1",
  user_id: "user-1",
  email: "alice@example.com",
  display_name: "Alice",
  role_id: "role-1",
  avatar_url: null,
  joined_at: "2026-01-01T00:00:00Z",
  status: "active",
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

describe("useWorkspaceMembers", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.getCurrentWorkspaceId.mockReturnValue(WS_ID);
    (api.fetchWorkspaceMembers as Mock).mockResolvedValue([MOCK_MEMBER]);
  });

  it("fetches members using the active workspace id from helpers", async () => {
    const { result } = renderHook(() => useWorkspaceMembers(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.getCurrentWorkspaceId).toHaveBeenCalled();
    expect(api.fetchWorkspaceMembers).toHaveBeenCalledWith(WS_ID);
    expect(result.current.members).toHaveLength(1);
    expect(result.current.members[0].email).toBe("alice@example.com");
  });

  it("prefers an explicitly passed workspace id over the helper", async () => {
    const { result } = renderHook(() => useWorkspaceMembers("ws-custom-999"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchWorkspaceMembers).toHaveBeenCalledWith("ws-custom-999");
  });

  it("defaults to an empty list when no members are loaded", () => {
    (api.fetchWorkspaceMembers as Mock).mockResolvedValue(undefined);
    api.getCurrentWorkspaceId.mockReturnValue("");

    const { result } = renderHook(() => useWorkspaceMembers(), { wrapper: createWrapper() });

    expect(result.current.members).toEqual([]);
  });

  it("invites a member by email and role and invalidates the members cache", async () => {
    (api.addWorkspaceMember as Mock).mockResolvedValue({ id: "user-2", email: "bob@example.com" });
    const { result } = renderHook(() => useWorkspaceMembers(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.addMember({ email: "bob@example.com", roleId: "role-1" });
    });

    expect(api.addWorkspaceMember).toHaveBeenCalledWith(WS_ID, "bob@example.com", "role-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: workspaceKeys.members(WS_ID),
      });
    });
  });

  it("toggles isAdding while the invite mutation is in flight", async () => {
    let resolveAdd: ((value: unknown) => void) | undefined;
    const deferred = new Promise((resolve) => { resolveAdd = resolve; });
    (api.addWorkspaceMember as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useWorkspaceMembers(), { wrapper: createWrapper() });

    act(() => {
      result.current.addMember({ email: "bob@example.com", roleId: "role-1" });
    });
    await waitFor(() => expect(result.current.isAdding).toBe(true));

    await act(async () => {
      resolveAdd?.({});
    });
    await waitFor(() => expect(result.current.isAdding).toBe(false));
  });

  it("removes a member and invalidates the members cache", async () => {
    (api.removeWorkspaceMember as Mock).mockResolvedValue(undefined);
    const { result } = renderHook(() => useWorkspaceMembers(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.removeMember("user-1");
    });

    expect(api.removeWorkspaceMember).toHaveBeenCalledWith(WS_ID, "user-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: workspaceKeys.members(WS_ID),
      });
    });
  });

  it("toggles isRemoving while the removal mutation is in flight", async () => {
    let resolveRemove: ((value: unknown) => void) | undefined;
    const deferred = new Promise((resolve) => { resolveRemove = resolve; });
    (api.removeWorkspaceMember as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useWorkspaceMembers(), { wrapper: createWrapper() });

    act(() => {
      result.current.removeMember("user-1");
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
    extra: [{ id: "extra-1", name: "Manager", expires_at: null }],
  };

  beforeEach(() => {
    vi.clearAllMocks();
    api.getCurrentWorkspaceId.mockReturnValue(WS_ID);
    (api.fetchMemberRoles as Mock).mockResolvedValue(MOCK_ROLES);
  });

  it("fetches a member's roles", async () => {
    const { result } = renderHook(() => useMemberRoles("user-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchMemberRoles).toHaveBeenCalledWith(WS_ID, "user-1");
    expect(result.current.roles).toEqual(MOCK_ROLES);
  });

  it("leaves roles undefined when the query is disabled", () => {
    const { result } = renderHook(() => useMemberRoles(""), { wrapper: createWrapper() });

    expect(result.current.roles).toBeUndefined();
    expect(api.fetchMemberRoles).not.toHaveBeenCalled();
  });

  it("updates the primary role and invalidates roles plus members", async () => {
    (api.updateWorkspaceMemberRole as Mock).mockResolvedValue(MOCK_ROLES);
    const { result } = renderHook(() => useMemberRoles("user-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.updatePrimaryRole("role-2");
    });

    expect(api.updateWorkspaceMemberRole).toHaveBeenCalledWith(WS_ID, "user-1", "role-2");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: workspaceKeys.roles(WS_ID, "user-1"),
      });
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: workspaceKeys.members(WS_ID),
      });
    });
  });

  it("assigns an extra role with an expiry and invalidates roles", async () => {
    (api.assignExtraRole as Mock).mockResolvedValue(MOCK_ROLES);
    const { result } = renderHook(() => useMemberRoles("user-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.assignRole({ role: "role-2", expiresAt: "2026-12-31" });
    });

    expect(api.assignExtraRole).toHaveBeenCalledWith(WS_ID, "user-1", "role-2", "2026-12-31");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: workspaceKeys.roles(WS_ID, "user-1"),
      });
    });
  });

  it("revokes an extra role and invalidates roles", async () => {
    (api.revokeExtraRole as Mock).mockResolvedValue(MOCK_ROLES);
    const { result } = renderHook(() => useMemberRoles("user-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.revokeRole("extra-1");
    });

    expect(api.revokeExtraRole).toHaveBeenCalledWith(WS_ID, "user-1", "extra-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: workspaceKeys.roles(WS_ID, "user-1"),
      });
    });
  });
});

describe("useMemberPermissions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.getCurrentWorkspaceId.mockReturnValue(WS_ID);
    (api.fetchMemberPermissions as Mock).mockResolvedValue([
      { permission: "attendance.view", expires_at: null },
    ]);
    (api.fetchAvailablePermissions as Mock).mockResolvedValue([
      { id: "perm-1", name: "View Attendance" },
    ]);
  });

  it("fetches member and available permissions", async () => {
    const { result } = renderHook(() => useMemberPermissions("user-1"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchMemberPermissions).toHaveBeenCalledWith(WS_ID, "user-1");
    expect(api.fetchAvailablePermissions).toHaveBeenCalledWith(WS_ID);
    expect(result.current.permissions).toHaveLength(1);
    expect(result.current.available).toHaveLength(1);
  });

  it("defaults permissions and available lists to empty arrays", () => {
    (api.fetchMemberPermissions as Mock).mockResolvedValue(undefined);
    (api.fetchAvailablePermissions as Mock).mockResolvedValue(undefined);

    const { result } = renderHook(() => useMemberPermissions("user-1"), {
      wrapper: createWrapper(),
    });

    expect(result.current.permissions).toEqual([]);
    expect(result.current.available).toEqual([]);
  });

  it("assigns an extra permission and invalidates the permissions cache", async () => {
    (api.assignExtraPermission as Mock).mockResolvedValue({ id: "perm-assign" });
    const { result } = renderHook(() => useMemberPermissions("user-1"), {
      wrapper: createWrapper(),
    });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.assignPermission({ permission: "attendance.edit", expiresAt: "2026-12-31" });
    });

    expect(api.assignExtraPermission).toHaveBeenCalledWith(
      WS_ID,
      "user-1",
      "attendance.edit",
      "2026-12-31",
    );
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: workspaceKeys.permissions(WS_ID, "user-1"),
      });
    });
  });

  it("toggles isAssigning while the permission assignment is in flight", async () => {
    let resolveAssign: ((value: unknown) => void) | undefined;
    const deferred = new Promise((resolve) => { resolveAssign = resolve; });
    (api.assignExtraPermission as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useMemberPermissions("user-1"), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.assignPermission({ permission: "attendance.edit", expiresAt: null });
    });
    await waitFor(() => expect(result.current.isAssigning).toBe(true));

    await act(async () => {
      resolveAssign?.({});
    });
    await waitFor(() => expect(result.current.isAssigning).toBe(false));
  });

  it("revokes a permission and invalidates the permissions cache", async () => {
    (api.revokeExtraPermission as Mock).mockResolvedValue(undefined);
    const { result } = renderHook(() => useMemberPermissions("user-1"), {
      wrapper: createWrapper(),
    });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.revokePermission("perm-assign");
    });

    expect(api.revokeExtraPermission).toHaveBeenCalledWith(WS_ID, "user-1", "perm-assign");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: workspaceKeys.permissions(WS_ID, "user-1"),
      });
    });
  });

  it("toggles isRevoking while the permission revocation is in flight", async () => {
    let resolveRevoke: ((value: unknown) => void) | undefined;
    const deferred = new Promise((resolve) => { resolveRevoke = resolve; });
    (api.revokeExtraPermission as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useMemberPermissions("user-1"), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.revokePermission("perm-assign");
    });
    await waitFor(() => expect(result.current.isRevoking).toBe(true));

    await act(async () => {
      resolveRevoke?.(undefined);
    });
    await waitFor(() => expect(result.current.isRevoking).toBe(false));
  });
});

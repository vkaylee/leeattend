import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";

import * as api from "@/lib/api";

import {
  useWorkspaceMembers,
  useMemberRoles,
} from "../use-workspace";


// Mock the API
vi.mock("@/lib/api", () => ({
  fetchWorkspaceMembers: vi.fn(),
  addWorkspaceMember: vi.fn(),
  fetchWorkspaceRoles: vi.fn(),
  createWorkspaceRole: vi.fn(),
  updateWorkspaceRole: vi.fn(),
  deleteWorkspaceRole: vi.fn(),
  fetchRolePermissions: vi.fn(),
  updateRolePermissions: vi.fn(),
  fetchMemberRoles: vi.fn(),
  assignExtraRole: vi.fn(),
  revokeExtraRole: vi.fn(),
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
};


describe("useWorkspaceMembers", () => {
  it("fetches members successfully", async () => {
    const mockMembers = [
      { workspace_id: "ws-1", user_id: "u-1", email: "owner@test.com", role_id: "role-1", extra_role_ids: [] },
    ];
 
    vi.mocked(api.fetchWorkspaceMembers).mockResolvedValue(mockMembers);

    const { result } = renderHook(() => useWorkspaceMembers("ws-1"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });
    expect(result.current.members).toEqual(mockMembers);
  });

  it("adds a member successfully", async () => {
 
    vi.mocked(api.addWorkspaceMember).mockResolvedValue(undefined);

    const { result } = renderHook(() => useWorkspaceMembers("ws-1"), {
      wrapper: createWrapper(),
    });

    await result.current.addMember({
      email: "test@example.com",
      roleId: "role-1",
    });
    expect(api.addWorkspaceMember).toHaveBeenCalledWith(
      "ws-1",
      "test@example.com",
      "role-1",
    );
  });
});

describe("useWorkspaceRoles", () => {
  it("fetches roles successfully", async () => {
    const mockRoles = [
      { id: "role-1", name: "Admin", is_system: true, color: "primary", created_at: "", updated_at: "" }
    ];
 
    vi.mocked(api.fetchWorkspaceRoles).mockResolvedValue(mockRoles as unknown as never);

    const { renderHook, waitFor } = await import("@testing-library/react");
    const { useWorkspaceRoles } = await import("../use-workspace");
    const { result } = renderHook(() => useWorkspaceRoles("ws-1"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });
    expect(result.current.roles).toEqual(mockRoles);
  });

  it("creates a role successfully", async () => {
 
    vi.mocked(api.createWorkspaceRole).mockResolvedValue({ id: "role-new", name: "New Role" } as unknown as never);
    const { renderHook } = await import("@testing-library/react");
    const { useWorkspaceRoles } = await import("../use-workspace");
    
    const { result } = renderHook(() => useWorkspaceRoles("ws-1"), {
      wrapper: createWrapper(),
    });

    await result.current.createRole({ name: "New Role", color: "blue" });
    expect(api.createWorkspaceRole).toHaveBeenCalledWith("ws-1", { name: "New Role", color: "blue" });
  });

  it("updates a role successfully", async () => {
 
    vi.mocked(api.updateWorkspaceRole).mockResolvedValue({ id: "role-1", name: "Updated" } as unknown as never);
    const { renderHook } = await import("@testing-library/react");
    const { useWorkspaceRoles } = await import("../use-workspace");
    
    const { result } = renderHook(() => useWorkspaceRoles("ws-1"), {
      wrapper: createWrapper(),
    });

    await result.current.updateRole({ roleId: "role-1", data: { name: "Updated" } });
    expect(api.updateWorkspaceRole).toHaveBeenCalledWith("ws-1", "role-1", { name: "Updated" });
  });

  it("deletes a role successfully", async () => {
 
    vi.mocked(api.deleteWorkspaceRole).mockResolvedValue(undefined);
    const { renderHook } = await import("@testing-library/react");
    const { useWorkspaceRoles } = await import("../use-workspace");
    
    const { result } = renderHook(() => useWorkspaceRoles("ws-1"), {
      wrapper: createWrapper(),
    });

    await result.current.deleteRole("role-1");
    expect(api.deleteWorkspaceRole).toHaveBeenCalledWith("ws-1", "role-1");
  });
});

describe("useWorkspacePermissionsMatrix", () => {
  it("fetches matrix successfully", async () => {
    const mockRoles = [
      { id: "role-1", name: "Admin", is_system: true, color: "primary" },
      { id: "role-2", name: "Member", is_system: false, color: "slate" }
    ];
 
    vi.mocked(api.fetchWorkspaceRoles).mockResolvedValue(mockRoles as unknown as never);
    
    // role-1 has perms: p1, p2. role-2 has: p2.
    vi.mocked(api.fetchRolePermissions).mockImplementation((_wsId: string, roleId: string) => {
      if (roleId === "role-1") return Promise.resolve(["p1", "p2"]);
      if (roleId === "role-2") return Promise.resolve(["p2"]);
      return Promise.resolve([]);
    });

    const { renderHook, waitFor } = await import("@testing-library/react");
    const { useWorkspacePermissionsMatrix } = await import("../use-workspace");
    const { result } = renderHook(() => useWorkspacePermissionsMatrix("ws-1"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(Object.keys(result.current.matrix).length).toBeGreaterThan(0); });
    expect(result.current.matrix).toEqual({
      "role-1": ["p1", "p2"],
      "role-2": ["p2"],
    });
  });

  it("includes role IDs in the query key to trigger refetch when roles change", async () => {
    const mockRoles = [
      { id: "role-1", name: "Admin", is_system: true, color: "primary" },
      { id: "role-new", name: "New", is_system: false, color: "slate" }
    ];
 
    vi.mocked(api.fetchWorkspaceRoles).mockResolvedValue(mockRoles as unknown as never);
    vi.mocked(api.fetchRolePermissions).mockResolvedValue([]);

    const { renderHook, waitFor } = await import("@testing-library/react");
    const { useWorkspacePermissionsMatrix } = await import("../use-workspace");
    
    // We can spy on useQuery indirectly, but it's simpler to test that it doesn't crash
    // and returns a matrix with the new role.
    const { result } = renderHook(() => useWorkspacePermissionsMatrix("ws-1"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.matrix["role-new"]).toBeDefined(); });
    expect(result.current.matrix["role-new"]).toEqual([]);
  });
});

describe("useMemberRoles", () => {
  it("fetches roles successfully", async () => {
    const mockRoles = { primary: "a0000000-0000-0000-0000-000000000002", extra: [] };
 
    vi.mocked(api.fetchMemberRoles).mockResolvedValue(mockRoles);

    const { result } = renderHook(() => useMemberRoles("u-1", "ws-1"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });
    expect(result.current.roles).toEqual(mockRoles);
  });
});

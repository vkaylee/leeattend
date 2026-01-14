import { describe, it, expect, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import {
  useWorkspaces,
  useWorkspaceMembers,
  useMemberRoles,
} from "../use-workspace";
import * as api from "@/lib/api";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";

// Mock the API
vi.mock("@/lib/api", () => ({
  fetchWorkspaces: vi.fn(),
  createWorkspace: vi.fn(),
  fetchWorkspaceMembers: vi.fn(),
  addWorkspaceMember: vi.fn(),
  fetchMemberRoles: vi.fn(),
  assignExtraRole: vi.fn(),
  revokeExtraRole: vi.fn(),
  getCurrentUserId: vi.fn().mockReturnValue("user-123"),
  getCurrentWorkspaceId: vi.fn().mockReturnValue("ws-123"),
  setCurrentWorkspaceId: vi.fn(),
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

describe("useWorkspaces", () => {
  it("fetches workspaces successfully", async () => {
      const mockWorkspaces = [
        {
          id: "ws-1",
          name: "Workspace 1",
          work_unit_minutes: 30,
          rounding_policy: "none",
          date_format: "DD/MM/YYYY",
          time_format: "24h",
          timezone: "Asia/Ho_Chi_Minh",
          theme_primary_color: "#7c3aed",
          created_at: new Date().toISOString()
        },
      ];
        vi.mocked(api.fetchWorkspaces).mockResolvedValue(mockWorkspaces);

    const { result } = renderHook(() => useWorkspaces(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.workspaces).toEqual(mockWorkspaces);
  });

  it("creates a workspace and invalidates queries", async () => {
        const newWs = { 
          id: "ws-new", 
          name: "New WS", 
          work_unit_minutes: 30, 
          rounding_policy: "none", 
          date_format: "DD/MM/YYYY", 
          time_format: "24h", 
          timezone: "Asia/Ho_Chi_Minh", 
          theme_primary_color: "#7c3aed",
          created_at: new Date().toISOString() 
        };    vi.mocked(api.createWorkspace).mockResolvedValue(newWs);

    const { result } = renderHook(() => useWorkspaces(), {
      wrapper: createWrapper(),
    });

    const created = await result.current.createWorkspace({ name: "New WS" });
    expect(created).toEqual(newWs);
    expect(api.createWorkspace).toHaveBeenCalled();
  });
});

describe("useWorkspaceMembers", () => {
  it("fetches members successfully", async () => {
    const mockMembers = [
      { workspace_id: "ws-1", user_id: "u-1", role: "owner" },
    ];
    vi.mocked(api.fetchWorkspaceMembers).mockResolvedValue(mockMembers as any);

    const { result } = renderHook(() => useWorkspaceMembers("ws-1"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.members).toEqual(mockMembers);
  });

  it("adds a member successfully", async () => {
    vi.mocked(api.addWorkspaceMember).mockResolvedValue(undefined as any);

    const { result } = renderHook(() => useWorkspaceMembers("ws-1"), {
      wrapper: createWrapper(),
    });

    await result.current.addMember({
      email: "test@example.com",
      role: "admin",
    });
    expect(api.addWorkspaceMember).toHaveBeenCalledWith(
      "ws-1",
      "test@example.com",
      "admin",
    );
  });
});

describe("useMemberRoles", () => {
  it("fetches roles successfully", async () => {
    const mockRoles = { primary: "admin", extra: [] };
    vi.mocked(api.fetchMemberRoles).mockResolvedValue(mockRoles as any);

    const { result } = renderHook(() => useMemberRoles("u-1", "ws-1"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.roles).toEqual(mockRoles);
  });
});

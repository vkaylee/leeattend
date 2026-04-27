import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import WorkspaceOverviewPage from "../workspace/WorkspaceOverviewPage";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "../../lib/api";
import React from "react";

vi.mock("../../lib/api", () => ({
  getCurrentWorkspaceId: vi.fn(),
  apiFetch: vi.fn(),
  API_BASE_URL: "http://localhost:3000/api",
}));

vi.mock("../../hooks/use-workspace", () => ({
  useWorkspace: vi.fn(),
  useWorkspaceMembers: vi.fn(),
  useWorkspaceRoles: vi.fn(),
  useWorkspacePermissionsMatrix: vi.fn(),
}));

import { useWorkspace, useWorkspaceMembers, useWorkspaceRoles } from "../../hooks/use-workspace";

describe("WorkspaceOverviewPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    
    (useWorkspace as any).mockReturnValue({
      workspace: {
        id: "test-ws-id",
        name: "Test Workspace",
        timezone: "Asia/Ho_Chi_Minh",
        created_at: "2024-01-01T00:00:00Z",
      },
      isLoading: false,
    });
    
    (useWorkspaceMembers as any).mockReturnValue({
      members: [
        { workspace_id: "ws-1", user_id: "user-1", email: "owner@test.com", full_name: "Owner User", role_id: "role-1", extra_role_ids: [] },
        { workspace_id: "ws-1", user_id: "user-2", email: "member@test.com", full_name: "Member User", role_id: "role-2", extra_role_ids: [] },
      ],
      isLoading: false,
    });
    
    (useWorkspaceRoles as any).mockReturnValue({ roles: [{ id: "role-1", name: "owner", is_system: true }], isLoading: false });
    
    (api.apiFetch as any).mockResolvedValue({
      ok: true,
      json: async () => ({
        total_employees: 42,
        present_today: 40,
        late_today: 2,
        on_leave_today: 1,
      }),
    });
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        {ui}
      </QueryClientProvider>
    );
  };

  it("renders workspace stats correctly", async () => {
    renderWithProviders(<WorkspaceOverviewPage />);

    // Check workspace name and timezone
    expect(await screen.findByText("Test Workspace")).toBeDefined();
    expect(await screen.findByText(/Asia\/Ho_Chi_Minh/)).toBeDefined();

    // Check stats numbers
    expect(await screen.findByText("42")).toBeDefined(); // total_employees
    expect(await screen.findByText("40")).toBeDefined(); // present_today
    expect(await screen.findByText("2")).toBeDefined(); // late_today
    expect(await screen.findByText("1")).toBeDefined(); // on_leave_today

    // Check admin members (should only render owner/admin)
    expect(screen.getByText("Owner User")).toBeDefined();
    expect(screen.queryByText("Member User")).toBeNull(); // members shouldn't appear in admin preview
  });
});

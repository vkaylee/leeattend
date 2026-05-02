import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { useWorkspace, useWorkspaceMembers, useWorkspaceRoles } from "../../hooks/use-workspace";
import * as api from "../../lib/api";
import WorkspaceOverviewPage from "../workspace/WorkspaceOverviewPage";

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

 
describe("WorkspaceOverviewPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("test-ws-id");
    
 
    vi.mocked(useWorkspace).mockReturnValue({
      workspace: {
 
        id: "test-ws-id",
        name: "Test Workspace",
        timezone: "Asia/Ho_Chi_Minh",
        created_at: "2024-01-01T00:00:00Z",
        work_unit_minutes: 0,
        rounding_policy: "",
        date_format: "",
        time_format: "",
        theme_primary_color: "",
        default_language: "",
 
      },
      isLoading: false,
      isError: false,
    });
    
    vi.mocked(useWorkspaceMembers).mockReturnValue({
      members: [
        { workspace_id: "ws-1", user_id: "user-1", email: "owner@test.com", full_name: "Owner User", role_id: "role-1", extra_role_ids: [] },
 
        { workspace_id: "ws-1", user_id: "user-2", email: "member@test.com", full_name: "Member User", role_id: "role-2", extra_role_ids: [] },
      ],
 
      isLoading: false,
      isAdding: false,
      isRemoving: false,
      addMember: vi.fn(),
      removeMember: vi.fn(),
    });
    
    vi.mocked(useWorkspaceRoles).mockReturnValue({ 
      roles: [
        { id: 'role-1', name: 'Admin', is_system: true, color: 'indigo' },
        { id: 'role-2', name: 'Member', is_system: false, color: 'slate' },
      ] as unknown as Record<string, unknown>[],
      isLoading: false,
      isCreating: false,
      isUpdating: false,
      isDeleting: false,
      createRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
      updateRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
      deleteRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
    } as unknown as ReturnType<typeof useWorkspaceRoles>);
    
    vi.mocked(api.apiFetch).mockResolvedValue({
      ok: true,
      json: async () => ({
        total_employees: 42,
        present_today: 40,
        late_today: 2,
        on_leave_today: 1,
      }),
    } as Response);
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

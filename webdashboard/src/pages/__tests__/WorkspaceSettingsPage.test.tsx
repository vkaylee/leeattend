import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import WorkspaceSettingsPage from "../WorkspaceSettingsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "../../lib/api";
import { useWorkspaceMembers } from "@/hooks/use-workspace";
import { ToastProvider } from "../../components/Toast";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchWorkspace: vi.fn(),
  updateWorkspace: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
}));

// Mock the hook
vi.mock("@/hooks/use-workspace", () => ({
  useWorkspaceMembers: vi.fn(),
  useWorkspace: vi.fn(() => ({
    workspace: {
      date_format: "DD/MM/YYYY",
      time_format: "24h",
    },
    isLoading: false,
  })),
  useMemberRoles: vi.fn(() => ({
    roles: { primary: "member", extra: [] },
    isLoading: false,
  })),
}));

describe("WorkspaceSettingsPage", () => {
  let queryClient: QueryClient;
  const mockAddMember = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
        },
      },
    });
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    (api.fetchWorkspace as any).mockResolvedValue({
      id: "test-ws-id",
      name: "Test Workspace",
      work_unit_minutes: 15,
      rounding_policy: "floor",
      date_format: "DD/MM/YYYY",
    });
    (useWorkspaceMembers as any).mockReturnValue({
      members: [
        { user_id: "u1", full_name: "Owner User", email: "owner@test.com", role: "owner" },
        { user_id: "u2", full_name: "Admin User", email: "admin@test.com", role: "admin" },
      ],
      isLoading: false,
      addMember: mockAddMember,
      isAdding: false,
    });
  });

  it("renders correctly and loads workspace config", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <WorkspaceSettingsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    expect(screen.getByText("Cài đặt Workspace")).toBeDefined();
    
    // Wait for data to load into selects
    await waitFor(() => {
      const workUnitSelect = screen.getByTestId("work-unit-select") as HTMLSelectElement;
      expect(workUnitSelect.value).toBe("15");
    });

    expect((screen.getByTestId("rounding-policy-select") as HTMLSelectElement).value).toBe("floor");
  });

  it("allows changing and saving configuration", async () => {
    const user = userEvent.setup();
    (api.updateWorkspace as any).mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <WorkspaceSettingsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    const workUnitSelect = await screen.findByTestId("work-unit-select");
    await user.selectOptions(workUnitSelect, "30");

    const roundingSelect = screen.getByTestId("rounding-policy-select");
    await user.selectOptions(roundingSelect, "ceil");

    const saveBtn = screen.getByText("Lưu cấu hình");
    await user.click(saveBtn);

    await waitFor(() => {
      expect(api.updateWorkspace).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
        work_unit_minutes: 30,
        rounding_policy: "ceil"
      }));
    });
  });

  it("handles inviting new members", async () => {
    const user = userEvent.setup();
    mockAddMember.mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <WorkspaceSettingsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    const emailInput = screen.getByPlaceholderText("dong-nghiep@congty.com");
    await user.type(emailInput, "new@test.com");

    const roleSelect = screen.getByLabelText("Vai trò");
    await user.selectOptions(roleSelect, "admin");

    const inviteBtn = screen.getByRole("button", { name: "Gửi lời mời" });
    await user.click(inviteBtn);

    await waitFor(() => {
      expect(mockAddMember).toHaveBeenCalledWith({
        email: "new@test.com",
        role: "admin"
      });
    });
    
    // Check if input was cleared
    expect((emailInput as HTMLInputElement).value).toBe("");
  });

  it("opens role management modal", async () => {
    const user = userEvent.setup();
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <WorkspaceSettingsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Find the settings button for the second member (u2)
    const settingsBtns = await screen.findAllByTitle("Quản lý quyền");
    await user.click(settingsBtns[1]);

    // Should see the modal title from RoleManagementModal
    expect(await screen.findByText("Quản lý quyền hạn")).toBeDefined();
  });
});
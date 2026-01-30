import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import RoleManagementModal from "../features/settings/RoleManagementModal";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../ui/Toast";
import * as api from "../../lib/api";
import React from "react";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchMemberRoles: vi.fn(),
  updateWorkspaceMemberRole: vi.fn(),
  assignExtraRole: vi.fn(),
  revokeExtraRole: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
}));

// Mock window.confirm
vi.spyOn(window, 'confirm').mockReturnValue(true);

describe("RoleManagementModal", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
    });
    (api.fetchMemberRoles as any).mockResolvedValue({
      primary: "member",
      extra: []
    });
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {ui}
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders correctly when open", async () => {
    renderWithProviders(<RoleManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);
    expect(await screen.findByText(/Quản lý quyền hạn/i)).toBeDefined();
    // In Premium UI, labels are uppercased in the component
    expect(await screen.findByText(/QUYỀN CHÍNH/i)).toBeDefined();
  });

  it("handles updating primary role", async () => {
    const user = userEvent.setup();
    (api.updateWorkspaceMemberRole as any).mockResolvedValue({});
    renderWithProviders(<RoleManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);
    
    // Find the current role text and click it to open the combobox
    const comboboxTrigger = await screen.findByText(/THÀNH VIÊN/i);
    await user.click(comboboxTrigger);

    // Select Admin option
    const primaryRoleSelect = screen.getByLabelText(/QUYỀN CHÍNH/i);
    fireEvent.change(primaryRoleSelect, { target: { value: 'admin' } });

    await waitFor(() => {
      expect(api.updateWorkspaceMemberRole).toHaveBeenCalledWith("test-ws-id", "user-1", "admin");
    });
  });

  it("handles assigning extra role", async () => {
    const user = userEvent.setup();
    (api.assignExtraRole as any).mockResolvedValue({});
    renderWithProviders(<RoleManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);

    const dateInput = screen.getByLabelText(/Ngày hết hạn/i);
    fireEvent.change(dateInput, { target: { value: "2026-12-31T23:59" } });

    // Use role button to target the assign button specifically
    await user.click(screen.getByRole("button", { name: /GÁN QUYỀN/i }));

    await waitFor(() => {
      expect(api.assignExtraRole).toHaveBeenCalled();
    });
  });

  it("handles revoking extra role", async () => {
    const user = userEvent.setup();
    (api.fetchMemberRoles as any).mockResolvedValue({
      primary: "member",
      extra: [{ id: "role-1", role: "admin", expires_at: "2027-01-01T00:00:00Z" }]
    });
    (api.revokeExtraRole as any).mockResolvedValue({});

    renderWithProviders(<RoleManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);

    // In Premium UI, role name is rendered in a p tag
    expect(await screen.findByText("admin")).toBeDefined();
    
    // Find the button within the extra role item container
    const extraRoleItem = screen.getByText("admin").closest("div");
    // The button is a sibling of the role name container
    const revokeBtn = extraRoleItem!.parentElement!.querySelector('button');
    await user.click(revokeBtn!);

    await waitFor(() => {
      expect(api.revokeExtraRole).toHaveBeenCalledWith("test-ws-id", "user-1", "role-1");
    });
  });
});
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useSystemRoles } from "@/hooks/use-system-roles";

import SystemRolesPage from "../system-roles-page";

// Mock the hooks
vi.mock("@/hooks/use-system-roles", () => ({
  useSystemRoles: vi.fn(),
}));

vi.mock("@/hooks/use-confirm", () => ({
  useConfirm: vi.fn(),
}));

describe("SystemRolesPage", () => {
  const mockAssignRole = vi.fn();
  const mockRevokeRole = vi.fn();
  const mockConfirm = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    (useSystemRoles as Mock).mockReturnValue({
      roles: [
        {
          id: "role-1",
          name: "platform_admin",
          description: "Full platform access.",
          assignees: [
            { user_id: "user-1", assigned_at: "2024-01-01T00:00:00Z" },
          ],
        },
      ],
      isLoading: false,
      assignRole: mockAssignRole,
      revokeRole: mockRevokeRole,
      isAssigning: false,
      isRevoking: false,
    });

    (useConfirm as Mock).mockReturnValue({
      confirm: mockConfirm,
      ConfirmationDialog: () => <div data-testid="confirmation-dialog"></div>,
    });
  });

  const renderComponent = () => render(
    <ToastProvider>
      <SystemRolesPage />
    </ToastProvider>
  );

  it("renders correctly", () => {
    renderComponent();
    expect(screen.getByText("Quản trị viên Nền tảng")).toBeInTheDocument();
    expect(screen.getByText("platform_admin")).toBeInTheDocument();
    expect(screen.getByText("user-1")).toBeInTheDocument();
  });

  it("opens assign role modal and submits", async () => {
    const user = userEvent.setup();
    renderComponent();

    // Click assign button (title uses vi translation)
    const assignButton = screen.getByTitle("Gán cho người dùng");
    await user.click(assignButton);

    // Modal should be open (with vi translation interpolated)
    expect(screen.getByText("Gán Vai trò platform_admin")).toBeInTheDocument();

    // Fill in user ID
    const input = screen.getByPlaceholderText("e.g. 123e4567-e89b-12d3-a456-426614174000");
    await user.type(input, "new-user-123");

    // Submit (vi translation)
    const submitButton = screen.getByText("Gán Vai trò");
    await user.click(submitButton);

    expect(mockAssignRole).toHaveBeenCalledWith({
      userId: "new-user-123",
      request: { role_name: "platform_admin" },
    });
  });

  it("revokes a role", async () => {
    const user = userEvent.setup();
    mockConfirm.mockResolvedValueOnce(true);

    renderComponent();

    const revokeButton = screen.getByTitle("Thu hồi vai trò");
    await user.click(revokeButton);

    expect(mockConfirm).toHaveBeenCalled();
    await waitFor(() => {
      expect(mockRevokeRole).toHaveBeenCalledWith({
        userId: "user-1",
        roleName: "platform_admin",
      });
    });
  });
});

import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import SystemRolesPage from "../SystemRolesPage";
import React from "react";
import userEvent from "@testing-library/user-event";

// Mock the hooks
vi.mock("@/hooks/use-system-roles", () => ({
  useSystemRoles: vi.fn(),
}));

vi.mock("@/hooks/use-confirm", () => ({
  useConfirm: vi.fn(),
}));

import { useSystemRoles } from "@/hooks/use-system-roles";
import { useConfirm } from "@/hooks/use-confirm";

describe("SystemRolesPage", () => {
  const mockAssignRole = vi.fn();
  const mockRevokeRole = vi.fn();
  const mockConfirm = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    (useSystemRoles as any).mockReturnValue({
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

    (useConfirm as any).mockReturnValue({
      confirm: mockConfirm,
      ConfirmationDialog: () => <div data-testid="confirmation-dialog"></div>,
    });
  });

  const renderComponent = () => render(<SystemRolesPage />);

  it("renders correctly", () => {
    renderComponent();
    expect(screen.getByText("Platform Administrators")).toBeInTheDocument();
    expect(screen.getByText("platform_admin")).toBeInTheDocument();
    expect(screen.getByText("user-1")).toBeInTheDocument();
  });

  it("opens assign role modal and submits", async () => {
    const user = userEvent.setup();
    renderComponent();

    // Click assign button
    const assignBtn = screen.getByTitle("Assign to user");
    await user.click(assignBtn);

    // Modal should be open
    expect(screen.getByText("Assign platform_admin Role")).toBeInTheDocument();

    // Fill in user ID
    const input = screen.getByPlaceholderText("e.g. 123e4567-e89b-12d3-a456-426614174000");
    await user.type(input, "new-user-123");

    // Submit
    const submitBtn = screen.getByText("Assign Role");
    await user.click(submitBtn);

    expect(mockAssignRole).toHaveBeenCalledWith({
      userId: "new-user-123",
      request: { role_name: "platform_admin" },
    });
  });

  it("revokes a role", async () => {
    const user = userEvent.setup();
    mockConfirm.mockResolvedValueOnce(true);

    renderComponent();

    const revokeBtn = screen.getByTitle("Revoke role");
    await user.click(revokeBtn);

    expect(mockConfirm).toHaveBeenCalled();
    await waitFor(() => {
      expect(mockRevokeRole).toHaveBeenCalledWith({
        userId: "user-1",
        roleName: "platform_admin",
      });
    });
  });
});

import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../lib/api";
import PermissionManagementModal from "../features/settings/PermissionManagementModal";
import { ToastProvider } from "../ui/Toast";


// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchMemberPermissions: vi.fn(),
  fetchAvailablePermissions: vi.fn(),
  assignExtraPermission: vi.fn(),
  revokeExtraPermission: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
}));

// Mock window.confirm
vi.spyOn(window, 'confirm').mockReturnValue(true);

describe("PermissionManagementModal", () => {
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
 
    vi.mocked(api.fetchAvailablePermissions).mockResolvedValue([
      "members:invite",
      "members:remove",
      "members:roles:manage",
      "settings:read",
    ] as unknown as never);
 
    vi.mocked(api.fetchMemberPermissions).mockResolvedValue([
      { id: "perm-1", permission: "members:invite", expires_at: "2027-01-01T00:00:00Z", assigned_at: "2026-01-01T00:00:00Z" }
    ] as unknown as never);
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
    renderWithProviders(<PermissionManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);
    
    // Check if title is present
    expect(await screen.findByText(/Manage Permissions|Quản lý quyền hạn/i)).toBeDefined();
    
    // Check if the current permission is listed
    expect(await screen.findByText(/members:invite/i)).toBeDefined();
  });

  it("handles revoking a permission", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.revokeExtraPermission).mockResolvedValue({} as never);

    renderWithProviders(<PermissionManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);

    // Wait for data to load
    await screen.findByText(/members:invite/i);

    // Find the revoke button (trash icon). It should be next to our permission
    const _revokeBtns = await screen.findAllByRole("button");
    // The first button might be the close modal button. The revoke one is the second one typically, or has a specific SVG.
    // Let's find it by clicking the one that is inside the same container as the permission text.
    const permText = await screen.findByText("members:invite");
    const container = permText.closest("div")?.parentElement;
    const revokeBtn = container?.querySelector("button");

    expect(revokeBtn).toBeDefined();
    await user.click(revokeBtn!);

    // Confirm the modal
    const confirmModalBtn = await screen.findByRole("button", { name: /Confirm|Xác nhận/i });
    await user.click(confirmModalBtn);

    await waitFor(() => {
      expect(api.revokeExtraPermission).toHaveBeenCalledWith("test-ws-id", "user-1", "perm-1");
    });
  });

  it("filters out existing permissions from search", async () => {
    renderWithProviders(<PermissionManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);
    
    await waitFor(() => {
      expect(api.fetchAvailablePermissions).toHaveBeenCalled();
    });

    const searchInput = screen.getByPlaceholderText(/Search permissions|Tìm kiếm quyền/i);
    fireEvent.change(searchInput, { target: { value: "members:" } });

    // "members:invite" should not be listed as an option to assign, because it's already assigned
    await waitFor(() => {
      expect(screen.queryByRole("button", { name: "members:invite" })).toBeNull();
      // "members:remove" should be available
      expect(screen.getByRole("button", { name: "members:remove" })).toBeDefined();
    });
  });

  it("handles assigning a new permission", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.assignExtraPermission).mockResolvedValue({} as never);

    renderWithProviders(<PermissionManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);

    await waitFor(() => {
      expect(api.fetchAvailablePermissions).toHaveBeenCalled();
    });

    const searchInput = screen.getByPlaceholderText(/Search permissions|Tìm kiếm quyền/i);
    fireEvent.change(searchInput, { target: { value: "settings:read" } });

    // Wait for the option to appear and click it
    const optionBtn = await screen.findByRole("button", { name: "settings:read" });
    await user.click(optionBtn);

    // Set expiration date
    const dateInput = screen.getByLabelText(/Expiration|Ngày hết hạn/i);
    fireEvent.change(dateInput, { target: { value: "2026-12-31T23:59" } });

    // Click assign button
    const assignBtn = screen.getByRole("button", { name: /Grant Permission|CẤP QUYỀN/i });
    await user.click(assignBtn);

    await waitFor(() => {
      expect(api.assignExtraPermission).toHaveBeenCalledWith(
        "test-ws-id",
        "user-1",
        "settings:read",
        expect.any(String) // ISO string
      );
    });
  });

  it("does not render when isOpen is false", () => {
    renderWithProviders(<PermissionManagementModal userId="user-1" isOpen={false} onClose={() => {}} />);
    expect(screen.queryByText(/Manage Permissions|Quản lý quyền hạn/i)).toBeNull();
  });
});

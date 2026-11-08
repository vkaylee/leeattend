import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../lib/api";
import PermissionManagementModal from "../features/settings/permission-management-modal";
import { ToastProvider } from "../ui/toast";


// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchMemberPermissions: vi.fn(),
  fetchPermissionCatalog: vi.fn(),
  assignExtraPermission: vi.fn(),
  revokeExtraPermission: vi.fn(),
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
 
    vi.mocked(api.fetchPermissionCatalog).mockResolvedValue([
      { id: "members:invite", group: "members", label_key: "permissions.items.members:invite.name" },
      { id: "members:remove", group: "members", label_key: "permissions.items.members:remove.name" },
      { id: "members:roles:manage", group: "members", label_key: "permissions.items.members:roles:manage.name" },
      { id: "settings:read", group: "workspace", label_key: "permissions.items.settings:read.name" },
    ]);
 
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
    renderWithProviders(<PermissionManagementModal isOpen={true} onClose={() => { /* empty */ }} userId="user-1" />);
    
    // Check if title is present
    expect(await screen.findByText(/Manage Permissions|Quản lý quyền hạn/i)).toBeDefined();
    
    // Check if the current permission is listed
    expect(await screen.findByText(/members:invite/i)).toBeDefined();
  });

  it("handles revoking a permission", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.revokeExtraPermission).mockResolvedValue({} as never);

    renderWithProviders(<PermissionManagementModal isOpen={true} onClose={() => { /* empty */ }} userId="user-1" />);

    // Wait for data to load
    await screen.findByText(/members:invite/i);

    // Find the revoke button (trash icon). It should be next to our permission
    const _revokeBtns = await screen.findAllByRole("button");
    // The first button might be the close modal button. The revoke one is the second one typically, or has a specific SVG.
    // Let's find it by clicking the one that is inside the same container as the permission text.
    const permText = await screen.findByText("members:invite");
    const container = permText.closest("div")?.parentElement;
    const revokeButton = container?.querySelector("button");

    expect(revokeButton).toBeDefined();
    await user.click(revokeButton!);

    // Confirm the modal
    const confirmModalButton = await screen.findByRole("button", { name: /Confirm|Xác nhận/i });
    await user.click(confirmModalButton);

    await waitFor(() => {
      expect(api.revokeExtraPermission).toHaveBeenCalledWith("user-1", "perm-1");
    });
  });

  it("filters out existing permissions from search", async () => {
    renderWithProviders(<PermissionManagementModal isOpen={true} onClose={() => { /* empty */ }} userId="user-1" />);
    
    await waitFor(() => {
      expect(api.fetchPermissionCatalog).toHaveBeenCalled();
    });

    const searchInput = screen.getByPlaceholderText(/Search permissions|Tìm kiếm quyền/i);
    fireEvent.change(searchInput, { target: { value: "members:" } });

    // "members:invite" should not be listed as an option to assign, because it's already assigned
    await waitFor(() => {
      expect(screen.queryByRole("button", { name: /members:invite/ })).toBeNull();
      // "members:remove" should be available
      expect(screen.getByRole("button", { name: /members:remove/ })).toBeDefined();
    });
  });

  it("handles assigning a new permission", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.assignExtraPermission).mockResolvedValue({} as never);

    renderWithProviders(<PermissionManagementModal isOpen={true} onClose={() => { /* empty */ }} userId="user-1" />);

    await waitFor(() => {
      expect(api.fetchPermissionCatalog).toHaveBeenCalled();
    });

    const searchInput = screen.getByPlaceholderText(/Search permissions|Tìm kiếm quyền/i);
    fireEvent.change(searchInput, { target: { value: "settings:read" } });

    // Wait for the option to appear and click it
    const optionButton = await screen.findByRole("button", { name: /settings:read/ });
    await user.click(optionButton);

    // Set expiration date
    const dateInput = screen.getByLabelText(/Expiration|Ngày hết hạn/i);
    fireEvent.change(dateInput, { target: { value: "2026-12-31T23:59" } });

    // Click assign button
    const assignButton = screen.getByRole("button", { name: /Grant Permission|CẤP QUYỀN/i });
    await user.click(assignButton);

    await waitFor(() => {
      expect(api.assignExtraPermission).toHaveBeenCalledWith("user-1",
        "settings:read",
        expect.any(String) // ISO string
      );
    });
  });

  it("does not render when isOpen is false", () => {
    renderWithProviders(<PermissionManagementModal isOpen={false} onClose={() => { /* empty */ }} userId="user-1" />);
    expect(screen.queryByText(/Manage Permissions|Quản lý quyền hạn/i)).toBeNull();
  });
});

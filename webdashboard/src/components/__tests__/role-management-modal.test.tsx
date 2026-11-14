import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../lib/api";
import RoleManagementModal from "../features/settings/role-management-modal";
import { ToastProvider } from "../ui/toast";




const OWNER_ROLE_ID = "a0000000-0000-0000-0000-000000000001";
const ADMIN_ROLE_ID = "a0000000-0000-0000-0000-000000000002";
const EMPLOYEE_ROLE_ID = "a0000000-0000-0000-0000-000000000006";
const TEAM_LEAD_ROLE_ID = "a0000000-0000-0000-0000-000000000005";

const mockWorkspaceRoles = [
  { id: OWNER_ROLE_ID, name: "owner", description: "Full access", color: "rose", is_system: true, scope_type: "global", created_at: "", updated_at: "" },
  { id: ADMIN_ROLE_ID, name: "admin", description: "Admin access", color: "rose", is_system: true, scope_type: "global", created_at: "", updated_at: "" },
  { id: TEAM_LEAD_ROLE_ID, name: "team_lead", description: "Team management", color: "blue", is_system: true, scope_type: "scoped", created_at: "", updated_at: "" },
  { id: EMPLOYEE_ROLE_ID, name: "employee", description: "Default access", color: "slate", is_system: true, scope_type: "scoped", created_at: "", updated_at: "" },
];

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchOrganization: vi.fn().mockResolvedValue({ name: "Test Organization" }),
  fetchMemberRoles: vi.fn(),
  updateOrganizationMemberRole: vi.fn(),
  assignExtraRole: vi.fn(),
  revokeExtraRole: vi.fn(),
  fetchOrganizationRoles: vi.fn(),
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
 
    vi.mocked(api.fetchMemberRoles).mockResolvedValue({
      primary: EMPLOYEE_ROLE_ID,
      extra: []
    });
 
    vi.mocked(api.fetchOrganizationRoles).mockResolvedValue(mockWorkspaceRoles);
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
    renderWithProviders(<RoleManagementModal isOpen={true} onClose={() => { /* empty */ }} userId="user-1" />);
    expect(await screen.findByText(/Quản lý quyền hạn/i)).toBeDefined();
    // In Premium UI, labels are uppercased in the component
    expect(await screen.findByText(/QUYỀN CHÍNH/i)).toBeDefined();
  });

  it("loads workspace roles dynamically and displays primary role", async () => {
    renderWithProviders(<RoleManagementModal isOpen={true} onClose={() => { /* empty */ }} userId="user-1" />);

    // Should have called fetchOrganizationRoles
    await waitFor(() => {
      expect(api.fetchOrganizationRoles).toHaveBeenCalledWith();
    });
  });

  it("handles updating primary role with UUID", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.updateOrganizationMemberRole).mockResolvedValue({} as never);
    renderWithProviders(<RoleManagementModal isOpen={true} onClose={() => { /* empty */ }} userId="user-1" />);

    // Wait for roles to load
    await waitFor(() => {
      expect(api.fetchOrganizationRoles).toHaveBeenCalled();
    });

    // Find and interact with the primary role combobox
    const primaryRoleSelect = await screen.findByLabelText(/QUYỀN CHÍNH/i);
    fireEvent.change(primaryRoleSelect, { target: { value: ADMIN_ROLE_ID } });

    // Confirm the modal
    const confirmModalButton = await screen.findByRole("button", { name: /Confirm|Xác nhận/i });
    await user.click(confirmModalButton);

    await waitFor(() => {
      expect(api.updateOrganizationMemberRole).toHaveBeenCalledWith("user-1", ADMIN_ROLE_ID);
    });
  });

  it("handles assigning extra role with UUID", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.assignExtraRole).mockResolvedValue({} as never);
    renderWithProviders(<RoleManagementModal isOpen={true} onClose={() => { /* empty */ }} userId="user-1" />);

    // Wait for roles to load
    await waitFor(() => {
      expect(api.fetchOrganizationRoles).toHaveBeenCalled();
    });

    const extraRoleSelect = screen.getByLabelText(/Vai trò gán/i);

    // Wait for options to populate
    await waitFor(() => {
      expect(extraRoleSelect.children.length).toBeGreaterThan(1);
    });

    await user.selectOptions(extraRoleSelect, ADMIN_ROLE_ID);

    const dateInput = screen.getByLabelText(/Ngày hết hạn/i);
    fireEvent.change(dateInput, { target: { value: "2026-12-31T23:59" } });

    await user.click(screen.getByRole("button", { name: /GÁN QUYỀN/i }));

    await waitFor(() => {
      expect(api.assignExtraRole).toHaveBeenCalled();
    });
  });

  it("handles revoking extra role with role_id", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.fetchMemberRoles).mockResolvedValue({
      primary: EMPLOYEE_ROLE_ID,
      extra: [{ id: "extra-role-1", role_id: ADMIN_ROLE_ID, workspace_id: "test-ws-id", user_id: "user-1", assigned_at: "2026-01-01T00:00:00Z", expires_at: "2027-01-01T00:00:00Z" }]
    });
 
    vi.mocked(api.revokeExtraRole).mockResolvedValue({} as never);

    renderWithProviders(<RoleManagementModal isOpen={true} onClose={() => { /* empty */ }} userId="user-1" />);

    // Should display the resolved role name from workspace roles
    const adminTexts = await screen.findAllByText(/admin/i);
    expect(adminTexts.length).toBeGreaterThan(0);

    // Find the revoke button
    const extraRoleItem = adminTexts[0].closest("div");
    const revokeButton = extraRoleItem!.parentElement!.querySelector('button');
    await user.click(revokeButton!);

    // Confirm the modal
    const confirmModalButton = await screen.findByRole("button", { name: /Confirm|Xác nhận/i });
    await user.click(confirmModalButton);

    await waitFor(() => {
      expect(api.revokeExtraRole).toHaveBeenCalledWith("user-1", "extra-role-1");
    });
  });

  it("filters out primary role and existing extra roles from available options", async () => {
 
    vi.mocked(api.fetchMemberRoles).mockResolvedValue({
      primary: EMPLOYEE_ROLE_ID,
      extra: [{ id: "extra-1", role_id: ADMIN_ROLE_ID, workspace_id: "test-ws-id", user_id: "user-1", assigned_at: "", expires_at: "2027-01-01T00:00:00Z" }]
    });

    renderWithProviders(<RoleManagementModal isOpen={true} onClose={() => { /* empty */ }} userId="user-1" />);

    await waitFor(() => {
      expect(api.fetchOrganizationRoles).toHaveBeenCalled();
    });

    // The extra role select should not contain the primary role (employee) or existing extra (admin)
    const extraRoleSelect = screen.getByLabelText(/Vai trò gán/i);
    
    // Wait for options to populate
    await waitFor(() => {
      expect(extraRoleSelect.children.length).toBeGreaterThan(1);
    });

    const options = extraRoleSelect.querySelectorAll("option");

    const optionValues = Array.from(options).map(o => (o).value);
    expect(optionValues).not.toContain(EMPLOYEE_ROLE_ID);
    expect(optionValues).not.toContain(ADMIN_ROLE_ID);
  });

  it("does not render when isOpen is false", () => {
    renderWithProviders(<RoleManagementModal isOpen={false} onClose={() => { /* empty */ }} userId="user-1" />);
    expect(screen.queryByText(/Quản lý quyền hạn/i)).toBeNull();
  });
});
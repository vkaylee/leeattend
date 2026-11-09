import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import i18n from "../../i18n/config";
import * as api from "../../lib/api";
import MembersPage from "../members-page";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
 
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchOrganizationMembers: vi.fn(),
  addOrganizationMember: vi.fn(),
  removeOrganizationMember: vi.fn(),
  updateOrganizationMemberRole: vi.fn(),
  fetchMemberRoles: vi.fn(),
  assignExtraRole: vi.fn(),
  revokeExtraRole: vi.fn(),
  fetchOrganizationRoles: vi.fn(),
  fetchUsers: vi.fn(),
  fetchMyPermissions: vi.fn(),
}));

describe("MembersPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    void i18n.changeLanguage("en");
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
 
    });
 
 
    vi.mocked(api.fetchMyPermissions).mockResolvedValue({ 
      permissions: ["members:invite", "members:remove", "members:roles:manage"], 
      roles: ["admin"],
      managed_department_ids: null
 
    });
    vi.mocked(api.fetchOrganizationMembers).mockResolvedValue([
        { workspace_id: "test-ws-id", user_id: "user-1", email: "owner@example.com", full_name: "Workspace Owner", role_id: "role-owner", extra_role_ids: [] },
        { workspace_id: "test-ws-id", user_id: "user-2", email: "admin@example.com", full_name: "Admin User", role_id: "role-admin", extra_role_ids: [] }
    ]);
    vi.mocked(api.fetchOrganizationRoles).mockResolvedValue([
      { id: "role-owner", name: "Owner", is_system: true, color: "rose", scope_type: "global", created_at: "", updated_at: "" },
      { id: "role-admin", name: "Admin", is_system: true, color: "primary", scope_type: "global", created_at: "", updated_at: "" },
      { id: "role-member", name: "Member", is_system: true, color: "slate", scope_type: "global", created_at: "", updated_at: "" }
    ]);
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

  it("renders members list with owner and admin", async () => {
    renderWithProviders(<MembersPage />);

    // ResponsiveTable renders both desktop and mobile views
    expect((await screen.findAllByText(/Workspace Owner/i, {}, { timeout: 30000 })).length).toBeGreaterThan(0);
    expect((await screen.findAllByText(/OWNER/i)).length).toBeGreaterThan(0);
    expect((await screen.findAllByText(/Admin User/i)).length).toBeGreaterThan(0);

    expect((await screen.findAllByText(/ADMIN/i)).length).toBeGreaterThan(0);
  });

  it("handles inviting a new member", async () => {
    const user = userEvent.setup();
    vi.mocked(api.addOrganizationMember).mockResolvedValue(undefined);

    renderWithProviders(<MembersPage />);

    const inviteButton = await screen.findByText(/Invite Member|Mời thành viên/i, {}, { timeout: 30000 });
    await user.click(inviteButton);

    // Wait for modal subtitle (members.invite.subtitle)
    expect(await screen.findByText(/Manage Staff|Quản lý nhân sự/i)).toBeDefined();

    const emailInput = screen.getByPlaceholderText(/dongnghiep@congty.com|email/i);
    await user.type(emailInput, "new@example.com");

    const submitButton = await screen.findByText(/Send Invitation|Gửi lời mời/i);
    await user.click(submitButton);

    await waitFor(() => {
      expect(api.addOrganizationMember).toHaveBeenCalledWith("new@example.com", "role-owner");
 
    });
  });

  it("handles removing a member", async () => {
    const user = userEvent.setup();
    vi.mocked(api.removeOrganizationMember).mockResolvedValue(undefined);

    renderWithProviders(<MembersPage />);

    // ResponsiveTable renders both desktop and mobile views
    await screen.findAllByText(/Admin User/i, {}, { timeout: 30000 });

    // Owner doesn't have a remove button, so the first one found is for the Admin User
    const removeBtns = await screen.findAllByTitle(/Remove Member|Xóa thành viên/i);
    await user.click(removeBtns[0]);

    // MemberTable uses useConfirm hook which renders a custom dialog.
    const confirmButton = await screen.findByText(/Confirm|Xác nhận|common\.confirm/i, {}, { timeout: 3000 });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(api.removeOrganizationMember).toHaveBeenCalledWith("user-2");

    });
  });

  it("opens role management modal", async () => {
    const user = userEvent.setup();
    vi.mocked(api.fetchMemberRoles).mockResolvedValue({
      primary: "role-admin",
      extra: []
    });

    renderWithProviders(<MembersPage />);

    // ResponsiveTable renders both desktop and mobile views
    await screen.findAllByText(/Admin User/i, {}, { timeout: 30000 });

    // Instead of relying on global button indices, find the specific row for the Admin User
    const adminUserElements = await screen.findAllByText(/Admin User/i);
    const adminRow = adminUserElements[0].closest('tr');
    if (!adminRow) throw new Error("Could not find Admin User row");

    // "Manage Roles" should be the button with the UserCog icon / role management title
    const manageButton = adminRow.querySelector('button[title*="Manage Roles"], button[title*="Quản lý quyền"]')!;
    if (!manageButton) throw new Error("Could not find Manage Roles button in Admin User row");

    await user.click(manageButton);

    expect(await screen.findByText(/Manage Member Roles|Quản lý quyền hạn/i)).toBeDefined();
    expect(api.fetchMemberRoles).toHaveBeenCalledWith("user-2");
  });
});

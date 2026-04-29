import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import MembersPage from "../MembersPage";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import React from "react";
import i18n from "../../i18n/config";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: any) => <a href={to} {...props}>{children}</a>,
}));

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchWorkspaceMembers: vi.fn(),
  addWorkspaceMember: vi.fn(),
  removeWorkspaceMember: vi.fn(),
  updateWorkspaceMemberRole: vi.fn(),
  fetchMemberRoles: vi.fn(),
  assignExtraRole: vi.fn(),
  revokeExtraRole: vi.fn(),
  fetchWorkspaceRoles: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  fetchUsers: vi.fn(),
  fetchMyPermissions: vi.fn(),
}));

describe("MembersPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    i18n.changeLanguage("en");
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
    });
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    (api.fetchMyPermissions as any).mockResolvedValue({ permissions: ["members:invite", "members:remove", "members:roles:manage"], roles: ["admin"] });
    (api.fetchWorkspaceMembers as any).mockResolvedValue([
        { workspace_id: "test-ws-id", user_id: "user-1", email: "owner@example.com", full_name: "Workspace Owner", role_id: "role-owner", extra_role_ids: [] },
        { workspace_id: "test-ws-id", user_id: "user-2", email: "admin@example.com", full_name: "Admin User", role_id: "role-admin", extra_role_ids: [] }
    ]);
    (api.fetchWorkspaceRoles as any).mockResolvedValue([
      { id: "role-owner", name: "Owner", is_system: true, color: "rose" },
      { id: "role-admin", name: "Admin", is_system: true, color: "indigo" },
      { id: "role-member", name: "Member", is_system: true, color: "slate" }
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

    expect(await screen.findByText(/Workspace Owner/i, {}, { timeout: 30000 })).toBeDefined();
    expect((await screen.findAllByText(/OWNER/i)).length).toBeGreaterThan(0);
    expect(await screen.findByText(/Admin User/i)).toBeDefined();
    expect((await screen.findAllByText(/ADMIN/i)).length).toBeGreaterThan(0);
  });

  it("handles inviting a new member", async () => {
    const user = userEvent.setup();
    (api.addWorkspaceMember as any).mockResolvedValue({});

    renderWithProviders(<MembersPage />);

    const inviteBtn = await screen.findByText(/Invite Member|Mời thành viên/i, {}, { timeout: 30000 });
    await user.click(inviteBtn);

    // Wait for modal subtitle (members.invite.subtitle)
    expect(await screen.findByText(/Manage Staff|Quản lý nhân sự/i)).toBeDefined();

    const emailInput = screen.getByPlaceholderText(/dongnghiep@congty.com|email/i);
    await user.type(emailInput, "new@example.com");

    const submitBtn = await screen.findByText(/Send Invitation|Gửi lời mời/i);
    await user.click(submitBtn);

    await waitFor(() => {
      expect(api.addWorkspaceMember).toHaveBeenCalledWith("test-ws-id", "new@example.com", "role-owner");
    });
  });

  it("handles removing a member", async () => {
    const user = userEvent.setup();
    (api.removeWorkspaceMember as any).mockResolvedValue({});

    renderWithProviders(<MembersPage />);

    await screen.findByText(/Admin User/i, {}, { timeout: 30000 });

    // Owner doesn't have a remove button, so the first one found is for the Admin User
    const removeBtns = await screen.findAllByTitle(/Remove Member|Xóa thành viên/i);
    await user.click(removeBtns[0]);

    // MemberTable uses useConfirm hook which renders a custom dialog.
    const confirmBtn = await screen.findByText(/Confirm|Xác nhận|common\.confirm/i, {}, { timeout: 3000 });
    await user.click(confirmBtn);

    await waitFor(() => {
      expect(api.removeWorkspaceMember).toHaveBeenCalledWith("test-ws-id", "user-2");
    });
  });

  it("opens role management modal", async () => {
    const user = userEvent.setup();
    (api.fetchMemberRoles as any).mockResolvedValue({
      primary: "role-admin",
      extra: []
    });

    renderWithProviders(<MembersPage />);

    await screen.findByText(/Admin User/i, {}, { timeout: 30000 });

    // Instead of relying on global button indices, find the specific row for the Admin User
    const adminRow = (await screen.findByText(/Admin User/i)).closest('tr');
    if (!adminRow) throw new Error("Could not find Admin User row");

    // "Manage Roles" should be the button with the UserCog icon / role management title
    const manageBtn = adminRow.querySelector('button[title*="Manage Roles"], button[title*="Quản lý quyền"]') as HTMLButtonElement;
    if (!manageBtn) throw new Error("Could not find Manage Roles button in Admin User row");

    await user.click(manageBtn);

    expect(await screen.findByText(/Manage Member Roles|Quản lý quyền hạn/i)).toBeDefined();
    expect(api.fetchMemberRoles).toHaveBeenCalledWith("test-ws-id", "user-2");
  });
});

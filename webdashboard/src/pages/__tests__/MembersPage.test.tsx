import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import MembersPage from "../MembersPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/Toast";
import * as api from "../../lib/api";
import React from "react";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchWorkspaceMembers: vi.fn(),
  addWorkspaceMember: vi.fn(),
  removeWorkspaceMember: vi.fn(),
  updateWorkspaceMemberRole: vi.fn(),
  fetchMemberRoles: vi.fn(),
  assignExtraRole: vi.fn(),
  revokeExtraRole: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  fetchUsers: vi.fn(),
}));

// Override i18n for this test to return keys
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("MembersPage", () => {
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
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    (api.fetchWorkspaceMembers as any).mockResolvedValue([
      { user_id: "user-1", email: "owner@example.com", full_name: "Workspace Owner", role: "owner" },
      { user_id: "user-2", email: "admin@example.com", full_name: "Admin User", role: "admin" }
    ]);
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            {ui}
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders members list with owner and admin", async () => {
    renderWithProviders(<MembersPage />);

    expect(await screen.findByText("Workspace Owner")).toBeDefined();
    expect(screen.getByText("OWNER")).toBeDefined();
    expect(screen.getByText("Admin User")).toBeDefined();
    expect(screen.getByText("ADMIN")).toBeDefined();
  });

  it("handles inviting a new member", async () => {
    const user = userEvent.setup();
    (api.addWorkspaceMember as any).mockResolvedValue({});

    renderWithProviders(<MembersPage />);

    await user.click(screen.getByText(/members.invite.title/i));

    // Wait for modal
    expect(await screen.findByText(/members.invite.subtitle/i)).toBeDefined();

    const emailInput = screen.getByPlaceholderText("dongnghiep@congty.com");
    await user.type(emailInput, "new@example.com");

    await user.click(screen.getByTestId("submit-invite-member-btn"));

    await waitFor(() => {
      expect(api.addWorkspaceMember).toHaveBeenCalledWith("test-ws-id", "new@example.com", "member");
    });
  });

  it("handles removing a member", async () => {
    const user = userEvent.setup();
    (api.removeWorkspaceMember as any).mockResolvedValue({});
    vi.spyOn(window, "confirm").mockReturnValue(true);

    renderWithProviders(<MembersPage />);

    await screen.findByText("Admin User");
    
    // Owner doesn't have a remove button, so the first one found is for the Admin User
    const removeBtns = await screen.findAllByTitle(/members.table.remove_member/i);
    await user.click(removeBtns[0]);

    expect(window.confirm).toHaveBeenCalled();
    await waitFor(() => {
      expect(api.removeWorkspaceMember).toHaveBeenCalledWith("test-ws-id", "user-2");
    });
  });

  it("opens role management modal", async () => {
    const user = userEvent.setup();
    (api.fetchMemberRoles as any).mockResolvedValue({
      primary: "admin",
      extra: []
    });

    renderWithProviders(<MembersPage />);

    await screen.findByText("Admin User");
    
    // There are 2 members, both have manage roles button. user-2 is the Admin User (second row)
    const manageBtns = await screen.findAllByTitle(/members.table.manage_roles/i);
    await user.click(manageBtns[1]);

    expect(await screen.findByText(/members.roles.modal_title/i)).toBeDefined();
    expect(api.fetchMemberRoles).toHaveBeenCalledWith("test-ws-id", "user-2");
  });
});

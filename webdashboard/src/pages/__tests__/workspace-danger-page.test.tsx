import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import { useWorkspace } from "../../hooks/use-workspace";
import * as api from "../../lib/api";
import * as apiWorkspaces from "../../lib/api/workspaces";
import WorkspaceDangerPage from "../workspace/workspace-danger-page";

vi.mock("../../lib/api/workspaces", () => ({
  clearWorkspaceData: vi.fn(),
}));

vi.mock("../../lib/api", () => ({
  getStandaloneOrganizationId: vi.fn(),
}));

vi.mock("../../hooks/use-workspace", () => ({
  useWorkspace: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));
describe("WorkspaceDangerPage", () => {
  let queryClient: QueryClient;
  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    vi.mocked(api.getStandaloneOrganizationId).mockReturnValue("test-ws-id");
    
    vi.mocked(useWorkspace).mockReturnValue({
      workspace: {
        id: "test-ws-id",
        name: "Test Workspace",
        date_format: "",
        time_format: "",
        timezone: "",
        theme_primary_color: "",
        default_language: "",
        created_at: "",
        default_holiday_ot_multiplier: "1.5",
        holiday_overnight_strategy: "split",
      },
      isLoading: false,
      isError: false,
    });
    
    // Mock window.location.reload
    Object.defineProperty(window, 'location', {
      configurable: true,
      value: { reload: vi.fn() }
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

  it("requires exact workspace name to clear data", async () => {
    const user = userEvent.setup();
    vi.mocked(apiWorkspaces.clearWorkspaceData).mockResolvedValue(undefined);

    renderWithProviders(<WorkspaceDangerPage />);

    const clearDataButton = await screen.findByText("danger.clearAllData");
    await user.click(clearDataButton);

    const modalTitles = await screen.findAllByText("danger.modalTitle");
    expect(modalTitles.length).toBeGreaterThan(0);

    // The confirm button inside the modal
    const confirmButton = screen.getByRole('button', { name: "danger.permanentlyDelete" });
    expect(confirmButton).toBeDisabled();

    const input = screen.getByPlaceholderText("Test Workspace");
    await user.type(input, "Wrong Name");
    expect(confirmButton).toBeDisabled();

    await user.clear(input);
    await user.type(input, "Test Workspace");
    expect(confirmButton).not.toBeDisabled();

    await user.click(confirmButton);

    expect(apiWorkspaces.clearWorkspaceData).toHaveBeenCalledWith("test-ws-id", "all");
  });
});

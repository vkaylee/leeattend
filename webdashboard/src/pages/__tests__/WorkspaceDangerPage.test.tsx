import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import WorkspaceDangerPage from "../workspace/WorkspaceDangerPage";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/ui/Toast";
import * as apiWorkspaces from "../../lib/api/workspaces";
import * as api from "../../lib/api";
import React from "react";

vi.mock("../../lib/api/workspaces", () => ({
  clearWorkspaceData: vi.fn(),
}));

vi.mock("../../lib/api", () => ({
  getCurrentWorkspaceId: vi.fn(),
}));

vi.mock("../../hooks/use-workspace", () => ({
  useWorkspace: vi.fn(),
}));

import { useWorkspace } from "../../hooks/use-workspace";

describe("WorkspaceDangerPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    
    (useWorkspace as any).mockReturnValue({
      workspace: {
        id: "test-ws-id",
        name: "Test Workspace",
      },
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
    (apiWorkspaces.clearWorkspaceData as any).mockResolvedValue({});

    renderWithProviders(<WorkspaceDangerPage />);

    const clearDataBtn = await screen.findByText("CLEAR ALL DATA");
    await user.click(clearDataBtn);

    const modalTitles = await screen.findAllByText("Clear Workspace Data");
    expect(modalTitles.length).toBeGreaterThan(0);

    // The confirm button inside the modal
    const confirmBtn = screen.getByRole('button', { name: "PERMANENTLY DELETE DATA" });
    expect(confirmBtn).toBeDisabled();

    const input = screen.getByPlaceholderText("Test Workspace");
    await user.type(input, "Wrong Name");
    expect(confirmBtn).toBeDisabled();

    await user.clear(input);
    await user.type(input, "Test Workspace");
    expect(confirmBtn).not.toBeDisabled();

    await user.click(confirmBtn);

    expect(apiWorkspaces.clearWorkspaceData).toHaveBeenCalledWith("test-ws-id", "all");
  });
});

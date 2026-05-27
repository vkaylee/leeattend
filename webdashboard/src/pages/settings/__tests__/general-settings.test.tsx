import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import * as api from "@/lib/api";

import GeneralSettings from "../general-settings";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ 
    t: (key: string) => key
  }),
}));

vi.mock("@/lib/api", async (importOriginal) => {
  const mod = await importOriginal<typeof import("@/lib/api")>();
  return {
    ...mod,
    fetchWorkspace: vi.fn(),
    updateWorkspace: vi.fn(),
    getCurrentWorkspaceId: vi.fn(),
  };
});

vi.mock("@/context/use-dark-mode-context", () => ({
  useDarkModeContext: () => ({
    isDarkMode: false,
    toggleDarkMode: vi.fn(),
    setDarkMode: vi.fn()
  })
}));

vi.mock("@/components/ui/toast", () => ({ useToast: () => ({ success: vi.fn(), error: vi.fn() }) }));

const mockWorkspace = {
  id: "test-ws-id",
  name: "Test Workspace",
  date_format: "DD/MM/YYYY",
  time_format: "24h",
  timezone: "UTC",
  theme_primary_color: "#2563eb",
  default_language: "en",
  logo_url: "logo.png",
  tagline: "Test Tagline"
};

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

describe("GeneralSettings Page", () => {
  const wsId = "test-ws-id";

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue(wsId);
    vi.mocked(api.fetchWorkspace).mockResolvedValue(mockWorkspace as unknown as api.Workspace);
  });

  it("renders loader initially and then content", async () => {
    const { container } = render(
      <QueryClientProvider client={queryClient}>
        <GeneralSettings />
      </QueryClientProvider>
    );

    // Initial loader
    expect(container.querySelector(".animate-spin")).toBeDefined();

    // Wait for content
    expect(await screen.findByTestId("general-settings-title")).toBeDefined();
    expect(screen.getByTestId("save-workspace-config-btn")).toBeDefined();
  });

  it("switches tabs correctly", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <GeneralSettings />
      </QueryClientProvider>
    );

    // Wait for content
    await screen.findByTestId("general-settings-title");

    // Click Appearance tab
    fireEvent.click(screen.getByTestId("tab-appearance"));

    // Workspace Name input should be visible in Appearance tab
    expect(await screen.findByTestId("workspace-name-input")).toBeDefined();
    
    // Click Localization tab
    fireEvent.click(screen.getByTestId("tab-localization"));
    
    // Wait until Appearance input is gone or Localization header appears
    expect(await screen.findByText("common.workspace_settings.general_config")).toBeDefined();
  });

  it("calls updateMutation on save", async () => {
    vi.mocked(api.updateWorkspace).mockResolvedValue(mockWorkspace as unknown as api.Workspace);

    render(
      <QueryClientProvider client={queryClient}>
        <GeneralSettings />
      </QueryClientProvider>
    );

    // Wait for content
    await screen.findByTestId("general-settings-title");

    const saveBtn = screen.getByTestId("save-workspace-config-btn");
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(api.updateWorkspace).toHaveBeenCalledWith(wsId, expect.objectContaining({
        name: "Test Workspace",
        date_format: "DD/MM/YYYY"
      }));
    });
  });
});

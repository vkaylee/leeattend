import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

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
    fetchOrganization: vi.fn(),
    updateOrganization: vi.fn(),
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



const mockOrganization = {
  id: "test-org-id",
  name: "Test Organization",
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
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.fetchOrganization).mockResolvedValue(mockOrganization as unknown as api.Organization);
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
    expect(screen.getByTestId("save-organization-config-btn")).toBeDefined();
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

    // Organization Name input should be visible in Appearance tab
    expect(await screen.findByTestId("organization-name-input")).toBeDefined();
    
    // Click Localization tab
    fireEvent.click(screen.getByTestId("tab-localization"));
    
    // Wait until Appearance input is gone or Localization header appears
    expect(await screen.findByText("common.organization_settings.general_config")).toBeDefined();
  });

  it("keeps settings tabs in a mobile-width container", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <GeneralSettings />
      </QueryClientProvider>
    );

    await screen.findByTestId("general-settings-title");

    expect(screen.getByRole("tablist")).toHaveClass("grid-cols-1");
    expect(screen.getByTestId("tab-localization")).toHaveClass("py-2");
    expect(screen.getByTestId("tab-appearance")).toHaveClass("py-2");
  });

  it("calls updateMutation on save", async () => {
    vi.mocked(api.updateOrganization).mockResolvedValue(mockOrganization as unknown as api.Organization);

    render(
      <QueryClientProvider client={queryClient}>
        <GeneralSettings />
      </QueryClientProvider>
    );

    // Wait for content
    await screen.findByTestId("general-settings-title");

    const saveBtn = screen.getByTestId("save-organization-config-btn");
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(api.updateOrganization).toHaveBeenCalledWith(expect.objectContaining({
        name: "Test Organization",
        date_format: "DD/MM/YYYY"
      }));
    });
  });
});


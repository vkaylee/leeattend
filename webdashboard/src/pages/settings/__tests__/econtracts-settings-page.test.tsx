import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";

import { ToastProvider } from "@/components/ui/toast";

import EContractsSettingsPage from "../econtracts-settings-page";

// Mock i18next translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: Record<string, unknown>) => options?.defaultValue as string ?? key,
  }),
}));

describe("EContractsSettingsPage", () => {
  const renderPage = () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, gcTime: 0 },
      },
    });
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <EContractsSettingsPage />
        </ToastProvider>
      </QueryClientProvider>,
    );
  };

  it("renders the page header and coming-soon placeholder", () => {
    renderPage();
    expect(screen.getByText("e-Contracts Integration")).toBeInTheDocument();
    expect(screen.getByText("Coming Soon")).toBeInTheDocument();
    expect(
      screen.getByText("The e-signature backend is under active development. Check back soon."),
    ).toBeInTheDocument();
  });
});

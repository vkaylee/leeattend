import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/Toast";

import * as api from "@/lib/api/payroll";

import PayrollSettingsSection from "../PayrollSettingsSection";


// Mock API
vi.mock("@/lib/api/payroll", () => ({
  fetchPayrollSettings: vi.fn(),
  updatePayrollSettings: vi.fn(),
  fetchPayrollSettingsHistory: vi.fn(),
}));

// Mock i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("PayrollSettingsSection", () => {
  let queryClient: QueryClient;
  const mockWsId = "ws-123";

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
    (api.fetchPayrollSettingsHistory as any).mockResolvedValue([
      { fixed_std_days: 26, hours_per_day: 8, effective_date: "2026-01-01" }
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

  it("renders settings correctly", async () => {
    renderWithProviders(<PayrollSettingsSection wsId={mockWsId} />);

    // In the new UI, standard days are wrapped in a span with text-primary-600
    // and followed by common.days_short.
    expect(await screen.findByText(/26/i, { selector: 'span' })).toBeInTheDocument();
    expect(screen.getByText(/01\/01\/2026/i)).toBeInTheDocument();
  });

  it("calls update API when save button is clicked", async () => {
    (api.updatePayrollSettings as any).mockResolvedValue({});
    renderWithProviders(<PayrollSettingsSection wsId={mockWsId} />);

    const addBtn = await screen.findByText(/common.add_new/i);
    fireEvent.click(addBtn);

    const daysInput = await screen.findByDisplayValue("26");
    fireEvent.change(daysInput, { target: { value: "24" } });

    const saveBtn = screen.getByText(/common.save/i);
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(api.updatePayrollSettings).toHaveBeenCalledWith(mockWsId, expect.objectContaining({
        fixed_std_days: 24
      }));
    });
  });
});

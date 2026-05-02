import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api/payroll";

import SalaryComponentManager from "../salary-component-manager";


// Mock API
vi.mock("@/lib/api/payroll", () => ({
  fetchSalaryComponents: vi.fn(),
  createSalaryComponent: vi.fn(),
}));

// Standard override for i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("SalaryComponentManager", () => {
  let queryClient: QueryClient;
  const mockWsId = "ws-123";
  const mockComponents = [
    { id: "1", name: "Lương cơ bản", code: "BASE", is_base_for_ot: true, is_taxable: true },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
 
    vi.mocked(api.fetchSalaryComponents).mockResolvedValue(mockComponents as unknown as never);
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

  it("renders list of components", async () => {
    renderWithProviders(<SalaryComponentManager wsId={mockWsId} />);

    expect(await screen.findByText(/Lương cơ bản/i)).toBeInTheDocument();
    expect(screen.getByText(/BASE/i)).toBeInTheDocument();
  });

  it("opens modal and submits new component", async () => {
 
    vi.mocked(api.createSalaryComponent).mockResolvedValue({ id: "2" } as unknown as never);
    renderWithProviders(<SalaryComponentManager wsId={mockWsId} />);

    const addButton = screen.getByText(/common.add/i);
    fireEvent.click(addButton);

    const nameInput = await screen.findByLabelText(/common.name/i);
    fireEvent.change(nameInput, { target: { value: "Gas" } });
    
    const codeInput = screen.getByLabelText(/payroll.components.code_label/i);
    fireEvent.change(codeInput, { target: { value: "GAS" } });

    const saveButton = await screen.findByRole("button", { name: /common.save/i });
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(api.createSalaryComponent).toHaveBeenCalledWith(mockWsId, expect.objectContaining({
        name: "Gas",
        code: "GAS"
      }));
    });
  });
});
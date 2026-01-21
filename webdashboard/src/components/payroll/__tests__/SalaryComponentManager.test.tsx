import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import SalaryComponentManager from "../SalaryComponentManager";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "@/components/Toast";
import * as api from "@/lib/api/payroll";
import React from "react";

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
    (api.fetchSalaryComponents as any).mockResolvedValue(mockComponents);
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
    (api.createSalaryComponent as any).mockResolvedValue({ id: "2" });
    renderWithProviders(<SalaryComponentManager wsId={mockWsId} />);

    const addBtn = screen.getByText(/common.add/i);
    fireEvent.click(addBtn);

    const nameInput = await screen.findByLabelText(/common.name/i);
    fireEvent.change(nameInput, { target: { value: "Gas" } });
    
    const codeInput = screen.getByLabelText(/payroll.components.code_label/i);
    fireEvent.change(codeInput, { target: { value: "GAS" } });

    const saveBtn = await screen.findByRole("button", { name: /common.save/i });
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(api.createSalaryComponent).toHaveBeenCalledWith(mockWsId, expect.objectContaining({
        name: "Gas",
        code: "GAS"
      }));
    });
  });
});
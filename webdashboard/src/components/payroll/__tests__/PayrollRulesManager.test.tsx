import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import PayrollRulesManager from "../PayrollRulesManager";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "@/components/ui/Toast";
import * as api from "@/lib/api/payroll";
import React from "react";

// Mock API
vi.mock("@/lib/api/payroll", () => ({
  fetchPayrollRules: vi.fn(),
  createPayrollRule: vi.fn(),
  fetchPayrollRuleTiers: vi.fn(),
  fetchPayrollRuleAssignments: vi.fn(),
}));

// We mock apiFetch for assignments since RuleAssignmentManager uses it directly
vi.mock("@/lib/api/helpers", async () => {
  const actual = await vi.importActual("@/lib/api/helpers");
  return {
    ...actual as any,
    apiFetch: vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve([]),
    }),
  };
});

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

describe("PayrollRulesManager", () => {
  let queryClient: QueryClient;
  const mockWsId = "ws-123";
  const mockRules = [
    { id: "1", name: "Bảo hiểm Xã hội", code: "BHXH", category: "statutory_deduction", calculation_method: "percentage", applies_to: "employee", is_mandatory: true },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
    (api.fetchPayrollRules as any).mockResolvedValue(mockRules);
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

  it("renders list of payroll rules", async () => {
    renderWithProviders(<PayrollRulesManager wsId={mockWsId} />);

    expect(await screen.findByText(/Bảo hiểm Xã hội/i)).toBeInTheDocument();
    expect(screen.getByText(/statutory_deduction • percentage/i)).toBeInTheDocument();
    expect(screen.getByText(/MANDATORY/i)).toBeInTheDocument();
  });

  it("opens modal and submits new payroll rule", async () => {
    (api.createPayrollRule as any).mockResolvedValue({ id: "2" });
    renderWithProviders(<PayrollRulesManager wsId={mockWsId} />);

    const addBtn = screen.getByText(/common.add/i);
    fireEvent.click(addBtn);

    const nameInput = await screen.findByLabelText(/common.name/i);
    fireEvent.change(nameInput, { target: { value: "Phạt đi muộn" } });
    
    const codeInput = screen.getByLabelText(/Code/i);
    fireEvent.change(codeInput, { target: { value: "LATE" } });

    const catSelect = screen.getByLabelText(/Category/i);
    fireEvent.change(catSelect, { target: { value: "deduction" } });

    const rateInput = screen.getByLabelText(/Rate \(%\)/i);
    fireEvent.change(rateInput, { target: { value: "10" } });

    const saveBtn = await screen.findByRole("button", { name: /common.save/i });
    fireEvent.submit(saveBtn.closest("form") as HTMLFormElement);

    await waitFor(() => {
      expect(api.createPayrollRule).toHaveBeenCalledWith(mockWsId, expect.objectContaining({
        name: "Phạt đi muộn",
        code: "LATE",
        category: "deduction"
      }));
    });
  });

  it("renders logic configuration dynamically", async () => {
    renderWithProviders(<PayrollRulesManager wsId={mockWsId} />);
    const addBtn = screen.getByText(/common.add/i);
    fireEvent.click(addBtn);

    const calcMethodSelect = screen.getByLabelText(/Calculation Method/i);
    fireEvent.change(calcMethodSelect, { target: { value: "fixed" } });

    expect(await screen.findByLabelText(/Fixed Amount \(₫\)/i)).toBeInTheDocument();
    
    fireEvent.change(calcMethodSelect, { target: { value: "tiered" } });
    expect(await screen.findByLabelText(/Tier Base Amount/i)).toBeInTheDocument();
  });

  it("expands a rule to show Tier and Assignment managers", async () => {
    (api.fetchPayrollRuleTiers as any).mockResolvedValue([]);
    renderWithProviders(<PayrollRulesManager wsId={mockWsId} />);
    
    const ruleCard = await screen.findByText(/Bảo hiểm Xã hội/i);
    fireEvent.click(ruleCard);

    expect(await screen.findByText(/Rule Assignments/i)).toBeInTheDocument();
  });
});

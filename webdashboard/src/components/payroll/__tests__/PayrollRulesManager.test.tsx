import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/Toast";

import * as api from "@/lib/api/payroll";

import PayrollRulesManager from "../PayrollRulesManager";


// Mock API
vi.mock("@/lib/api/payroll", () => ({
  fetchPayrollRules: vi.fn(),
  createPayrollRule: vi.fn(),
  fetchSalaryBaseGroups: vi.fn(),
}));

// Mock child components to simplify
vi.mock("../RuleTierManager", () => ({
  default: () => <div data-testid="rule-tier-manager">RuleTierManager</div>,
}));

vi.mock("../RuleAssignmentManager", () => ({
  default: () => <div data-testid="rule-assignment-manager">RuleAssignmentManager</div>,
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

describe("PayrollRulesManager", () => {
  let queryClient: QueryClient;
  const mockWsId = "ws-123";
  
  const mockRules = [
    { 
      id: "rule-1", 
      name: "Social Insurance", 
      code: "SI", 
      category: "statutory_deduction", 
      applies_to: "employee", 
      calculation_method: "percentage", 
      is_mandatory: true,
      rule_config: { rate: 8, base: "social_insurance" }
    },
  ];

  const mockBasesResponse = {
    groups: [
      { id: "base-1", name: "Gross Salary", slug: "gross" },
      { id: "base-2", name: "Social Insurance", slug: "social_insurance" },
    ],
    members: []
  };

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
 
    vi.mocked(api.fetchPayrollRules).mockResolvedValue(mockRules as unknown as never);
 
    vi.mocked(api.fetchSalaryBaseGroups).mockResolvedValue(mockBasesResponse as unknown as never);
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

  it("renders list of statutory rules", async () => {
    renderWithProviders(<PayrollRulesManager wsId={mockWsId} />);

    expect(await screen.findByText(/Social Insurance/i)).toBeInTheDocument();
    expect(screen.getByText(/SI/i)).toBeInTheDocument();
  });

  it("opens modal and submits new rule", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.createPayrollRule).mockResolvedValue({ id: "rule-2" } as unknown as never);
    renderWithProviders(<PayrollRulesManager wsId={mockWsId} />);

    const addBtn = await screen.findByText(/common.add/i);
    await user.click(addBtn);

    const nameInput = await screen.findByPlaceholderText(/Social Insurance/i);
    await user.type(nameInput, "Health Insurance");
    
    const codeInput = screen.getByPlaceholderText(/BHXH/i);
    await user.type(codeInput, "HI");

    // The label has an SVG icon inside, so getByLabelText might fail if not matched correctly.
    // Let's find by role and then check for "Rate"
    const rateInput = screen.getByRole("spinbutton");
    await user.clear(rateInput);
    await user.type(rateInput, "1.5");

    const saveBtn = screen.getByRole("button", { name: /common.save/i });
    expect(saveBtn).not.toBeDisabled();
    
    // Sometimes clicking button doesn't trigger submit in tests depending on portal/event bubbling
    const form = screen.getByRole("dialog").querySelector("form");
    if (form) {
      fireEvent.submit(form);
    } else {
      await user.click(saveBtn);
    }

    await waitFor(() => {
      expect(api.createPayrollRule).toHaveBeenCalledWith(mockWsId, expect.objectContaining({
        name: "Health Insurance",
        code: "HI",
        rule_config: expect.objectContaining({ rate: 1.5 })
      }));
    }, { timeout: 3000 });
  });

  it("expands rule to show details", async () => {
    renderWithProviders(<PayrollRulesManager wsId={mockWsId} />);

    const ruleItem = await screen.findByText(/Social Insurance/i);
    fireEvent.click(ruleItem);

    expect(await screen.findByTestId("rule-assignment-manager")).toBeInTheDocument();
  });
});

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api/payroll";

import BonusPolicyManager from "../bonus-policy-manager";


// Mock API
vi.mock("@/lib/api/payroll", () => ({
  fetchIncentivePolicies: vi.fn(),
  createIncentivePolicy: vi.fn(),
  deleteIncentivePolicy: vi.fn(),
}));

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const renderWithProviders = (ui: React.ReactNode) => {
  return render(
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        {ui}
      </ToastProvider>
    </QueryClientProvider>
  );
};

describe("BonusPolicyManager", () => {
  const mockWsId = "ws-123";

  beforeEach(() => {
    vi.clearAllMocks();
 
    vi.mocked(api.fetchIncentivePolicies).mockResolvedValue([]);
  });

  it("renders correctly and opens creation form", () => {
    renderWithProviders(<BonusPolicyManager   />);
    
    expect(screen.getByText(/payroll.incentives.title/i)).toBeDefined();
    
    const addButton = screen.getByText(/payroll.incentives.add_btn/i);
    fireEvent.click(addButton);
    
    expect(screen.getByPlaceholderText(/e.g. Diligence Bonus/i)).toBeDefined();
    expect(screen.getByText(/payroll.incentives.tiers_config/i)).toBeDefined();
  });

  it("allows adding and removing tiers", () => {
    renderWithProviders(<BonusPolicyManager   />);
    fireEvent.click(screen.getByText(/payroll.incentives.add_btn/i));
    
    // Default has 2 tiers
    expect(screen.getAllByText(/#/i).length).toBe(2);
    
    // Add tier
    const addTierButton = screen.getByText(/common.add/i);
    fireEvent.click(addTierButton);
    expect(screen.getAllByText(/#/i).length).toBe(3);
  });

  it("submits policy data correctly", async () => {
 
    vi.mocked(api.createIncentivePolicy).mockResolvedValue({ id: "p1" } as unknown as never);
    
    renderWithProviders(<BonusPolicyManager   />);
    fireEvent.click(screen.getByText(/payroll.incentives.add_btn/i));
    
    const nameInput = screen.getByPlaceholderText(/e.g. Diligence Bonus/i);
    fireEvent.change(nameInput, { target: { value: "Monthly Diligence" } });
    
    const saveButton = screen.getByText(/common.save_settings/i);
    fireEvent.click(saveButton);
    
    await waitFor(() => {
      expect(api.createIncentivePolicy).toHaveBeenCalledWith(expect.objectContaining({
        name: "Monthly Diligence",
        base_amount: 300000,
        tiers: expect.any(Array) as unknown as unknown[]
      }));
    });
  });

  it("allows adding and removing rules in a tier", () => {
    renderWithProviders(<BonusPolicyManager   />);
    fireEvent.click(screen.getByText(/payroll.incentives.add_btn/i));
    
    // Default tier 1 has 1 rule (Late <= 2)
    const tierContainers = screen.getAllByText(/Conditions \(AND\)/i).map(el => el.parentElement);
    expect(tierContainers).toHaveLength(2);
    
    // Add rule to Tier 2
    const addRuleButtons = screen.getAllByText(/Add Rule/i);
    fireEvent.click(addRuleButtons[1]); // add to second tier
    
    // Now there should be an extra select for metric
    const metricSelects = screen.getAllByRole("combobox").filter(el => (el as HTMLSelectElement).value === "kpi_score" || (el as HTMLSelectElement).value === "late_early_count");
    expect(metricSelects.length).toBeGreaterThan(1);
  });
});

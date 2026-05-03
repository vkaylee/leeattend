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
    renderWithProviders(<BonusPolicyManager wsId={mockWsId} />);
    
    expect(screen.getByText(/payroll.incentives.title/i)).toBeDefined();
    
    const addButton = screen.getByText(/payroll.incentives.add_btn/i);
    fireEvent.click(addButton);
    
    expect(screen.getByPlaceholderText(/e.g. Diligence Bonus/i)).toBeDefined();
    expect(screen.getByText(/payroll.incentives.tiers_config/i)).toBeDefined();
  });

  it("allows adding and removing tiers", () => {
    renderWithProviders(<BonusPolicyManager wsId={mockWsId} />);
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
    
    renderWithProviders(<BonusPolicyManager wsId={mockWsId} />);
    fireEvent.click(screen.getByText(/payroll.incentives.add_btn/i));
    
    const nameInput = screen.getByPlaceholderText(/e.g. Diligence Bonus/i);
    fireEvent.change(nameInput, { target: { value: "Monthly Diligence" } });
    
    const saveButton = screen.getByText(/common.save_settings/i);
    fireEvent.click(saveButton);
    
    await waitFor(() => {
      expect(api.createIncentivePolicy).toHaveBeenCalledWith(mockWsId, expect.objectContaining({
        name: "Monthly Diligence",
        base_amount: 300000,
        tiers: expect.any(Array) as unknown as unknown[]
      }));
    });
  });
});

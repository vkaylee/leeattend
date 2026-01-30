import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import BonusPolicyManager from "../BonusPolicyManager";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "@/lib/api/payroll";
import { ToastProvider } from "@/components/ui/Toast";

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
    (api.fetchIncentivePolicies as any).mockResolvedValue([]);
  });

  it("renders correctly and opens creation form", async () => {
    renderWithProviders(<BonusPolicyManager wsId={mockWsId} />);
    
    expect(screen.getByText(/payroll.incentives.title/i)).toBeDefined();
    
    const addBtn = screen.getByText(/payroll.incentives.add_btn/i);
    fireEvent.click(addBtn);
    
    expect(screen.getByPlaceholderText(/e.g. Diligence Bonus/i)).toBeDefined();
    expect(screen.getByText(/payroll.incentives.tiers_config/i)).toBeDefined();
  });

  it("allows adding and removing tiers", async () => {
    renderWithProviders(<BonusPolicyManager wsId={mockWsId} />);
    fireEvent.click(screen.getByText(/payroll.incentives.add_btn/i));
    
    // Default has 2 tiers
    expect(screen.getAllByText(/#/i).length).toBe(2);
    
    // Add tier
    const addTierBtn = screen.getByText(/common.add/i);
    fireEvent.click(addTierBtn);
    expect(screen.getAllByText(/#/i).length).toBe(3);
  });

  it("submits policy data correctly", async () => {
    (api.createIncentivePolicy as any).mockResolvedValue({ id: "p1" });
    
    renderWithProviders(<BonusPolicyManager wsId={mockWsId} />);
    fireEvent.click(screen.getByText(/payroll.incentives.add_btn/i));
    
    const nameInput = screen.getByPlaceholderText(/e.g. Diligence Bonus/i);
    fireEvent.change(nameInput, { target: { value: "Monthly Diligence" } });
    
    const saveBtn = screen.getByText(/common.save_settings/i);
    fireEvent.click(saveBtn);
    
    await waitFor(() => {
      expect(api.createIncentivePolicy).toHaveBeenCalledWith(mockWsId, expect.objectContaining({
        name: "Monthly Diligence",
        base_amount: 300000,
        tiers: expect.any(Array)
      }));
    });
  });
});

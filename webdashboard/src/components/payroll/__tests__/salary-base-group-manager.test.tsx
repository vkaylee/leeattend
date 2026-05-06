import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api/payroll";

import SalaryBaseGroupManager from "../salary-base-group-manager";


// Mock API
vi.mock("@/lib/api/payroll", () => ({
  fetchSalaryBaseGroups: vi.fn(),
  fetchSalaryComponents: vi.fn(),
  assignComponentToSalaryBase: vi.fn(),
  unassignComponentFromSalaryBase: vi.fn(),
}));

// Standard override for i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: any) => options?.defaultValue || key,
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("SalaryBaseGroupManager", () => {
  let queryClient: QueryClient;
  const mockWsId = "ws-123";
  
  const mockGroups = [
    { id: "base-1", name: "Gross Salary", slug: "gross", description: "All earnings", is_system: true },
    { id: "base-2", name: "Social Insurance", slug: "social_insurance", description: "SI base", is_system: false },
  ];
  
  const mockMembers = [
    { base_group_id: "base-1", component_definition_id: "comp-1" },
    { base_group_id: "base-2", component_definition_id: "comp-1" },
  ];
  
  const mockComponents = [
    { id: "comp-1", name: "Basic Salary", code: "BASIC" },
    { id: "comp-2", name: "Allowance", code: "ALW" },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
 
    vi.mocked(api.fetchSalaryBaseGroups).mockResolvedValue({ groups: mockGroups, members: mockMembers } as unknown as never);
 
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

  it("renders list of salary base groups", async () => {
    renderWithProviders(<SalaryBaseGroupManager wsId={mockWsId} />);

    // Wait for the loading spinner to disappear
    await waitFor(() => {
      expect(screen.queryByTestId("loader")).not.toBeInTheDocument();
    }, { timeout: 3000 });

    expect(await screen.findByRole("heading", { name: /Gross Salary/i })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: /Social Insurance/i })).toBeInTheDocument();
  });

  it("shows components and their assignment status", async () => {
    renderWithProviders(<SalaryBaseGroupManager wsId={mockWsId} />);

    // Wait for the loading spinner to disappear
    await waitFor(() => {
      expect(screen.queryByTestId("loader")).not.toBeInTheDocument();
    }, { timeout: 3000 });

    await screen.findByRole("heading", { name: /Gross Salary/i });

    // Basic Salary should be in Gross Salary (mockMembers says so)
    const grossSalarySection = screen.getByRole("heading", { name: /Gross Salary/i }).closest('div.bg-white, div.dark\\:bg-slate-800\\/50');
    expect(grossSalarySection).toBeInTheDocument();
    
    const basicSalaries = screen.getAllByText(/Basic Salary/i);
    expect(basicSalaries.length).toBeGreaterThan(0);
  });

  it("handles assigning a component", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.assignComponentToSalaryBase).mockResolvedValue({} as unknown as never);
    renderWithProviders(<SalaryBaseGroupManager wsId={mockWsId} />);

    await waitFor(() => {
      expect(screen.queryByTestId("loader")).not.toBeInTheDocument();
    }, { timeout: 3000 });

    await screen.findByRole("heading", { name: /Social Insurance/i });
    
    // Find the "+" button for Allowance in Social Insurance group
    const siGroup = screen.getByRole("heading", { name: /Social Insurance/i }).closest('div.bg-white, div.dark\\:bg-slate-800\\/50');
    expect(siGroup).toBeInTheDocument();
    
    const allowanceInSI = screen.getAllByTitle("Assign Component").find(button => {
      const container = button.closest('div.flex.items-center.justify-between');
      return siGroup?.contains(button) && container?.textContent.includes("Allowance");
    });
    expect(allowanceInSI).toBeDefined();
    
    if (allowanceInSI) {
        await user.click(allowanceInSI);
    }

    await waitFor(() => {
      expect(api.assignComponentToSalaryBase).toHaveBeenCalledWith(mockWsId, "base-2", "comp-2");
    });
  });

  it("handles unassigning a component", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.unassignComponentFromSalaryBase).mockResolvedValue({} as unknown as never);
    renderWithProviders(<SalaryBaseGroupManager wsId={mockWsId} />);

    await waitFor(() => {
      expect(screen.queryByTestId("loader")).not.toBeInTheDocument();
    }, { timeout: 3000 });

    await screen.findByRole("heading", { name: /Social Insurance/i });
    
    // In Social Insurance (base-2), Basic Salary (comp-1) IS assigned.
    const siGroup = screen.getByRole("heading", { name: /Social Insurance/i }).closest('div.bg-white, div.dark\\:bg-slate-800\\/50');
    
    const basicInSI = screen.getAllByTitle("Unassign Component").find(button => {
      const container = button.closest('div.flex.items-center.justify-between');
      return siGroup?.contains(button) && container?.textContent.includes("Basic Salary");
    });
    expect(basicInSI).toBeDefined();
    
    if (basicInSI) {
        await user.click(basicInSI);
    }

    await waitFor(() => {
      expect(api.unassignComponentFromSalaryBase).toHaveBeenCalledWith(mockWsId, "base-2", "comp-1");
    });
  });
});

import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/Toast";

import * as api from "@/lib/api/payroll";

import EmployeeSalaryConfig from "../EmployeeSalaryConfig";


// Mock API
vi.mock("@/lib/api/payroll", () => ({
  fetchSalaryComponents: vi.fn(),
  fetchEmployeeSalaryComponents: vi.fn(),
  assignEmployeeSalaryComponent: vi.fn(),
  fetchEmployeePayrollConfigs: vi.fn(),
  setEmployeePayrollConfig: vi.fn(),
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

describe("EmployeeSalaryConfig", () => {
  let queryClient: QueryClient;
  const mockWsId = "ws-123";
  const mockEmpId = "emp-456";

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
 
    vi.mocked(api.fetchSalaryComponents).mockResolvedValue([
      { id: "comp-1", name: "Base Salary", code: "BASE" }
    ] as unknown as never);
 
    vi.mocked(api.fetchEmployeeSalaryComponents).mockResolvedValue([
      { id: "esc-1", component_definition_id: "comp-1", name: "Lương cơ bản", code: "BASE", amount: "1000", effective_date: "2026-01-01", is_base_for_ot: true }
    ] as unknown as never);
 
    vi.mocked(api.fetchEmployeePayrollConfigs).mockResolvedValue([] as unknown as never);
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

  it("renders employee salary components", async () => {
    renderWithProviders(<EmployeeSalaryConfig wsId={mockWsId} employeeId={mockEmpId} />);

    expect(await screen.findByText(/Lương cơ bản/i)).toBeInTheDocument();
    expect(screen.getByText(/1[.,]000/)).toBeInTheDocument();
  });

  it("allows adding a new salary component to employee", async () => {
 
    vi.mocked(api.assignEmployeeSalaryComponent).mockResolvedValue({} as unknown as never);
    renderWithProviders(<EmployeeSalaryConfig wsId={mockWsId} employeeId={mockEmpId} />);

    const addBtn = await screen.findByRole("button", { name: /payroll.employee_config.add_btn/i });
    fireEvent.click(addBtn);

    // Select component type from Combobox
    const combobox = await screen.findByLabelText(/payroll.employee_config.item_type/i);
    fireEvent.change(combobox, { target: { value: 'comp-1' } });


    

    const amountInput = screen.getByPlaceholderText(/1[.,]000/);
    fireEvent.change(amountInput, { target: { value: "5000" } });

    const saveBtn = screen.getByRole("button", { name: /common.save/i });
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(api.assignEmployeeSalaryComponent).toHaveBeenCalled();
    });
  });
});
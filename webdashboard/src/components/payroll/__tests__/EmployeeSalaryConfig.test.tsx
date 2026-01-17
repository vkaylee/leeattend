import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import EmployeeSalaryConfig from "../EmployeeSalaryConfig";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "@/lib/api/payroll";

vi.mock("@/lib/api/payroll", () => ({
  fetchSalaryComponents: vi.fn(),
  fetchEmployeeSalaryComponents: vi.fn(),
  assignEmployeeSalaryComponent: vi.fn(),
}));

vi.mock("@/components/Toast", () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}));

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

describe("EmployeeSalaryConfig", () => {
  const mockWsId = "ws-123";
  const mockEmpId = "emp-456";
  const mockDefinitions = [
    { id: "def-1", name: "Lương cơ bản", code: "BASE" },
  ];
  const mockEmployeeComps = [
    { id: "1", name: "Lương cơ bản", code: "BASE", amount: "1000", effective_date: "2026-01-01", is_base_for_ot: true },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchSalaryComponents as any).mockResolvedValue(mockDefinitions);
    (api.fetchEmployeeSalaryComponents as any).mockResolvedValue(mockEmployeeComps);
  });

  it("renders employee salary components", async () => {
    render(<EmployeeSalaryConfig wsId={mockWsId} employeeId={mockEmpId} />, { wrapper });

    await waitFor(() => {
      expect(screen.getByText("1.000")).toBeInTheDocument(); // Formatted
      expect(screen.getByText("2026-01-01")).toBeInTheDocument();
    });
  });

  it("allows adding a new salary component to employee", async () => {
    (api.assignEmployeeSalaryComponent as any).mockResolvedValue({ id: "2" });
    render(<EmployeeSalaryConfig wsId={mockWsId} employeeId={mockEmpId} />, { wrapper });

    const addBtn = screen.getByTestId("add-salary-item-btn");
    fireEvent.click(addBtn);

    // Select component type (Mocked Combobox is a select)
    const typeSelect = screen.getByRole("combobox");
    fireEvent.change(typeSelect, { target: { value: "def-1" } });

    const amountInput = screen.getByPlaceholderText("10,000,000");
    fireEvent.change(amountInput, { target: { value: "5000" } });

    const saveBtn = screen.getByTestId("save-salary-item-btn");
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(api.assignEmployeeSalaryComponent).toHaveBeenCalledWith(mockWsId, mockEmpId, expect.objectContaining({
        amount: "5000"
      }));
    });
  });
});

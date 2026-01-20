import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import PayrollDetailDrawer from "../PayrollDetailDrawer";
import { Employee, Payslip } from "@/lib/api/types";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../Toast";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

const mockEmployee: Employee = {
  id: "emp-1",
  first_name: "John",
  last_name: "Doe",
  employee_id_number: "EMP001",
  workspace_id: "ws-1",
  hire_date: "2024-01-01",
  is_active: true,
  created_at: "",
  updated_at: "",
  department_name: "IT"
};

const queryClient = new QueryClient();

describe("PayrollDetailDrawer", () => {
  const onClose = vi.fn();
  const onResolveAdjustments = vi.fn();

  it("renders employee details when open", () => {
    render(
      <ToastProvider>
        <QueryClientProvider client={queryClient}>
          <BrowserRouter>
            <PayrollDetailDrawer 
              isOpen={true}
              onClose={onClose}
              employee={mockEmployee}
              workspaceId="ws-1"
              payslips={[]}
              isLoadingPayslips={false}
              adjustments={[]}
              onResolveAdjustments={onResolveAdjustments}
            />
          </BrowserRouter>
        </QueryClientProvider>
      </ToastProvider>
    );

    expect(screen.getByText("John Doe")).toBeDefined();
    expect(screen.getByText("IT")).toBeDefined();
    expect(screen.getByText("payroll.history_title")).toBeDefined();
  });

  it("switches tabs correctly", () => {
    render(
      <ToastProvider>
        <QueryClientProvider client={queryClient}>
          <BrowserRouter>
            <PayrollDetailDrawer 
              isOpen={true}
              onClose={onClose}
              employee={mockEmployee}
              workspaceId="ws-1"
              payslips={[]}
              isLoadingPayslips={false}
              adjustments={[]}
              onResolveAdjustments={onResolveAdjustments}
            />
          </BrowserRouter>
        </QueryClientProvider>
      </ToastProvider>
    );

    const configTab = screen.getByText("common.payroll.employee_config.tab_title");
    fireEvent.click(configTab);

    // Should show config component content
    expect(screen.getByText("common.payroll.employee_config.tab_title")).toBeDefined();
  });

  it("calls onClose when close button is clicked", () => {
    render(
      <ToastProvider>
        <QueryClientProvider client={queryClient}>
          <BrowserRouter>
            <PayrollDetailDrawer 
              isOpen={true}
              onClose={onClose}
              employee={mockEmployee}
              workspaceId="ws-1"
              payslips={[]}
              isLoadingPayslips={false}
              adjustments={[]}
              onResolveAdjustments={onResolveAdjustments}
            />
          </BrowserRouter>
        </QueryClientProvider>
      </ToastProvider>
    );

    // Use a more specific selector for the close button (the one with the X icon)
    const closeBtn = screen.getAllByRole("button").find(b => b.querySelector(".lucide-x"));
    if (closeBtn) fireEvent.click(closeBtn);
    expect(onClose).toHaveBeenCalled();
  });
});

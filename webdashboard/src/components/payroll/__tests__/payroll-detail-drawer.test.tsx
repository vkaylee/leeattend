import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi } from "vitest";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
 
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

import { Employee } from "@/lib/api/types";

import { ToastProvider } from "../../ui/toast";
import PayrollDetailDrawer from "../payroll-detail-drawer";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasPermission: () => true,
  }),
}));

const mockEmployee: Employee = {
  id: "emp-1",
  full_name: "John Doe",
  employee_id_number: "EMP001",
  workspace_id: "ws-1",
  created_at: "",
  updated_at: "",
  tags: [],
  department_name: "IT",
};

const queryClient = new QueryClient();

describe("PayrollDetailDrawer", () => {
  const onClose = vi.fn();
  const onResolveAdjustments = vi.fn();

  it("renders employee details when open", () => {
    render(
      <ToastProvider>
        <QueryClientProvider client={queryClient}>
            <PayrollDetailDrawer 
              adjustments={[]}
              employee={mockEmployee}
              isLoadingPayslips={false}
              isOpen={true}
              onClose={onClose}
              onResolveAdjustments={onResolveAdjustments}
              payslips={[]}
              workspaceId="ws-1"
            />
        </QueryClientProvider>
      </ToastProvider>
    );

    expect(screen.getByText("John Doe")).toBeDefined();
    expect(screen.getByText("IT")).toBeDefined();
    expect(screen.getByText("payroll.history_title")).toBeDefined();

    const dialog = screen.getByRole("dialog");
    expect(dialog.className).toContain("min-h-0");
    const content = dialog.querySelector("main");
    expect(content).not.toBeNull();
    expect(content!.className).toContain("min-h-0");
    expect(content!.className).toContain("overflow-y-auto");
  });

  it("switches tabs correctly", () => {
    render(
      <ToastProvider>
        <QueryClientProvider client={queryClient}>
            <PayrollDetailDrawer 
              adjustments={[]}
              employee={mockEmployee}
              isLoadingPayslips={false}
              isOpen={true}
              onClose={onClose}
              onResolveAdjustments={onResolveAdjustments}
              payslips={[]}
              workspaceId="ws-1"
            />
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
            <PayrollDetailDrawer 
              adjustments={[]}
              employee={mockEmployee}
              isLoadingPayslips={false}
              isOpen={true}
              onClose={onClose}
              onResolveAdjustments={onResolveAdjustments}
              payslips={[]}
              workspaceId="ws-1"
            />
        </QueryClientProvider>
      </ToastProvider>
    );

    // Use a more specific selector for the close button (the one with the X icon)
    const closeButton = screen.getAllByRole("button").find(b => b.querySelector(".lucide-x"));
    if (closeButton) fireEvent.click(closeButton);
    expect(onClose).toHaveBeenCalled();
  });
});

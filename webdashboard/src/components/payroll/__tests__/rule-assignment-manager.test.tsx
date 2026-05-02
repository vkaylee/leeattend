import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as departmentsApi from "@/lib/api/departments";
import * as payrollApi from "@/lib/api/payroll";

import RuleAssignmentManager from "../rule-assignment-manager";


// --- Mocks ---
vi.mock("@/lib/api/payroll");
vi.mock("@/lib/api/departments");
vi.mock("@/lib/api", async () => {
  const actual = await vi.importActual("@/lib/api");
  return {
    ...actual,
    searchEmployees: vi.fn(),
  };
});

// Avoid modal rendering in portals for simpler DOM queries
vi.mock("@/components/ui/modal", () => {
  return {
 
    default: ({ isOpen, children, title }: { isOpen: boolean; children: React.ReactNode; title: string }) => {
      if (!isOpen) return null;
      return (
        <div data-testid="mock-modal">
          <h2>{title}</h2>
          {children}
        </div>
      );
    },
  };
});

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback || key,
  }),
}));

describe("RuleAssignmentManager", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    vi.clearAllMocks();
  });

  const renderComponent = (props: { calculationMethod?: "percentage" | "fixed" | "tiered" } = {}) => {
    return render(
      <ToastProvider>
        <QueryClientProvider client={queryClient}>
          <RuleAssignmentManager ruleId="rule-1" wsId="ws-1" {...props} />
        </QueryClientProvider>
      </ToastProvider>
    );
  };

  it("renders empty state when there are no assignments", async () => {
    vi.mocked(payrollApi.fetchPayrollRuleAssignments).mockResolvedValueOnce([]);
    vi.mocked(departmentsApi.fetchDepartments).mockResolvedValueOnce([]);

    renderComponent();

    expect(await screen.findByText(/No assignments yet/i)).toBeInTheDocument();
  });

  it("renders existing assignments grouped by scope", async () => {
    vi.mocked(payrollApi.fetchPayrollRuleAssignments).mockResolvedValueOnce([
      {
        id: "a-1",
        workspace_id: "ws-1",
        rule_definition_id: "rule-1",
        assignee_type: "workspace",
        assignee_id: null,
        effective_from: "2026-01-01",
        created_at: "2026-01-01T00:00:00Z",
      },
      {
        id: "a-2",
        workspace_id: "ws-1",
        rule_definition_id: "rule-1",
        assignee_type: "department",
        assignee_id: "dept-1",
        effective_from: "2026-02-01",
        override_config: { rate: 10, base: "gross" },
        created_at: "2026-02-01T00:00:00Z",
      },
    ]);

    vi.mocked(departmentsApi.fetchDepartments).mockResolvedValueOnce([
      { id: "dept-1", workspace_id: "ws-1", name: "Engineering", active_employee_count: 5, created_at: "", updated_at: "" },
    ]);

    renderComponent();

    // Verify groups and items are rendered
    expect(await screen.findByText(/workspace level/i)).toBeInTheDocument();
    expect(await screen.findByText(/department level/i)).toBeInTheDocument();
    
    // Check specific assignment renders
    expect(await screen.findByText("Entire Workspace")).toBeInTheDocument();
    expect(await screen.findByText("Engineering")).toBeInTheDocument();
    
    // Check override config renders
    expect(screen.getByText(/10%/i)).toBeInTheDocument();
  });

  it("opens modal and can submit workspace level assignment", async () => {
    vi.mocked(payrollApi.fetchPayrollRuleAssignments).mockResolvedValueOnce([]);
    vi.mocked(departmentsApi.fetchDepartments).mockResolvedValueOnce([]);
 
    vi.mocked(payrollApi.createPayrollRuleAssignment).mockResolvedValueOnce({} as unknown as never);

    renderComponent();
    
    const user = userEvent.setup();
    
    // Open modal
    await user.click(await screen.findByRole("button", { name: /Assign/i }));
    
    expect(screen.getByTestId("mock-modal")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Assign Rule" })).toBeInTheDocument();

    // Select Workspace scope
    await user.click(screen.getByRole("button", { name: /workspace/i }));
    
    // Effective From has a default value (today)
    const effectiveFromInput = screen.getByLabelText(/Effective From/i);
    expect(effectiveFromInput).toHaveValue(new Date().toISOString().split("T")[0]);

    // Submit form
    const submitButton = screen.getByRole("button", { name: "Assign Rule" });
    await user.click(submitButton);

    await waitFor(() => {
      expect(payrollApi.createPayrollRuleAssignment).toHaveBeenCalledWith(
        "ws-1",
        "rule-1",
        expect.objectContaining({
          assignee_type: "workspace",
          assignee_id: undefined,
        })
      );
    });
  });

  it("renders override config inputs based on calculationMethod", async () => {
    vi.mocked(payrollApi.fetchPayrollRuleAssignments).mockResolvedValueOnce([]);
    vi.mocked(departmentsApi.fetchDepartments).mockResolvedValueOnce([]);

    renderComponent({ calculationMethod: "percentage" });
    
    const user = userEvent.setup();
    
    // Open modal
    await user.click(await screen.findByRole("button", { name: /Assign/i }));
    
    // Enable override
    const overrideToggle = screen.getByRole("checkbox");
    await user.click(overrideToggle);
    
    // Should show rate input for percentage method
    expect(await screen.findByText("Override Rate (%)")).toBeInTheDocument();
    expect(screen.getByText("Override Base")).toBeInTheDocument();
  });

  it("handles deletion flow correctly", async () => {
    vi.mocked(payrollApi.fetchPayrollRuleAssignments).mockResolvedValueOnce([
      {
        id: "a-1",
        workspace_id: "ws-1",
        rule_definition_id: "rule-1",
        assignee_type: "workspace",
        assignee_id: null,
        effective_from: "2026-01-01",
        created_at: "2026-01-01T00:00:00Z",
      },
    ]);
    vi.mocked(departmentsApi.fetchDepartments).mockResolvedValueOnce([]);
    vi.mocked(payrollApi.deletePayrollRuleAssignment).mockResolvedValueOnce();

    renderComponent();
    
    const user = userEvent.setup();
    
    // Find the assignment item (it should render the trash button icon inside)
    const deleteButton = await screen.findByRole("button", { name: /Delete assignment/i });
    
    // Click delete -> goes into confirm mode
    await user.click(deleteButton);
    
    // Should see Confirm and Cancel buttons
    const confirmButton = await screen.findByRole("button", { name: /Confirm/i });
    expect(screen.getByRole("button", { name: /Cancel/i })).toBeInTheDocument();
    
    // Click Confirm
    await user.click(confirmButton);
    
    await waitFor(() => {
      expect(payrollApi.deletePayrollRuleAssignment).toHaveBeenCalledWith("ws-1", "rule-1", "a-1");
    });
  });
});

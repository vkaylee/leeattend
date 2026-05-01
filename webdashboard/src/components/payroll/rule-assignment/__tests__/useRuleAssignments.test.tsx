import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as departmentsApi from "@/lib/api/departments";
import * as payrollApi from "@/lib/api/payroll";

import { useRuleAssignments } from "../useRuleAssignments";

// --- Mocks ---
vi.mock("@/lib/api/payroll");
vi.mock("@/lib/api/departments");

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback || key,
  }),
}));

const mockToastSuccess = vi.fn();
const mockToastError = vi.fn();
vi.mock("@/components/ui/Toast", () => ({
  useToast: () => ({
    success: mockToastSuccess,
    error: mockToastError,
  }),
}));

describe("useRuleAssignments", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    vi.clearAllMocks();
  });

  const getWrapper = () => {
    return ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };

  it("fetches and groups assignments correctly", async () => {
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
      {
        id: "dept-1",
        workspace_id: "ws-1",
        name: "Engineering",
        active_employee_count: 5,
        created_at: "",
        updated_at: "",
      },
    ]);

    const { result } = renderHook(() => useRuleAssignments("ws-1", "rule-1"), {
      wrapper: getWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.assignments).toHaveLength(2);
    expect(result.current.totalCount).toBe(2);
    expect(result.current.deptMap["dept-1"].name).toBe("Engineering");

    // Check grouping
    expect(result.current.grouped.workspace).toHaveLength(1);
    expect(result.current.grouped.department).toHaveLength(1);
    expect(result.current.grouped.employee).toHaveLength(0);
  });

  it("handles successful create mutation", async () => {
    vi.mocked(payrollApi.fetchPayrollRuleAssignments).mockResolvedValueOnce([]);
    vi.mocked(departmentsApi.fetchDepartments).mockResolvedValueOnce([]);
    vi.mocked(payrollApi.createPayrollRuleAssignment).mockResolvedValueOnce(
      {} as never
    );

    const { result } = renderHook(() => useRuleAssignments("ws-1", "rule-1"), {
      wrapper: getWrapper(),
    });

    result.current.createMutation.mutate({
      assignee_type: "workspace",
    });

    await waitFor(() => {
      expect(result.current.createMutation.isSuccess).toBe(true);
    });

    expect(payrollApi.createPayrollRuleAssignment).toHaveBeenCalledWith(
      "ws-1",
      "rule-1",
      { assignee_type: "workspace" }
    );
    expect(mockToastSuccess).toHaveBeenCalledWith("Success");
  });

  it("handles failed delete mutation", async () => {
    vi.mocked(payrollApi.fetchPayrollRuleAssignments).mockResolvedValueOnce([]);
    vi.mocked(departmentsApi.fetchDepartments).mockResolvedValueOnce([]);
    vi.mocked(payrollApi.deletePayrollRuleAssignment).mockRejectedValueOnce(
      new Error("API Error")
    );

    const { result } = renderHook(() => useRuleAssignments("ws-1", "rule-1"), {
      wrapper: getWrapper(),
    });

    result.current.deleteMutation.mutate("a-1");

    await waitFor(() => {
      expect(result.current.deleteMutation.isError).toBe(true);
    });

    expect(payrollApi.deletePayrollRuleAssignment).toHaveBeenCalledWith(
      "ws-1",
      "rule-1",
      "a-1"
    );
    expect(mockToastError).toHaveBeenCalledWith("API Error");
  });
});

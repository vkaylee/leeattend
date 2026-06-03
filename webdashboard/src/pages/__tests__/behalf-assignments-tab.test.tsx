import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useBehalfAssignments } from "@/hooks/use-behalf-assignments";
import { useAllEmployees } from "@/hooks/use-employees";
import { fetchDepartments } from "@/lib/api";

import { ToastProvider } from "../../components/ui/toast";
import { BehalfAssignmentsTab } from "../approvals/components/behalf-assignments-tab";

// Mock toast
const mockToastSuccess = vi.fn();
const mockToastError = vi.fn();
vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({
    success: mockToastSuccess,
    error: mockToastError,
  }),
  ToastProvider: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue: string }) => {
      const translations: Record<string, string> = {
        "common.loading": "Loading...",
      };
      return translations[key] || options?.defaultValue || key;
    },
    i18n: { language: "en" },
  }),
}));

// Mock EmployeeSearch
vi.mock("@/components/common/employee-search", () => ({
  EmployeeSearch: ({ id, value, onChange, placeholder }: any) => (
    <input
      data-testid={id}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      value={value}
    />
  ),
}));

// Mock Combobox
vi.mock("@/components/ui/combobox", () => ({
  Combobox: ({ id, value, onChange, options }: any) => (
    <select
      data-testid={id}
      onChange={(e) => onChange(e.target.value)}
      value={value}
    >
      <option value="">Select...</option>
      {options.map((opt: any) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  ),
}));

// Mock Modal (render inline to bypass portals in JSDOM)
vi.mock("@/components/ui/modal", () => ({
  default: ({ isOpen, children, title, onClose }: any) => {
    if (!isOpen) return null;
    return (
      <div data-testid="mock-modal">
        <div aria-level={2} role="heading">
          {title}
        </div>
        <div
          aria-label="Close"
          onClick={onClose}
          onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") onClose(); }}
          role="button"
          tabIndex={0}
        >
          X
        </div>
        {children}
      </div>
    );
  },
}));

// Mock hooks
vi.mock("@/hooks/use-behalf-assignments", () => ({
  useBehalfAssignments: vi.fn(),
}));

vi.mock("@/hooks/use-employees", () => ({
  useAllEmployees: vi.fn(),
}));

vi.mock("@/lib/api", () => ({
  fetchDepartments: vi.fn(),
}));

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>
    <ToastProvider>{children}</ToastProvider>
  </QueryClientProvider>
);

describe("BehalfAssignmentsTab", () => {
  const mockEmployees = [
    { id: "emp-1", full_name: "Alice Smith", employee_id_number: "E001" },
    { id: "emp-2", full_name: "Bob Jones", employee_id_number: "E002" },
    { id: "emp-3", full_name: "Charlie Brown", employee_id_number: "E003" },
  ];

  const mockDepartments = [
    { id: "dept-1", name: "Engineering", description: "Eng department" },
    { id: "dept-2", name: "Marketing", description: "Mkt department" },
  ];

  const mockAssignments = [
    {
      id: "assign-1",
      assignee_employee_id: "emp-1",
      target_employee_id: "emp-2",
      target_department_id: null,
    },
    {
      id: "assign-2",
      assignee_employee_id: "emp-1",
      target_employee_id: null,
      target_department_id: "dept-1",
    },
  ];

  const mockCreateAssignment = vi.fn();
  const mockDeleteAssignment = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    vi.mocked(useBehalfAssignments).mockReturnValue({
      assignments: [],
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
      createAssignment: mockCreateAssignment,
      isCreating: false,
      deleteAssignment: mockDeleteAssignment,
      isDeleting: false,
    });

    vi.mocked(useAllEmployees).mockReturnValue({
      employees: mockEmployees,
      employeesData: { data: mockEmployees, total: mockEmployees.length, page: 1, limit: 3000 },
      isLoading: false,
      isError: false,
      error: null,
      refetch: vi.fn(),
    });

    vi.mocked(fetchDepartments).mockResolvedValue(mockDepartments);
  });

  it("renders loading state when initial loading is true", () => {
    vi.mocked(useBehalfAssignments).mockReturnValue({
      assignments: [],
      isLoading: true,
      isError: false,
      refetch: vi.fn(),
      createAssignment: mockCreateAssignment,
      isCreating: false,
      deleteAssignment: mockDeleteAssignment,
      isDeleting: false,
    });

    render(<BehalfAssignmentsTab wsId="ws-1" />, { wrapper });
    expect(screen.getByText("Loading...")).toBeDefined();
  });

  it("renders empty state when no assignments exist", () => {
    render(<BehalfAssignmentsTab wsId="ws-1" />, { wrapper });
    expect(screen.getByText("No Behalf Assignments Configured")).toBeDefined();
  });

  it("renders assignments list", async () => {
    vi.mocked(useBehalfAssignments).mockReturnValue({
      assignments: mockAssignments,
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
      createAssignment: mockCreateAssignment,
      isCreating: false,
      deleteAssignment: mockDeleteAssignment,
      isDeleting: false,
    });

    render(<BehalfAssignmentsTab wsId="ws-1" />, { wrapper });

    await waitFor(() => {
      // Assignments claimant
      expect(screen.getAllByText("Alice Smith").length).toBe(2);

      // Employee target scope
      expect(screen.getByText("Employee")).toBeDefined();
      expect(screen.getByText("Bob Jones")).toBeDefined();
      expect(screen.getByText("(E002)")).toBeDefined();

      // Department target scope
      expect(screen.getByText("Department")).toBeDefined();
      expect(screen.getByText("Engineering")).toBeDefined();
    });
  });

  it("opens modal on Add Assignment click", async () => {
    const user = userEvent.setup();
    render(<BehalfAssignmentsTab wsId="ws-1" />, { wrapper });

    await user.click(screen.getByText("Add Assignment"));
    expect(screen.getByTestId("mock-modal")).toBeDefined();
    expect(screen.getByText("Add Behalf Assignment")).toBeDefined();
  });

  it("validates form inputs and shows error toasts", async () => {
    const user = userEvent.setup();
    render(<BehalfAssignmentsTab wsId="ws-1" />, { wrapper });

    await user.click(screen.getByText("Add Assignment"));

    const submitBtn = screen.getByText("Create Assignment");
    await user.click(submitBtn);

    expect(mockToastError).toHaveBeenCalledWith("Please select an assignee employee");

    // Fill in assignee
    const submitterInput = screen.getByTestId("submitter-select");
    fireEvent.change(submitterInput, { target: { value: "emp-1" } });

    await user.click(submitBtn);
    expect(mockToastError).toHaveBeenCalledWith("Please select a target employee");

    // Choose Department target scope type
    const deptRadio = screen.getByLabelText("Department");
    await user.click(deptRadio);

    await user.click(submitBtn);
    expect(mockToastError).toHaveBeenCalledWith("Please select a target department");
  });

  it("submits the form successfully for employee target", async () => {
    mockCreateAssignment.mockResolvedValue({});
    const user = userEvent.setup();
    render(<BehalfAssignmentsTab wsId="ws-1" />, { wrapper });

    await user.click(screen.getByText("Add Assignment"));

    const submitterInput = screen.getByTestId("submitter-select");
    fireEvent.change(submitterInput, { target: { value: "emp-1" } });

    const targetEmpInput = screen.getByTestId("target-employee-select");
    fireEvent.change(targetEmpInput, { target: { value: "emp-2" } });

    const submitBtn = screen.getByText("Create Assignment");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(mockCreateAssignment).toHaveBeenCalledWith({
        assignee_employee_id: "emp-1",
        target_employee_id: "emp-2",
        target_department_id: null,
      });
      expect(mockToastSuccess).toHaveBeenCalledWith("Behalf assignment created successfully");
      expect(screen.queryByTestId("mock-modal")).toBeNull();
    });
  });

  it("submits the form successfully for department target", async () => {
    mockCreateAssignment.mockResolvedValue({});
    const user = userEvent.setup();
    render(<BehalfAssignmentsTab wsId="ws-1" />, { wrapper });

    await user.click(screen.getByText("Add Assignment"));

    const submitterInput = screen.getByTestId("submitter-select");
    fireEvent.change(submitterInput, { target: { value: "emp-1" } });

    // Choose Department target scope type
    const deptRadio = screen.getByLabelText("Department");
    await user.click(deptRadio);

    const targetDeptSelect = screen.getByTestId("target-department-select");
    fireEvent.change(targetDeptSelect, { target: { value: "dept-1" } });

    const submitBtn = screen.getByText("Create Assignment");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(mockCreateAssignment).toHaveBeenCalledWith({
        assignee_employee_id: "emp-1",
        target_employee_id: null,
        target_department_id: "dept-1",
      });
      expect(mockToastSuccess).toHaveBeenCalledWith("Behalf assignment created successfully");
      expect(screen.queryByTestId("mock-modal")).toBeNull();
    });
  });

  it("calls deleteAssignment when trash button is clicked", async () => {
    vi.mocked(useBehalfAssignments).mockReturnValue({
      assignments: mockAssignments,
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
      createAssignment: mockCreateAssignment,
      isCreating: false,
      deleteAssignment: mockDeleteAssignment,
      isDeleting: false,
    });

    mockDeleteAssignment.mockResolvedValue({});
    const user = userEvent.setup();
    render(<BehalfAssignmentsTab wsId="ws-1" />, { wrapper });

    const deleteButtons = screen.getAllByRole("button");
    // The trash icons are in ghost buttons. Let's find buttons containing SVGs or specific classes
    // We have 1 Add Assignment button, 2 Trash buttons. So 3 buttons total.
    expect(deleteButtons.length).toBe(3);

    await user.click(deleteButtons[1]); // First trash button

    await waitFor(() => {
      expect(mockDeleteAssignment).toHaveBeenCalledWith("assign-1");
      expect(mockToastSuccess).toHaveBeenCalledWith("Assignment deleted successfully");
    });
  });
});

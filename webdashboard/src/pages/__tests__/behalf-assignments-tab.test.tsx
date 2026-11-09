import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import { useBehalfAssignments } from "../../hooks/use-behalf-assignments";
import { fetchAllOrganizationMembers, fetchDepartments } from "../../lib/api";
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
const { enCommon } = vi.hoisted(() => ({
  enCommon: {
    cancel: "Cancel",
    behalf_assignments: {
      scope_type_employee: "Employee",
      scope_type_department: "Department",
      scope_type_none: "None",
      unknown_employee: "Unknown Employee",
      assignee_labels: {
        ot: "Authorized Claimant",
        overtime_request: "Authorized Submitter",
        leave: "Authorized Requester",
        attendance_adjustment: "Authorized Requester",
      },
      target_labels: {
        ot: "Claim Target Scope",
        overtime_request: "Pre-OT Target Scope",
        leave: "Request Target Scope",
        attendance_adjustment: "Request Target Scope",
      },
      form: {
        assignee_label_ot: "Authorized Claimant (User)",
        assignee_label_overtime_request: "Authorized Submitter (User)",
        assignee_label_other: "Authorized Requester (User)",
        assignee_placeholder: "Search and select user...",
        assignee_help_ot: "Select the workspace user who is allowed to claim on behalf.",
        assignee_help_overtime_request: "Select the workspace user who is allowed to submit Pre-OT requests on behalf.",
        assignee_help_leave: "Select the workspace user who is allowed to submit leave requests on behalf.",
        assignee_help_adjustment: "Select the workspace user who is allowed to submit attendance adjustment requests on behalf.",
        no_assignees: "No workspace users found",
        target_type_label: "Target Scope Type",
        target_type_employee: "Employee",
        target_type_department: "Department",
        target_employee_label: "Target Employee",
        target_employee_placeholder: "Search and select target employee...",
        target_department_label: "Target Department",
        no_departments: "No departments found",
        target_department_placeholder: "Search and select target department...",
        submit_btn: "Create Assignment",
      },
      header: {
        title_ot: "Behalf Claim Assignments",
        title_overtime_request: "Behalf Pre-OT Request Assignments",
        title_leave: "Behalf Leave Request Assignments",
        desc_ot: "Configure which employees are authorized to claim overtime on behalf of specific employees or entire departments.",
        desc_overtime_request: "Configure which employees are authorized to request Pre-OT on behalf of specific employees or entire departments.",
        desc_leave: "Configure which employees are authorized to request leave on behalf of specific employees or entire departments.",
        add_btn: "Add Assignment",
      },
      empty: {
        title_ot: "No Behalf Assignments Configured",
        title_leave: "No Behalf Leave Assignments Configured",
        title_overtime_request: "No Behalf Pre-OT Assignments Configured",
        title_adjustment: "No Behalf Adjustment Assignments Configured",
        desc_ot: "Users with Claim on Behalf permission will fall back to their managed departments. Assign specific employees or departments to scope their claim access.",
        desc_leave: "Users with Submit on Behalf permission will fall back to their managed departments. Assign specific employees or departments to scope their leave request access.",
        desc_overtime_request: "Users with Pre-OT Create on Behalf permission will fall back to their managed departments. Assign specific employees or departments to scope their Pre-OT request access.",
        desc_adjustment: "Users with Submit on Behalf permission will fall back to their managed departments. Assign specific employees or departments to scope their adjustment access.",
      },
      modal: {
        add_title: "Add Behalf Assignment",
      },
      toasts: {
        select_assignee: "Please select an assignee user",
        select_target_employee: "Please select a target employee",
        select_target_department: "Please select a target department",
        create_success: "Behalf assignment created successfully",
        delete_success: "Assignment deleted successfully",
      },
    },
  },
}));

vi.mock("react-i18next", () => {
  const resolveKey = (key: string, object: Record<string, any>): string => {
    let result = key.split('.').reduce((o: any, index) => o?.[index], object) as string | undefined;
    if (!result && key.startsWith('common.')) {
      const fallbackKey = key.substring(7);
      result = fallbackKey.split('.').reduce((o: any, index) => o?.[index], object) as string | undefined;
    }
    return result || key;
  };

  return {
    useTranslation: () => ({
      t: (key: string) => {
        if (key === "common.loading") return "Loading...";
        return resolveKey(key, enCommon);
      },
      i18n: { language: "en" },
    }),
  };
});


// Mock EmployeeSearch
vi.mock("@/components/common/employee-search", () => ({
  EmployeeSearch: ({ id, value, onChange, placeholder, onSelectEmployee }: any) => {
    const mockEmployees = [
      { id: "emp-1", full_name: "Alice Smith", employee_id_number: "E001", user_id: "user-1" },
      { id: "emp-2", full_name: "Bob Jones", employee_id_number: "E002", user_id: "user-2" },
      { id: "emp-3", full_name: "Charlie Brown", employee_id_number: "E003", user_id: "user-3" },
    ];
    return (
      <input
        data-testid={id}
        onChange={(e) => {
          const val = e.target.value;
          onChange(val);
          const emp = mockEmployees.find((emp) => emp.id === val);
          if (emp && onSelectEmployee) {
            onSelectEmployee(emp);
          }
        }}
        placeholder={placeholder}
        value={value}
      />
    );
  },
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

vi.mock("@/lib/api", () => ({
  fetchAllOrganizationMembers: vi.fn(),
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
  const mockDepartments = [
    { id: "dept-1", name: "Engineering", description: "Eng department", active_employee_count: 5, created_at: "", updated_at: "", workspace_id: "ws-1" },
    { id: "dept-2", name: "Marketing", description: "Mkt department", active_employee_count: 5, created_at: "", updated_at: "", workspace_id: "ws-1" },
  ];

  const mockMembers = [
    { email: "alice@example.com", full_name: "Alice Smith", role_id: "role-1", user_id: "user-1", workspace_id: "ws-1" },
    { email: "bob@example.com", full_name: "Bob Jones", role_id: "role-1", user_id: "user-2", workspace_id: "ws-1" },
  ];

  const mockAssignments = [
    {
      id: "assign-1",
      assignee_user_id: "user-1",
      target_employee_id: "emp-2",
      target_employee_name: "Bob Jones",
      target_employee_id_number: "E002",
      target_department_id: null,
      target_department_name: null,
      assignment_type: "ot",
      created_at: "",
      updated_at: "",
      workspace_id: "ws-1",
    },
    {
      id: "assign-2",
      assignee_user_id: "user-1",
      target_employee_id: null,
      target_employee_name: null,
      target_employee_id_number: null,
      target_department_id: "dept-1",
      target_department_name: "Engineering",
      assignment_type: "ot",
      created_at: "",
      updated_at: "",
      workspace_id: "ws-1",
    },
  ];

  const mockCreateAssignment = vi.fn();
  const mockDeleteAssignment = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    const defaultHookReturn = {
      assignments: [],
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
      createAssignment: mockCreateAssignment,
      isCreating: false,
      deleteAssignment: mockDeleteAssignment,
      isDeleting: false,
    };

    vi.mocked(useBehalfAssignments).mockReturnValue(defaultHookReturn);

    vi.mocked(fetchDepartments).mockResolvedValue(mockDepartments);
    vi.mocked(fetchAllOrganizationMembers).mockResolvedValue(mockMembers);
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

    render(<BehalfAssignmentsTab />, { wrapper });
    expect(screen.getByText("Loading...")).toBeDefined();
  });

  it("renders empty state when no assignments exist", () => {
    render(<BehalfAssignmentsTab />, { wrapper });
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

    render(<BehalfAssignmentsTab />, { wrapper });

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
    render(<BehalfAssignmentsTab />, { wrapper });

    await user.click(screen.getByText("Add Assignment"));
    expect(screen.getByTestId("mock-modal")).toBeDefined();
    expect(screen.getByText("Add Behalf Assignment")).toBeDefined();
  });

  it("validates form inputs and shows error toasts", async () => {
    const user = userEvent.setup();
    render(<BehalfAssignmentsTab />, { wrapper });

    await user.click(screen.getByText("Add Assignment"));

    const submitBtn = screen.getByText("Create Assignment");
    await user.click(submitBtn);

    expect(mockToastError).toHaveBeenCalledWith("Please select an assignee user");

    // Fill in assignee
    const submitterInput = screen.getByTestId("submitter-select");
    fireEvent.change(submitterInput, { target: { value: "user-1" } });

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
    render(<BehalfAssignmentsTab />, { wrapper });

    await user.click(screen.getByText("Add Assignment"));

    const submitterInput = screen.getByTestId("submitter-select");
    fireEvent.change(submitterInput, { target: { value: "user-1" } });

    const targetEmpInput = screen.getByTestId("target-employee-select");
    fireEvent.change(targetEmpInput, { target: { value: "emp-2" } });

    const submitBtn = screen.getByText("Create Assignment");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(mockCreateAssignment).toHaveBeenCalledWith({
        assignee_user_id: "user-1",
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
    render(<BehalfAssignmentsTab />, { wrapper });

    await user.click(screen.getByText("Add Assignment"));

    const submitterInput = screen.getByTestId("submitter-select");
    fireEvent.change(submitterInput, { target: { value: "user-1" } });

    // Choose Department target scope type
    const deptRadio = screen.getByLabelText("Department");
    await user.click(deptRadio);

    const targetDeptSelect = screen.getByTestId("target-department-select");
    fireEvent.change(targetDeptSelect, { target: { value: "dept-1" } });

    const submitBtn = screen.getByText("Create Assignment");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(mockCreateAssignment).toHaveBeenCalledWith({
        assignee_user_id: "user-1",
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
    render(<BehalfAssignmentsTab />, { wrapper });

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

  describe("assignmentType=attendance_adjustment", () => {
    it("renders correct empty state text for attendance_adjustment type", () => {
      render(<BehalfAssignmentsTab assignmentType="attendance_adjustment" />, { wrapper });
      expect(screen.getByText("No Behalf Adjustment Assignments Configured")).toBeDefined();
    });

    it("uses useBehalfAssignments hook with 'attendance_adjustment' when type is attendance_adjustment", () => {
      const adjustmentCreateMock = vi.fn();
      vi.mocked(useBehalfAssignments).mockImplementation((type?: any) => {
        if (type === "attendance_adjustment") {
          return {
            assignments: mockAssignments,
            isLoading: false,
            isError: false,
            refetch: vi.fn(),
            createAssignment: adjustmentCreateMock,
            isCreating: false,
            deleteAssignment: vi.fn(),
            isDeleting: false,
          };
        }
        return {
          assignments: [],
          isLoading: false,
          isError: false,
          refetch: vi.fn(),
          createAssignment: vi.fn(),
          isCreating: false,
          deleteAssignment: vi.fn(),
          isDeleting: false,
        };
      });

      render(<BehalfAssignmentsTab assignmentType="attendance_adjustment" />, { wrapper });

      expect(useBehalfAssignments).toHaveBeenCalledWith("attendance_adjustment");
      expect(screen.getAllByText("Alice Smith").length).toBe(2);
    });

    it("renders correct empty state description for attendance_adjustment type", () => {
      render(<BehalfAssignmentsTab assignmentType="attendance_adjustment" />, { wrapper });
      expect(
        screen.getByText(/scope their adjustment access/)
      ).toBeDefined();
    });

    it("creates assignment via useBehalfAssignments with 'attendance_adjustment'", async () => {
      const adjustmentCreateMock = vi.fn().mockResolvedValue({});
      vi.mocked(useBehalfAssignments).mockImplementation((type?: any) => {
        if (type === "attendance_adjustment") {
          return {
            assignments: [],
            isLoading: false,
            isError: false,
            refetch: vi.fn(),
            createAssignment: adjustmentCreateMock,
            isCreating: false,
            deleteAssignment: vi.fn(),
            isDeleting: false,
          };
        }
        return {
          assignments: [],
          isLoading: false,
          isError: false,
          refetch: vi.fn(),
          createAssignment: vi.fn(),
          isCreating: false,
          deleteAssignment: vi.fn(),
          isDeleting: false,
        };
      });

      const user = userEvent.setup();
      render(<BehalfAssignmentsTab assignmentType="attendance_adjustment" />, { wrapper });

      await user.click(screen.getByText("Add Assignment"));

      const submitterInput = screen.getByTestId("submitter-select");
      fireEvent.change(submitterInput, { target: { value: "user-1" } });

      const targetEmpInput = screen.getByTestId("target-employee-select");
      fireEvent.change(targetEmpInput, { target: { value: "emp-2" } });

      const submitBtn = screen.getByText("Create Assignment");
      await user.click(submitBtn);

      await waitFor(() => {
        expect(adjustmentCreateMock).toHaveBeenCalledWith({
          assignee_user_id: "user-1",
          target_employee_id: "emp-2",
          target_department_id: null,
        });
        expect(mockToastSuccess).toHaveBeenCalledWith("Behalf assignment created successfully");
      });
    });
  });

  describe("assignmentType=leave", () => {
    it("renders correct empty state text for leave type", () => {
      render(<BehalfAssignmentsTab assignmentType="leave" />, { wrapper });
      expect(screen.getByText("No Behalf Leave Assignments Configured")).toBeDefined();
    });

    it("renders correct empty state description for leave type", () => {
      render(<BehalfAssignmentsTab assignmentType="leave" />, { wrapper });
      expect(
        screen.getByText(/scope their leave request access/)
      ).toBeDefined();
    });
  });

  describe("assignmentType=overtime_request", () => {
    it("renders correct empty state text for overtime_request type", () => {
      render(<BehalfAssignmentsTab assignmentType="overtime_request" />, { wrapper });
      expect(screen.getByText("No Behalf Pre-OT Assignments Configured")).toBeDefined();
    });

    it("renders correct empty state description for overtime_request type", () => {
      render(<BehalfAssignmentsTab assignmentType="overtime_request" />, { wrapper });
      expect(
        screen.getByText(/scope their Pre-OT request access/)
      ).toBeDefined();
    });

    it("uses useBehalfAssignments hook with 'overtime_request'", () => {
      render(<BehalfAssignmentsTab assignmentType="overtime_request" />, { wrapper });
      expect(useBehalfAssignments).toHaveBeenCalledWith("overtime_request");
    });
  });
});

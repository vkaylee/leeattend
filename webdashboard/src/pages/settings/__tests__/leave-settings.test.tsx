import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import { useAllEmployees } from "@/hooks/use-employees";
import * as departmentsApi from "@/lib/api/departments";
import * as helpers from "@/lib/api/helpers";
import * as api from "@/lib/api/leaves";

import LeaveSettings from "../leave-settings";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ 
    t: (key: string) => {
      const translations: Record<string, string> = {
        "settings.leave.tabs.types": "Leave Types",
        "settings.leave.tabs.policies": "Accrual Policies",
        "settings.leave.tabs.assignments": "Policy Assignments",
        "settings.leave.actions.assign": "Assign Policy",
        "settings.leave.actions.delete": "Delete",
        "common.edit": "Edit",
        "settings.leave.dialogs.delete_assignment_title": "Delete Policy Assignment",
        "settings.leave.dialogs.delete_assignment_message": "Are you sure you want to delete this leave policy assignment? This will soft-delete the assignment, and the previous timeline will be restored/healed automatically.",
        "settings.leave.messages.delete_assignment_success": "Leave policy assignment deleted successfully",
        "settings.leave.messages.delete_assignment_error": "Failed to delete leave policy assignment",
        "common.confirm": "Confirm",
        "common.cancel": "Cancel",
        "common.forever": "Forever",
        "common.important_note": "Important Note",
        "settings.leave.title": "Leave Settings",
        "settings.leave.desc": "Description",
        "leave_requests.policy.edit_assignment_title": "EDIT POLICY ASSIGNMENT",
        "leave_requests.policy.edit_assignment_subtitle": "Modify assignment rules and effective dates.",
        "leave_requests.policy.messages.update_assignment_success": "Leave policy assignment updated successfully!"
      };
      return translations[key] || key;
    }
  }),
}));

vi.mock("@/components/ui/toast", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
  useToast: () => ({ success: vi.fn(), error: vi.fn() }),
}));

vi.mock("@/lib/api/leaves");
vi.mock("@/lib/api/helpers", () => ({
  getStandaloneOrganizationId: vi.fn(() => "standalone"),
}));
vi.mock("@/hooks/use-employees", () => ({
  useAllEmployees: vi.fn(),
}));
vi.mock("@/lib/api/departments", () => ({
  fetchDepartments: vi.fn(),
}));
vi.mock("@/components/ui/toast", () => ({ useToast: () => ({ success: vi.fn(), error: vi.fn() }) }));


const mockAssignments = [
  {
    id: "assign-1",
    workspace_id: "ws-1",
    assignee_type: "workspace",
    assignee_id: null,
    policy_id: "pol-1",
    effective_from: "2026-01-01",
    effective_to: null,
    created_at: "2026-01-01T00:00:00Z",
    policy_name: "General Vacation",
    leave_type_name: "Vacation"
  },
  {
    id: "assign-2",
    workspace_id: "ws-1",
    assignee_type: "department",
    assignee_id: "dept-1",
    policy_id: "pol-1",
    effective_from: "2026-01-01",
    effective_to: null,
    created_at: "2026-01-01T00:00:00Z",
    policy_name: "Dept Policy",
    leave_type_name: "Vacation"
  },
  {
    id: "assign-3",
    workspace_id: "ws-1",
    assignee_type: "employee",
    assignee_id: "emp-1",
    policy_id: "pol-1",
    effective_from: "2026-01-01",
    effective_to: null,
    created_at: "2026-01-01T00:00:00Z",
    policy_name: "Employee Policy",
    leave_type_name: "Vacation"
  }
];

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

describe("LeaveSettings Page", () => {
  const wsId = "standalone";

  beforeEach(() => {
    vi.clearAllMocks();
 
 
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([] as never);
 
    vi.mocked(api.fetchLeavePolicies).mockResolvedValue([] as never);
 
    vi.mocked(api.fetchLeavePolicyAssignments).mockResolvedValue(mockAssignments);

    vi.mocked(useAllEmployees).mockReturnValue({
      employees: [{ id: "emp-1", full_name: "Alice Smith" }],
      isLoading: false,
    } as any);

    vi.mocked(departmentsApi.fetchDepartments).mockResolvedValue([
      { id: "dept-1", name: "Engineering" }
    ] as any);
  });

  it("renders all three main tabs", () => {
    render(
      <QueryClientProvider client={queryClient}>
        <LeaveSettings />
      </QueryClientProvider>
    );

    // Query buttons specifically to avoid duplicate text in headings/desc
    expect(screen.getByRole("button", { name: /Leave Types/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /Accrual Policies/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /Policy Assignments/i })).toBeDefined();
  });

  it("switches to assignments tab and shows content", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <LeaveSettings />
      </QueryClientProvider>
    );

    const assignTab = screen.getByRole("button", { name: /Policy Assignments/i });
    fireEvent.click(assignTab);

    // Check for the heading in the Assignments tab
    expect(await screen.findByRole("heading", { name: /Policy Assignments/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /Assign Policy/i })).toBeDefined();
  });

  it("renders assignments table with data and resolved target names", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <LeaveSettings />
      </QueryClientProvider>
    );

    const assignTab = screen.getByRole("button", { name: /Policy Assignments/i });
    fireEvent.click(assignTab);

    // Wait for data to load — use getAllByText since ResponsiveTable renders both mobile+desktop
    expect(await screen.findAllByText("General Vacation"));
    expect(screen.getAllByText("Dept Policy")[0]).toBeDefined();
    expect(screen.getAllByText("Employee Policy")[0]).toBeDefined();
    expect(screen.getAllByText("workspace")[0]).toBeDefined();
    expect(screen.getAllByText("department")[0]).toBeDefined();
    expect(screen.getAllByText("employee")[0]).toBeDefined();

    // Verify resolved department and employee names
    expect(screen.getAllByText("Engineering")[0]).toBeDefined();
    expect(screen.getAllByText("Alice Smith")[0]).toBeDefined();
  });

  it("allows deleting a policy assignment after confirming", async () => {
    vi.mocked(api.deleteLeavePolicyAssignment).mockResolvedValue(undefined);

    render(
      <QueryClientProvider client={queryClient}>
        <LeaveSettings />
      </QueryClientProvider>
    );

    const assignTab = screen.getByRole("button", { name: /Policy Assignments/i });
    fireEvent.click(assignTab);

    // Verify first assignment is rendered
    expect(await screen.findAllByText("General Vacation"));

    // Find and click the delete button for General Vacation (assign-1)
    const deleteButtons = screen.getAllByRole("button", { name: /Delete/i });
    fireEvent.click(deleteButtons[0]);

    // Confirm dialog should be open
    expect(screen.getByText(/Are you sure you want to delete this leave policy assignment/i)).toBeDefined();

    // Click confirm (which has label "Delete" inside the dialog)
    const dialogButtons = screen.getAllByRole("button", { name: /Delete/i });
    fireEvent.click(dialogButtons[dialogButtons.length - 1]);

    // Verify API call
    await waitFor(() => {
      expect(api.deleteLeavePolicyAssignment).toHaveBeenCalledWith(wsId, "assign-1");
    });
  });

  it("opens edit assignment modal when clicking edit button", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <LeaveSettings />
      </QueryClientProvider>
    );

    const assignTab = screen.getByRole("button", { name: /Policy Assignments/i });
    fireEvent.click(assignTab);

    // Wait for data to load
    expect(await screen.findAllByText("General Vacation"));

    // Find and click the edit button for General Vacation (assign-1)
    const editButtons = screen.getAllByRole("button", { name: /Edit/i });
    fireEvent.click(editButtons[0]);

    // Verify modal is open and has Edit Policy Assignment title
    expect(await screen.findByText("EDIT POLICY ASSIGNMENT")).toBeDefined();
  });
});

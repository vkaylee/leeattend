import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { AssignPolicyModal } from "../AssignPolicyModal";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "../../../lib/api";
import { ToastProvider } from "../../Toast";

// Mock the API library
vi.mock("../../../lib/api", () => ({
  fetchEmployees: vi.fn(),
  fetchDepartments: vi.fn(),
  fetchLeavePolicies: vi.fn(),
  assignLeavePolicy: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
}));

describe("AssignPolicyModal", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
    });
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "Alice", last_name: "Doe", employee_id_number: "EMP001" }
    ]);
    (api.fetchDepartments as any).mockResolvedValue([
      { id: "dept-1", name: "Engineering" }
    ]);
    (api.fetchLeavePolicies as any).mockResolvedValue([
      { id: "pol-1", name: "Standard Vacation", leave_type: "vacation" }
    ]);
  });

  const renderModal = (isOpen: boolean = true, onClose = vi.fn()) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <AssignPolicyModal isOpen={isOpen} onClose={onClose} wsId="test-ws-id" />
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("does not fetch data when closed", () => {
    renderModal(false);
    expect(api.fetchEmployees).not.toHaveBeenCalled();
  });

  it("fetches data when opened", async () => {
    renderModal(true);
    await waitFor(() => {
      expect(api.fetchEmployees).toHaveBeenCalledWith("test-ws-id");
      expect(api.fetchDepartments).toHaveBeenCalledWith("test-ws-id");
      expect(api.fetchLeavePolicies).toHaveBeenCalledWith("test-ws-id");
    });
  });

  it("allows selecting an employee and policy then submitting", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    (api.assignLeavePolicy as any).mockResolvedValue({});

    renderModal(true, onClose);

    // Wait for options to load and select Employee
    const empSelect = await screen.findByLabelText(/Nhân viên/i);
    await waitFor(() => {
      expect(empSelect.querySelector('option[value="emp-1"]')).not.toBeNull();
    });
    await user.selectOptions(empSelect, "emp-1");

    // Select Policy
    const policySelect = screen.getByLabelText(/Chính sách/i);
    await waitFor(() => {
      expect(policySelect.querySelector('option[value="pol-1"]')).not.toBeNull();
    });
    await user.selectOptions(policySelect, "pol-1");

    // Submit
    const submitBtn = screen.getByText(/Gán ngay|Assign Now/i);
    await user.click(submitBtn);

    await waitFor(() => {
      expect(api.assignLeavePolicy).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
        assignee_type: "employee",
        assignee_id: "emp-1",
        policy_id: "pol-1"
      }));
      expect(onClose).toHaveBeenCalled();
    });
  });

  it("allows selecting a department", async () => {
    const user = userEvent.setup();
    renderModal(true);

    // Switch to Department type
    const typeSelect = screen.getByLabelText(/Loại phép|Type/i);
    await user.selectOptions(typeSelect, "department");

    // Look for department select
    const deptSelect = await screen.findByLabelText(/Phòng ban|Department/i);
    await user.selectOptions(deptSelect, "dept-1");

    expect(deptSelect).toHaveValue("dept-1");
  });
});

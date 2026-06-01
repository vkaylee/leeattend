import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { useCreateApprovalWorkflow, useUpdateApprovalWorkflow } from "../../../../../hooks/use-approval-workflows";
import { useWorkspaceRoles, useWorkspaceMembers } from "../../../../../hooks/use-workspace";
import { UpsertWorkflowModal } from "../upsert-workflow-modal";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string }) => {
      if (options && typeof options === 'object' && options.defaultValue) return options.defaultValue;
      return key;
    },
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: "en",
    },
  }),
}));

// Mock toast notification
vi.mock("../../../../../components/ui/toast", () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}));

// Mock Tanstack Query hooks
vi.mock("../../../../../hooks/use-approval-workflows", () => ({
  useCreateApprovalWorkflow: vi.fn(),
  useUpdateApprovalWorkflow: vi.fn(),
}));

vi.mock("../../../../../hooks/use-workspace", () => ({
  useWorkspaceRoles: vi.fn(),
  useWorkspaceMembers: vi.fn(),
}));

// Mock Combobox to render a simple select box for easier testing
vi.mock("../../../../../components/ui/combobox", () => ({
  Combobox: ({ options, value, onChange, placeholder }: any) => (
    <select
      aria-label={placeholder}
      data-testid="mock-combobox"
      onChange={(e) => onChange(e.target.value)}
      value={value || ""}
    >
      <option value="">{placeholder}</option>
      {options.map((opt: any) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  ),
}));

describe("UpsertWorkflowModal", () => {
  let queryClient: QueryClient;
  const mockCreateWorkflow = vi.fn();
  const mockUpdateWorkflow = vi.fn();

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

    vi.mocked(useCreateApprovalWorkflow).mockReturnValue({
      mutateAsync: mockCreateWorkflow,
      isPending: false,
    } as any);

    vi.mocked(useUpdateApprovalWorkflow).mockReturnValue({
      mutateAsync: mockUpdateWorkflow,
      isPending: false,
    } as any);

    vi.mocked(useWorkspaceRoles).mockReturnValue({
      roles: [
        { id: "role-admin", name: "Administrator" },
        { id: "role-hr", name: "HR Manager" },
      ],
      isLoading: false,
    } as any);

    vi.mocked(useWorkspaceMembers).mockReturnValue({
      members: [
        { user_id: "user-1", email: "user1@test.com", full_name: "Alice Smith" },
        { user_id: "user-2", email: "user2@test.com", full_name: "Bob Jones" },
      ],
      isLoading: false,
    } as any);
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        {ui}
      </QueryClientProvider>
    );
  };

  it("renders correctly with default single step", () => {
    renderWithProviders(
      <UpsertWorkflowModal
        isOpen={true}
        onClose={vi.fn()}
        workflow={null}
        wsId="test-ws-id"
      />
    );

    expect(screen.getByLabelText(/^Name$/i)).toBeDefined();
    expect(screen.getByLabelText(/Entity Type/i)).toBeDefined();
    expect(screen.getByLabelText(/Description/i)).toBeDefined();
    expect(screen.getByText("Step 1")).toBeDefined();
    expect(screen.getByText("Department Manager Routing")).toBeDefined();
  });

  it("handles adding, shifting, and deleting steps", async () => {
    const user = userEvent.setup();
    renderWithProviders(
      <UpsertWorkflowModal
        isOpen={true}
        onClose={vi.fn()}
        workflow={null}
        wsId="test-ws-id"
      />
    );

    // Add another step
    const addStepButton = screen.getByRole("button", { name: /Add Step/i });
    await user.click(addStepButton);

    expect(screen.getByText("Step 2")).toBeDefined();

    // Check that we can move steps
    const step1NameInput = screen.getByLabelText("Step Name", { selector: "#step-name-0" }) as HTMLInputElement;
    const step2NameInput = screen.getByLabelText("Step Name", { selector: "#step-name-1" }) as HTMLInputElement;

    await user.clear(step1NameInput);
    await user.type(step1NameInput, "First Step");
    await user.clear(step2NameInput);
    await user.type(step2NameInput, "Second Step");

    // Move step 2 up (which makes it step 1)
    const moveUpButtons = screen.getAllByTitle("Move Up") as HTMLButtonElement[];
    // Only Step 2 has move up enabled, step 1 has it disabled
    expect(moveUpButtons[0].disabled).toBe(true); // index 0 (Step 1)
    expect(moveUpButtons[1].disabled).toBe(false); // index 1 (Step 2)

    await user.click(moveUpButtons[1]);

    // Values should swap
    expect((screen.getByLabelText("Step Name", { selector: "#step-name-0" }) as HTMLInputElement).value).toBe("Second Step");
    expect((screen.getByLabelText("Step Name", { selector: "#step-name-1" }) as HTMLInputElement).value).toBe("First Step");

    // Delete step 2
    const deleteButtons = screen.getAllByTitle("Delete Step") as HTMLButtonElement[];
    await user.click(deleteButtons[1]);

    expect(screen.queryByText("Step 2")).toBeNull();
  });

  it("transitions dynamic fields properly on approver type change", async () => {
    const user = userEvent.setup();
    renderWithProviders(
      <UpsertWorkflowModal
        isOpen={true}
        onClose={vi.fn()}
        workflow={null}
        wsId="test-ws-id"
      />
    );

    const typeSelect = screen.getByLabelText("Approver Type", { selector: "#step-approver-0" }) as HTMLSelectElement;

    // 1. Check Role selection
    await user.selectOptions(typeSelect, "role");
    expect(screen.getByLabelText("Select Role")).toBeDefined();
    
    // Select HR Manager
    const roleSelect = screen.getByLabelText("Select Role") as HTMLSelectElement;
    await user.selectOptions(roleSelect, "role-hr");
    expect(roleSelect.value).toBe("role-hr");

    // 2. Check Permission selection
    await user.selectOptions(typeSelect, "permission");
    expect(screen.getByLabelText("Select Permission")).toBeDefined();

    const permSelect = screen.getByLabelText("Select Permission") as HTMLSelectElement;
    await user.selectOptions(permSelect, "overtime:approve");
    expect(permSelect.value).toBe("overtime:approve");

    // 3. Check Specific User selection
    await user.selectOptions(typeSelect, "specific_user");
    const userSelect = screen.getByTestId("mock-combobox") as HTMLSelectElement;
    expect(userSelect).toBeDefined();
    
    await user.selectOptions(userSelect, "user-2");
    expect(userSelect.value).toBe("user-2");
  });

  it("submits correct payload format on workflow creation", async () => {
    const user = userEvent.setup();
    mockCreateWorkflow.mockResolvedValue({ id: "wf-123" });

    renderWithProviders(
      <UpsertWorkflowModal
        isOpen={true}
        onClose={vi.fn()}
        workflow={null}
        wsId="test-ws-id"
      />
    );

    // Fill in workflow metadata
    const nameInput = screen.getByLabelText(/^Name$/i);
    await user.type(nameInput, "Custom Overtime Approval");

    // Switch to role and select a role
    const typeSelect = screen.getByLabelText("Approver Type", { selector: "#step-approver-0" }) as HTMLSelectElement;
    await user.selectOptions(typeSelect, "role");
    
    const roleSelect = screen.getByLabelText("Select Role") as HTMLSelectElement;
    await user.selectOptions(roleSelect, "role-admin");

    // Submit
    const submitBtn = screen.getByRole("button", { name: /save/i });
    await user.click(submitBtn);

    await waitFor(() => {
      expect(mockCreateWorkflow).toHaveBeenCalledWith({
        name: "Custom Overtime Approval",
        entity_type: "ot_request",
        description: "",
        is_active: true,
        steps: [
          {
            step_order: 1,
            step_name: "L1 Approval",
            approver_type: "role",
            approver_config: { role_id: "role-admin" },
            action_required: "approve",
            can_skip: false,
          },
        ],
      });
    });
  });

  it("loads existing workflow values correctly in edit mode", () => {
    const existingWorkflow = {
      id: "wf-edit",
      name: "Existing Flow",
      entity_type: "leave_request",
      description: "Existing desc",
      is_active: false,
      steps: [
        {
          id: "step-edit-1",
          step_order: 1,
          step_name: "L1 Leave Step",
          approver_type: "specific_user",
          approver_config: { user_id: "user-1" },
          action_required: "review",
          can_skip: true,
        },
      ],
    };

    renderWithProviders(
      <UpsertWorkflowModal
        isOpen={true}
        onClose={vi.fn()}
        workflow={existingWorkflow}
        wsId="test-ws-id"
      />
    );

    expect((screen.getByLabelText(/^Name$/i) as HTMLInputElement).value).toBe("Existing Flow");
    expect((screen.getByLabelText(/Entity Type/i) as HTMLSelectElement).value).toBe("leave_request");
    expect((screen.getByLabelText(/Description/i) as HTMLInputElement).value).toBe("Existing desc");
    expect((screen.getByLabelText("Active Workflow") as HTMLInputElement).checked).toBe(false);

    expect((screen.getByLabelText("Step Name", { selector: "#step-name-0" }) as HTMLInputElement).value).toBe("L1 Leave Step");
    expect((screen.getByLabelText("Approver Type", { selector: "#step-approver-0" }) as HTMLSelectElement).value).toBe("specific_user");
    expect((screen.getByLabelText("Action", { selector: "#step-action-0" }) as HTMLSelectElement).value).toBe("review");
    expect((screen.getByLabelText("Can Skip if Not Found", { selector: "#skip-0" }) as HTMLInputElement).checked).toBe(true);

    const userSelect = screen.getByTestId("mock-combobox") as HTMLSelectElement;
    expect(userSelect.value).toBe("user-1");
  });
});

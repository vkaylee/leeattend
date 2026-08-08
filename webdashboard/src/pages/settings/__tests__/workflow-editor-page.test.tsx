import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useNavigate, useParams, useSearch } from "@tanstack/react-router";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import * as approvalHooks from "@/hooks/use-approval-workflows";
import * as workspaceHooks from "@/hooks/use-workspace";

import WorkflowEditorPage from "../workflow-editor-page";

vi.mock("@tanstack/react-router", () => ({
  useNavigate: vi.fn(),
  useParams: vi.fn(),
  useSearch: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: unknown) => {
      if (options && typeof options === "object" && "defaultValue" in options) {
        return (options as Record<string, unknown>).defaultValue as string;
      }
      if (typeof options === "string") {
        return options;
      }
      return key;
    },
  }),
}));

vi.mock("@/hooks/use-approval-workflows", () => ({
  useApprovalWorkflow: vi.fn(),
  useCreateApprovalWorkflow: vi.fn(),
  useUpdateApprovalWorkflow: vi.fn(),
}));

vi.mock("@/hooks/use-workspace", () => ({
  useWorkspace: vi.fn(),
  useWorkspaceRoles: vi.fn(),
  useWorkspaceMembers: vi.fn(),
  usePermissionCatalog: vi.fn(),
}));

vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}));

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

describe("WorkflowEditorPage", () => {
  const mockNavigate = vi.fn();
  const mockCreateMutate = vi.fn();
  const mockUpdateMutate = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(useNavigate).mockReturnValue(mockNavigate);
    vi.mocked(useParams).mockReturnValue({ workflowId: undefined });
    vi.mocked(useSearch).mockReturnValue({ tab: "overtime" });

    vi.mocked(workspaceHooks.useWorkspace).mockReturnValue({
      workspace: { id: "ws-123", name: "Test Workspace" },
      isLoading: false,
    } as any);

    vi.mocked(workspaceHooks.useWorkspaceRoles).mockReturnValue({
      roles: [
        { id: "role-1", name: "Admin" },
        { id: "role-2", name: "HR Manager" },
      ],
      isLoading: false,
    } as any);

    vi.mocked(workspaceHooks.useWorkspaceMembers).mockReturnValue({
      members: [
        { user_id: "user-1", email: "user1@example.com", full_name: "Alice" },
        { user_id: "user-2", email: "user2@example.com", full_name: "Bob" },
      ],
      isLoading: false,
    } as any);

    vi.mocked(workspaceHooks.usePermissionCatalog).mockReturnValue({
      catalog: [],
      isError: false,
      isLoading: false,
    });

    vi.mocked(approvalHooks.useApprovalWorkflow).mockReturnValue({
      data: undefined,
      isLoading: false,
    } as any);

    vi.mocked(approvalHooks.useCreateApprovalWorkflow).mockReturnValue({
      mutateAsync: mockCreateMutate,
      isPending: false,
    } as any);

    vi.mocked(approvalHooks.useUpdateApprovalWorkflow).mockReturnValue({
      mutateAsync: mockUpdateMutate,
      isPending: false,
    } as any);
  });

  it("renders new workflow page with default step", () => {
    render(
      <QueryClientProvider client={queryClient}>
        <WorkflowEditorPage />
      </QueryClientProvider>
    );

    expect(screen.getByText("New Workflow")).toBeDefined();
    expect(screen.getByLabelText("Name")).toBeDefined();
    expect(screen.getByText("Entity Type")).toBeDefined();
    expect(screen.getByText("Overtime Requests")).toBeDefined();
    expect(screen.getByLabelText("Step Name")).toBeDefined();
    expect(screen.getByText("Department Manager Routing")).toBeDefined();
  });

  it("allows adding and shifting steps", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <WorkflowEditorPage />
      </QueryClientProvider>
    );

    const addStepButton = screen.getByText("Add Step");
    fireEvent.click(addStepButton);

    const steps = screen.getAllByLabelText("Step Name");
    expect(steps.length).toBe(2);

    // Set name on second step
    fireEvent.change(steps[1], { target: { value: "L2 Approval" } });
    expect((steps[1] as HTMLInputElement).value).toBe("L2 Approval");

    // Click Move Up on Step 2
    const moveUpButtons = screen.getAllByTitle("Move Up");
    fireEvent.click(moveUpButtons[1]);

    const updatedSteps = screen.getAllByLabelText("Step Name");
    expect((updatedSteps[0] as HTMLInputElement).value).toBe("L2 Approval");
  });

  it("calls createWorkflow on form submit", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <WorkflowEditorPage />
      </QueryClientProvider>
    );

    fireEvent.change(screen.getByLabelText("Name"), { target: { value: "Overtime workflow" } });
    fireEvent.change(screen.getByLabelText("Step Name"), { target: { value: "Manager approval" } });

    const submitButton = screen.getByText("common.save");
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(mockCreateMutate).toHaveBeenCalledWith(
        expect.objectContaining({
          name: "Overtime workflow",
          entity_type: "ot_request",
          is_active: true,
          steps: [
            expect.objectContaining({
              step_order: 1,
              step_name: "Manager approval",
              approver_type: "department_manager",
            }),
          ],
        })
      );
    });
  });

  it("pre-fills and saves in edit mode", async () => {
    vi.mocked(useParams).mockReturnValue({ workflowId: "wf-1" });
    vi.mocked(approvalHooks.useApprovalWorkflow).mockReturnValue({
      data: {
        id: "wf-1",
        name: "Edit workflow",
        entity_type: "leave_request",
        description: "Test description",
        is_active: true,
        steps: [
          {
            id: "step-1",
            step_order: 1,
            step_name: "Leave approval",
            approver_type: "department_manager",
            approver_config: {},
            action_required: "approve",
            can_skip: false,
          },
        ],
      },
      isLoading: false,
    } as any);

    render(
      <QueryClientProvider client={queryClient}>
        <WorkflowEditorPage />
      </QueryClientProvider>
    );

    expect(screen.getByText("Edit Workflow")).toBeDefined();
    expect(screen.getByDisplayValue("Edit workflow")).toBeDefined();
    expect(screen.getByDisplayValue("Leave approval")).toBeDefined();

    const submitButton = screen.getByText("common.save");
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(mockUpdateMutate).toHaveBeenCalledWith(
        expect.objectContaining({
          name: "Edit workflow",
          entity_type: "leave_request",
        })
      );
    });
  });
});

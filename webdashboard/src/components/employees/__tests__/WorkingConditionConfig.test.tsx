import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/Toast";

import WorkingConditionConfig from "../WorkingConditionConfig";



// Mock API
const mockDelete = vi.fn().mockResolvedValue(undefined);
vi.mock("@/lib/api/employees", () => ({
  fetchWorkingConditions: vi.fn().mockResolvedValue([
    {
      id: "cond-1",
      type: "EARLY_LEAVE",
      minute_allowance: 60,
      is_paid: true,
      start_date: "2026-01-01"
    }
  ]),
  createWorkingCondition: vi.fn(),
  deleteWorkingCondition: (...args: any[]) => mockDelete(...args)
}));

// Mock translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      if (key === "common.confirm_delete") return "Delete policy?";
      if (key === "common.warning") return "Warning";
      if (key === "common.confirm") return "Confirm";
      if (key === "employees.policies.types.early_leave") return "Early Leave";
      return key;
    }
  })
}));

const renderComponent = (props: any) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <WorkingConditionConfig {...props} />
      </ToastProvider>
    </QueryClientProvider>
  );
};

describe("WorkingConditionConfig", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should show useConfirm dialog when deleting a condition", async () => {
    const user = userEvent.setup();
    renderComponent({
      wsId: "ws-1",
      employeeId: "emp-1"
    });

    // Wait for the mock condition to render
    expect(await screen.findByText("Early Leave")).toBeDefined();

    // Find the delete button. The icon is Trash2.
    // We can find the button by getting all buttons and picking the one that looks like delete
    const buttons = screen.getAllByRole("button");
    // The first button is "Add btn" or "Cancel", the second one is inside the condition
    const deleteBtn = buttons[buttons.length - 1];
    await user.click(deleteBtn);

    // Confirm dialog should appear
    expect(await screen.findByText("Warning")).toBeDefined();
    expect(await screen.findByText("Delete policy?")).toBeDefined();

    // Click confirm
    const confirmBtn = screen.getByRole("button", { name: "Confirm" });
    await user.click(confirmBtn);

    await waitFor(() => {
      expect(mockDelete).toHaveBeenCalledWith("ws-1", "emp-1", "cond-1");
    });
  });
});

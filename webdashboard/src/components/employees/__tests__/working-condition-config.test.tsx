import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";

import WorkingConditionConfig from "../working-condition-config";


// Mock useWorkingConditions hook
const { mockDeleteFn } = vi.hoisted(() => ({
  mockDeleteFn: vi.fn(),
}));

vi.mock("@/hooks/use-employees", () => ({
  useWorkingConditions: () => ({
    data: [
      {
        id: "cond-1",
        type: "REGULAR",
        minute_allowance: 60,
        is_paid: true,
        start_date: "2026-01-01",
      },
    ],
    isLoading: false,
  }),
  useEmployeeContract: () => ({ data: null, isLoading: false }),
}));

// Mock the condition types API call (via useQuery)
vi.mock("@/lib/api", async () => {
  const actual = await vi.importActual("@/lib/api");
  return {
    ...actual as any,
    fetchConditionTypes: vi.fn().mockResolvedValue([
      { id: "ct-1", code: "REGULAR", name_i18n: { en: "Regular", vi: "Chính thức" }, category: "working_hours", is_system: true },
    ]),
    createWorkingCondition: vi.fn().mockResolvedValue({ id: "cond-2", type_: "REGULAR", start_date: "2026-01-01", is_paid: true }),
    deleteWorkingCondition: vi.fn(),
  };
});

// Mock translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      if (key === "common.delete_confirm") return "Warning";
      if (key === "employees.policies.delete_warning") return "Delete policy?";
      if (key === "common.confirm") return "Confirm";
      if (key === "employees.policies.types.regular") return "Regular";
      return key;
    },
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: "en",
    },
  })
}));

 
const renderComponent = (props: {}) => {
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

  afterEach(() => {
    localStorage.clear();
  });

  it("should show useConfirm dialog when deleting a condition", async () => {
    const user = userEvent.setup();
    renderComponent({employeeId: "emp-1"
    });

    // Wait for the mock condition to render
    expect(await screen.findByText("Regular")).toBeDefined();

    // Find the delete button. The icon is Trash2.
    // We can find the button by getting all buttons and picking the one that looks like delete
    const buttons = screen.getAllByRole("button");
    // The first button is "Add btn" or "Cancel", the second one is inside the condition
    const deleteButton = buttons[buttons.length - 1];
    await user.click(deleteButton);

    // Confirm dialog should appear
    expect(await screen.findByText("Warning")).toBeDefined();
    expect(await screen.findByText("Delete policy?")).toBeDefined();

    // Click confirm
    const confirmButton = screen.getByRole("button", { name: "Confirm" });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(mockDeleteFn).toHaveBeenCalledWith("emp-1", "cond-1");
    });
  });
});

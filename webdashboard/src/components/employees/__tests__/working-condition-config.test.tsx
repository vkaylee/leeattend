import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";

import WorkingConditionConfig from "../working-condition-config";



// Mock API
const mockDelete = vi.fn().mockResolvedValue(undefined);
vi.stubGlobal("fetch", vi.fn((url: string, options?: RequestInit) => {
  let responseData: any = null;
  if (options?.method === "DELETE") {
    mockDelete("ws-1", "emp-1", "cond-1");
    responseData = null;
  } else if (options?.method === "POST") {
    responseData = { id: "cond-2", type_: "REGULAR", start_date: "2026-01-01", is_paid: true };
  } else {
    // GET
    responseData = {
      data: [
        {
          id: "cond-1",
          type_: "REGULAR",
          minute_allowance: 60,
          is_paid: true,
          start_date: "2026-01-01"
        }
      ]
    };
  }

  return Promise.resolve({
    ok: true,
    status: 200,
    headers: new Headers({ "content-type": "application/json" }),
    json: () => Promise.resolve(responseData),
    text: () => Promise.resolve(responseData ? JSON.stringify(responseData) : ""),
    clone: function() { return this; }
  } as unknown as Response);
}));


// Mock translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      if (key === "common.delete_confirm") return "Warning";
      if (key === "employees.policies.delete_warning") return "Delete policy?";
      if (key === "common.confirm") return "Confirm";
      if (key === "employees.policies.types.regular") return "Regular";
      return key;
    }
  })
}));

 
const renderComponent = (props: { wsId: string; employeeId: string }) => {
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
    localStorage.setItem("active_workspace_id", "ws-1");
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    localStorage.clear();
  });

  it("should show useConfirm dialog when deleting a condition", async () => {
    const user = userEvent.setup();
    renderComponent({
      wsId: "ws-1",
      employeeId: "emp-1"
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
      expect(mockDelete).toHaveBeenCalledWith("ws-1", "emp-1", "cond-1");
    });
  });
});

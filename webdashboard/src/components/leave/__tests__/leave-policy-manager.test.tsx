import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";

import { LeavePolicyManager } from "../leave-policy-manager";



const mockDelete = vi.fn().mockResolvedValue(undefined);
vi.mock("@/lib/api/leaves", () => ({
  fetchLeavePolicies: vi.fn().mockResolvedValue([
    {
      id: "policy-1",
      name: "Annual Leave",
      leave_type_id: "type-1",
      rule_config: { accrual_amount: 1, frequency: "monthly" }
    }
  ]),
  fetchLeaveTypes: vi.fn().mockResolvedValue([
    { id: "type-1", name: "Vacation" }
  ]),
  createLeavePolicy: vi.fn(),
  updateLeavePolicy: vi.fn(),
 
  deleteLeavePolicy: (wsId: string, policyId: string) => mockDelete(wsId, policyId) as Promise<void>
}));

// Mock translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      if (key === "leave.policy.confirm_delete") return "Delete this policy?";
      if (key === "common.warning") return "Warning";
      if (key === "common.confirm") return "Confirm";
      return key;
    }
  })
}));

 
const renderComponent = (props: { wsId: string }) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <LeavePolicyManager {...props} />
      </ToastProvider>
    </QueryClientProvider>
  );
};

describe("LeavePolicyManager", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should show useConfirm dialog when deleting a policy", async () => {
    const user = userEvent.setup();
    renderComponent({ wsId: "ws-1" });

    // Wait for data
    expect(await screen.findByText("Annual Leave")).toBeDefined();

    // Click delete
    const deleteButton = screen.getByTestId("delete-policy-btn");
    await user.click(deleteButton);

    // Confirm dialog should appear
    expect(await screen.findByText("Warning")).toBeDefined();
    expect(await screen.findByText("Delete this policy?")).toBeDefined();

    // Click confirm
    const confirmButton = screen.getByRole("button", { name: "Confirm" });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(mockDelete).toHaveBeenCalledWith("ws-1", "policy-1");
    });
  });
});

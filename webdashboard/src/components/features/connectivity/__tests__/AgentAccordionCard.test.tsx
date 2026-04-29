import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import React from "react";
import AgentAccordionCard from "../AgentAccordionCard";
import { ToastProvider } from "@/components/ui/Toast";
import { AuthorizedAgent } from "@/lib/api";

// Mock translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: any) => {
      if (key === "agents.messages.confirm_delete") return `Delete ${options?.name}?`;
      if (key === "agents.messages.confirm_revoke") return `Revoke ${options?.name}?`;
      if (key === "agents.messages.confirm_approve") return `Approve ${options?.name}?`;
      if (key === "common.confirm") return "Confirm";
      if (key === "common.warning") return "Warning";
      return key;
    }
  })
}));

const mockAgent: AuthorizedAgent = {
  id: "agent-1",
  name: "Test Agent",
  is_active: true,
  is_trusted: true,
  last_seen_at: "2026-01-01T00:00:00Z"
} as any;

const renderComponent = (props: any) => {
  return render(
    <ToastProvider>
      <AgentAccordionCard {...props} />
    </ToastProvider>
  );
};

describe("AgentAccordionCard", () => {
  const onDelete = vi.fn().mockResolvedValue(undefined);
  const onToggleTrust = vi.fn().mockResolvedValue(undefined);

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should handle delete flow using useConfirm modal", async () => {
    const user = userEvent.setup();
    renderComponent({
      agent: mockAgent,
      devices: [],
      isExpanded: false,
      onToggleExpand: vi.fn(),
      onEdit: vi.fn(),
      onToggleTrust,
      onDelete,
      onUpdateDevice: vi.fn(),
      onDeleteDevice: vi.fn(),
      isOnline: true
    });

    // Click delete button
    const deleteBtn = screen.getByTestId(`agent-delete-${mockAgent.id}`);
    await user.click(deleteBtn);

    // Verify modal appears
    expect(await screen.findByText("Warning")).toBeDefined();
    expect(await screen.findByText("Delete Test Agent?")).toBeDefined();

    // Click confirm in the modal
    const confirmBtn = screen.getByRole("button", { name: "Confirm" });
    await user.click(confirmBtn);

    // Verify onDelete was called
    await waitFor(() => {
      expect(onDelete).toHaveBeenCalledWith(mockAgent);
    });
  });

  it("should handle toggle trust flow using useConfirm modal", async () => {
    const user = userEvent.setup();
    renderComponent({
      agent: mockAgent,
      devices: [],
      isExpanded: false,
      onToggleExpand: vi.fn(),
      onEdit: vi.fn(),
      onToggleTrust,
      onDelete,
      onUpdateDevice: vi.fn(),
      onDeleteDevice: vi.fn(),
      isOnline: true
    });

    // Click toggle trust button
    const trustBtn = screen.getByTestId(`agent-trust-toggle-${mockAgent.id}`);
    await user.click(trustBtn);

    // Verify modal appears
    expect(await screen.findByText("Warning")).toBeDefined();
    expect(await screen.findByText("Revoke Test Agent?")).toBeDefined();

    // Click confirm in the modal
    const confirmBtn = screen.getByRole("button", { name: "Confirm" });
    await user.click(confirmBtn);

    // Verify onToggleTrust was called
    await waitFor(() => {
      expect(onToggleTrust).toHaveBeenCalledWith(mockAgent);
    });
  });
});

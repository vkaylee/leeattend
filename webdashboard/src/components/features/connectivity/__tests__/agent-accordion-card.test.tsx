import { render, screen, waitFor } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import { AuthorizedAgent, Device } from "@/lib/api";

import AgentAccordionCard from "../agent-accordion-card";

// Mock translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { name?: string }) => {
      if (key === "agents.messages.confirm_delete") return `Delete ${options?.name ?? ""}?`;
      if (key === "agents.messages.confirm_revoke") return `Revoke ${options?.name ?? ""}?`;
 
      if (key === "agents.messages.confirm_approve") return `Approve ${options?.name ?? ""}?`;
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
} as unknown as AuthorizedAgent;

const renderComponent = (props: {
 
  agent: AuthorizedAgent;
  devices: Device[];
 
  isExpanded: boolean;
  onToggleExpand: () => void;
  onEdit: () => void;
  onToggleTrust: (agent: AuthorizedAgent) => Promise<void>;
  onDelete: (agent: AuthorizedAgent) => Promise<void>;
  onUpdateDevice: (id: string, data: Partial<Device>) => Promise<void>;
  onDeleteDevice: (id: string) => Promise<void>;
  isOnline: boolean;
}) => {
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
    const deleteButton = screen.getByTestId(`agent-delete-${mockAgent.id ?? ""}`);
    await user.click(deleteButton);

    // Verify modal appears
    expect(await screen.findByText("Warning")).toBeDefined();
    expect(await screen.findByText("Delete Test Agent?")).toBeDefined();

    // Click confirm in the modal
    const confirmButton = screen.getByRole("button", { name: "Confirm" });
    await user.click(confirmButton);

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
    const trustButton = screen.getByTestId(`agent-trust-toggle-${mockAgent.id ?? ""}`);
    await user.click(trustButton);

    // Verify modal appears
    expect(await screen.findByText("Warning")).toBeDefined();
    expect(await screen.findByText("Revoke Test Agent?")).toBeDefined();

    // Click confirm in the modal
    const confirmButton = screen.getByRole("button", { name: "Confirm" });
    await user.click(confirmButton);

    // Verify onToggleTrust was called
    await waitFor(() => {
      expect(onToggleTrust).toHaveBeenCalledWith(mockAgent);
    });
  });
});

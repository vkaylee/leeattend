import { screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import AgentEditModal from "./AgentEditModal";
import * as api from "@/lib/api";
import { agentFactory } from "@/test/factories";
import { renderWithClient } from "@/test/utils";
import * as agentsHook from "@/hooks/use-agents";

// Mock the API
vi.mock("@/lib/api", async () => {
  const actual = await vi.importActual<typeof api>("@/lib/api");
  return {
    ...actual,
    updateAuthorizedAgent: vi.fn(),
  };
});

// Mock the Hook
vi.mock("@/hooks/use-agents", () => ({
  useAuthorizedAgents: vi.fn(),
}));

const mockAgent = agentFactory({
  id: "TEST_001",
  name: "Test Agent",
  token: "secret123",
  max_concurrent_workers: 3,
});

describe("AgentEditModal", () => {
  it("renders agent data when open", async () => {
    vi.mocked(agentsHook.useAuthorizedAgents).mockReturnValue({
      agents: [],
      isLoading: false,
      isFetching: false,
      error: null,
      refetch: vi.fn(),
      updateAgent: vi.fn() as any,
      isUpdating: false,
      deleteAgent: vi.fn() as any,
      isDeleting: false,
      includeInactive: false,
      setIncludeInactive: vi.fn(),
      createAgent: vi.fn() as any,
      isCreating: false,
      updateTrust: vi.fn() as any,
      isUpdatingTrust: false,
      sendCommand: vi.fn() as any,
      isSendingCommand: false,
    });

    renderWithClient(
      <AgentEditModal
        agent={mockAgent}
        isOpen={true}
        onClose={() => {}}
        onSave={() => {}}
      />,
    );

    expect(screen.getByText("Chỉnh sửa Agent")).toBeInTheDocument();
    expect(await screen.findByDisplayValue("Test Agent")).toBeInTheDocument();
    expect(screen.getByDisplayValue("secret123")).toBeInTheDocument();
    expect(screen.getByDisplayValue("3")).toBeInTheDocument();
    expect(screen.getByText("TEST_001")).toBeInTheDocument();
  });

  it("calls updateAgent and onSave when form is submitted", async () => {
    const onSave = vi.fn();
    const onClose = vi.fn();
    const updateAgentMock = vi
      .fn()
      .mockResolvedValue({ ...mockAgent, name: "New Name" });

    vi.mocked(agentsHook.useAuthorizedAgents).mockReturnValue({
      agents: [],
      isLoading: false,
      isFetching: false,
      error: null,
      refetch: vi.fn(),
      updateAgent: updateAgentMock as any,
      isUpdating: false,
      deleteAgent: vi.fn() as any,
      isDeleting: false,
      includeInactive: false,
      setIncludeInactive: vi.fn(),
      createAgent: vi.fn() as any,
      isCreating: false,
      updateTrust: vi.fn() as any,
      isUpdatingTrust: false,
      sendCommand: vi.fn() as any,
      isSendingCommand: false,
    });

    renderWithClient(
      <AgentEditModal
        agent={mockAgent}
        isOpen={true}
        onClose={onClose}
        onSave={onSave}
      />,
    );

    const nameInput = await screen.findByDisplayValue("Test Agent");
    fireEvent.change(nameInput, { target: { value: "New Name" } });

    const strategyInput = screen.getByDisplayValue("3");
    fireEvent.change(strategyInput, { target: { value: "10" } });

    const saveButton = screen.getByText(/LƯU THAY ĐỔI/i);
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(updateAgentMock).toHaveBeenCalledWith(
        expect.objectContaining({
          id: "TEST_001",
          agent: expect.objectContaining({
            name: "New Name",
            max_concurrent_workers: 10,
          }),
        }),
      );
      expect(onSave).toHaveBeenCalled();
      expect(onClose).toHaveBeenCalled();
    });
  });
});

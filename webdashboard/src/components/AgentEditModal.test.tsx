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
});

describe("AgentEditModal", () => {
  it("renders agent data when open", () => {
    vi.mocked(agentsHook.useAuthorizedAgents).mockReturnValue({
      agents: [],
      isLoading: false,
      updateAgent: vi.fn() as any,
      isUpdating: false,
      deleteAgent: vi.fn() as any,
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
    expect(screen.getByDisplayValue("Test Agent")).toBeInTheDocument();
    expect(screen.getByDisplayValue("secret123")).toBeInTheDocument();
    expect(screen.getByDisplayValue("TEST_001")).toBeDisabled();
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
      updateAgent: updateAgentMock as any,
      isUpdating: false,
      deleteAgent: vi.fn() as any,
    });

    renderWithClient(
      <AgentEditModal
        agent={mockAgent}
        isOpen={true}
        onClose={onClose}
        onSave={onSave}
      />,
    );

    const nameInput = screen.getByDisplayValue("Test Agent");
    fireEvent.change(nameInput, { target: { value: "New Name" } });

    const saveButton = screen.getByText("Lưu thay đổi");
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(updateAgentMock).toHaveBeenCalledWith(
        expect.objectContaining({
          id: "TEST_001",
          agent: expect.objectContaining({
            name: "New Name",
          }),
        }),
      );
      expect(onSave).toHaveBeenCalled();
      expect(onClose).toHaveBeenCalled();
    });
  });

  it("toggles trust and active states", () => {
    vi.mocked(agentsHook.useAuthorizedAgents).mockReturnValue({
      agents: [],
      isLoading: false,
      updateAgent: vi.fn() as any,
      isUpdating: false,
      deleteAgent: vi.fn() as any,
    });

    renderWithClient(
      <AgentEditModal
        agent={mockAgent}
        isOpen={true}
        onClose={() => {}}
        onSave={() => {}}
      />,
    );

    const trustButton = screen.getByText("Đã duyệt");
    const activeButton = screen.getByText("Hoạt động");

    fireEvent.click(trustButton);
    expect(screen.getByText("Chờ duyệt")).toBeInTheDocument();

    fireEvent.click(activeButton);
    expect(screen.getByText("Tạm dừng")).toBeInTheDocument();
  });

  it("does not render when isOpen is false", () => {
    const { container } = renderWithClient(
      <AgentEditModal
        agent={mockAgent}
        isOpen={false}
        onClose={() => {}}
        onSave={() => {}}
      />,
    );

    expect(container.firstChild).toBeNull();
  });
});

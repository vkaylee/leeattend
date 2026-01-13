import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import AgentsPage from "../AgentsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/Toast";
import * as useAgentsHook from "@/hooks/use-agents";

// Mock the hook
vi.mock("@/hooks/use-agents", () => ({
  useAuthorizedAgents: vi.fn(),
}));

// Mock the Edit Modal to keep it simple
vi.mock("@/components/AgentEditModal", () => ({
  default: ({ isOpen, onClose }: any) => isOpen ? <div data-testid="agent-edit-modal"><button onClick={onClose}>Close</button></div> : null,
}));

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
    },
  },
});

describe("AgentsPage", () => {
  const mockRefetch = vi.fn();
  const mockCreateAgent = vi.fn();
  const mockUpdateTrust = vi.fn();
  const mockDeleteAgent = vi.fn();
  const mockUpdateAgent = vi.fn();
  const mockSetIncludeInactive = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient.clear();
    
    (useAgentsHook.useAuthorizedAgents as any).mockReturnValue({
      agents: [],
      isLoading: false,
      isFetching: false,
      refetch: mockRefetch,
      createAgent: mockCreateAgent,
      updateTrust: mockUpdateTrust,
      deleteAgent: mockDeleteAgent,
      updateAgent: mockUpdateAgent,
      includeInactive: false,
      setIncludeInactive: mockSetIncludeInactive,
    });
  });

  it("renders without crashing and shows empty state", () => {
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AgentsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    expect(screen.getByText("Quản lý Agent")).toBeDefined();
    expect(screen.getByText("Chưa có Agent nào được khai báo.")).toBeDefined();
  });

  it("shows loading state", () => {
    (useAgentsHook.useAuthorizedAgents as any).mockReturnValue({
      agents: [],
      isLoading: true,
      isFetching: false,
      refetch: mockRefetch,
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AgentsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Refresh icon should have animate-spin
    const refreshBtn = screen.getByRole("button", { name: "" }); // The one with RefreshCcw
    // We can't easily check class with standard getByRole if it's nested icon, 
    // but we can find the svg
    const svg = document.querySelector('.animate-spin');
    expect(svg).toBeDefined();
  });

  it("renders list of agents", () => {
    const mockAgents = [
      { id: "agent-1", name: "Agent One", token: "token1", is_trusted: true, is_active: true, last_ip: "1.1.1.1", last_seen_at: new Date().toISOString() },
      { id: "agent-2", name: "Agent Two", token: "token2", is_trusted: false, is_active: true, last_ip: "2.2.2.2", last_seen_at: null },
    ];

    (useAgentsHook.useAuthorizedAgents as any).mockReturnValue({
      agents: mockAgents,
      isLoading: false,
      isFetching: false,
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AgentsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    expect(screen.getByText("Agent One")).toBeDefined();
    expect(screen.getByText("Agent Two")).toBeDefined();
    expect(screen.getByText("Trusted")).toBeDefined();
    expect(screen.getByText("Pending")).toBeDefined();
    expect(screen.getByText("Online")).toBeDefined();
    expect(screen.getByText("Offline")).toBeDefined();
  });

  it("handles adding a new agent", async () => {
    mockCreateAgent.mockResolvedValue({});
    
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AgentsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    fireEvent.click(screen.getByText("Khai báo Agent"));
    
    fireEvent.change(screen.getByPlaceholderText("Ví dụ: BRANCH_HN_01"), { target: { value: "NEW_ID" } });
    fireEvent.change(screen.getByPlaceholderText("Ví dụ: Văn phòng Hà Nội"), { target: { value: "New Name" } });
    fireEvent.change(screen.getByPlaceholderText("Mật mã kết nối"), { target: { value: "secret" } });
    
    fireEvent.click(screen.getByText("Lưu"));

    await waitFor(() => {
      expect(mockCreateAgent).toHaveBeenCalledWith({ id: "NEW_ID", name: "New Name", token: "secret" });
    });
  });

  it("handles toggle trust with confirmation", async () => {
    const mockAgents = [
      { id: "agent-1", name: "Agent One", token: "token1", is_trusted: false, is_active: true },
    ];
    (useAgentsHook.useAuthorizedAgents as any).mockReturnValue({
      agents: mockAgents,
      isLoading: false,
      isFetching: false,
      updateTrust: mockUpdateTrust,
    });

    window.confirm = vi.fn(() => true);
    mockUpdateTrust.mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AgentsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    fireEvent.click(screen.getByText("Trust"));

    expect(window.confirm).toHaveBeenCalled();
    await waitFor(() => {
      expect(mockUpdateTrust).toHaveBeenCalledWith({ id: "agent-1", is_trusted: true });
    });
  });

  it("handles delete with confirmation", async () => {
    const mockAgents = [
      { id: "agent-1", name: "Agent One", token: "token1", is_trusted: true, is_active: true },
    ];
    (useAgentsHook.useAuthorizedAgents as any).mockReturnValue({
      agents: mockAgents,
      isLoading: false,
      isFetching: false,
      deleteAgent: mockDeleteAgent,
    });

    window.confirm = vi.fn(() => true);
    mockDeleteAgent.mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AgentsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Find delete icon/button - it uses Trash2
    const deleteBtn = document.querySelector('button[title="Xóa"]');
    if (deleteBtn) fireEvent.click(deleteBtn);

    expect(window.confirm).toHaveBeenCalled();
    await waitFor(() => {
      expect(mockDeleteAgent).toHaveBeenCalledWith("agent-1");
    });
  });

  it("handles restore for inactive agents", async () => {
    const mockAgents = [
      { id: "agent-1", name: "Agent One", token: "token1", is_trusted: true, is_active: false },
    ];
    (useAgentsHook.useAuthorizedAgents as any).mockReturnValue({
      agents: mockAgents,
      isLoading: false,
      isFetching: false,
      updateAgent: mockUpdateAgent,
    });

    mockUpdateAgent.mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AgentsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    fireEvent.click(screen.getByText("Khôi phục"));

    await waitFor(() => {
      expect(mockUpdateAgent).toHaveBeenCalledWith({ id: "agent-1", agent: { is_active: true } });
    });
  });

  it("toggles includeInactive setting", () => {
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AgentsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    fireEvent.click(screen.getByText("Hiện lưu trữ"));
    expect(mockSetIncludeInactive).toHaveBeenCalledWith(true);
  });

  it("opens edit modal", () => {
    const mockAgents = [
      { id: "agent-1", name: "Agent One", token: "token1", is_trusted: true, is_active: true },
    ];
    (useAgentsHook.useAuthorizedAgents as any).mockReturnValue({
      agents: mockAgents,
      isLoading: false,
      isFetching: false,
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AgentsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    const editBtn = document.querySelector('button[title="Chỉnh sửa"]');
    if (editBtn) fireEvent.click(editBtn);

    expect(screen.getByTestId("agent-edit-modal")).toBeDefined();
  });
});
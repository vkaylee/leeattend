import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import AgentsPage from "../AgentsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

// Mock the hook
vi.mock("@/hooks/use-agents", () => ({
  useAuthorizedAgents: vi.fn(() => ({
    agents: [],
    isLoading: false,
    isFetching: false,
    refetch: vi.fn(),
    createAgent: vi.fn(),
    updateTrust: vi.fn(),
    deleteAgent: vi.fn(),
  })),
}));

// Mock the Edit Modal to keep it simple
vi.mock("@/components/AgentEditModal", () => ({
  default: () => <div data-testid="agent-edit-modal">Mock Modal</div>,
}));

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
    },
  },
});

describe("AgentsPage Smoke Test", () => {
  it("renders without crashing", () => {
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AgentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    // Kiểm tra tiêu đề trang tồn tại
    expect(screen.getByText("Quản lý Agent")).toBeDefined();
    // Kiểm tra nút thêm agent tồn tại
    expect(screen.getByText("Khai báo Agent")).toBeDefined();
    // Kiểm tra nút Hiện lưu trữ tồn tại
    expect(screen.getByText("Hiện lưu trữ")).toBeDefined();
  });

  it("shows empty state when no agents are present", () => {
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AgentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    expect(screen.getByText("Chưa có Agent nào được khai báo.")).toBeDefined();
  });
});

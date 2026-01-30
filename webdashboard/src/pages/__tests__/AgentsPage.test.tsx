import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import AgentsPage from "../AgentsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import React from "react";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchAgents: vi.fn(),
  createAgent: vi.fn(),
  deleteAgent: vi.fn(),
  updateAgentTrust: vi.fn(),
  updateAgent: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
}));

// Mock the hooks
vi.mock("@/hooks/use-agents", () => ({
  useAuthorizedAgents: () => ({
    agents: [
      { id: "agent-1", name: "Test Agent", is_trusted: true, is_active: true, last_heartbeat: new Date().toISOString() }
    ],
    isLoading: false,
    isFetching: false,
    refetch: vi.fn(),
    createAgent: vi.fn(),
    updateTrust: vi.fn(),
    deleteAgent: vi.fn(),
    updateAgent: vi.fn(),
    includeInactive: false,
    setIncludeInactive: vi.fn(),
  }),
}));

// Override i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("AgentsPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
    });
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            {ui}
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders list of agents", async () => {
    renderWithProviders(<AgentsPage />);
    expect(await screen.findByText("Test Agent")).toBeDefined();
    expect(screen.getByText(/ID: agent-1/i)).toBeDefined();
  });

  it("opens edit modal", async () => {
    const user = userEvent.setup();
    renderWithProviders(<AgentsPage />);
    
    await screen.findByText("Test Agent");
    const editBtn = screen.getByTitle(/common.edit/i);
    await user.click(editBtn);

    expect(await screen.findByText(/common.agents.edit_modal.title/i)).toBeDefined();
  });
});

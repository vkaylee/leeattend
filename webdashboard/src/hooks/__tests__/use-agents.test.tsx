import { renderHook, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { useAuthorizedAgents } from "../use-agents";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "@/lib/api";

// Mock the API module
vi.mock("@/lib/api", async () => {
  const actual = await vi.importActual("@/lib/api");
  return {
    ...actual,
    fetchAuthorizedAgents: vi.fn(),
    updateAuthorizedAgent: vi.fn(),
  };
});

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

describe("useAuthorizedAgents hook", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches only active agents by default", async () => {
    const mockAgents = [{ id: "A1", is_active: true }];
    (api.fetchAuthorizedAgents as any).mockResolvedValue(mockAgents);

    const { result } = renderHook(() => useAuthorizedAgents(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(api.fetchAuthorizedAgents).toHaveBeenCalledWith(false);
    expect(result.current.agents).toEqual(mockAgents);
  });

  it("fetches all agents when includeInactive is true", async () => {
    (api.fetchAuthorizedAgents as any).mockResolvedValue([]);

    const { result } = renderHook(() => useAuthorizedAgents(), {
      wrapper: createWrapper(),
    });

    // 1. Initial call should be with false
    await waitFor(() => expect(api.fetchAuthorizedAgents).toHaveBeenCalledWith(false));

    // 2. Toggle includeInactive
    result.current.setIncludeInactive(true);

    // 3. Wait for the re-fetch with true
    await waitFor(() => {
      expect(api.fetchAuthorizedAgents).toHaveBeenCalledWith(true);
    });
  });

  it("calls updateAuthorizedAgent when restoring an agent", async () => {
    const { result } = renderHook(() => useAuthorizedAgents(), {
      wrapper: createWrapper(),
    });

    await result.current.updateAgent({
      id: "A1",
      agent: { is_active: true },
    });

    expect(api.updateAuthorizedAgent).toHaveBeenCalledWith("A1", {
      is_active: true,
    });
  });
});
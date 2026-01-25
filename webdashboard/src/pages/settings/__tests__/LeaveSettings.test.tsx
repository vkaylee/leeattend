import { render, screen, fireEvent } from "@testing-library/react";
import LeaveSettings from "../LeaveSettings";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { vi, describe, it, expect, beforeEach } from "vitest";
import * as api from "@/lib/api/leaves";
import * as helpers from "@/lib/api/helpers";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ 
    t: (key: string) => {
      if (key.includes("types")) return "Leave Types";
      if (key.includes("policies")) return "Accrual Policies";
      return key;
    }
  }),
}));

vi.mock("@/lib/api/leaves");
vi.mock("@/lib/api/helpers");
vi.mock("@/components/Toast", () => ({ useToast: () => ({ success: vi.fn(), error: vi.fn() }) }));

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

describe("LeaveSettings Page", () => {
  const wsId = "test-ws-id";

  beforeEach(() => {
    vi.clearAllMocks();
    (helpers.getCurrentWorkspaceId as any).mockReturnValue(wsId);
    (api.fetchLeaveTypes as any).mockResolvedValue([]);
    (api.fetchLeavePolicies as any).mockResolvedValue([]);
  });

  it("renders all three main tabs", () => {
    render(
      <QueryClientProvider client={queryClient}>
        <LeaveSettings />
      </QueryClientProvider>
    );

    // Query buttons specifically to avoid duplicate text in headings/desc
    expect(screen.getByRole("button", { name: /Leave Types/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /Accrual Policies/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /Assignments/i })).toBeDefined();
  });

  it("switches to assignments tab and shows content", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <LeaveSettings />
      </QueryClientProvider>
    );

    const assignTab = screen.getByRole("button", { name: /Assignments/i });
    fireEvent.click(assignTab);

    // Check for the heading in the Assignments tab
    expect(await screen.findByRole("heading", { name: /Policy Assignments/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /Assign Policy/i })).toBeDefined();
  });
});

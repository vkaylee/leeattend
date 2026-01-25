import { render, screen, fireEvent } from "@testing-library/react";
import LeaveSettings from "../LeaveSettings";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { vi, describe, it, expect, beforeEach } from "vitest";
import * as api from "@/lib/api/leaves";
import * as helpers from "@/lib/api/helpers";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ 
    t: (key: string) => {
      const translations: Record<string, string> = {
        "settings.leave.tabs.types": "Leave Types",
        "settings.leave.tabs.policies": "Accrual Policies",
        "settings.leave.tabs.assignments": "Policy Assignments",
        "settings.leave.actions.assign": "Assign Policy",
        "common.forever": "Forever",
        "common.important_note": "Important Note",
        "settings.leave.title": "Leave Settings",
        "settings.leave.desc": "Description"
      };
      return translations[key] || key;
    }
  }),
}));

vi.mock("@/lib/api/leaves");
vi.mock("@/lib/api/helpers");
vi.mock("@/components/Toast", () => ({ useToast: () => ({ success: vi.fn(), error: vi.fn() }) }));


const mockAssignments = [
  {
    id: "assign-1",
    workspace_id: "ws-1",
    assignee_type: "workspace",
    assignee_id: null,
    policy_id: "pol-1",
    effective_from: "2026-01-01",
    effective_to: null,
    created_at: "2026-01-01T00:00:00Z",
    policy_name: "General Vacation",
    leave_type_name: "Vacation"
  }
];

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

describe("LeaveSettings Page", () => {
  const wsId = "test-ws-id";

  beforeEach(() => {
    vi.clearAllMocks();
    (helpers.getCurrentWorkspaceId as any).mockReturnValue(wsId);
    (api.fetchLeaveTypes as any).mockResolvedValue([]);
    (api.fetchLeavePolicies as any).mockResolvedValue([]);
    (api.fetchLeavePolicyAssignments as any).mockResolvedValue(mockAssignments);
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
    expect(screen.getByRole("button", { name: /Policy Assignments/i })).toBeDefined();
  });

  it("switches to assignments tab and shows content", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <LeaveSettings />
      </QueryClientProvider>
    );

    const assignTab = screen.getByRole("button", { name: /Policy Assignments/i });
    fireEvent.click(assignTab);

    // Check for the heading in the Assignments tab
    expect(await screen.findByRole("heading", { name: /Policy Assignments/i })).toBeDefined();
    expect(screen.getByRole("button", { name: /Assign Policy/i })).toBeDefined();
  });

  it("renders assignments table with data", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <LeaveSettings />
      </QueryClientProvider>
    );

    const assignTab = screen.getByRole("button", { name: /Policy Assignments/i });
    fireEvent.click(assignTab);

    // Wait for data to load
    expect(await screen.findByText("General Vacation")).toBeDefined();
    expect(screen.getByText("Vacation")).toBeDefined();
    expect(screen.getByText("workspace")).toBeDefined();
  });
});

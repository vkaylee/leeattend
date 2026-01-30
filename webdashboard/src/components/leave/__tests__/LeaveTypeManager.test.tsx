import { render, screen } from "@testing-library/react";
import { LeaveTypeManager } from "../LeaveTypeManager";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { vi, describe, it, expect, beforeEach } from "vitest";
import * as api from "@/lib/api/leaves";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string, fallback: string) => fallback || key }),
}));

vi.mock("@/lib/api/leaves");
vi.mock("@/components/ui/Toast", () => ({ useToast: () => ({ success: vi.fn(), error: vi.fn() }) }));

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

describe("LeaveTypeManager", () => {
  const wsId = "test-ws-id";
  const mockLeaveTypes = [{ id: "1", code: "AL", name: "Annual Leave", is_paid: true, policy_config: {} }];

  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchLeaveTypes as any).mockResolvedValue(mockLeaveTypes);
  });

  it("renders leave types list", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <LeaveTypeManager wsId={wsId} />
      </QueryClientProvider>
    );

    // Find the heading specifically to avoid duplicate text issues
    expect(await screen.findByRole("heading", { name: /Annual Leave/i })).toBeDefined();
    expect(screen.getByText(/Code: AL/i)).toBeDefined();
  });
});

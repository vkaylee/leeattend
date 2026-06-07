import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { vi, describe, it, expect, beforeEach } from "vitest";
import React from "react";

import * as api from "@/lib/api";
import TimeSettings from "../time-settings";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ 
    t: (key: string) => key
  }),
}));

vi.mock("@/lib/api", async (importOriginal) => {
  const mod = await importOriginal<typeof import("@/lib/api")>();
  return {
    ...mod,
    fetchWorkspace: vi.fn(),
    fetchDefaultAttendanceRules: vi.fn(),
    fetchAttendancePolicyVersions: vi.fn(),
    getCurrentWorkspaceId: vi.fn(),
  };
});

vi.mock("@/components/ui/toast", () => ({ useToast: () => ({ success: vi.fn(), error: vi.fn() }) }));

vi.mock("@/components/features/attendance/policy-assignments-section", () => ({
  PolicyAssignmentsSection: () => <div data-testid="policy-assignments-section" />
}));
vi.mock("../time-settings/bulk-reprocess", () => ({
  BulkReprocessSection: () => <div data-testid="bulk-reprocess-section" />
}));
vi.mock("../time-settings/timeline", () => ({
  AttendanceVersionTimeline: () => <div data-testid="timeline-section" />
}));

const mockWorkspace = {
  id: "test-ws-id",
  name: "Test Workspace",
};

const mockRules = {
  work_unit_minutes: 15,
  rounding_policy: "none",
  early_arrival_strategy: "block",
};

const mockVersions = [
  {
    id: "version-1",
    work_unit_minutes: 15,
    rounding_policy: "none",
    early_arrival_strategy: "block",
    valid_from: "2026-06-01T00:00:00Z",
    created_at: "2026-06-01T00:00:00Z",
  }
];

const createQueryClient = () => new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
    },
  },
});

describe("TimeSettings Page", () => {
  const wsId = "test-ws-id";

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue(wsId);
  });

  it("renders loader initially and then content", async () => {
    vi.mocked(api.fetchWorkspace).mockResolvedValue(mockWorkspace as any);
    vi.mocked(api.fetchDefaultAttendanceRules).mockResolvedValue(mockRules as any);
    vi.mocked(api.fetchAttendancePolicyVersions).mockResolvedValue(mockVersions as any);

    const queryClient = createQueryClient();
    render(
      <QueryClientProvider client={queryClient}>
        <TimeSettings />
      </QueryClientProvider>
    );

    expect(await screen.findByText("common.settings.categories.time")).toBeDefined();
    expect(screen.getByTestId("policy-assignments-section")).toBeDefined();
  });

  it("renders premium error state when API fails", async () => {
    vi.mocked(api.fetchWorkspace).mockResolvedValue(mockWorkspace as any);
    vi.mocked(api.fetchDefaultAttendanceRules).mockRejectedValue(new Error("API Error fetching rules"));
    vi.mocked(api.fetchAttendancePolicyVersions).mockResolvedValue(mockVersions as any);

    const queryClient = createQueryClient();
    render(
      <QueryClientProvider client={queryClient}>
        <TimeSettings />
      </QueryClientProvider>
    );

    expect(await screen.findByText("common.settings.time.error_title")).toBeDefined();
    expect(screen.getByText("API Error fetching rules")).toBeDefined();
    expect(screen.getByText("common.retry")).toBeDefined();
  });

  it("renders empty warnings when rules are empty/null", async () => {
    vi.mocked(api.fetchWorkspace).mockResolvedValue(mockWorkspace as any);
    vi.mocked(api.fetchDefaultAttendanceRules).mockResolvedValue(null as any);
    vi.mocked(api.fetchAttendancePolicyVersions).mockResolvedValue(mockVersions as any);

    const queryClient = createQueryClient();
    render(
      <QueryClientProvider client={queryClient}>
        <TimeSettings />
      </QueryClientProvider>
    );

    expect(await screen.findByText("common.settings.time.missing_title")).toBeDefined();
  });
});

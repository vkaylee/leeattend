import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { vi, describe, it, expect, beforeEach } from "vitest";

import Button from "@/components/ui/button";
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
    fetchOrganization: vi.fn(),
    fetchDefaultAttendanceRules: vi.fn(),
    fetchAttendancePolicyVersions: vi.fn(),
    updateDefaultAttendanceRules: vi.fn(),
    reprocessAllAttendance: vi.fn(),
    checkReprocessStatus: vi.fn(),
  };
});

vi.mock("@/components/ui/toast", () => ({ useToast: () => ({ success: vi.fn(), error: vi.fn() }) }));

vi.mock("@/components/features/attendance/policy-assignments-section", () => ({
  PolicyAssignmentsSection: () => <div data-testid="policy-assignments-section" />
}));

vi.mock("../time-settings/bulk-reprocess", () => ({
  BulkReprocessSection: ({ setShowReprocessConfirm }: any) => (
    <div data-testid="bulk-reprocess-section">
      <Button data-testid="reprocess-trigger" onClick={() => setShowReprocessConfirm(true)}>Reprocess</Button>
    </div>
  )
}));

vi.mock("../time-settings/timeline", () => ({
  AttendanceVersionTimeline: () => <div data-testid="timeline-section" />
}));

vi.mock("../time-settings/create-version-modal", () => ({
  CreateVersionModal: ({ isOpen, onSave, onClose, setWorkUnit }: any) => {
    if (!isOpen) return null;
    return (
      <div data-testid="create-version-modal">
        <Button data-testid="modal-save" onClick={onSave}>Save</Button>
        <Button data-testid="modal-close" onClick={onClose}>Close</Button>
        <Button data-testid="modal-change-unit" onClick={() => setWorkUnit(30)}>Change Unit</Button>
      </div>
    );
  }
}));

vi.mock("../time-settings/reprocess-modal", () => ({
  ReprocessModal: ({ isOpen, onSubmit, onClose }: any) => {
    if (!isOpen) return null;
    return (
      <div data-testid="reprocess-modal">
        <Button data-testid="reprocess-submit" onClick={onSubmit}>Submit</Button>
        <Button data-testid="reprocess-close" onClick={onClose}>Close</Button>
      </div>
    );
  }
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

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders loader initially and then content", async () => {
    vi.mocked(api.fetchOrganization).mockResolvedValue(mockWorkspace as any);
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
    vi.mocked(api.fetchOrganization).mockResolvedValue(mockWorkspace as any);
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
    vi.mocked(api.fetchOrganization).mockResolvedValue(mockWorkspace as any);
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

  it("opens create version modal and saves new version successfully", async () => {
    vi.mocked(api.fetchOrganization).mockResolvedValue(mockWorkspace as any);
    vi.mocked(api.fetchDefaultAttendanceRules).mockResolvedValue(mockRules as any);
    vi.mocked(api.fetchAttendancePolicyVersions).mockResolvedValue(mockVersions as any);
    vi.mocked(api.updateDefaultAttendanceRules).mockResolvedValue({} as any);

    const user = userEvent.setup();
    const queryClient = createQueryClient();
    render(
      <QueryClientProvider client={queryClient}>
        <TimeSettings />
      </QueryClientProvider>
    );

    // Find and click new version button
    const newVersionBtn = await screen.findByText("common.organization_settings.new_version");
    await user.click(newVersionBtn);

    // Modal should be open
    expect(screen.getByTestId("create-version-modal")).toBeDefined();

    // Click Save in modal
    const saveBtn = screen.getByTestId("modal-save");
    await user.click(saveBtn);

    // Verify api call
    expect(api.updateDefaultAttendanceRules).toHaveBeenCalledWith({
      early_arrival_strategy: "block",
      rounding_policy: "none",
      valid_from: expect.any(String),
      work_unit_minutes: 15,
      grace_period_minutes: 15,
    });
  });

  it("triggers bulk reprocess flow and submits successfully", async () => {
    vi.mocked(api.fetchOrganization).mockResolvedValue(mockWorkspace as any);
    vi.mocked(api.fetchDefaultAttendanceRules).mockResolvedValue(mockRules as any);
    vi.mocked(api.fetchAttendancePolicyVersions).mockResolvedValue(mockVersions as any);
    vi.mocked(api.reprocessAllAttendance).mockResolvedValue({ job_id: "job-1" } as any);

    const user = userEvent.setup();
    const queryClient = createQueryClient();
    render(
      <QueryClientProvider client={queryClient}>
        <TimeSettings />
      </QueryClientProvider>
    );

    // Wait for content to render
    await screen.findByText("common.settings.categories.time");

    // Click reprocess trigger button in BulkReprocessSection mock
    const reprocessBtn = screen.getByTestId("reprocess-trigger");
    await user.click(reprocessBtn);

    // Verify confirmation modal is open
    expect(screen.getByTestId("reprocess-modal")).toBeDefined();

    // Click submit in modal
    const submitBtn = screen.getByTestId("reprocess-submit");
    await user.click(submitBtn);

    // Verify api call
    expect(api.reprocessAllAttendance).toHaveBeenCalledWith(expect.any(String), expect.any(String));
  });
});

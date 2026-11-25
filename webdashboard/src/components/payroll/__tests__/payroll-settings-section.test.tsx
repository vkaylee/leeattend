import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as employeesApi from "@/lib/api/employees";
import * as api from "@/lib/api/payroll";

import PayrollSettingsSection from "../payroll-settings-section";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (k: string, d?: string) => d ?? k }),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasPermission: () => true,
  }),
}));

vi.mock("@/lib/api/payroll", () => ({
  fetchWorkPolicyGroups: vi.fn(),
  createWorkPolicyGroup: vi.fn(),
  fetchWorkPolicyGroupVersions: vi.fn(),
  createWorkPolicyVersion: vi.fn(),
  patchWorkPolicyVersion: vi.fn(),
  validateWorkPolicyVersion: vi.fn(),
  publishWorkPolicyVersion: vi.fn(),
  fetchWorkPolicyMemberships: vi.fn(),
  createWorkPolicyMembership: vi.fn(),
  transferWorkPolicyMembership: vi.fn(),
  fetchWorkPolicyReferenceOptions: vi.fn(),
  simulateWorkPolicyVersionBatch: vi.fn(),
}));

vi.mock("@/lib/api/employees", () => ({
  fetchEmployees: vi.fn(),
  fetchEmployeeRelationships: vi.fn(),
}));

const sampleDraft: api.WorkPolicyVersion = {
  id: "ver-2",
  group_id: "grp-1",
  group_code: "STD-OFFICE",
  group_name: "Standard Office",
  version_no: 2,
  status: "draft",
  row_version: 1,
  effective_from: "2026-06-01",
  effective_to: null,
  minutes_per_credit: 480,
  work_unit_minutes: 1,
  rounding_policy: "none",
  early_arrival_strategy: "clamp",
  late_arrival_grace_minutes: 5,
  early_ot_enabled: false,
  early_ot_qualification_grace_minutes: 0,
  early_ot_max_minutes: null,
  break_auto_deduct: true,
  break_is_overtime: false,
  ot_aggregation_mode: "aggregate",
  working_pattern_id: null,
  pay_method: "monthly",
  monthly_denominator: "fixed",
  monthly_standard_days: "22",
  rate_component_definition_id: null,
  paid_leave_credit_mode: "standard_day",
  overtime_policy_version_id: "ot-ver-1",
  created_by: "u-1",
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

describe("PayrollSettingsSection (Work Policy v2)", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

    vi.mocked(api.fetchWorkPolicyGroups).mockResolvedValue([
      {
        id: "grp-1",
        code: "STD-OFFICE",
        name: "Standard Office",
        active_membership_count: 1,
        latest_draft: sampleDraft,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
      },
      {
        id: "grp-2",
        code: "SHIFT-PROD",
        name: "Production Shift",
        active_membership_count: 0,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
      },
    ]);
    vi.mocked(api.fetchWorkPolicyGroupVersions).mockResolvedValue([sampleDraft]);
    vi.mocked(api.fetchWorkPolicyMemberships).mockResolvedValue([
      { id: "m-1", group_id: "grp-1", employment_relationship_id: "rel-1", effective_from: "2026-01-01", effective_to: null },
    ]);
    vi.mocked(api.fetchWorkPolicyReferenceOptions).mockResolvedValue({
      overtimeVersions: [{ id: "ot-ver-1", label: "Standard OT (v1)" }],
      patterns: [],
      salaryComponents: [{ id: "sc-1", label: "Hourly Rate (RATE)" }],
    });
    vi.mocked(employeesApi.fetchEmployees).mockResolvedValue({
      data: [{ id: "emp-1", full_name: "Alice", employee_id_number: "EMP001" } as unknown as never],
      total: 1,
      page: 1,
      limit: 100,
    });
    vi.mocked(employeesApi.fetchEmployeeRelationships).mockResolvedValue([
      { id: "rel-1", employee_id: "emp-1", employer_id: "er-1", tenure_id: "t-1", status: "active", start_date: "2026-01-01" },
    ]);
  });

  const renderSection = (onDirtyChange?: (dirty: boolean) => void) =>
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <PayrollSettingsSection onDirtyChange={onDirtyChange} />
        </ToastProvider>
      </QueryClientProvider>,
    );

  it("renders group list, version editor, memberships, and simulation panels", async () => {
    renderSection();
    expect(await screen.findByTestId("work-policy-group-card-STD-OFFICE")).toBeInTheDocument();
    expect(await screen.findByTestId("work-policy-version-editor")).toBeInTheDocument();
    expect(screen.getByTestId("work-policy-memberships-panel")).toBeInTheDocument();
    expect(screen.getByTestId("work-policy-simulation-panel")).toBeInTheDocument();
  });

  it("hides inapplicable pay fields when switching pay method and notifies dirty state", async () => {
    const onDirty = vi.fn();
    renderSection(onDirty);
    const payMethod = await screen.findByTestId("wp-pay-method");
    expect(screen.getByTestId("wp-monthly-standard-days")).toBeInTheDocument();
    expect(screen.queryByTestId("wp-rate-component")).not.toBeInTheDocument();

    fireEvent.change(payMethod, { target: { value: "hourly" } });
    expect(screen.queryByTestId("wp-monthly-standard-days")).not.toBeInTheDocument();
    expect(screen.getByTestId("wp-rate-component")).toBeInTheDocument();
    expect(onDirty).toHaveBeenCalledWith(true);
  });

  it("surfaces 409 optimistic lock conflict banner on stale draft save", async () => {
    const err = Object.assign(new Error("conflict"), { status: 409 });
    vi.mocked(api.patchWorkPolicyVersion).mockRejectedValue(err);
    renderSection();

    const minsInput = await screen.findByTestId("wp-minutes-per-credit");
    fireEvent.change(minsInput, { target: { value: "450" } });
    fireEvent.click(screen.getByTestId("save-work-policy-draft-btn"));

    expect(await screen.findByTestId("work-policy-conflict-banner")).toBeInTheDocument();
  });

  it("validates, runs batch simulation with uncomputable + trace, and publishes", async () => {
    vi.mocked(api.validateWorkPolicyVersion).mockResolvedValue({
      version_id: "ver-2",
      valid: true,
      blockers: [],
      warnings: [],
      checked_at: "2026-03-01T00:00:00Z",
    });
    vi.mocked(api.simulateWorkPolicyVersionBatch).mockResolvedValue({
      group_id: "grp-1",
      version_id: "ver-2",
      period_year: 2026,
      period_month: 3,
      total: 2,
      computed: 1,
      affected: 1,
      uncomputable: 1,
      input_fingerprint: "abcdef1234567890",
      evaluated_at: "2026-03-01T00:00:00Z",
      expires_at: "2099-03-01T00:15:00Z",
      stale_after_seconds: 900,
      items: [
        {
          employment_relationship_id: "rel-1",
          status: "computed",
          affected: true,
          baseline_net: "1000",
          candidate_net: "1100",
          net_delta: "100",
          trace: [{ step: "net", rule: "gross_minus_deductions", expression: "1100 - 0", result: "1100.00" }],
        },
        {
          employment_relationship_id: "rel-missing",
          status: "uncomputable",
          affected: false,
          reason: "no active employment segment covers period",
        },
      ],
    });
    vi.mocked(api.publishWorkPolicyVersion).mockResolvedValue({ ...sampleDraft, status: "published", row_version: 2 });

    renderSection();
    fireEvent.click(await screen.findByTestId("validate-work-policy-btn"));
    expect(await screen.findByTestId("work-policy-validation-results")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("wp-run-simulation-btn"));
    expect(await screen.findByTestId("wp-simulation-results")).toBeInTheDocument();
    expect(screen.getByText(/uncomputable · no active employment segment/)).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("publish-work-policy-btn"));
    await waitFor(() => {
      expect(api.publishWorkPolicyVersion).toHaveBeenCalledWith(
        "grp-1",
        "ver-2",
        expect.objectContaining({ expected_row_version: 1, auto_close_previous: true }),
        expect.stringContaining("pub-ver-2"),
      );
    });
  });

  it("performs atomic membership transfer to another group", async () => {
    vi.mocked(api.transferWorkPolicyMembership).mockResolvedValue({
      new_membership: { id: "m-2", group_id: "grp-2", employment_relationship_id: "rel-1", effective_from: "2026-05-01" },
    });
    renderSection();

    fireEvent.click(await screen.findByTestId("wp-open-transfer-rel-1"));
    expect(screen.getByTestId("wp-transfer-modal")).toBeInTheDocument();
    fireEvent.click(screen.getByTestId("wp-confirm-transfer-btn"));

    await waitFor(() => {
      expect(api.transferWorkPolicyMembership).toHaveBeenCalledWith(
        expect.objectContaining({ employment_relationship_id: "rel-1", target_group_id: "grp-2" }),
      );
    });
  });
});

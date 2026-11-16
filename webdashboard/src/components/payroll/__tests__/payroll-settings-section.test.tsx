import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api/payroll";

import PayrollSettingsSection from "../payroll-settings-section";


// Mock API
vi.mock("@/lib/api/payroll", () => ({
  fetchPayrollSettings: vi.fn(),
  updatePayrollSettings: vi.fn(),
  fetchPayrollSettingsHistory: vi.fn(),
  fetchWorkPolicyGroups: vi.fn(),
  fetchWorkPolicyGroupHistory: vi.fn(),
  createWorkPolicyGroupVersion: vi.fn(),
  fetchWorkPolicyMemberships: vi.fn(),
  createWorkPolicyMembership: vi.fn(),
  fetchWorkPolicyReferenceOptions: vi.fn(),
  validateWorkPolicyGroupVersion: vi.fn(),
  fetchPayrollImpactBatch: vi.fn(),
  simulatePayrollWithPolicy: vi.fn(),
}));

// Mock i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("PayrollSettingsSection", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
 
    vi.mocked(api.fetchPayrollSettingsHistory).mockResolvedValue([
      { fixed_std_days: 26, hours_per_day: 8, effective_date: "2026-01-01" }
    ] as unknown as never);
    vi.mocked(api.fetchWorkPolicyGroups).mockResolvedValue([]);
    vi.mocked(api.fetchWorkPolicyGroupHistory).mockResolvedValue([]);
    vi.mocked(api.fetchWorkPolicyMemberships).mockResolvedValue([]);
    vi.mocked(api.fetchWorkPolicyReferenceOptions).mockResolvedValue({ attendance: [], patterns: [], overtime: [], salaryComponents: [] });
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {ui}
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("shows bounded impact preview results and stale state", async () => {
    vi.mocked(api.fetchPayrollImpactBatch).mockResolvedValue({ total: 1, computed: 1, affected: 1, uncomputable: 0, items: [{ employment_id: "e-1", computed: true, stale: true, stale_reasons: ["policy_version_changed"], delta: { net_pay_delta: "240" } as never }] });
    renderWithProviders(<PayrollSettingsSection />);
    fireEvent.change(await screen.findByLabelText(/impact_employments/i), { target: { value: "e-1" } });
    fireEvent.change(screen.getByLabelText(/impact_before_run_id/i), { target: { value: "r-1" } });
    fireEvent.change(screen.getByLabelText(/impact_after_run_id/i), { target: { value: "r-2" } });
    fireEvent.click(screen.getByRole("button", { name: "common.payroll.settings.impact_run" }));
    expect(await screen.findByText("policy_version_changed")).toBeInTheDocument();
    expect(screen.getByText(/240/)).toBeInTheDocument();
  });

  it("renders settings correctly", async () => {
    renderWithProviders(<PayrollSettingsSection   />);

    // In the new UI, standard days are wrapped in a span with text-primary-600
    // and followed by common.days_short.
    expect(await screen.findByText(/26/i, { selector: 'span' })).toBeInTheDocument();
    expect(screen.getByText(/01\/01\/2026/i)).toBeInTheDocument();
  });

  it("does not submit until configurable standard values are entered", async () => {
    renderWithProviders(<PayrollSettingsSection />);

    fireEvent.click(await screen.findByText(/common.add_new/i));
    fireEvent.click(screen.getByText(/common.save/i));

    await waitFor(() => {
      expect(api.updatePayrollSettings).not.toHaveBeenCalled();
    });
    expect(screen.getAllByText(/payroll.settings.validation.positive/i).length).toBeGreaterThanOrEqual(2);
  });

  it("shows save errors without closing the form", async () => {
    vi.mocked(api.updatePayrollSettings).mockRejectedValue(new Error("conflict"));
    renderWithProviders(<PayrollSettingsSection />);

    fireEvent.click(await screen.findByText(/common.add_new/i));
    const numberInputs = await screen.findAllByRole("spinbutton");
    fireEvent.change(numberInputs[0], { target: { value: "24" } });
    fireEvent.change(numberInputs[1], { target: { value: "8" } });
    fireEvent.click(screen.getByText(/common.save/i));

    await waitFor(() => {
      expect(api.updatePayrollSettings).toHaveBeenCalled();
    });
    expect(screen.getByText(/common.save/i)).toBeInTheDocument();
  });

  it("creates a new effective policy version from selected policy references", async () => {
    vi.mocked(api.fetchWorkPolicyGroups).mockResolvedValue([{
      id: "v-1",
      group_id: "g-1",
      name: "Operations",
      attendance_policy_id: "attendance-1",
      working_pattern_id: "pattern-1",
      overtime_policy_id: "overtime-1",
      minutes_per_credit: 480,
      pay_method: "monthly",
      monthly_standard_days: "26",
      effective_from: "2026-01-01",
    }]);
    vi.mocked(api.fetchWorkPolicyGroupHistory).mockResolvedValue([]);
    vi.mocked(api.fetchWorkPolicyMemberships).mockResolvedValue([]);
    vi.mocked(api.fetchWorkPolicyReferenceOptions).mockResolvedValue({ attendance: [], patterns: [], overtime: [], salaryComponents: [] });
    vi.mocked(api.createWorkPolicyGroupVersion).mockResolvedValue({} as never);
    renderWithProviders(<PayrollSettingsSection />);

    fireEvent.click(await screen.findByRole("button", { name: "Operations" }));
    fireEvent.change((await screen.findAllByLabelText("common.payroll.settings.work_policy_effective_from"))[0], { target: { value: "2026-07-01" } });
    fireEvent.change(await screen.findByLabelText("common.payroll.settings.work_policy_minutes"), { target: { value: "450" } });
    fireEvent.click(screen.getByRole("button", { name: "common.payroll.settings.work_policy_create" }));

    await waitFor(() => {
      expect(api.createWorkPolicyGroupVersion).toHaveBeenCalledWith("g-1", expect.objectContaining({
        effective_from: "2026-07-01",
        minutes_per_credit: 450,
        attendance_policy_id: "attendance-1",
        working_pattern_id: "pattern-1",
        overtime_policy_id: "overtime-1",
      }));
    });
  });

  it("validates selected policy version and shows blockers", async () => {
    vi.mocked(api.fetchWorkPolicyGroups).mockResolvedValue([{
      id: "v-1", group_id: "g-1", name: "Operations", attendance_policy_id: "attendance-1",
      working_pattern_id: "pattern-1", overtime_policy_id: "overtime-1", minutes_per_credit: 480,
      pay_method: "monthly", effective_from: "2026-01-01",
    }]);
    vi.mocked(api.fetchWorkPolicyGroupHistory).mockResolvedValue([{
      id: "v-1", group_id: "g-1", name: "Operations", attendance_policy_id: "attendance-1",
      working_pattern_id: "pattern-1", overtime_policy_id: "overtime-1", minutes_per_credit: 480,
      pay_method: "monthly", effective_from: "2026-01-01",
    }]);
    vi.mocked(api.fetchWorkPolicyMemberships).mockResolvedValue([]);
    vi.mocked(api.validateWorkPolicyGroupVersion).mockResolvedValue({
      version_id: "v-1", valid: false, blockers: [{ code: "reference_missing_or_inactive", field: "policy_references", severity: "blocker", message: "working pattern inactive" }], warnings: [], checked_at: "2026-11-16T00:00:00Z",
    });
    renderWithProviders(<PayrollSettingsSection />);

    fireEvent.click(await screen.findByRole("button", { name: "Operations" }));
    fireEvent.click(await screen.findByRole("button", { name: "common.payroll.settings.work_policy_validate" }));

    await waitFor(() => expect(api.validateWorkPolicyGroupVersion).toHaveBeenCalledWith("g-1", "v-1"));
    expect(await screen.findByText(/working pattern inactive/i)).toBeInTheDocument();
    expect(screen.getByText("common.payroll.settings.work_policy_blocked")).toBeInTheDocument();
  });

  it("rejects policy version without date or positive whole minutes", async () => {
    vi.mocked(api.fetchWorkPolicyGroups).mockResolvedValue([{
      id: "v-1", group_id: "g-1", name: "Operations", attendance_policy_id: "attendance-1",
      working_pattern_id: "pattern-1", overtime_policy_id: "overtime-1", minutes_per_credit: 480,
      pay_method: "monthly", effective_from: "2026-01-01",
    }]);
    vi.mocked(api.fetchWorkPolicyGroupHistory).mockResolvedValue([]);
    vi.mocked(api.fetchWorkPolicyMemberships).mockResolvedValue([]);
    vi.mocked(api.fetchWorkPolicyReferenceOptions).mockResolvedValue({ attendance: [], patterns: [], overtime: [], salaryComponents: [] });
    renderWithProviders(<PayrollSettingsSection />);

    fireEvent.click(await screen.findByRole("button", { name: "Operations" }));
    fireEvent.click(await screen.findByRole("button", { name: "common.payroll.settings.work_policy_create" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("common.payroll.settings.work_policy_validation");
    expect(api.createWorkPolicyGroupVersion).not.toHaveBeenCalled();
  });

  it("calls update API when save button is clicked", async () => {
 
    vi.mocked(api.updatePayrollSettings).mockResolvedValue({} as unknown as never);
    renderWithProviders(<PayrollSettingsSection   />);

    const addButton = await screen.findByText(/common.add_new/i);
    fireEvent.click(addButton);

    const daysInput = await screen.findByLabelText("payroll.settings.std_days");
    const hoursInput = screen.getByLabelText("payroll.settings.hours_per_day");
    fireEvent.change(daysInput, { target: { value: "24" } });
    fireEvent.change(hoursInput, { target: { value: "8" } });

    const saveButton = screen.getByText(/common.save/i);
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(api.updatePayrollSettings).toHaveBeenCalledWith(expect.objectContaining({
        fixed_std_days: "24"
      }));
    });
  });
});

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ShieldAlert, Clock, History } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { fetchEmployeeEmployments, fetchEmployees } from "@/lib/api/employees";
import { ApiError } from "@/lib/api/errors";
import { createWorkPolicyMembership, createWorkPolicyGroupVersion, fetchPayrollImpactBatch, fetchWorkPolicyGroupHistory, fetchWorkPolicyGroups, fetchWorkPolicyMemberships, fetchWorkPolicyReferenceOptions, simulatePayroll, simulatePayrollWithPolicy, updatePayrollSettings, fetchPayrollSettingsHistory, validateWorkPolicyGroupVersion, type PayrollImpactBatchResponse, type WorkPolicyGroupVersion, type WorkPolicyValidationResponse } from "@/lib/api/payroll";
import { employeeKeys, payrollKeys } from "@/lib/query-keys";

import TemporalConfigTable from "../common/temporal-config-table";


interface PayrollSettingsSectionProps {
  onDirtyChange?: (isDirty: boolean) => void;
}

export default function PayrollSettingsSection({ onDirtyChange }: PayrollSettingsSectionProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [selectedPolicyGroupId, setSelectedPolicyGroupId] = useState<string>();
  const [isCreatingVersion, setIsCreatingVersion] = useState(false);
  const [versionForm, setVersionForm] = useState({ effective_from: "", minutes_per_credit: "", pay_method: "monthly" as "monthly" | "hourly" | "shift", monthly_denominator: "fixed" as "fixed" | "schedule", monthly_standard_days: "26", attendance_policy_id: "", working_pattern_id: "", overtime_policy_id: "", rate_component_definition_id: "" });
  const [versionError, setVersionError] = useState<string>();
  const [versionConflict, setVersionConflict] = useState(false);
  const [membershipEmployeeId, setMembershipEmployeeId] = useState("");
  const [membershipForm, setMembershipForm] = useState({ employment_id: "", effective_from: "", effective_to: "" });
  const [membershipError, setMembershipError] = useState<string>();
  const [membershipConflict, setMembershipConflict] = useState(false);
  const [draftSimulationForm, setDraftSimulationForm] = useState({ employee_id: "", year: "2026", month: "1", policy_version_id: "" });
  const [draftSimulationResult, setDraftSimulationResult] = useState<{ base_pay_amount: string; net_pay_amount: string }>();
  const [currentSimulationResult, setCurrentSimulationResult] = useState<{ base_pay_amount: string; net_pay_amount: string }>();
  const [draftSimulationError, setDraftSimulationError] = useState<string>();
  const [isLoadingDraftSimulation, setIsLoadingDraftSimulation] = useState(false);
  const [isDraftDirty, setIsDraftDirty] = useState(false);
  const [validationByVersion, setValidationByVersion] = useState<Record<string, WorkPolicyValidationResponse>>({});
  const [validatingVersionId, setValidatingVersionId] = useState<string>();
  const [impactForm, setImpactForm] = useState({ employment_ids: "", year: "2026", month: "1", before_run_id: "", after_run_id: "" });
  const [impactResult, setImpactResult] = useState<PayrollImpactBatchResponse>();
  const [impactError, setImpactError] = useState<string>();
  const [now] = useState(Date.now);
  const [isLoadingImpact, setIsLoadingImpact] = useState(false);

  const { data: history = [], isLoading: isHistoryLoading } = useQuery({ queryKey: payrollKeys.settingsHistory(), queryFn: () => fetchPayrollSettingsHistory() });
  const { data: workPolicies = [], isLoading: isWorkPoliciesLoading, isError: isWorkPoliciesError } = useQuery({
    queryKey: payrollKeys.workPolicyGroups(),
    queryFn: fetchWorkPolicyGroups,
  });
  const { data: selectedPolicyHistory = [], isLoading: isPolicyHistoryLoading, isError: isPolicyHistoryError, refetch: refetchPolicyHistory } = useQuery({
    queryKey: payrollKeys.workPolicyHistory(selectedPolicyGroupId),
    queryFn: () => fetchWorkPolicyGroupHistory(selectedPolicyGroupId!),
    enabled: Boolean(selectedPolicyGroupId),
  });
  const { data: selectedMemberships = [], isLoading: isMembershipsLoading } = useQuery({
    queryKey: payrollKeys.workPolicyMemberships(selectedPolicyGroupId),
    queryFn: () => fetchWorkPolicyMemberships(selectedPolicyGroupId!),
    enabled: Boolean(selectedPolicyGroupId),
  });
  const { data: referenceOptions, isLoading: isReferenceOptionsLoading, isError: isReferenceOptionsError, refetch: refetchReferenceOptions } = useQuery({ queryKey: payrollKeys.workPolicyReferences(), queryFn: fetchWorkPolicyReferenceOptions });
  const { data: employeesResponse } = useQuery({ queryKey: employeeKeys.list({ status: "active", emptyQueryBehavior: "all" }), queryFn: () => fetchEmployees(1, 100, "employees:read") });
  const employees = employeesResponse?.data ?? [];
  const { data: membershipEmployments = [] } = useQuery({ queryKey: payrollKeys.membershipEmployments(membershipEmployeeId), queryFn: () => fetchEmployeeEmployments(membershipEmployeeId), enabled: Boolean(membershipEmployeeId) });

  const selectPolicyForEdit = (policy: WorkPolicyGroupVersion) => {
    setSelectedPolicyGroupId(policy.group_id);
    setVersionForm((form) => ({ ...form, attendance_policy_id: policy.attendance_policy_id ?? "", working_pattern_id: policy.working_pattern_id ?? "", overtime_policy_id: policy.overtime_policy_id }));
  };

  useEffect(() => {
    onDirtyChange?.(isDraftDirty);
  }, [isDraftDirty, onDirtyChange]);

  useEffect(() => {
    if (!isDraftDirty) return;
    const handleBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
    };
    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => { window.removeEventListener("beforeunload", handleBeforeUnload); };
  }, [isDraftDirty]);

  const handleCreateVersion = async () => {
    if (!selectedPolicyGroupId) return;
    if (isPolicyHistoryError || isReferenceOptionsError || !referenceOptions) {
      setVersionError(t("common.payroll.settings.work_policy_references_unavailable", "Policy references are unavailable. Refresh before creating a version."));
      return;
    }
    const minutes = Number(versionForm.minutes_per_credit);
    if (!versionForm.effective_from || !Number.isInteger(minutes) || minutes <= 0) {
      setVersionError(t("common.payroll.settings.work_policy_validation", "Enter effective date and positive whole minutes."));
      return;
    }
    setIsCreatingVersion(true);
    setVersionError(undefined);
    setVersionConflict(false);
    try {
      const selected = workPolicies.find((policy) => policy.group_id === selectedPolicyGroupId);
      const attendancePolicyID = versionForm.attendance_policy_id || selected?.attendance_policy_id;
      const workingPatternID = versionForm.working_pattern_id || selected?.working_pattern_id;
      const overtimePolicyID = versionForm.overtime_policy_id || selected?.overtime_policy_id;
      if (!selected || !attendancePolicyID || !workingPatternID || !overtimePolicyID) {
        throw new Error(t("common.payroll.settings.work_policy_missing_refs", "Selected policy lacks required references."));
      }
      await createWorkPolicyGroupVersion(selectedPolicyGroupId, {
        name: selected.name,
        minutes_per_credit: minutes,
        attendance_policy_id: attendancePolicyID,
        working_pattern_id: workingPatternID,
        pay_method: versionForm.pay_method,
        monthly_denominator: versionForm.pay_method === "monthly" ? versionForm.monthly_denominator : undefined,
        monthly_standard_days: versionForm.pay_method === "monthly" && versionForm.monthly_denominator === "fixed" ? versionForm.monthly_standard_days : undefined,
        rate_component_definition_id: versionForm.pay_method === "monthly" ? null : (versionForm.rate_component_definition_id || selected.rate_component_definition_id || null),
        overtime_policy_id: overtimePolicyID,
        effective_from: versionForm.effective_from,
      });
      toast.success(t("common.success"));
      setIsDraftDirty(false);
      setVersionForm({ effective_from: "", minutes_per_credit: "", pay_method: "monthly", monthly_denominator: "fixed", monthly_standard_days: "26", attendance_policy_id: "", working_pattern_id: "", overtime_policy_id: "", rate_component_definition_id: "" });
      void queryClient.invalidateQueries({ queryKey: payrollKeys.workPolicyGroups() });
      void queryClient.invalidateQueries({ queryKey: payrollKeys.workPolicyHistory(selectedPolicyGroupId) });
    } catch (error: unknown) {
      const conflict = error instanceof ApiError && error.isConflict;
      setVersionConflict(conflict);
      setVersionError(conflict ? t("common.payroll.settings.work_policy_conflict", "Policy changed on server. Review current versions, then retry your draft.") : error instanceof Error ? error.message : t("common.error"));
    } finally {
      setIsCreatingVersion(false);
    }
  };

  const handleValidateVersion = async (version: WorkPolicyGroupVersion) => {
    setValidatingVersionId(version.id);
    try {
      const result = await validateWorkPolicyGroupVersion(version.group_id, version.id);
      setValidationByVersion((current) => ({ ...current, [version.id]: result }));
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : t("common.error"));
    } finally {
      setValidatingVersionId(undefined);
    }
  };

  const handleDraftSimulation = async () => {
    const year = Number(draftSimulationForm.year);
    const month = Number(draftSimulationForm.month);
    if (!draftSimulationForm.employee_id || !draftSimulationForm.policy_version_id || !Number.isInteger(year) || !Number.isInteger(month) || month < 1 || month > 12) {
      setDraftSimulationError(t("common.payroll.settings.draft_simulation_validation", "Choose employee, policy version, and valid period."));
      return;
    }
    setIsLoadingDraftSimulation(true);
    setDraftSimulationError(undefined);
    setCurrentSimulationResult(undefined);
    setDraftSimulationResult(undefined);
    try {
      const results = await Promise.allSettled([
        simulatePayroll(draftSimulationForm.employee_id, year, month),
        simulatePayrollWithPolicy(draftSimulationForm.employee_id, year, month, draftSimulationForm.policy_version_id),
      ]);
      const [current, draft] = results;
      if (current.status !== "fulfilled" || draft.status !== "fulfilled") {
        setDraftSimulationError(t("common.payroll.settings.draft_simulation_partial", "Simulation incomplete: both current and draft results are required."));
        return;
      }
      setCurrentSimulationResult({ base_pay_amount: current.value.base_pay_amount, net_pay_amount: current.value.net_pay_amount });
      setDraftSimulationResult({ base_pay_amount: draft.value.base_pay_amount, net_pay_amount: draft.value.net_pay_amount });
    } catch (error: unknown) {
      setDraftSimulationError(error instanceof Error ? error.message : t("common.error"));
    } finally {
      setIsLoadingDraftSimulation(false);
    }
  };

  const handleCreateMembership = async () => {
    if (!selectedPolicyGroupId || !membershipForm.employment_id || !membershipForm.effective_from) {
      setMembershipError(t("common.payroll.settings.work_policy_membership_validation", "Enter employment ID and effective date."));
      return;
    }
    setMembershipError(undefined);
    setMembershipConflict(false);
    try {
      await createWorkPolicyMembership(selectedPolicyGroupId, {
        employment_id: membershipForm.employment_id,
        effective_from: membershipForm.effective_from,
        effective_to: membershipForm.effective_to || undefined,
      });
      toast.success(t("common.success"));
      setMembershipForm({ employment_id: "", effective_from: "", effective_to: "" });
      void queryClient.invalidateQueries({ queryKey: payrollKeys.workPolicyMemberships(selectedPolicyGroupId) });
    } catch (error: unknown) {
      const conflict = error instanceof ApiError && error.isConflict;
      setMembershipConflict(conflict);
      setMembershipError(conflict ? t("common.payroll.settings.work_policy_membership_conflict", "Assignment overlaps a current membership. Refresh assignments and choose a non-overlapping range.") : error instanceof Error ? error.message : t("common.error"));
    }
  };

  const handleImpactPreview = async () => {
    const employment_ids = impactForm.employment_ids.split(/[\s,]+/).map((id) => id.trim()).filter(Boolean);
    const hasDuplicateEmploymentIDs = new Set(employment_ids).size !== employment_ids.length;
    const year = Number(impactForm.year);
    const month = Number(impactForm.month);
    if (employment_ids.length === 0 || employment_ids.length > 100 || hasDuplicateEmploymentIDs || !Number.isInteger(year) || !Number.isInteger(month) || month < 1 || month > 12 || !impactForm.before_run_id || !impactForm.after_run_id || impactForm.before_run_id === impactForm.after_run_id) {
      setImpactError(t("common.payroll.settings.impact_validation", "Enter up to 100 unique employment IDs, a valid period, and two different calculation run IDs."));
      return;
    }
    setIsLoadingImpact(true);
    setImpactError(undefined);
    try {
      setImpactResult(await fetchPayrollImpactBatch({ employment_ids, year, month, before_run_id: impactForm.before_run_id, after_run_id: impactForm.after_run_id }));
    } catch (error: unknown) {
      setImpactError(error instanceof Error ? error.message : t("common.error"));
    } finally {
      setIsLoadingImpact(false);
    }
  };

  const handleSavePolicy = async (data: { std_working_days: number; hours_per_day: number; max_ot_hours_per_day: number | null; max_ot_hours_per_month: number | null; effective_date: string }) => {
    try {
      await updatePayrollSettings({
        fixed_std_days: String(data.std_working_days),
        hours_per_day: String(data.hours_per_day),
        max_ot_hours_per_day: data.max_ot_hours_per_day !== null ? String(data.max_ot_hours_per_day) : null,
        max_ot_hours_per_month: data.max_ot_hours_per_month !== null ? String(data.max_ot_hours_per_month) : null,

        effective_date: data.effective_date,
      });
      toast.success(t("common.success"));
      void queryClient.invalidateQueries({ queryKey: payrollKeys.settings() });
      void queryClient.invalidateQueries({ queryKey: payrollKeys.settingsHistory() });
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : t("common.error"));
      throw error;
    }
  };

  return (
    <div className="space-y-6">
      <TemporalConfigTable
        isLoading={isHistoryLoading}
        onSave={handleSavePolicy}
        records={history.map((record) => ({
          std_working_days: parseFloat(record.fixed_std_days),
          hours_per_day: parseFloat(record.hours_per_day),
          max_ot_hours_per_day: record.max_ot_hours_per_day ? parseFloat(record.max_ot_hours_per_day) : null,
          max_ot_hours_per_month: record.max_ot_hours_per_month ? parseFloat(record.max_ot_hours_per_month) : null,
          effective_date: record.effective_date,
        }))}
        title={t("common.payroll.settings.title")}
      />

      <section aria-labelledby="work-policy-groups-title" className="rounded-2xl border border-slate-200/80 bg-slate-50/60 p-6 dark:border-slate-800/80 dark:bg-slate-900/60">
        <div className="mb-4 flex items-center gap-3">
          <History aria-hidden="true" className="h-5 w-5 text-primary-600" />
          <div>
            <Typography className="font-bold" id="work-policy-groups-title" variant="h2">
              {t("common.payroll.settings.work_policy_title", "Labor policy groups")}
            </Typography>
            <Typography className="text-xs text-slate-500" variant="small">
              {t("common.payroll.settings.work_policy_desc", "Read-only effective versions used by payroll")}
            </Typography>
          </div>
        </div>
        {isWorkPoliciesLoading ? (
          <div aria-label={t("common.loading", "Loading")} className="h-16 animate-pulse rounded-lg bg-slate-200/70 dark:bg-slate-800" />
        ) : isWorkPoliciesError ? (
          <Typography className="text-sm text-red-600"role="alert" variant="p">{t("common.payroll.settings.work_policy_error", "Unable to load labor policy groups")}</Typography>
        ) : workPolicies.length === 0 ? (
          <Typography className="text-sm text-slate-500"variant="p">{t("common.payroll.settings.work_policy_empty", "No labor policy groups configured")}</Typography>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full min-w-[640px] text-left text-sm">
              <thead className="text-xs uppercase text-slate-500">
                <tr>
                  <th className="px-3 py-2">{t("common.payroll.settings.work_policy_name", "Group")}</th>
                  <th className="px-3 py-2">{t("common.payroll.settings.work_policy_method", "Pay method")}</th>
                  <th className="px-3 py-2">{t("common.payroll.settings.work_policy_credit", "Minutes / credit")}</th>
                  <th className="px-3 py-2">{t("common.payroll.settings.work_policy_effective", "Effective")}</th>
                </tr>
              </thead>
              <tbody>
                {workPolicies.map((policy) => (
                  <tr className="border-t border-slate-200/70 dark:border-slate-800" key={policy.id}>
                    <td className="px-3 py-3 font-medium">
                      <Button className="text-left text-primary-700 underline-offset-2 hover:underline" onClick={() => { selectPolicyForEdit(policy); }} size="sm" type="button" variant="ghost">
                        {policy.name}
                      </Button>
                    </td>
                    <td className="px-3 py-3">{policy.pay_method}</td>
                    <td className="px-3 py-3">{policy.minutes_per_credit}</td>
                    <td className="px-3 py-3">{policy.effective_from}{policy.effective_to ? ` – ${policy.effective_to}` : ""}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        {!!selectedPolicyGroupId && (
          <div className="mt-5 rounded-lg border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-slate-950">
            <div className="mb-3 flex items-center justify-between">
              <Typography className="font-semibold" variant="h3">{t("common.payroll.settings.work_policy_history", "Version history")}</Typography>
              <Button className="text-sm text-slate-500 underline" onClick={() => { setSelectedPolicyGroupId(undefined); }} size="sm" type="button" variant="ghost">{t("common.close", "Close")}</Button>
            </div>
            {isPolicyHistoryLoading || isReferenceOptionsLoading ? <div className="h-10 animate-pulse rounded bg-slate-200 dark:bg-slate-800" /> : isPolicyHistoryError || isReferenceOptionsError ? (
              <div className="space-y-2" role="alert">
                <Typography className="text-sm text-red-600"variant="p">{t("common.payroll.settings.work_policy_references_unavailable", "Policy references are unavailable. Refresh before editing.")}</Typography>
                <Button className="underline" onClick={() => { void refetchPolicyHistory(); void refetchReferenceOptions(); }} size="sm" type="button" variant="ghost">{t("common.retry", "Retry")}</Button>
              </div>
            ) : (
              <>
              <div aria-label={t("common.payroll.settings.work_policy_create", "Create policy version")} className="mb-4 grid gap-3 sm:grid-cols-3">
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-effective-from">
                  {t("common.payroll.settings.work_policy_effective_from", "Effective from")}
                  <input className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-effective-from" onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, effective_from: event.target.value })); }} type="date" value={versionForm.effective_from} />
                </label>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-minutes-per-credit">
                  {t("common.payroll.settings.work_policy_minutes", "Minutes / credit")}
                  <input className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-minutes-per-credit" min="1" onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, minutes_per_credit: event.target.value })); }} step="1" type="number" value={versionForm.minutes_per_credit} />
                </label>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-attendance-policy">
                  {t("common.payroll.settings.work_policy_attendance", "Attendance policy")}
                  <select className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-attendance-policy" onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, attendance_policy_id: event.target.value })); }} value={versionForm.attendance_policy_id}><option value="">{t("common.select", "Select")}</option>{referenceOptions?.attendance.map((option) => <option key={option.id} value={option.id}>{option.label}</option>)}</select>
                </label>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-working-pattern">
                  {t("common.payroll.settings.work_policy_pattern", "Working pattern")}
                  <select className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-working-pattern" onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, working_pattern_id: event.target.value })); }} value={versionForm.working_pattern_id}><option value="">{t("common.select", "Select")}</option>{referenceOptions?.patterns.map((option) => <option key={option.id} value={option.id}>{option.label}</option>)}</select>
                </label>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-overtime">
                  {t("common.payroll.settings.work_policy_overtime", "Overtime policy")}
                  <select className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-overtime" onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, overtime_policy_id: event.target.value })); }} value={versionForm.overtime_policy_id}><option value="">{t("common.select", "Select")}</option>{referenceOptions?.overtime.map((option) => <option key={option.id} value={option.id}>{option.label}</option>)}</select>
                </label>
                {versionForm.pay_method === "monthly" ? <>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-monthly-denominator">{t("common.payroll.settings.work_policy_denominator", "Monthly denominator")}<select className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-monthly-denominator" onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, monthly_denominator: event.target.value as typeof form.monthly_denominator })); }} value={versionForm.monthly_denominator}><option value="fixed">fixed</option><option value="schedule">schedule</option></select></label>
                {versionForm.monthly_denominator === "fixed" ? <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-monthly-days">{t("common.payroll.settings.work_policy_days", "Monthly standard days")}<input className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-monthly-days" min="1" onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, monthly_standard_days: event.target.value })); }} step="0.01" type="number" value={versionForm.monthly_standard_days} /></label> : null}
                </> : <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-rate-component">{t("common.payroll.settings.work_policy_rate_component", "Rate component")}<select className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-rate-component" onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, rate_component_definition_id: event.target.value })); }} value={versionForm.rate_component_definition_id}><option value="">{t("common.select", "Select")}</option>{referenceOptions?.salaryComponents.map((option) => <option key={option.id} value={option.id}>{option.label}</option>)}</select></label>}
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-pay-method">
                  {t("common.payroll.settings.work_policy_method", "Pay method")}
                  <select className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-pay-method" onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, pay_method: event.target.value as typeof form.pay_method })); }} value={versionForm.pay_method}>
                    <option value="monthly">monthly</option><option value="hourly">hourly</option><option value="shift">shift</option>
                  </select>
                </label>
              </div>
              {isDraftDirty ? <Typography className="mb-3 text-sm text-amber-700"role="status" variant="p">{t("common.payroll.settings.unsaved_changes", "Unsaved policy changes")}</Typography> : null}
              {versionError ? <Typography className="mb-3 text-sm text-red-600"role="alert" variant="p">{versionError}</Typography> : null}
              {versionConflict ? <Button className="mb-3 underline" onClick={() => { void queryClient.invalidateQueries({ queryKey: payrollKeys.workPolicyHistory(selectedPolicyGroupId) }); setVersionConflict(false); }} size="sm" type="button" variant="ghost">{t("common.payroll.settings.work_policy_refresh", "Refresh current versions")}</Button> : null}
              <Button disabled={isCreatingVersion} onClick={() => void handleCreateVersion()} size="sm" type="button">{isCreatingVersion ? t("common.loading", "Loading") : t("common.payroll.settings.work_policy_create", "Create policy version")}</Button>
              <div className="mt-5 border-t border-slate-200 pt-4 dark:border-slate-800">
                <Typography className="font-semibold" variant="h4">{t("common.payroll.settings.work_policy_memberships", "Assignments")}</Typography>
                <div className="mt-3 grid gap-3 sm:grid-cols-3">
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-membership-employee">{t("common.payroll.settings.work_policy_employee", "Employee")}<select className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-membership-employee" onChange={(event) => { setMembershipEmployeeId(event.target.value); setMembershipForm((form) => ({ ...form, employment_id: "" })); }} value={membershipEmployeeId}><option value="">{t("common.select", "Select")}</option>{employees.map((employee) => <option key={employee.id} value={employee.id}>{employee.full_name ?? employee.employee_id_number ?? employee.id}</option>)}</select></label>
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-employment-id">{t("common.payroll.settings.work_policy_employment_id", "Employment ID")}<select className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-employment-id" onChange={(event) => { setMembershipForm((form) => ({ ...form, employment_id: event.target.value })); }} value={membershipForm.employment_id}><option value="">{t("common.select", "Select")}</option>{membershipEmployments.map((employment) => <option key={employment.id} value={employment.id}>{employment.id} · {employment.status}</option>)}</select></label>
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-membership-from">{t("common.payroll.settings.work_policy_effective_from", "Effective from")}<input className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-membership-from" onChange={(event) => { setMembershipForm((form) => ({ ...form, effective_from: event.target.value })); }} type="date" value={membershipForm.effective_from} /></label>
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-membership-to">{t("common.payroll.settings.work_policy_effective_to", "Effective to")}<input className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="policy-membership-to" onChange={(event) => { setMembershipForm((form) => ({ ...form, effective_to: event.target.value })); }} type="date" value={membershipForm.effective_to} /></label>
                </div>
                {membershipError ? <Typography className="mt-2 text-sm text-red-600"role="alert" variant="p">{membershipError}</Typography> : null}
                {membershipConflict ? <Button className="mt-2 underline" onClick={() => { void queryClient.invalidateQueries({ queryKey: payrollKeys.workPolicyMemberships(selectedPolicyGroupId) }); setMembershipConflict(false); }} size="sm" type="button" variant="ghost">{t("common.payroll.settings.work_policy_membership_refresh", "Refresh current assignments")}</Button> : null}
                <Button className="mt-3" onClick={() => void handleCreateMembership()} size="sm" type="button">{t("common.payroll.settings.work_policy_assign", "Assign membership")}</Button>
                {isMembershipsLoading ? <div className="mt-3 h-8 animate-pulse rounded bg-slate-200 dark:bg-slate-800" /> : <ul className="mt-3 space-y-2 text-sm">{selectedMemberships.map((membership) => <li className="flex justify-between gap-4 border-b border-slate-100 py-2 last:border-0 dark:border-slate-800" key={membership.id}><span>{membership.employment_id}</span><span>{membership.effective_from}{membership.effective_to ? ` – ${membership.effective_to}` : ""}</span></li>)}</ul>}
              </div>
              <section aria-labelledby="draft-policy-simulation-title" className="mt-5 border-t border-slate-200 pt-4 dark:border-slate-800">
                <div aria-label={t("common.payroll.settings.work_policy_parts", "Policy editor sections")} className="mb-4 grid gap-3 sm:grid-cols-4">
                  <div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><Typography className="font-semibold" variant="small">{t("common.payroll.settings.work_policy_schedule", "Schedule")}</Typography><Typography className="mt-1 text-xs text-slate-500"variant="p">{t("common.payroll.settings.work_policy_schedule_desc", "Working pattern reference")}</Typography></div>
                  <div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><Typography className="font-semibold" variant="small">{t("common.payroll.settings.work_policy_credit_section", "Work credit")}</Typography><Typography className="mt-1 text-xs text-slate-500"variant="p">{t("common.payroll.settings.work_policy_credit_desc", "Attendance source and minutes")}</Typography></div>
                  <div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><Typography className="font-semibold" variant="small">{t("common.payroll.settings.work_policy_pay_basis", "Pay basis")}</Typography><Typography className="mt-1 text-xs text-slate-500"variant="p">{t("common.payroll.settings.work_policy_pay_basis_desc", "Monthly, hourly, or shift")}</Typography></div>
                  <div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><Typography className="font-semibold" variant="small">{t("common.payroll.settings.work_policy_overtime_section", "Overtime")}</Typography><Typography className="mt-1 text-xs text-slate-500"variant="p">{t("common.payroll.settings.work_policy_overtime_desc", "Approval and quota reference")}</Typography></div>
                </div>
                <Typography className="font-semibold" id="draft-policy-simulation-title" variant="h4">{t("common.payroll.settings.draft_simulation_title", "Draft policy simulation")}</Typography>
                <Typography className="mt-1 text-xs text-slate-500" variant="small">{t("common.payroll.settings.draft_simulation_desc", "Runs same payroll calculation path and rolls back all writes.")}</Typography>
                <div className="mt-3 grid gap-3 sm:grid-cols-4">
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="draft-simulation-employee">{t("common.payroll.settings.draft_simulation_employee", "Employee")}
                    <select className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="draft-simulation-employee" onChange={(event) => { setDraftSimulationForm((form) => ({ ...form, employee_id: event.target.value })); }} value={draftSimulationForm.employee_id}><option value="">{t("common.select", "Select")}</option>{employees.map((employee) => <option key={employee.id} value={employee.id}>{employee.full_name ?? employee.employee_id_number ?? employee.id}</option>)}</select>
                  </label>
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="draft-simulation-policy">{t("common.payroll.settings.draft_simulation_policy", "Policy version")}
                    <select className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="draft-simulation-policy" onChange={(event) => { setDraftSimulationForm((form) => ({ ...form, policy_version_id: event.target.value })); }} value={draftSimulationForm.policy_version_id}><option value="">{t("common.select", "Select")}</option>{selectedPolicyHistory.map((version) => <option key={version.id} value={version.id}>{version.id} · {version.effective_from}</option>)}</select>
                  </label>
                  {(["year", "month"] as const).map((field) => <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor={`draft-simulation-${field}`} key={field}>{t(`common.payroll.settings.draft_simulation_${field}`, field)}<input className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id={`draft-simulation-${field}`} onChange={(event) => { setDraftSimulationForm((form) => ({ ...form, [field]: event.target.value })); }} type="number" value={draftSimulationForm[field]} /></label>)}
                </div>
                {draftSimulationError ? <Typography className="mt-2 text-sm text-red-700"role="alert" variant="p">{draftSimulationError}</Typography> : null}
                <Button className="mt-3" disabled={isLoadingDraftSimulation} onClick={() => void handleDraftSimulation()} type="button">{isLoadingDraftSimulation ? t("common.loading", "Loading") : t("common.payroll.settings.draft_simulation_run", "Run draft simulation")}</Button>
                {draftSimulationResult ? <div className="mt-3 grid gap-3 sm:grid-cols-2" role="status"><div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><span className="text-xs text-slate-500">{t("common.payroll.settings.draft_simulation_base", "Base pay")}</span><Typography className="font-bold"variant="p">{draftSimulationResult.base_pay_amount}</Typography><Typography className="text-xs text-slate-500"variant="p">{t("common.payroll.settings.draft_simulation_draft", "Draft")}</Typography></div><div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><span className="text-xs text-slate-500">{t("common.payroll.settings.draft_simulation_net", "Net pay")}</span><Typography className="font-bold"variant="p">{draftSimulationResult.net_pay_amount}</Typography><Typography className="text-xs text-slate-500"variant="p">{t("common.payroll.settings.draft_simulation_draft", "Draft")}</Typography></div>{currentSimulationResult ? <><div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><span className="text-xs text-slate-500">{t("common.payroll.settings.draft_simulation_current_base", "Current base pay")}</span><Typography className="font-bold"variant="p">{currentSimulationResult.base_pay_amount}</Typography></div><div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><span className="text-xs text-slate-500">{t("common.payroll.settings.draft_simulation_current_net", "Current net pay")}</span><Typography className="font-bold"variant="p">{currentSimulationResult.net_pay_amount}</Typography></div></> : null}</div> : null}
              </section>
              <ul className="mt-4 space-y-2 text-sm">
                {selectedPolicyHistory.map((version) => (
                  <li className="border-b border-slate-100 py-2 last:border-0 dark:border-slate-800" key={version.id}>
                    <div className="flex justify-between gap-4">
                      <span>{version.effective_from}{version.effective_to ? ` – ${version.effective_to}` : ""}</span>
                      <span className="font-medium">{version.pay_method}</span>
                    </div>
                    <Button className="mt-1 text-xs underline" disabled={validatingVersionId === version.id} onClick={() => void handleValidateVersion(version)} size="sm" type="button" variant="ghost">
                      {validatingVersionId === version.id ? t("common.loading", "Loading") : t("common.payroll.settings.work_policy_validate", "Validate")}
                    </Button>
                    {validationByVersion[version.id] ? (
                      <div className="mt-2 space-y-1 text-xs" role="status">
                        <Typography className={validationByVersion[version.id].valid ? "text-emerald-700" : "text-red-700"}variant="p">
                          {validationByVersion[version.id].valid ? t("common.payroll.settings.work_policy_valid", "Valid") : t("common.payroll.settings.work_policy_blocked", "Blocked")}
                        </Typography>
                        {[...validationByVersion[version.id].blockers, ...validationByVersion[version.id].warnings].map((issue) => (
                          <Typography className={issue.severity === "blocker" ? "text-red-700" : "text-amber-700"}key={`${issue.code}-${issue.field}`} variant="p">
                            {issue.field}: {issue.message}
                          </Typography>
                        ))}
                      </div>
                    ) : null}
                  </li>
                ))}
              </ul>
              </>
            )}
          </div>
        )}
      </section>

      <section aria-labelledby="payroll-impact-preview-title" className="mt-8 rounded-2xl border border-slate-200/80 bg-white p-6 dark:border-slate-800 dark:bg-slate-950">
        <Typography className="font-bold" id="payroll-impact-preview-title" variant="h2">{t("common.payroll.settings.impact_title", "Impact preview")}</Typography>
        <Typography className="mt-1 text-xs text-slate-500" variant="small">{t("common.payroll.settings.impact_desc", "Compare persisted policy-only runs. Preview never changes payroll.")}</Typography>
        <div className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
          <label className="text-xs font-semibold text-slate-600 dark:text-slate-300 sm:col-span-2 lg:col-span-2" htmlFor="payroll-impact-employments">{t("common.payroll.settings.impact_employments", "Employment IDs")}
            <input aria-describedby="payroll-impact-employments-help" className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id="payroll-impact-employments" onChange={(event) => { setImpactForm((form) => ({ ...form, employment_ids: event.target.value })); }} placeholder="UUIDs separated by comma or space" value={impactForm.employment_ids} />
            <span className="mt-1 block font-normal text-slate-500" id="payroll-impact-employments-help">{t("common.payroll.settings.impact_limit", "Maximum 100 IDs")}</span>
          </label>
          {(["year", "month", "before_run_id", "after_run_id"] as const).map((field) => (
            <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor={`payroll-impact-${field}`} key={field}>{t(`common.payroll.settings.impact_${field}`, field.replaceAll("_", " "))}
              <input className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" id={`payroll-impact-${field}`} onChange={(event) => { setImpactForm((form) => ({ ...form, [field]: event.target.value })); }} type={field === "year" || field === "month" ? "number" : "text"} value={impactForm[field]} />
            </label>
          ))}
        </div>
        {impactError ? <Typography className="mt-3 text-sm text-red-700"role="alert" variant="p">{impactError}</Typography> : null}
        <Button className="mt-4" disabled={isLoadingImpact} onClick={() => void handleImpactPreview()} type="button">{isLoadingImpact ? t("common.loading", "Loading") : t("common.payroll.settings.impact_run", "Run impact preview")}</Button>
        {impactResult ? <div className="mt-5 grid gap-3 sm:grid-cols-4" role="status">
          {impactResult.evaluated_at ? <Typography className="sm:col-span-4 text-xs text-slate-500" variant="p">{t("common.payroll.settings.impact_evaluated_at", "Evaluated at")}: {impactResult.evaluated_at}{impactResult.expires_at ? ` · ${t("common.payroll.settings.impact_expires_at", "Expires at")}: ${impactResult.expires_at}` : ""}</Typography> : null}
          {impactResult.expires_at && Date.parse(impactResult.expires_at) <= now ? <Typography className="sm:col-span-4 text-sm text-amber-700" role="alert" variant="p">{t("common.payroll.settings.impact_expired", "Impact preview expired. Refresh before using it.")}</Typography> : null}
          {(["total", "computed", "affected", "uncomputable"] as const).map((key) => <div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800" key={key}><Typography className="text-xs text-slate-500"variant="p">{t(`common.payroll.settings.impact_${key}`, key)}</Typography><Typography className="text-xl font-bold"variant="p">{impactResult[key]}</Typography></div>)}
          <div className="sm:col-span-4 space-y-2 text-sm">{impactResult.items.map((item) => <div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800" key={item.employment_id}><div className="flex flex-wrap justify-between gap-2"><span>{item.employment_id}</span><span>{item.computed ? t("common.payroll.settings.impact_computed", "Computed") : t("common.payroll.settings.impact_uncomputable", "Uncomputable")}{item.stale ? ` · ${t("common.payroll.settings.impact_stale", "Stale")}` : ""}</span></div>{item.stale_reasons?.length ? <Typography className="mt-1 text-amber-700"variant="p">{item.stale_reasons.join(", ")}</Typography> : null}{item.reason ? <Typography className="mt-1 text-red-700"variant="p">{item.reason}</Typography> : null}{item.delta ? <Typography className="mt-1 text-slate-600"variant="p">{t("common.payroll.settings.impact_net_delta", "Net delta")}: {item.delta.net_pay_delta}</Typography> : null}</div>)}</div>
        </div> : null}
      </section>

      <div className="mt-8 p-6 bg-blue-50 dark:bg-blue-950/50 border border-blue-100 dark:border-blue-800 rounded-2xl flex gap-4">
        <div className="w-10 h-10 rounded-xl bg-blue-100 dark:bg-blue-900 flex items-center justify-center text-blue-600 dark:text-blue-300 shrink-0">
          <ShieldAlert size={20} />
        </div>
        <div className="text-xs text-blue-800 leading-relaxed font-medium">
          <Typography className="font-bold uppercase tracking-widest mb-1 text-blue-900" variant="p">{t("common.settings.notes_title", "Note") || t("common.payroll.settings.note_title")}</Typography>
          <Typography className="opacity-80" variant="p">{t("common.payroll.settings.note_desc")}</Typography>
        </div>
      </div>

      <div className="mt-4 p-6 bg-amber-50 dark:bg-amber-950/50 border border-amber-100 dark:border-amber-800 rounded-2xl flex gap-4">
        <div className="w-10 h-10 rounded-xl bg-amber-100 dark:bg-amber-900 flex items-center justify-center text-amber-600 dark:text-amber-300 shrink-0">
          <Clock size={20} />
        </div>
        <div className="text-xs text-amber-800 leading-relaxed font-medium">
          <Typography className="font-bold uppercase tracking-widest mb-1 text-amber-900" variant="p">{t("payroll.settings.max_ot_guide_title")}</Typography>
          <Typography className="opacity-80" variant="p">{t("payroll.settings.max_ot_guide_desc")}</Typography>
        </div>
      </div>
    </div>
  );
}

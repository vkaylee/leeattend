import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ShieldAlert, Clock, History } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { ApiError } from "@/lib/api/errors";
import { createWorkPolicyMembership, createWorkPolicyGroupVersion, fetchPayrollImpactBatch, fetchWorkPolicyGroupHistory, fetchWorkPolicyGroups, fetchWorkPolicyMemberships, fetchWorkPolicyReferenceOptions, simulatePayroll, simulatePayrollWithPolicy, updatePayrollSettings, fetchPayrollSettingsHistory, validateWorkPolicyGroupVersion, type PayrollImpactBatchResponse, type WorkPolicyGroupVersion, type WorkPolicyValidationResponse } from "@/lib/api/payroll";
import { fetchEmployeeEmployments, fetchEmployees } from "@/lib/api/employees";
import { employeeKeys, payrollKeys } from "@/lib/query-keys";

import TemporalConfigTable from "../common/temporal-config-table";


export default function PayrollSettingsSection() {
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
  const [isLoadingImpact, setIsLoadingImpact] = useState(false);

  const { data: history = [], isLoading: isHistoryLoading } = useQuery({ queryKey: payrollKeys.settingsHistory(), queryFn: () => fetchPayrollSettingsHistory() });
  const { data: workPolicies = [], isLoading: isWorkPoliciesLoading, isError: isWorkPoliciesError } = useQuery({
    queryKey: payrollKeys.workPolicyGroups(),
    queryFn: fetchWorkPolicyGroups,
  });
  const { data: selectedPolicyHistory = [], isLoading: isPolicyHistoryLoading } = useQuery({
    queryKey: payrollKeys.workPolicyHistory(selectedPolicyGroupId),
    queryFn: () => fetchWorkPolicyGroupHistory(selectedPolicyGroupId!),
    enabled: Boolean(selectedPolicyGroupId),
  });
  const { data: selectedMemberships = [], isLoading: isMembershipsLoading } = useQuery({
    queryKey: payrollKeys.workPolicyMemberships(selectedPolicyGroupId),
    queryFn: () => fetchWorkPolicyMemberships(selectedPolicyGroupId!),
    enabled: Boolean(selectedPolicyGroupId),
  });
  const { data: referenceOptions } = useQuery({ queryKey: payrollKeys.workPolicyReferences(), queryFn: fetchWorkPolicyReferenceOptions });
  const { data: employeesResponse } = useQuery({ queryKey: employeeKeys.list({ status: "active", emptyQueryBehavior: "all" }), queryFn: () => fetchEmployees(1, 100, "employees:read") });
  const employees = employeesResponse?.data ?? [];
  const { data: membershipEmployments = [] } = useQuery({ queryKey: ["payroll-settings-membership-employments", membershipEmployeeId], queryFn: () => fetchEmployeeEmployments(membershipEmployeeId), enabled: Boolean(membershipEmployeeId) });

  const selectPolicyForEdit = (policy: WorkPolicyGroupVersion) => {
    setSelectedPolicyGroupId(policy.group_id);
    setVersionForm((form) => ({ ...form, attendance_policy_id: policy.attendance_policy_id ?? "", working_pattern_id: policy.working_pattern_id ?? "", overtime_policy_id: policy.overtime_policy_id }));
  };

  useEffect(() => {
    if (!isDraftDirty) return;
    const handleBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => window.removeEventListener("beforeunload", handleBeforeUnload);
  }, [isDraftDirty]);

  const handleCreateVersion = async () => {
    if (!selectedPolicyGroupId) return;
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
    try {
      const [current, draft] = await Promise.all([
        simulatePayroll(draftSimulationForm.employee_id, year, month),
        simulatePayrollWithPolicy(draftSimulationForm.employee_id, year, month, draftSimulationForm.policy_version_id),
      ]);
      setCurrentSimulationResult({ base_pay_amount: current.base_pay_amount, net_pay_amount: current.net_pay_amount });
      setDraftSimulationResult({ base_pay_amount: draft.base_pay_amount, net_pay_amount: draft.net_pay_amount });
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
      setMembershipError(error instanceof Error ? error.message : t("common.error"));
    }
  };

  const handleImpactPreview = async () => {
    const employment_ids = impactForm.employment_ids.split(/[\s,]+/).map((id) => id.trim()).filter(Boolean);
    const year = Number(impactForm.year);
    const month = Number(impactForm.month);
    if (employment_ids.length === 0 || employment_ids.length > 100 || !Number.isInteger(year) || !Number.isInteger(month) || month < 1 || month > 12 || !impactForm.before_run_id || !impactForm.after_run_id || impactForm.before_run_id === impactForm.after_run_id) {
      setImpactError(t("common.payroll.settings.impact_validation", "Enter up to 100 employment IDs, a valid period, and two different calculation run IDs."));
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

        effective_date: data.effective_date
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
         records={history.map(h => ({
          std_working_days: parseFloat(h.fixed_std_days),
          hours_per_day: parseFloat(h.hours_per_day),
          max_ot_hours_per_day: h.max_ot_hours_per_day ? parseFloat(h.max_ot_hours_per_day) : null,
          max_ot_hours_per_month: h.max_ot_hours_per_month ? parseFloat(h.max_ot_hours_per_month) : null,
          effective_date: h.effective_date
        }))}
        title={t("common.payroll.settings.title")}
      />

      <section className="rounded-2xl border border-slate-200/80 bg-slate-50/60 p-6 dark:border-slate-800/80 dark:bg-slate-900/60" aria-labelledby="work-policy-groups-title">
        <div className="mb-4 flex items-center gap-3">
          <History className="h-5 w-5 text-primary-600" aria-hidden="true" />
          <div>
            <Typography id="work-policy-groups-title" className="font-bold" variant="h2">
              {t("common.payroll.settings.work_policy_title", "Labor policy groups")}
            </Typography>
            <Typography className="text-xs text-slate-500" variant="small">
              {t("common.payroll.settings.work_policy_desc", "Read-only effective versions used by payroll")}
            </Typography>
          </div>
        </div>
        {isWorkPoliciesLoading ? (
          <div className="h-16 animate-pulse rounded-lg bg-slate-200/70 dark:bg-slate-800" aria-label={t("common.loading", "Loading")} />
        ) : isWorkPoliciesError ? (
          <p className="text-sm text-red-600" role="alert">{t("common.payroll.settings.work_policy_error", "Unable to load labor policy groups")}</p>
        ) : workPolicies.length === 0 ? (
          <p className="text-sm text-slate-500">{t("common.payroll.settings.work_policy_empty", "No labor policy groups configured")}</p>
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
                  <tr key={policy.id} className="border-t border-slate-200/70 dark:border-slate-800">
                    <td className="px-3 py-3 font-medium">
                      <Button className="text-left text-primary-700 underline-offset-2 hover:underline" onClick={() => selectPolicyForEdit(policy)} size="sm" type="button" variant="ghost">
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
        {selectedPolicyGroupId && (
          <div className="mt-5 rounded-lg border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-slate-950">
            <div className="mb-3 flex items-center justify-between">
              <Typography className="font-semibold" variant="h3">{t("common.payroll.settings.work_policy_history", "Version history")}</Typography>
              <Button className="text-sm text-slate-500 underline" onClick={() => setSelectedPolicyGroupId(undefined)} size="sm" type="button" variant="ghost">{t("common.close", "Close")}</Button>
            </div>
            {isPolicyHistoryLoading ? <div className="h-10 animate-pulse rounded bg-slate-200 dark:bg-slate-800" /> : (
              <>
              <div className="mb-4 grid gap-3 sm:grid-cols-3" aria-label={t("common.payroll.settings.work_policy_create", "Create policy version")}>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-effective-from">
                  {t("common.payroll.settings.work_policy_effective_from", "Effective from")}
                  <input id="policy-effective-from" type="date" value={versionForm.effective_from} onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, effective_from: event.target.value })); }} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" />
                </label>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-minutes-per-credit">
                  {t("common.payroll.settings.work_policy_minutes", "Minutes / credit")}
                  <input id="policy-minutes-per-credit" type="number" min="1" step="1" value={versionForm.minutes_per_credit} onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, minutes_per_credit: event.target.value })); }} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" />
                </label>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-attendance-policy">
                  {t("common.payroll.settings.work_policy_attendance", "Attendance policy")}
                  <select id="policy-attendance-policy" value={versionForm.attendance_policy_id} onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, attendance_policy_id: event.target.value })); }} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900"><option value="">{t("common.select", "Select")}</option>{referenceOptions?.attendance.map((option) => <option key={option.id} value={option.id}>{option.label}</option>)}</select>
                </label>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-working-pattern">
                  {t("common.payroll.settings.work_policy_pattern", "Working pattern")}
                  <select id="policy-working-pattern" value={versionForm.working_pattern_id} onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, working_pattern_id: event.target.value })); }} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900"><option value="">{t("common.select", "Select")}</option>{referenceOptions?.patterns.map((option) => <option key={option.id} value={option.id}>{option.label}</option>)}</select>
                </label>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-overtime">
                  {t("common.payroll.settings.work_policy_overtime", "Overtime policy")}
                  <select id="policy-overtime" value={versionForm.overtime_policy_id} onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, overtime_policy_id: event.target.value })); }} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900"><option value="">{t("common.select", "Select")}</option>{referenceOptions?.overtime.map((option) => <option key={option.id} value={option.id}>{option.label}</option>)}</select>
                </label>
                {versionForm.pay_method === "monthly" ? <>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-monthly-denominator">{t("common.payroll.settings.work_policy_denominator", "Monthly denominator")}<select id="policy-monthly-denominator" value={versionForm.monthly_denominator} onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, monthly_denominator: event.target.value as typeof form.monthly_denominator })); }} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900"><option value="fixed">fixed</option><option value="schedule">schedule</option></select></label>
                {versionForm.monthly_denominator === "fixed" ? <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-monthly-days">{t("common.payroll.settings.work_policy_days", "Monthly standard days")}<input id="policy-monthly-days" type="number" min="1" step="0.01" value={versionForm.monthly_standard_days} onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, monthly_standard_days: event.target.value })); }} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" /></label> : null}
                </> : <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-rate-component">{t("common.payroll.settings.work_policy_rate_component", "Rate component")}<select id="policy-rate-component" value={versionForm.rate_component_definition_id} onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, rate_component_definition_id: event.target.value })); }} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900"><option value="">{t("common.select", "Select")}</option>{referenceOptions?.salaryComponents.map((option) => <option key={option.id} value={option.id}>{option.label}</option>)}</select></label>}
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-pay-method">
                  {t("common.payroll.settings.work_policy_method", "Pay method")}
                  <select id="policy-pay-method" value={versionForm.pay_method} onChange={(event) => { setIsDraftDirty(true); setVersionForm((form) => ({ ...form, pay_method: event.target.value as typeof form.pay_method })); }} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900">
                    <option value="monthly">monthly</option><option value="hourly">hourly</option><option value="shift">shift</option>
                  </select>
                </label>
              </div>
              {isDraftDirty ? <p className="mb-3 text-sm text-amber-700" role="status">{t("common.payroll.settings.unsaved_changes", "Unsaved policy changes")}</p> : null}
              {versionError ? <p className="mb-3 text-sm text-red-600" role="alert">{versionError}</p> : null}
              {versionConflict ? <Button className="mb-3 underline" type="button" size="sm" variant="ghost" onClick={() => { void queryClient.invalidateQueries({ queryKey: payrollKeys.workPolicyHistory(selectedPolicyGroupId) }); setVersionConflict(false); }}>{t("common.payroll.settings.work_policy_refresh", "Refresh current versions")}</Button> : null}
              <Button type="button" size="sm" disabled={isCreatingVersion} onClick={() => void handleCreateVersion()}>{isCreatingVersion ? t("common.loading", "Loading") : t("common.payroll.settings.work_policy_create", "Create policy version")}</Button>
              <div className="mt-5 border-t border-slate-200 pt-4 dark:border-slate-800">
                <Typography className="font-semibold" variant="h4">{t("common.payroll.settings.work_policy_memberships", "Assignments")}</Typography>
                <div className="mt-3 grid gap-3 sm:grid-cols-3">
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-membership-employee">{t("common.payroll.settings.work_policy_employee", "Employee")}<select id="policy-membership-employee" value={membershipEmployeeId} onChange={(event) => { setMembershipEmployeeId(event.target.value); setMembershipForm((form) => ({ ...form, employment_id: "" })); }} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900"><option value="">{t("common.select", "Select")}</option>{employees.map((employee) => <option key={employee.id} value={employee.id}>{employee.full_name ?? employee.employee_id_number ?? employee.id}</option>)}</select></label>
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-employment-id">{t("common.payroll.settings.work_policy_employment_id", "Employment ID")}<select id="policy-employment-id" value={membershipForm.employment_id} onChange={(event) => setMembershipForm((form) => ({ ...form, employment_id: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900"><option value="">{t("common.select", "Select")}</option>{membershipEmployments.map((employment) => <option key={employment.id} value={employment.id}>{employment.id} · {employment.status}</option>)}</select></label>
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-membership-from">{t("common.payroll.settings.work_policy_effective_from", "Effective from")}<input id="policy-membership-from" type="date" value={membershipForm.effective_from} onChange={(event) => setMembershipForm((form) => ({ ...form, effective_from: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" /></label>
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-membership-to">{t("common.payroll.settings.work_policy_effective_to", "Effective to")}<input id="policy-membership-to" type="date" value={membershipForm.effective_to} onChange={(event) => setMembershipForm((form) => ({ ...form, effective_to: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" /></label>
                </div>
                {membershipError ? <p className="mt-2 text-sm text-red-600" role="alert">{membershipError}</p> : null}
                <Button className="mt-3" type="button" size="sm" onClick={() => void handleCreateMembership()}>{t("common.payroll.settings.work_policy_assign", "Assign membership")}</Button>
                {isMembershipsLoading ? <div className="mt-3 h-8 animate-pulse rounded bg-slate-200 dark:bg-slate-800" /> : <ul className="mt-3 space-y-2 text-sm">{selectedMemberships.map((membership) => <li key={membership.id} className="flex justify-between gap-4 border-b border-slate-100 py-2 last:border-0 dark:border-slate-800"><span>{membership.employment_id}</span><span>{membership.effective_from}{membership.effective_to ? ` – ${membership.effective_to}` : ""}</span></li>)}</ul>}
              </div>
              <section className="mt-5 border-t border-slate-200 pt-4 dark:border-slate-800" aria-labelledby="draft-policy-simulation-title">
                <div className="mb-4 grid gap-3 sm:grid-cols-4" aria-label={t("common.payroll.settings.work_policy_parts", "Policy editor sections")}>
                  <div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><Typography className="font-semibold" variant="small">{t("common.payroll.settings.work_policy_schedule", "Schedule")}</Typography><p className="mt-1 text-xs text-slate-500">{t("common.payroll.settings.work_policy_schedule_desc", "Working pattern reference")}</p></div>
                  <div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><Typography className="font-semibold" variant="small">{t("common.payroll.settings.work_policy_credit_section", "Work credit")}</Typography><p className="mt-1 text-xs text-slate-500">{t("common.payroll.settings.work_policy_credit_desc", "Attendance source and minutes")}</p></div>
                  <div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><Typography className="font-semibold" variant="small">{t("common.payroll.settings.work_policy_pay_basis", "Pay basis")}</Typography><p className="mt-1 text-xs text-slate-500">{t("common.payroll.settings.work_policy_pay_basis_desc", "Monthly, hourly, or shift")}</p></div>
                  <div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><Typography className="font-semibold" variant="small">{t("common.payroll.settings.work_policy_overtime_section", "Overtime")}</Typography><p className="mt-1 text-xs text-slate-500">{t("common.payroll.settings.work_policy_overtime_desc", "Approval and quota reference")}</p></div>
                </div>
                <Typography id="draft-policy-simulation-title" className="font-semibold" variant="h4">{t("common.payroll.settings.draft_simulation_title", "Draft policy simulation")}</Typography>
                <Typography className="mt-1 text-xs text-slate-500" variant="small">{t("common.payroll.settings.draft_simulation_desc", "Runs same payroll calculation path and rolls back all writes.")}</Typography>
                <div className="mt-3 grid gap-3 sm:grid-cols-4">
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="draft-simulation-employee">{t("common.payroll.settings.draft_simulation_employee", "Employee")}
                    <select id="draft-simulation-employee" value={draftSimulationForm.employee_id} onChange={(event) => setDraftSimulationForm((form) => ({ ...form, employee_id: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900"><option value="">{t("common.select", "Select")}</option>{employees.map((employee) => <option key={employee.id} value={employee.id}>{employee.full_name ?? employee.employee_id_number ?? employee.id}</option>)}</select>
                  </label>
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="draft-simulation-policy">{t("common.payroll.settings.draft_simulation_policy", "Policy version")}
                    <select id="draft-simulation-policy" value={draftSimulationForm.policy_version_id} onChange={(event) => setDraftSimulationForm((form) => ({ ...form, policy_version_id: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900"><option value="">{t("common.select", "Select")}</option>{selectedPolicyHistory.map((version) => <option key={version.id} value={version.id}>{version.id} · {version.effective_from}</option>)}</select>
                  </label>
                  {(["year", "month"] as const).map((field) => <label key={field} className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor={`draft-simulation-${field}`}>{t(`common.payroll.settings.draft_simulation_${field}`, field)}<input id={`draft-simulation-${field}`} type="number" value={draftSimulationForm[field]} onChange={(event) => setDraftSimulationForm((form) => ({ ...form, [field]: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" /></label>)}
                </div>
                {draftSimulationError ? <p className="mt-2 text-sm text-red-700" role="alert">{draftSimulationError}</p> : null}
                <Button className="mt-3" type="button" disabled={isLoadingDraftSimulation} onClick={() => void handleDraftSimulation()}>{isLoadingDraftSimulation ? t("common.loading", "Loading") : t("common.payroll.settings.draft_simulation_run", "Run draft simulation")}</Button>
                {draftSimulationResult ? <div className="mt-3 grid gap-3 sm:grid-cols-2" role="status"><div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><span className="text-xs text-slate-500">{t("common.payroll.settings.draft_simulation_base", "Base pay")}</span><p className="font-bold">{draftSimulationResult.base_pay_amount}</p><p className="text-xs text-slate-500">{t("common.payroll.settings.draft_simulation_draft", "Draft")}</p></div><div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><span className="text-xs text-slate-500">{t("common.payroll.settings.draft_simulation_net", "Net pay")}</span><p className="font-bold">{draftSimulationResult.net_pay_amount}</p><p className="text-xs text-slate-500">{t("common.payroll.settings.draft_simulation_draft", "Draft")}</p></div>{currentSimulationResult ? <><div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><span className="text-xs text-slate-500">{t("common.payroll.settings.draft_simulation_current_base", "Current base pay")}</span><p className="font-bold">{currentSimulationResult.base_pay_amount}</p></div><div className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><span className="text-xs text-slate-500">{t("common.payroll.settings.draft_simulation_current_net", "Current net pay")}</span><p className="font-bold">{currentSimulationResult.net_pay_amount}</p></div></> : null}</div> : null}
              </section>
              <ul className="mt-4 space-y-2 text-sm">
                {selectedPolicyHistory.map((version) => (
                  <li key={version.id} className="border-b border-slate-100 py-2 last:border-0 dark:border-slate-800">
                    <div className="flex justify-between gap-4">
                      <span>{version.effective_from}{version.effective_to ? ` – ${version.effective_to}` : ""}</span>
                      <span className="font-medium">{version.pay_method}</span>
                    </div>
                    <Button className="mt-1 text-xs underline" disabled={validatingVersionId === version.id} onClick={() => void handleValidateVersion(version)} size="sm" type="button" variant="ghost">
                      {validatingVersionId === version.id ? t("common.loading", "Loading") : t("common.payroll.settings.work_policy_validate", "Validate")}
                    </Button>
                    {validationByVersion[version.id] ? (
                      <div className="mt-2 space-y-1 text-xs" role="status">
                        <p className={validationByVersion[version.id].valid ? "text-emerald-700" : "text-red-700"}>
                          {validationByVersion[version.id].valid ? t("common.payroll.settings.work_policy_valid", "Valid") : t("common.payroll.settings.work_policy_blocked", "Blocked")}
                        </p>
                        {[...validationByVersion[version.id].blockers, ...validationByVersion[version.id].warnings].map((issue) => (
                          <p key={`${issue.code}-${issue.field}`} className={issue.severity === "blocker" ? "text-red-700" : "text-amber-700"}>
                            {issue.field}: {issue.message}
                          </p>
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

      <section className="mt-8 rounded-2xl border border-slate-200/80 bg-white p-6 dark:border-slate-800 dark:bg-slate-950" aria-labelledby="payroll-impact-preview-title">
        <Typography id="payroll-impact-preview-title" className="font-bold" variant="h2">{t("common.payroll.settings.impact_title", "Impact preview")}</Typography>
        <Typography className="mt-1 text-xs text-slate-500" variant="small">{t("common.payroll.settings.impact_desc", "Compare persisted policy-only runs. Preview never changes payroll.")}</Typography>
        <div className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
          <label className="text-xs font-semibold text-slate-600 dark:text-slate-300 sm:col-span-2 lg:col-span-2" htmlFor="payroll-impact-employments">{t("common.payroll.settings.impact_employments", "Employment IDs")}
            <input id="payroll-impact-employments" aria-describedby="payroll-impact-employments-help" value={impactForm.employment_ids} onChange={(event) => setImpactForm((form) => ({ ...form, employment_ids: event.target.value }))} placeholder="UUIDs separated by comma or space" className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" />
            <span id="payroll-impact-employments-help" className="mt-1 block font-normal text-slate-500">{t("common.payroll.settings.impact_limit", "Maximum 100 IDs")}</span>
          </label>
          {(["year", "month", "before_run_id", "after_run_id"] as const).map((field) => (
            <label key={field} className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor={`payroll-impact-${field}`}>{t(`common.payroll.settings.impact_${field}`, field.replaceAll("_", " "))}
              <input id={`payroll-impact-${field}`} type={field === "year" || field === "month" ? "number" : "text"} value={impactForm[field]} onChange={(event) => setImpactForm((form) => ({ ...form, [field]: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" />
            </label>
          ))}
        </div>
        {impactError ? <p className="mt-3 text-sm text-red-700" role="alert">{impactError}</p> : null}
        <Button className="mt-4" type="button" disabled={isLoadingImpact} onClick={() => void handleImpactPreview()}>{isLoadingImpact ? t("common.loading", "Loading") : t("common.payroll.settings.impact_run", "Run impact preview")}</Button>
        {impactResult ? <div className="mt-5 grid gap-3 sm:grid-cols-4" role="status">
          {(["total", "computed", "affected", "uncomputable"] as const).map((key) => <div key={key} className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><p className="text-xs text-slate-500">{t(`common.payroll.settings.impact_${key}`, key)}</p><p className="text-xl font-bold">{impactResult[key]}</p></div>)}
          <div className="sm:col-span-4 space-y-2 text-sm">{impactResult.items.map((item) => <div key={item.employment_id} className="rounded-lg border border-slate-200 p-3 dark:border-slate-800"><div className="flex flex-wrap justify-between gap-2"><span>{item.employment_id}</span><span>{item.computed ? t("common.payroll.settings.impact_computed", "Computed") : t("common.payroll.settings.impact_uncomputable", "Uncomputable")}{item.stale ? ` · ${t("common.payroll.settings.impact_stale", "Stale")}` : ""}</span></div>{item.stale_reasons?.length ? <p className="mt-1 text-amber-700">{item.stale_reasons.join(", ")}</p> : null}{item.reason ? <p className="mt-1 text-red-700">{item.reason}</p> : null}{item.delta ? <p className="mt-1 text-slate-600">{t("common.payroll.settings.impact_net_delta", "Net delta")}: {item.delta.net_pay_delta}</p> : null}</div>)}</div>
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



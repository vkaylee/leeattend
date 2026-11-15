import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ShieldAlert, Clock, History } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { createWorkPolicyMembership, createWorkPolicyGroupVersion, fetchWorkPolicyGroupHistory, fetchWorkPolicyGroups, fetchWorkPolicyMemberships, updatePayrollSettings, fetchPayrollSettingsHistory } from "@/lib/api/payroll";
import { payrollKeys } from "@/lib/query-keys";

import TemporalConfigTable from "../common/temporal-config-table";


export default function PayrollSettingsSection() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [selectedPolicyGroupId, setSelectedPolicyGroupId] = useState<string>();
  const [isCreatingVersion, setIsCreatingVersion] = useState(false);
  const [versionForm, setVersionForm] = useState({ effective_from: "", minutes_per_credit: "", pay_method: "monthly" as "monthly" | "hourly" | "shift" });
  const [versionError, setVersionError] = useState<string>();
  const [membershipForm, setMembershipForm] = useState({ employment_id: "", effective_from: "", effective_to: "" });
  const [membershipError, setMembershipError] = useState<string>();

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

  const handleCreateVersion = async () => {
    if (!selectedPolicyGroupId) return;
    const minutes = Number(versionForm.minutes_per_credit);
    if (!versionForm.effective_from || !Number.isInteger(minutes) || minutes <= 0) {
      setVersionError(t("common.payroll.settings.work_policy_validation", "Enter effective date and positive whole minutes."));
      return;
    }
    setIsCreatingVersion(true);
    setVersionError(undefined);
    try {
      const selected = workPolicies.find((policy) => policy.group_id === selectedPolicyGroupId);
      if (!selected?.attendance_policy_id || !selected.working_pattern_id || !selected.overtime_policy_id) {
        throw new Error(t("common.payroll.settings.work_policy_missing_refs", "Selected policy lacks required references."));
      }
      await createWorkPolicyGroupVersion(selectedPolicyGroupId, {
        name: selected.name,
        minutes_per_credit: minutes,
        attendance_policy_id: selected.attendance_policy_id,
        working_pattern_id: selected.working_pattern_id,
        pay_method: versionForm.pay_method,
        monthly_denominator: versionForm.pay_method === "monthly" ? "fixed" : undefined,
        monthly_standard_days: versionForm.pay_method === "monthly" ? (selected.monthly_standard_days ?? "26") : undefined,
        rate_component_definition_id: selected.rate_component_definition_id ?? null,
        overtime_policy_id: selected.overtime_policy_id,
        effective_from: versionForm.effective_from,
      });
      toast.success(t("common.success"));
      setVersionForm({ effective_from: "", minutes_per_credit: "", pay_method: "monthly" });
      void queryClient.invalidateQueries({ queryKey: payrollKeys.workPolicyGroups() });
      void queryClient.invalidateQueries({ queryKey: payrollKeys.workPolicyHistory(selectedPolicyGroupId) });
    } catch (error: unknown) {
      setVersionError(error instanceof Error ? error.message : t("common.error"));
    } finally {
      setIsCreatingVersion(false);
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
                      <Button className="text-left text-primary-700 underline-offset-2 hover:underline" onClick={() => setSelectedPolicyGroupId(policy.group_id)} size="sm" type="button" variant="ghost">
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
                  <input id="policy-effective-from" type="date" value={versionForm.effective_from} onChange={(event) => setVersionForm((form) => ({ ...form, effective_from: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" />
                </label>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-minutes-per-credit">
                  {t("common.payroll.settings.work_policy_minutes", "Minutes / credit")}
                  <input id="policy-minutes-per-credit" type="number" min="1" step="1" value={versionForm.minutes_per_credit} onChange={(event) => setVersionForm((form) => ({ ...form, minutes_per_credit: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" />
                </label>
                <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-pay-method">
                  {t("common.payroll.settings.work_policy_method", "Pay method")}
                  <select id="policy-pay-method" value={versionForm.pay_method} onChange={(event) => setVersionForm((form) => ({ ...form, pay_method: event.target.value as typeof form.pay_method }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900">
                    <option value="monthly">monthly</option><option value="hourly">hourly</option><option value="shift">shift</option>
                  </select>
                </label>
              </div>
              {versionError ? <p className="mb-3 text-sm text-red-600" role="alert">{versionError}</p> : null}
              <Button type="button" size="sm" disabled={isCreatingVersion} onClick={() => void handleCreateVersion()}>{isCreatingVersion ? t("common.loading", "Loading") : t("common.payroll.settings.work_policy_create", "Create policy version")}</Button>
              <div className="mt-5 border-t border-slate-200 pt-4 dark:border-slate-800">
                <Typography className="font-semibold" variant="h4">{t("common.payroll.settings.work_policy_memberships", "Assignments")}</Typography>
                <div className="mt-3 grid gap-3 sm:grid-cols-3">
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-employment-id">{t("common.payroll.settings.work_policy_employment_id", "Employment ID")}<input id="policy-employment-id" value={membershipForm.employment_id} onChange={(event) => setMembershipForm((form) => ({ ...form, employment_id: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" /></label>
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-membership-from">{t("common.payroll.settings.work_policy_effective_from", "Effective from")}<input id="policy-membership-from" type="date" value={membershipForm.effective_from} onChange={(event) => setMembershipForm((form) => ({ ...form, effective_from: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" /></label>
                  <label className="text-xs font-semibold text-slate-600 dark:text-slate-300" htmlFor="policy-membership-to">{t("common.payroll.settings.work_policy_effective_to", "Effective to")}<input id="policy-membership-to" type="date" value={membershipForm.effective_to} onChange={(event) => setMembershipForm((form) => ({ ...form, effective_to: event.target.value }))} className="mt-1 block h-10 w-full rounded-lg border border-slate-300 bg-white px-3 text-sm dark:border-slate-700 dark:bg-slate-900" /></label>
                </div>
                {membershipError ? <p className="mt-2 text-sm text-red-600" role="alert">{membershipError}</p> : null}
                <Button className="mt-3" type="button" size="sm" onClick={() => void handleCreateMembership()}>{t("common.payroll.settings.work_policy_assign", "Assign membership")}</Button>
                {isMembershipsLoading ? <div className="mt-3 h-8 animate-pulse rounded bg-slate-200 dark:bg-slate-800" /> : <ul className="mt-3 space-y-2 text-sm">{selectedMemberships.map((membership) => <li key={membership.id} className="flex justify-between gap-4 border-b border-slate-100 py-2 last:border-0 dark:border-slate-800"><span>{membership.employment_id}</span><span>{membership.effective_from}{membership.effective_to ? ` – ${membership.effective_to}` : ""}</span></li>)}</ul>}
              </div>
              <ul className="mt-4 space-y-2 text-sm">
                {selectedPolicyHistory.map((version) => (
                  <li key={version.id} className="flex justify-between gap-4 border-b border-slate-100 py-2 last:border-0 dark:border-slate-800">
                    <span>{version.effective_from}{version.effective_to ? ` – ${version.effective_to}` : ""}</span>
                    <span className="font-medium">{version.pay_method}</span>
                  </li>
                ))}
              </ul>
              </>
            )}
          </div>
        )}
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



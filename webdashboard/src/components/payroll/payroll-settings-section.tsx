import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ShieldAlert, Clock, History } from "lucide-react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { fetchWorkPolicyGroups, updatePayrollSettings, fetchPayrollSettingsHistory } from "@/lib/api/payroll";
import { payrollKeys } from "@/lib/query-keys";

import TemporalConfigTable from "../common/temporal-config-table";


export default function PayrollSettingsSection() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: history = [], isLoading: isHistoryLoading } = useQuery({ queryKey: payrollKeys.settingsHistory(), queryFn: () => fetchPayrollSettingsHistory() });
  const { data: workPolicies = [], isLoading: isWorkPoliciesLoading, isError: isWorkPoliciesError } = useQuery({
    queryKey: payrollKeys.workPolicyGroups(),
    queryFn: fetchWorkPolicyGroups,
  });

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
                    <td className="px-3 py-3 font-medium">{policy.name}</td>
                    <td className="px-3 py-3">{policy.pay_method}</td>
                    <td className="px-3 py-3">{policy.minutes_per_credit}</td>
                    <td className="px-3 py-3">{policy.effective_from}{policy.effective_to ? ` – ${policy.effective_to}` : ""}</td>
                  </tr>
                ))}
              </tbody>
            </table>
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



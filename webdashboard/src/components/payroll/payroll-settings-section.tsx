import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ShieldAlert, Clock } from "lucide-react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { updatePayrollSettings, fetchPayrollSettingsHistory } from "@/lib/api/payroll";

import TemporalConfigTable from "../common/temporal-config-table";


interface Props {
  wsId: string;
}

export default function PayrollSettingsSection({ wsId }: Props) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: history = [], isLoading: isHistoryLoading } = useQuery({
    queryKey: ["payroll-settings-history", wsId],
    queryFn: () => fetchPayrollSettingsHistory(wsId),
    enabled: !!wsId,
  });

  const handleSavePolicy = async (data: { std_working_days: number; hours_per_day: number; max_ot_hours_per_day: number | null; max_ot_hours_per_month: number | null; effective_date: string }) => {
    try {
      await updatePayrollSettings(wsId, {
        fixed_std_days: data.std_working_days,
        hours_per_day: data.hours_per_day,
        max_ot_hours_per_day: data.max_ot_hours_per_day,
        max_ot_hours_per_month: data.max_ot_hours_per_month,
 
        effective_date: data.effective_date
      });
      toast.success(t("common.success"));
      queryClient.invalidateQueries({ queryKey: ["payroll-settings", wsId] });
      queryClient.invalidateQueries({ queryKey: ["payroll-settings-history", wsId] });
 
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : t("common.error"));
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-white dark:bg-slate-900 p-8 rounded-[32px] border border-slate-100 dark:border-slate-800 shadow-sm">
        <TemporalConfigTable
          isLoading={isHistoryLoading}
          onSave={handleSavePolicy}
          records={history.map(h => ({
            std_working_days: Number(h.fixed_std_days),
            hours_per_day: Number(h.hours_per_day),
            max_ot_hours_per_day: h.max_ot_hours_per_day != null ? Number(h.max_ot_hours_per_day) : null,
            max_ot_hours_per_month: h.max_ot_hours_per_month != null ? Number(h.max_ot_hours_per_month) : null,
            effective_date: h.effective_date
          }))}
          title={t("common.payroll.settings.title")}
        />

        <div className="mt-8 p-6 bg-blue-50 dark:bg-blue-950/50 border border-blue-100 dark:border-blue-800 rounded-2xl flex gap-4">
          <div className="w-10 h-10 rounded-xl bg-blue-100 dark:bg-blue-900 flex items-center justify-center text-blue-600 dark:text-blue-300 shrink-0">
            <ShieldAlert size={20} />
          </div>
          <div className="text-xs text-blue-800 leading-relaxed font-medium">
            <p className="font-bold uppercase tracking-widest mb-1 text-blue-900">{t("common.payroll.settings.note_title")}</p>
            <p className="opacity-80">{t("common.payroll.settings.note_desc")}</p>
          </div>
        </div>

        <div className="mt-4 p-6 bg-amber-50 dark:bg-amber-950/50 border border-amber-100 dark:border-amber-800 rounded-2xl flex gap-4">
          <div className="w-10 h-10 rounded-xl bg-amber-100 dark:bg-amber-900 flex items-center justify-center text-amber-600 dark:text-amber-300 shrink-0">
            <Clock size={20} />
          </div>
          <div className="text-xs text-amber-800 leading-relaxed font-medium">
            <p className="font-bold uppercase tracking-widest mb-1 text-amber-900">{t("payroll.settings.max_ot_guide_title")}</p>
            <p className="opacity-80">{t("payroll.settings.max_ot_guide_desc")}</p>
          </div>
        </div>
      </div>
    </div>
  );
}



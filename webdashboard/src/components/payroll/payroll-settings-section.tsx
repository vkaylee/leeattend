import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ShieldAlert, Clock } from "lucide-react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { updatePayrollSettings, fetchPayrollSettingsHistory } from "@/lib/api/payroll";

import TemporalConfigTable from "../common/temporal-config-table";


export default function PayrollSettingsSection() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const { data: history = [], isLoading: isHistoryLoading } = useQuery({ queryKey: ["payroll-settings-history"], queryFn: () => fetchPayrollSettingsHistory(),
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
      void queryClient.invalidateQueries({ queryKey: ["payroll-settings"] });
      void queryClient.invalidateQueries({ queryKey: ["payroll-settings-history"] });
 
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



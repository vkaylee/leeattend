import { useQuery, useQueryClient } from "@tanstack/react-query";
import { fetchPayrollSettings, updatePayrollSettings, fetchPayrollSettingsHistory } from "@/lib/api/payroll";
import { useToast } from "@/components/ui/Toast";
import { useTranslation } from "react-i18next";
import TemporalConfigTable from "../common/TemporalConfigTable";
import { ShieldAlert } from "lucide-react";

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

  const handleSavePolicy = async (data: { std_working_days: number; hours_per_day: number; effective_date: string }) => {
    try {
      await updatePayrollSettings(wsId, {
        fixed_std_days: data.std_working_days,
        hours_per_day: data.hours_per_day,
        effective_date: data.effective_date as any
      });
      toast.success(t("common.success"));
      queryClient.invalidateQueries({ queryKey: ["payroll-settings", wsId] });
      queryClient.invalidateQueries({ queryKey: ["payroll-settings-history", wsId] });
    } catch (error: any) {
      toast.error(error.message || t("common.error"));
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-white p-8 rounded-[32px] border border-slate-100 shadow-sm">
        <TemporalConfigTable 
          title={t("common.payroll.settings.title")}
          records={history.map(h => ({
            std_working_days: Number(h.fixed_std_days),
            hours_per_day: Number(h.hours_per_day),
            effective_date: h.effective_date
          }))}
          isLoading={isHistoryLoading}
          onSave={handleSavePolicy}
        />

        <div className="mt-8 p-6 bg-blue-50/50 border border-blue-100 rounded-2xl flex gap-4">
          <div className="w-10 h-10 rounded-xl bg-blue-100 flex items-center justify-center text-blue-600 shrink-0">
            <ShieldAlert size={20} />
          </div>
          <div className="text-xs text-blue-800 leading-relaxed font-medium">
            <p className="font-bold uppercase tracking-widest mb-1 text-blue-900">{t("common.payroll.settings.note_title")}</p>
            <p className="opacity-80">{t("common.payroll.settings.note_desc")}</p>
          </div>
        </div>
      </div>
    </div>
  );
}


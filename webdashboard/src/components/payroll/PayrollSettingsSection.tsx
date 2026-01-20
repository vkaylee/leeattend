import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Calculator, Save, ShieldAlert, History } from "lucide-react";
import { fetchPayrollSettings, updatePayrollSettings, fetchPayrollSettingsHistory } from "@/lib/api/payroll";
import { useToast } from "@/components/Toast";
import { Combobox } from "@/components/ui/Combobox";
import { useTranslation } from "react-i18next";
import { PayrollSettings } from "@/lib/api/types";
import TemporalConfigTable from "../common/TemporalConfigTable";

interface Props {
  wsId: string;
}

export default function PayrollSettingsSection({ wsId }: Props) {
  const { t } = useTranslation();
  const { data: settings, isLoading } = useQuery({
    queryKey: ["payroll-settings", wsId],
    queryFn: () => fetchPayrollSettings(wsId),
    enabled: !!wsId,
  });

  const { data: history = [], isLoading: isHistoryLoading } = useQuery({
    queryKey: ["payroll-settings-history", wsId],
    queryFn: () => fetchPayrollSettingsHistory(wsId),
    enabled: !!wsId,
  });

  if (isLoading) return <div className="p-8 text-center text-slate-400 italic">{t("common.loading")}</div>;

  return (
    <div className="bg-white rounded-3xl border border-slate-100 shadow-sm overflow-hidden">
      {/* Configuration Form */}
      <div className="p-8 border-b border-slate-50">
        <PayrollSettingsForm wsId={wsId} initialSettings={settings} />
      </div>
      
      {/* History Table */}
      <div className="p-8 bg-slate-50/30">
        <TemporalConfigTable 
          title={t("common.payroll.settings.global_history")}
          records={history.map(h => ({
            std_working_days: Number(h.fixed_std_days),
            hours_per_day: Number(h.hours_per_day),
            effective_date: h.effective_date
          }))}
          isLoading={isHistoryLoading}
          onSave={async (data) => {
            await updatePayrollSettings(wsId, {
              fixed_std_days: data.std_working_days,
              hours_per_day: data.hours_per_day,
              effective_date: data.effective_date as any
            });
          }}
        />
      </div>
    </div>
  );
}

interface FormProps {
  wsId: string;
  initialSettings?: PayrollSettings;
}

function PayrollSettingsForm({ wsId, initialSettings }: FormProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();

  const [mode, setMode] = useState<"fixed" | "actual_month">(initialSettings?.std_working_day_mode as any || "fixed");
  const [fixedDays, setFixedDays] = useState(Number(initialSettings?.fixed_std_days || 26));
  const [daysPerWeek, setDaysPerWeek] = useState(initialSettings?.working_days_per_week || 6);
  const [hoursPerDay, setHoursPerDay] = useState(Number(initialSettings?.hours_per_day || 8));
  
  // Default effective date to the first of the current month
  const firstOfOfMonth = new Date();
  firstOfOfMonth.setDate(1);
  const [effectiveDate, setEffectiveDate] = useState(firstOfOfMonth.toISOString().split('T')[0]);

  const updateMutation = useMutation({
    mutationFn: (data: Partial<PayrollSettings>) => updatePayrollSettings(wsId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      queryClient.invalidateQueries({ queryKey: ["payroll-settings", wsId] });
      queryClient.invalidateQueries({ queryKey: ["payroll-settings-history", wsId] });
    },
    onError: () => {
      toast.error(t("common.error"));
    }
  });

  const handleSave = () => {
    updateMutation.mutate({
      std_working_day_mode: mode,
      fixed_std_days: fixedDays,
      working_days_per_week: daysPerWeek,
      hours_per_day: hoursPerDay,
      effective_date: effectiveDate as any,
    });
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center mb-2">
        <h2 className="text-xl font-bold flex items-center gap-2 text-slate-800">
          <Calculator className="w-5 h-5 text-primary-500" />
          {t("common.payroll.settings.title")}
        </h2>
        <button
          onClick={handleSave}
          data-testid="payroll-settings-save"
          disabled={updateMutation.isPending}
          className="flex items-center gap-2 bg-primary-600 text-white px-4 py-2 rounded-xl font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 disabled:opacity-50 text-sm"
        >
          <Save className="w-4 h-4" />
          {t("common.workspace_settings.save_config")}
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="space-y-2">
          <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
            {t("common.payroll.settings.effective_date")}
          </label>
          <input
            type="date"
            value={effectiveDate}
            onChange={(e) => setEffectiveDate(e.target.value)}
            className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2 focus:ring-2 focus:ring-primary-500 outline-none transition-all font-bold text-sm"
          />
        </div>

        <div className="space-y-2">
          <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
            {t("common.payroll.settings.working_day_mode")}
          </label>
          <Combobox
            options={[
              { value: "fixed", label: t("common.payroll.settings.mode_fixed"), description: t("common.payroll.settings.mode_fixed_desc") },
              { value: "actual_month", label: t("common.payroll.settings.mode_actual"), description: t("common.payroll.settings.mode_actual_desc") },
            ]}
            value={mode}
            onChange={(val) => setMode(val as any)}
            className="bg-slate-50 border-slate-200"
          />
        </div>

        {mode === "fixed" ? (
          <div className="space-y-2">
            <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
              {t("common.payroll.settings.fixed_days")}
            </label>
            <input
              type="number"
              value={fixedDays}
              onChange={(e) => setFixedDays(parseFloat(e.target.value))}
              className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 focus:ring-2 focus:ring-primary-500 outline-none transition-all font-bold text-sm"
            />
          </div>
        ) : (
          <div className="space-y-2">
            <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
              {t("common.payroll.settings.days_per_week")}
            </label>
            <Combobox
              options={[
                { value: "5", label: t("common.payroll.settings.5days") },
                { value: "6", label: t("common.payroll.settings.6days") },
                { value: "7", label: t("common.payroll.settings.7days") },
              ]}
              value={daysPerWeek.toString()}
              onChange={(val) => setDaysPerWeek(parseInt(val))}
              className="bg-slate-50 border-slate-200"
            />
          </div>
        )}

        <div className="space-y-2">
          <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
            {t("common.payroll.settings.hours_per_day")}
          </label>
          <input
            type="number"
            value={hoursPerDay}
            onChange={(e) => setHoursPerDay(parseFloat(e.target.value))}
            className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 focus:ring-2 focus:ring-primary-500 outline-none transition-all font-bold text-sm"
          />
        </div>
      </div>
      
      <div className="p-4 bg-blue-50 border border-blue-100 rounded-2xl flex gap-3">
        <ShieldAlert className="w-5 h-5 text-blue-500 shrink-0 mt-0.5" />
        <div className="text-xs text-blue-700 leading-relaxed">
          <p className="font-bold mb-1">{t("common.payroll.settings.note_title")}</p>
          <p>{t("common.payroll.settings.note_desc")}</p>
        </div>
      </div>
    </div>
  );
}

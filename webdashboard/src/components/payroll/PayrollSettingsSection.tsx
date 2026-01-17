import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Calculator, Save, ShieldAlert } from "lucide-react";
import { fetchPayrollSettings, updatePayrollSettings } from "@/lib/api/payroll";
import { useToast } from "@/components/Toast";
import { Combobox } from "@/components/ui/Combobox";
import { useTranslation } from "react-i18next";
import { PayrollSettings } from "@/lib/api/types";

interface Props {
  wsId: string;
}

export default function PayrollSettingsSection({ wsId }: Props) {
  const { data: settings, isLoading } = useQuery({
    queryKey: ["payroll-settings", wsId],
    queryFn: () => fetchPayrollSettings(wsId),
    enabled: !!wsId,
  });

  if (isLoading) return <div className="p-8 text-center text-slate-400 italic">Loading settings...</div>;

  return <PayrollSettingsForm wsId={wsId} initialSettings={settings} />;
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

  const updateMutation = useMutation({
    mutationFn: (data: Partial<PayrollSettings>) => updatePayrollSettings(wsId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      queryClient.invalidateQueries({ queryKey: ["payroll-settings", wsId] });
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
    });
  };

  return (
    <div className="bg-white p-8 rounded-3xl border border-slate-100 shadow-sm">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-xl font-bold flex items-center gap-2 text-slate-800">
          <Calculator className="w-5 h-5 text-primary-500" />
          {t("payroll.settings.title", "Cấu hình ngày công chuẩn")}
        </h2>
        <button
          onClick={handleSave}
          data-testid="payroll-settings-save"
          disabled={updateMutation.isPending}
          className="flex items-center gap-2 bg-primary-600 text-white px-4 py-2 rounded-xl font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 disabled:opacity-50 text-sm"
        >
          <Save className="w-4 h-4" />
          {t("workspace_settings.save_config")}
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="space-y-2">
          <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
            {t("payroll.settings.working_day_mode", "Chế độ tính công chuẩn")}
          </label>
          <Combobox
            options={[
              { value: "fixed", label: t("payroll.settings.mode_fixed", "Số ngày cố định"), description: t("payroll.settings.mode_fixed_desc", "Ví dụ: Luôn tính là 26 ngày/tháng") },
              { value: "actual_month", label: t("payroll.settings.mode_actual", "Theo lịch thực tế"), description: t("payroll.settings.mode_actual_desc", "Tổng số ngày trừ đi ngày nghỉ (T7/CN)") },
            ]}
            value={mode}
            onChange={(val) => setMode(val as any)}
            className="bg-slate-50 border-slate-200"
          />
        </div>

        {mode === "fixed" ? (
          <div className="space-y-2">
            <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
              {t("payroll.settings.fixed_days", "Số ngày công cố định")}
            </label>
            <input
              type="number"
              value={fixedDays}
              onChange={(e) => setFixedDays(parseFloat(e.target.value))}
              className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 focus:ring-2 focus:ring-primary-500 outline-none transition-all"
            />
          </div>
        ) : (
          <div className="space-y-2">
            <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
              {t("payroll.settings.days_per_week", "Số ngày làm việc/tuần")}
            </label>
            <Combobox
              options={[
                { value: "5", label: t("payroll.settings.5days", "5 ngày (Nghỉ T7, CN)") },
                { value: "6", label: t("payroll.settings.6days", "6 ngày (Nghỉ CN)") },
                { value: "7", label: t("payroll.settings.7days", "7 ngày (Không ngày nghỉ)") },
              ]}
              value={daysPerWeek.toString()}
              onChange={(val) => setDaysPerWeek(parseInt(val))}
              className="bg-slate-50 border-slate-200"
            />
          </div>
        )}

        <div className="space-y-2">
          <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
            {t("payroll.settings.hours_per_day", "Số giờ làm việc chuẩn/ngày")}
          </label>
          <input
            type="number"
            value={hoursPerDay}
            onChange={(e) => setHoursPerDay(parseFloat(e.target.value))}
            className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 focus:ring-2 focus:ring-primary-500 outline-none transition-all"
          />
        </div>
      </div>
      
      <div className="mt-6 p-4 bg-blue-50 border border-blue-100 rounded-2xl flex gap-3">
        <ShieldAlert className="w-5 h-5 text-blue-500 shrink-0 mt-0.5" />
        <div className="text-xs text-blue-700 leading-relaxed">
          <p className="font-bold mb-1">{t("payroll.settings.note_title", "Lưu ý về cách tính lương giờ")}</p>
          <p>{t("payroll.settings.note_desc", "Lương giờ dùng để tính OT = (Tổng các khoản Lương cơ bản) / (Số ngày công chuẩn × Số giờ làm chuẩn/ngày).")}</p>
        </div>
      </div>
    </div>
  );
}

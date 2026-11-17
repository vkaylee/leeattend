import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Clock, ShieldAlert, type LucideIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { fetchPayrollSettingsHistory, updatePayrollSettings } from "@/lib/api/payroll";
import { payrollKeys } from "@/lib/query-keys";

import TemporalConfigTable from "../common/temporal-config-table";

interface PayrollPolicyData {
  effective_date: string;
  hours_per_day: number;
  max_ot_hours_per_day: number | null;
  max_ot_hours_per_month: number | null;
  std_working_days: number;
}

export interface PayrollSettingsSectionProps {
  onDirtyChange?: (isDirty: boolean) => void;
}

type PayrollSettingsHistory = Awaited<ReturnType<typeof fetchPayrollSettingsHistory>>;
type PayrollSettingsHistoryRecord = PayrollSettingsHistory[number];

function toPayrollPolicyRecord(record: PayrollSettingsHistoryRecord): PayrollPolicyData {
  return {
    effective_date: record.effective_date,
    hours_per_day: parseFloat(record.hours_per_day),
    max_ot_hours_per_day: record.max_ot_hours_per_day ? parseFloat(record.max_ot_hours_per_day) : null,
    max_ot_hours_per_month: record.max_ot_hours_per_month ? parseFloat(record.max_ot_hours_per_month) : null,
    std_working_days: parseFloat(record.fixed_std_days),
  };
}

interface NoticeCardProps {
  description: string;
  icon: LucideIcon;
  title: string;
  tone: "amber" | "blue";
}

function NoticeCard({ description, icon: Icon, title, tone }: NoticeCardProps) {
  const colors = tone === "blue"
    ? {
        background: "bg-blue-50 dark:bg-blue-950/50",
        border: "border-blue-100 dark:border-blue-800",
        icon: "bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300",
        text: "text-blue-800",
        title: "text-blue-900",
      }
    : {
        background: "bg-amber-50 dark:bg-amber-950/50",
        border: "border-amber-100 dark:border-amber-800",
        icon: "bg-amber-100 dark:bg-amber-900 text-amber-600 dark:text-amber-300",
        text: "text-amber-800",
        title: "text-amber-900",
      };

  return (
    <div className={`mt-4 flex gap-4 rounded-2xl p-6 ${colors.background} ${colors.border}`}>
      <div className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-xl ${colors.icon}`}>
        <Icon size={20} />
      </div>
      <div className={`text-xs font-medium leading-relaxed ${colors.text}`}>
        <Typography className={`mb-1 font-bold uppercase tracking-widest ${colors.title}`} variant="p">{title}</Typography>
        <Typography className="opacity-80" variant="p">{description}</Typography>
      </div>
    </div>
  );
}

function handleBeforeUnload(event: BeforeUnloadEvent) {
  event.preventDefault();
  Reflect.set(event, "returnValue", "");
}

export default function PayrollSettingsSection({ onDirtyChange }: PayrollSettingsSectionProps = {}) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const [isDirty, setIsDirty] = useState(false);
  const { data: history = [], isLoading: isHistoryLoading } = useQuery({
    queryFn: fetchPayrollSettingsHistory,
    queryKey: payrollKeys.settingsHistory(),
  });
  const noteTitle = t("common.settings.notes_title", "Note");
  const resolvedNoteTitle = noteTitle.length > 0 ? noteTitle : t("common.payroll.settings.note_title");

  useEffect(() => {
    onDirtyChange?.(isDirty);
  }, [isDirty, onDirtyChange]);

  useEffect(() => {
    if (!isDirty) return undefined;
    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => {
      window.removeEventListener("beforeunload", handleBeforeUnload);
    };
  }, [isDirty]);

  const handleSavePolicy = async (data: PayrollPolicyData) => {
    setIsDirty(true);
    try {
      await updatePayrollSettings({
        effective_date: data.effective_date,
        fixed_std_days: String(data.std_working_days),
        hours_per_day: String(data.hours_per_day),
        max_ot_hours_per_day: data.max_ot_hours_per_day !== null ? String(data.max_ot_hours_per_day) : null,
        max_ot_hours_per_month: data.max_ot_hours_per_month !== null ? String(data.max_ot_hours_per_month) : null,
      });
      toast.success(t("common.success"));
      setIsDirty(false);
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
        records={history.map(toPayrollPolicyRecord)}
        title={t("common.payroll.settings.title")}
      />

      <NoticeCard
        description={t("common.payroll.settings.note_desc")}
        icon={ShieldAlert}
        title={resolvedNoteTitle}
        tone="blue"
      />
      <NoticeCard
        description={t("payroll.settings.max_ot_guide_desc")}
        icon={Clock}
        title={t("payroll.settings.max_ot_guide_title")}
        tone="amber"
      />
    </div>
  );
}

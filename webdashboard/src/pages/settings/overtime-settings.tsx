import { Clock } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { useWorkspace } from "@/hooks/use-workspace";

import { OvertimeQuotasTab } from "./components/overtime/overtime-quotas-tab";

export default function OvertimeSettings() {
  const { t } = useTranslation();
  const { workspace: ws } = useWorkspace();

  if (!ws) return null;

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-2 border-b border-slate-200/60 dark:border-slate-800/60">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-blue-50 dark:bg-blue-950/70 flex items-center justify-center text-blue-600 dark:text-blue-400 shadow-2xs border border-blue-100/50 dark:border-blue-900/50">
            <Clock size={20} />
          </div>
          <div>
            <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100 normal-case" variant="h3">
              {t("settings.categories.overtime", { defaultValue: "Cài đặt tăng ca" })}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">
              {t("settings.categories.overtime_desc", { defaultValue: "Quản lý hạn mức tăng ca và luồng phê duyệt" })}
            </Typography>
          </div>
        </div>
      </div>

      <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 p-6 sm:p-8 shadow-2xs">
        <div className="animate-in fade-in duration-300">
          <OvertimeQuotasTab wsId={ws.id} />
        </div>
      </div>
    </div>
  );
}

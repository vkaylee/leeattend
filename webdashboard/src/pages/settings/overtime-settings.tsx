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
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-blue-50 dark:bg-blue-950/30 flex items-center justify-center text-blue-600 dark:text-blue-400 shadow-sm border border-blue-100 dark:border-blue-900/50">
            <Clock size={24} />
          </div>
          <div>
            <Typography className="text-2xl font-bold tracking-tight text-slate-850 dark:text-slate-100 normal-case mb-0.5" variant="h3">
              {t("settings.categories.overtime", { defaultValue: "Overtime Settings" })}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">
              {t("settings.categories.overtime_desc", { defaultValue: "Manage overtime quotas and approvals" })}
            </Typography>
          </div>
        </div>
      </div>

      <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-8 shadow-sm">
        <div className="animate-in fade-in slide-in-from-bottom-2 duration-500">
          <OvertimeQuotasTab wsId={ws.id} />
        </div>
      </div>
    </div>
  );
}

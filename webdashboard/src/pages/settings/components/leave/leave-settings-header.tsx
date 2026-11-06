import { CalendarDays } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

export function LeaveSettingsHeader() {
  const { t } = useTranslation();
  return (
    <header className="flex flex-col gap-2 pb-2 border-b border-slate-200/60 dark:border-slate-800/60">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950/70 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-2xs border border-primary-100/50 dark:border-primary-900/50">
          <CalendarDays size={20} />
        </div>
        <div>
          <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100 normal-case" variant="h3">
            {t("settings.leave.title")}
          </Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">
            {t("settings.leave.desc")}
          </Typography>
        </div>
      </div>
    </header>
  );
}

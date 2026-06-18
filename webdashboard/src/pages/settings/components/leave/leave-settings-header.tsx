import { CalendarDays } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

export function LeaveSettingsHeader() {
  const { t } = useTranslation();
  return (
    <header className="flex flex-col gap-2">
      <div className="flex items-center gap-4">
        <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50">
          <CalendarDays size={24} />
        </div>
        <div>
          <Typography className="text-2xl font-bold tracking-tight text-slate-850 dark:text-slate-100 normal-case mb-0.5" variant="h3">
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

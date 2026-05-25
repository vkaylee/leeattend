import { CalendarDays } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

export function LeaveSettingsHeader() {
  const { t } = useTranslation();
  return (
    <header className="flex flex-col gap-2">
      <div className="flex items-center gap-4">
        <div className="bg-primary-600 p-3 rounded-2xl shadow-xl shadow-primary-100 dark:shadow-black/30 rotate-3">
          <CalendarDays className="text-white w-6 h-6" />
        </div>
        <div>
          <Typography className="text-3xl font-bold tracking-tighter text-slate-800 dark:text-slate-100 uppercase leading-none" variant="h2">
            {t("settings.leave.title")}
          </Typography>
          <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mt-1 opacity-60" variant="p">
            {t("settings.leave.desc")}
          </Typography>
        </div>
      </div>
    </header>
  );
}

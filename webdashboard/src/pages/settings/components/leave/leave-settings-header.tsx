import { CalendarDays } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

export function LeaveSettingsHeader() {
  const { t } = useTranslation();
  return (
    <header className="flex flex-col gap-2">
      <div className="flex items-center gap-4">
        <div className="bg-primary-600 p-3 rounded-2xl shadow-xl shadow-primary-100 dark:shadow-black/30 rotate-3">
          <CalendarDays className="text-white w-6 h-6" />
        </div>
        <div>
          <h2 className="text-3xl font-bold tracking-tighter text-slate-800 dark:text-slate-100 uppercase leading-none">
            {t("settings.leave.title")}
          </h2>
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mt-1 opacity-60">
            {t("settings.leave.desc")}
          </p>
        </div>
      </div>
    </header>
  );
}

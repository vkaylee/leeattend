import { User } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

export function EmptyEmployeeState() {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-[40px] h-[500px] flex flex-col items-center justify-center text-center p-12 shadow-sm">
      <div className="bg-slate-50 dark:bg-slate-800 p-6 rounded-[32px] mb-6 animate-pulse text-slate-300 dark:text-slate-600">
        <User className="w-12 h-12" />
      </div>
      <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200" variant="h3">
        {t("shifts.empty_state.title")}
      </Typography>
      <Typography className="text-slate-500 dark:text-slate-400 mt-2 max-w-xs mx-auto text-sm" variant="p">
        {t("shifts.empty_state.desc")}
      </Typography>
    </div>
  );
}

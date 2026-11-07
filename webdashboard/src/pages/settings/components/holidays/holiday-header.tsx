import { Palmtree, Plus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface HolidayHeaderProps {
  onAdd: () => void;
}

export function HolidayHeader({ onAdd }: HolidayHeaderProps) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-2 border-b border-slate-200/60 dark:border-slate-800/60">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-emerald-50 dark:bg-emerald-950/70 flex items-center justify-center text-emerald-600 dark:text-emerald-400 shadow-2xs border border-emerald-100/50 dark:border-emerald-900/50">
          <Palmtree size={20} />
        </div>
        <div>
          <Typography className="text-xl font-bold text-slate-900 dark:text-white tracking-tight" variant="h1">
            {t("holidays.title")}
          </Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium" variant="p">
            {t("holidays.subtitle")}
          </Typography>
        </div>
      </div>
      <Button
        className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-xl font-semibold text-xs hover:bg-primary-700 transition-colors shadow-xs"
        onClick={onAdd}
        variant="ghost"
      >
        <Plus size={15} />
        {t("holidays.add_btn")}
      </Button>
    </div>
  );
}

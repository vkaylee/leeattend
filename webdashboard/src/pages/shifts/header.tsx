import { Link } from "@tanstack/react-router";
import { CalendarRange, Plus, Zap } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { ROUTES } from "@/lib/routes";

export function ShiftsHeader() {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
      <div>
        <Typography className="font-bold mb-0.5 normal-case"
          data-testid="shifts-title"
          variant="h3"
        >
          {t("shifts.title")}
        </Typography>
        <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium opacity-80" variant="small">
          {t("shifts.subtitle")}
        </Typography>
      </div>
      <div className="flex items-center gap-3">
        <Link
          className="bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-200 font-bold px-4 py-2 rounded-xl flex items-center gap-2 border border-slate-200 dark:border-slate-700 shadow-sm transition-all"
          search={{ tab: "patterns" }}
          to={ROUTES.SHIFTS_RULES}
        >
          <CalendarRange className="w-4 h-4 text-primary-500" />
          <span className="text-xs uppercase tracking-widest">
            {t("patterns.manage_btn")}
          </span>
        </Link>
        <Link
          className="bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-200 font-bold px-4 py-2 rounded-xl flex items-center gap-2 border border-slate-200 dark:border-slate-700 shadow-sm transition-all"
          search={{ tab: "rules" }}
          to={ROUTES.SHIFTS_RULES}
        >
          <Zap className="w-4 h-4 text-amber-500" />
          <span className="text-xs uppercase tracking-widest">
            {t("rules.title", "Assignment Rules")}
          </span>
        </Link>
        <Link className="bg-primary-600 hover:bg-primary-700 text-white font-bold px-5 py-2 rounded-xl flex items-center gap-2 shadow-md transition-all text-center"
          to="/shifts/new"
        >
          <Plus className="w-5 h-5" />
          <span className="text-xs uppercase tracking-widest">
            {t("shifts.create_btn")}
          </span>
        </Link>
      </div>
    </div>
  );
}

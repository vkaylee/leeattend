import { Link } from "@tanstack/react-router";
import { CalendarRange, Plus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { ROUTES } from "@/lib/routes";

interface ShiftsHeaderProps {
  onCreateClick: () => void;
}

export function ShiftsHeader({ onCreateClick }: ShiftsHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
      <div>
        <h1
          className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tight"
          data-testid="shifts-title"
        >
          {t("shifts.title")}
        </h1>
        <p className="text-slate-500 dark:text-slate-400 mt-1">
          {t("shifts.subtitle")}
        </p>
      </div>
      <div className="flex items-center gap-3">
        <Link
          className="bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-200 font-bold px-4 py-2.5 rounded-2xl flex items-center gap-2 border border-slate-200 dark:border-slate-700 shadow-sm transition-all"
          search={{ tab: "patterns" } as Record<string, string>}
          to={ROUTES.SETTINGS.TIME}
        >
          <CalendarRange className="w-4 h-4" />
          <span className="text-xs uppercase tracking-widest">
            {t("patterns.manage_btn")}
          </span>
        </Link>
        <button
          className="bg-primary-600 hover:bg-primary-700 text-white font-bold px-6 py-2.5 rounded-2xl flex items-center gap-2 shadow-lg shadow-primary-100 transition-all"
          onClick={() => {
            onCreateClick();
          }}
          type="button"
        >
          <Plus className="w-5 h-5" />
          <span className="text-xs uppercase tracking-widest">
            {t("shifts.create_btn")}
          </span>
        </button>
      </div>
    </div>
  );
}

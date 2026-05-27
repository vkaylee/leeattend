import { Link } from "@tanstack/react-router";
import { CalendarRange, Plus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { ROUTES } from "@/lib/routes";

interface ShiftsHeaderProps {
  onCreateClick: () => void;
}

export function ShiftsHeader({ onCreateClick }: ShiftsHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
      <div>
        <Typography className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tight"
          data-testid="shifts-title"
          variant="h1"
        >
          {t("shifts.title")}
        </Typography>
        <Typography className="text-slate-500 dark:text-slate-400 mt-1" variant="p">
          {t("shifts.subtitle")}
        </Typography>
      </div>
      <div className="flex items-center gap-3">
        <Link
          className="bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-200 font-bold px-4 py-2.5 rounded-2xl flex items-center gap-2 border border-slate-200 dark:border-slate-700 shadow-sm transition-all"
          to={ROUTES.WORKING_PATTERNS}
        >
          <CalendarRange className="w-4 h-4" />
          <span className="text-xs uppercase tracking-widest">
            {t("patterns.manage_btn")}
          </span>
        </Link>
        <Button className="bg-primary-600 hover:bg-primary-700 text-white font-bold px-6 py-2.5 rounded-2xl flex items-center gap-2 shadow-lg shadow-primary-100 transition-all"
          onClick={() => {
            onCreateClick();
          }}
          type="button"
          variant="primary"
        >
          <Plus className="w-5 h-5" />
          <span className="text-xs uppercase tracking-widest">
            {t("shifts.create_btn")}
          </span>
        </Button>
      </div>
    </div>
  );
}

import { PlayCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface LeaveBalancesHeaderProps {
  isPending: boolean;
  onRunAccrual: () => void;
  selectedYear: number;
}

export const LeaveBalancesHeader: React.FC<LeaveBalancesHeaderProps> = ({
  isPending,
  onRunAccrual,
  selectedYear,
}) => {
  const { t } = useTranslation();
  const currentYear = new Date().getFullYear();

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
      <div className="space-y-1">
        <Typography className="text-3xl md:text-5xl font-bold tracking-tighter text-slate-800 dark:text-slate-100 uppercase leading-none" variant="h1">
          {t("leave_balances.title")}
        </Typography>
        <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60" variant="p">
          {t("leave_balances.subtitle")}
        </Typography>
      </div>
      <Button
        className="w-full md:w-auto shadow-primary-100 h-auto py-3.5 md:py-3 px-6 md:px-8"
        disabled={isPending || selectedYear !== currentYear}
        onClick={onRunAccrual}
        size="md"
        title={selectedYear !== currentYear ? t("leave_balances.accrual_current_year_only") : undefined}
        variant="primary"
      >
        <PlayCircle size={18} /> 
        <span className="whitespace-nowrap">
          {isPending ? t("common.processing").toUpperCase() : t("leave_balances.run_accrual").toUpperCase()}
        </span>
      </Button>
    </div>
  );
};

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
      <div>
        <Typography className="font-bold mb-0.5 normal-case" variant="h3">
          {t("leave_balances.title")}
        </Typography>
        <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium opacity-80" variant="small">
          {t("leave_balances.subtitle")}
        </Typography>
      </div>
      <Button
        className="w-full md:w-auto shadow-sm h-10 px-5 rounded-xl font-bold"
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

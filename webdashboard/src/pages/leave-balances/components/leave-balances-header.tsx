import { PlayCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

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
        <h1 className="text-3xl md:text-5xl font-bold tracking-tighter text-slate-800 dark:text-slate-100 uppercase leading-none">
          {t("leave_balances.title")}
        </h1>
        <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">
          {t("leave_balances.subtitle")}
        </p>
      </div>
      <button
        className="bg-primary-600 hover:bg-primary-700 text-white font-bold px-6 md:px-8 py-3.5 md:py-3 rounded-2xl flex items-center justify-center gap-3 shadow-xl shadow-primary-100 transition-all active:scale-95 text-xs uppercase tracking-widest disabled:opacity-50 min-h-[44px] w-full md:w-auto"
        disabled={isPending || selectedYear !== currentYear}
        onClick={onRunAccrual}
        title={selectedYear !== currentYear ? t("leave_balances.accrual_current_year_only") : undefined}
      >
        <PlayCircle size={18} /> 
        <span className="whitespace-nowrap">
          {isPending ? t("common.processing").toUpperCase() : t("leave_balances.run_accrual").toUpperCase()}
        </span>
      </button>
    </div>
  );
};

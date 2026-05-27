import { ArrowLeft, ArrowRight, Calculator } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { MonthYearPicker } from "@/components/common/month-year-picker";
import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface PayrollHeaderProps {
  calcMonth: number;
  calcYear: number;
  onChangeMonth: (delta: number) => void;
  onMonthYearChange: (month: number, year: number) => void;
  onOpenBatchCalc: () => void;
  selectedCount: number;
}

export const PayrollHeader: React.FC<PayrollHeaderProps> = ({
  calcMonth,
  calcYear,
  onChangeMonth,
  onMonthYearChange,
  onOpenBatchCalc,
  selectedCount,
}) => {
  const { t } = useTranslation();

  return (
    <header className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
      <div>
        <Typography className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200" data-testid="payroll-title" variant="h1">
          {t("payroll.title")}
        </Typography>
        <Typography className="text-slate-500 dark:text-slate-400 mt-1 font-medium" variant="p">
          {t("payroll.subtitle")}
        </Typography>
      </div>

      <div className="flex items-center gap-4 bg-white dark:bg-slate-900 p-2 rounded-[2rem] border border-slate-200 dark:border-slate-700 shadow-sm">
        <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-2xl p-1 shadow-inner border border-slate-200 dark:border-slate-700/50">
          <Button 
            className="w-10 h-10 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-sm rounded-xl transition-all" 
            onClick={() => { onChangeMonth(-1); }}
          >
            <ArrowLeft size={18} />
          </Button>
          
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button className="px-6 flex flex-col items-center min-w-[160px] cursor-pointer hover:bg-white dark:hover:bg-slate-900 hover:shadow-sm rounded-xl transition-all py-1.5 group outline-none border border-transparent hover:border-slate-100 dark:border-slate-800">
                <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest leading-none mb-1 group-hover:text-primary-500 dark:group-hover:text-primary-400 transition-colors">
                  {t("payroll.table.period")}
                </span>
                <span className="text-sm font-bold text-slate-700 dark:text-slate-200 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors uppercase tracking-tighter">
                  {calcMonth} / {calcYear}
                </span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="center" className="w-80 p-4 rounded-[2rem] shadow-2xl border-slate-100 dark:border-slate-800" sideOffset={8}>
              <DropdownMenuLabel className="text-center text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 mb-4">
                {t("common.select_month", { defaultValue: "SELECT PAY PERIOD" })}
              </DropdownMenuLabel>
              <MonthYearPicker 
                month={calcMonth} 
                onChange={(m, y) => { onMonthYearChange(m, y); }} 
                year={calcYear} 
              />
            </DropdownMenuContent>
          </DropdownMenu>

          <Button 
            className="w-10 h-10 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-sm rounded-xl transition-all" 
            onClick={() => { onChangeMonth(1); }}
          >
            <ArrowRight size={18} />
          </Button>
        </div>

        <div className="h-10 w-px bg-slate-100 dark:bg-slate-700" />

        <Button 
          className={cn(
            "flex items-center gap-3 px-8 py-3 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all shadow-lg active:scale-95",
            selectedCount > 0 
              ? "bg-primary-600 text-white shadow-primary-100 hover:bg-primary-700" 
              : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 shadow-none cursor-not-allowed"
          )}
          data-testid="calculate-batch-btn"
          disabled={selectedCount === 0}
          onClick={() => { onOpenBatchCalc(); }}
        >
          <Calculator size={18} />
          {t("payroll.calculate_btn")} {selectedCount > 0 && (
            <span className="ml-1 bg-white dark:bg-slate-900/20 px-2 py-0.5 rounded-lg text-white">
              {selectedCount}
            </span>
          )}
        </Button>
      </div>
    </header>
  );
};

import * as Popover from "@radix-ui/react-popover";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import { MonthYearPicker } from "@/components/common/month-year-picker";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface AttendanceHeaderProps {
  currentMonth: Date;
  handleMonthYearChange: (month: number, year: number) => void;
  handleNextMonth: () => void;
  handlePreviousMonth: () => void;
}

export function AttendanceHeader({
  currentMonth,
  handleMonthYearChange,
  handleNextMonth,
  handlePreviousMonth,
}: AttendanceHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
      <div>
        <Typography className="font-bold mb-0.5 normal-case" variant="h3">
          {t("ess.attendance.title")}
        </Typography>
        <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium opacity-80" variant="small">
          {t("ess.attendance.subtitle")}
        </Typography>
      </div>

      <div className="flex items-center gap-4 bg-white dark:bg-slate-900 p-2.5 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-900/5">
        <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-[1.5rem] p-1.5 shadow-inner border border-slate-200 dark:border-slate-700/50">
          <Button
            className="w-12 h-12 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all active:scale-90 border-none shadow-none"
            onClick={handlePreviousMonth}
            size="icon"
            variant="ghost"
          >
            <ChevronLeft size={24} />
          </Button>

          <Popover.Root>
            <Popover.Trigger asChild>
              <Button 
                className="px-10 h-auto flex flex-col items-center min-w-[200px] cursor-pointer hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all py-2 group outline-none border border-transparent hover:border-slate-100 dark:border-slate-800 shadow-none normal-case tracking-normal"
                variant="ghost"
              >
                <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] leading-none mb-1.5 group-hover:text-primary-500 dark:group-hover:text-primary-400 transition-colors">
                  {t("payroll.table.period").toUpperCase()}
                </span>
                <span className="text-sm font-bold text-slate-700 dark:text-slate-200 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors uppercase tracking-tight">
                  {format(currentMonth, "MMMM yyyy")}
                </span>
              </Button>
            </Popover.Trigger>
            <Popover.Portal>
              <Popover.Content
                align="center"
                className="z-50 w-80 p-4 rounded-[2.5rem] shadow-premium-2xl border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 animate-in zoom-in-95 slide-in-from-top-4 origin-top"
                sideOffset={12}
              >
                <MonthYearPicker
                  month={currentMonth.getMonth() + 1}
                  onChange={handleMonthYearChange}
                  year={currentMonth.getFullYear()}
                />
              </Popover.Content>
            </Popover.Portal>
          </Popover.Root>

          <Button
            className="w-12 h-12 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all active:scale-90 border-none shadow-none"
            onClick={handleNextMonth}
            size="icon"
            variant="ghost"
          >
            <ChevronRight size={24} />
          </Button>
        </div>
      </div>
    </div>
  );
}


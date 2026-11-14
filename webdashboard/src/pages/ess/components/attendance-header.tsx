import * as Popover from "@radix-ui/react-popover";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import { MonthYearPicker } from "@/components/common/month-year-picker";
import Button from "@/components/ui/button";
import { PageHeader } from "@/components/ui/page-header";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

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
  const { formatMonthLongYear } = useDateTimeFormatter();

  return (
    <PageHeader
      actions={
        <div className="flex items-center gap-4 bg-white dark:bg-slate-900 p-2.5 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-900/5">
          <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-[1.5rem] p-1.5 shadow-inner border border-slate-200 dark:border-slate-700/50">
            <Button
              aria-label={t("common.prev_period", { defaultValue: "Kỳ trước" })}
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
                    {t("payroll.table.period")}
                  </span>
                  <span className="text-sm font-bold text-slate-700 dark:text-slate-200 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors capitalize tracking-tight">
                    {formatMonthLongYear(currentMonth)}
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
              aria-label={t("common.next_period", { defaultValue: "Kỳ tiếp theo" })}
              className="w-12 h-12 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all active:scale-90 border-none shadow-none"
              onClick={handleNextMonth}
              size="icon"
              variant="ghost"
            >
              <ChevronRight size={24} />
            </Button>
          </div>
        </div>
      }
      className="px-2"
      subtitle={t("ess.attendance.subtitle")}
      title={t("ess.attendance.title")}
    />
  );
}


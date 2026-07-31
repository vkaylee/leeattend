import { CalendarRange } from "lucide-react";

import { MonthYearPicker } from "@/components/common/month-year-picker";
import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useMonthPair } from "@/hooks/use-month-pair";
import { cn } from "@/lib/utils";

export type FilterMode = "this_month" | "last_month" | "custom" | "specific_month";

interface FilterModeButtonsProps {
  className?: string;
  filterMode: FilterMode;
  onFilterModeChange: (mode: FilterMode) => void;
  onSpecificMonthChange: (month: number, year: number) => void;
  specificMonth: { month: number; year: number };
  t: (key: string) => string;
  uppercase?: boolean;
}

export function FilterModeButtons({
  className,
  filterMode,
  onFilterModeChange,
  onSpecificMonthChange,
  specificMonth,
  t,
  uppercase = false,
}: FilterModeButtonsProps) {
  const { curMonth, curYear, prevMonth, prevYear } = useMonthPair();
  const fmt = (s: string) => uppercase ? s.toUpperCase() : s;
  const modeLabel = (mode: string) =>
    mode === "this_month"
      ? fmt(`Tháng ${curMonth}/${curYear}`)
      : fmt(`Tháng ${prevMonth}/${prevYear}`);

  return (
    <div className={cn(
      "flex flex-wrap items-center gap-1 bg-slate-100 dark:bg-slate-700/50 shadow-inner border border-slate-200 dark:border-slate-700/50",
      className ?? "rounded-2xl p-1",
    )}>
      {(["this_month", "last_month"] as const).map(mode => (
        <Button
          className={cn(
            "px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
            filterMode === mode
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          )}
          key={mode}
          onClick={() => { onFilterModeChange(mode); }}
          type="button"
          variant="ghost"
        >
          {modeLabel(mode)}
        </Button>
      ))}
      <div className="h-6 w-px bg-slate-200 dark:bg-slate-600 mx-1" />
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            className={cn(
              "flex items-center gap-2 px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
              filterMode === "specific_month"
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            )}
            type="button"
            variant="ghost"
          >
            <CalendarRange size={14} />
            {filterMode === "specific_month"
              ? `${specificMonth.month}/${specificMonth.year}`
              : fmt(t("common.filters.specific_month"))}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="end"
          className="w-80 p-4 rounded-[2.5rem] shadow-premium-2xl border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900/95 backdrop-blur-xl"
          sideOffset={12}
        >
          <MonthYearPicker month={specificMonth.month} onChange={onSpecificMonthChange} year={specificMonth.year} />
        </DropdownMenuContent>
      </DropdownMenu>
      <Button
        className={cn(
          "px-5 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all",
          filterMode === "custom"
            ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
        )}
        onClick={() => { onFilterModeChange("custom"); }}
        type="button"
        variant="ghost"
      >
        {fmt(t("common.filters.custom"))}
      </Button>
    </div>
  );
}

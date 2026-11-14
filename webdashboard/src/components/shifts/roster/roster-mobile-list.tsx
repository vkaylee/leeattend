import type { Locale } from "date-fns/locale";

import { Check, Clock3 } from "lucide-react";
import React, { memo } from "react";
import { useTranslation } from "react-i18next";

import { format, isToday as isDateToday } from "date-fns";

import type { ComplianceViolation } from "@/lib/api/errors";
import type { EmployeeSchedule, Shift } from "@/lib/api/types";

import { Typography } from "@/components/ui/typography";
import { maybeNull } from "@/types/api-overrides";

import { RosterCellContent } from "./roster-cell";
import { getCellStyles, getComplianceStatus, getHighestPrioritySource, isWeekend } from "./roster-utils";

interface RosterMobileListProps {
  complianceViolations?: ComplianceViolation[];
  currentLocale: Locale;
  monthDays: Date[];
  monthSchedules: EmployeeSchedule[];
  onCellClick: (employeeId: string, dateStr: string) => void;
  onCellKeyDown: (employeeId: string, dateStr: string, e: React.KeyboardEvent) => void;
  rowStatus?: Map<string, "saving" | "saved" | "error">;
  selectedCells: Set<string>;
  shifts?: Shift[];
}

export const RosterMobileList = memo(function RosterMobileList({
  complianceViolations,
  currentLocale,
  monthDays,
  monthSchedules,
  onCellClick,
  onCellKeyDown,
  rowStatus,
  selectedCells,
  shifts,
}: RosterMobileListProps) {
  const { t } = useTranslation();

  return (
    <div
      className="lg:hidden space-y-3 p-3"
      data-testid="monthly-roster-mobile"
    >
      {monthSchedules.map((employeeSchedule) => {
        const employeeId = maybeNull(employeeSchedule.employee_id) ?? "";
        const subtitle = [
          employeeSchedule.employee_id_number,
          employeeSchedule.department_name,
        ]
          .filter(Boolean)
          .join(" · ");
        const status = rowStatus?.get(employeeId);

        return (
          <section
            className="overflow-hidden rounded-2xl border border-slate-100 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900"
            data-testid={`monthly-roster-mobile-employee-${employeeId}`}
            key={employeeSchedule.employment_id || employeeId}
          >
            <header className="flex items-center justify-between gap-3 border-b border-slate-100 bg-slate-50/70 px-4 py-3 dark:border-slate-800 dark:bg-slate-850/60">
              <div className="flex min-w-0 items-center gap-3">
                <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-primary-50 text-xs font-extrabold text-primary-700 dark:bg-primary-950/50 dark:text-primary-300">
                  {employeeSchedule.full_name.substring(0, 2).toUpperCase()}
                </div>
                <div className="min-w-0">
                  <Typography
                    className="truncate text-sm font-bold text-slate-800 dark:text-slate-100"
                    noWrap
                    variant="h3"
                  >
                    {employeeSchedule.full_name}
                  </Typography>
                  {subtitle ? (
                    <Typography
                      className="truncate text-[10px] font-medium text-slate-500 dark:text-slate-400"
                      noWrap
                      variant="p"
                    >
                      {subtitle}
                    </Typography>
                  ) : null}
                </div>
              </div>
              <div
                aria-live="polite"
                className="flex shrink-0 items-center gap-1 text-[10px] font-bold uppercase tracking-wide text-slate-500 dark:text-slate-400"
              >
                {status === "saving" ? <Clock3 aria-hidden="true" size={13} /> : null}
                {status === "saved" ? <Check aria-hidden="true" size={13} /> : null}
                <span className="sr-only">
                  {status === "saving"
                    ? t("common.saving", { defaultValue: "Saving..." })
                    : status === "saved"
                    ? t("common.saved", { defaultValue: "Saved" })
                    : ""}
                </span>
              </div>
            </header>

            <div className="divide-y divide-slate-100 dark:divide-slate-800">
              {monthDays.map((day) => {
                const dateStr = format(day, "yyyy-MM-dd");
                const daySchedules = employeeSchedule.schedules.filter(
                  (schedule) => schedule.date === dateStr,
                );
                const isSelected = selectedCells.has(`${employeeId}:${dateStr}`);
                const isToday = isDateToday(day);
                const isWe = isWeekend(day);
                const source = getHighestPrioritySource(daySchedules);
                const cellStatus = rowStatus?.get(employeeId);
                const statusStyles =
                  cellStatus === "saving"
                    ? " opacity-60 animate-pulse"
                    : cellStatus === "error"
                    ? " ring-1 ring-inset ring-rose-400 dark:ring-rose-500"
                    : "";

                return (
                  <div
                    aria-label={`${employeeSchedule.full_name}, ${format(day, "PPP", { locale: currentLocale })}`}
                    aria-pressed={isSelected}
                    className={`flex min-h-[68px] items-center gap-3 px-3 py-2 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary-500 ${isToday ? "bg-primary-50/40 dark:bg-primary-950/20" : ""}`}
                    data-date={dateStr}
                    data-employee-id={employeeId}
                    data-testid={`monthly-roster-mobile-day-${employeeId}-${dateStr}`}
                    key={dateStr}
                    onClick={() => {
                      onCellClick(employeeId, dateStr);
                    }}
                    onKeyDown={(event) => {
                      onCellKeyDown(employeeId, dateStr, event);
                    }}
                    role="button"
                    tabIndex={0}
                  >
                    <div className="w-14 shrink-0 text-center">
                      <div
                        className={`text-[10px] font-bold uppercase tracking-tight ${isToday ? "text-primary-600 dark:text-primary-400" : isWe ? "text-amber-600 dark:text-amber-400" : "text-slate-500 dark:text-slate-400"}`}
                      >
                        {format(day, "EEE", { locale: currentLocale })}
                      </div>
                      <div
                        className={`mt-0.5 text-lg font-extrabold ${isToday ? "text-primary-600 dark:text-primary-400" : isWe ? "text-amber-600 dark:text-amber-400" : "text-slate-700 dark:text-slate-200"}`}
                      >
                        {format(day, "d")}
                      </div>
                    </div>
                    <div
                      className={`relative min-w-0 flex-1 rounded-xl ${getCellStyles(source, isSelected)}${statusStyles}`}
                    >
                      <RosterCellContent
                        complianceStatus={getComplianceStatus(
                          daySchedules,
                          employeeId,
                          dateStr,
                          complianceViolations,
                        )}
                        daySchedules={daySchedules}
                        shifts={shifts}
                        status={cellStatus}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          </section>
        );
      })}
    </div>
  );
});

RosterMobileList.displayName = "RosterMobileList";

import React, { memo } from "react";
import { useTranslation } from "react-i18next";

import type { Shift } from "@/lib/api/types";

import { ScheduleTracePopover } from "../schedule-trace-popover";
import { getCellStyles, getHighestPrioritySource, getShiftSymbol } from "./roster-utils";

interface RosterCellProps {
  complianceStatus?: "ok" | "warning" | "violation";
  dateStr: string;
  daySchedules: import("@/lib/api/types").EffectiveSchedule[];
  employeeId: string;
  isSelected: boolean;
  shifts?: Shift[];
  status?: "saving" | "saved" | "error";
}

function CellOverlays({ status, complianceStatus, isHoliday, holidayName, hasTraces, workingSchedules, shifts, highestSource, allTraces }: {
  status?: "saving" | "saved" | "error";
  complianceStatus?: "ok" | "warning" | "violation";
  isHoliday: boolean;
  holidayName?: string;
  hasTraces: boolean;
  workingSchedules: import("@/lib/api/types").EffectiveSchedule[];
  shifts?: Shift[];
  highestSource?: string;
  allTraces: import("@/lib/api/types").ScheduleTrace[];
}) {
  const { t } = useTranslation();
  return (
    <>
      {status === "saved" ? (
        <span className="absolute top-0.5 left-0.5 z-30 text-[9px] font-extrabold text-emerald-600 dark:text-emerald-400 animate-in fade-in">
          ✓
        </span>
      ) : null}
      {complianceStatus === "warning" ? (
        <span
          aria-label={t("shifts.compliance.badge_warning")}
          className="absolute top-1 right-1 w-2 h-2 rounded-full bg-amber-500 z-25"
          title={t("shifts.compliance.badge_warning")}
        />
      ) : null}
      {complianceStatus === "violation" ? (
        <span
          aria-label={t("shifts.compliance.badge_violation")}
          className="absolute top-1 right-1 w-2 h-2 rounded-full bg-rose-500 z-25"
          title={t("shifts.compliance.badge_violation")}
        />
      ) : null}
      {isHoliday ? (
        <span
          className="absolute top-1 left-1 text-[8px] bg-amber-100 dark:bg-amber-950 text-amber-600 rounded-full px-0.5 font-extrabold scale-90 border border-amber-200 dark:border-amber-900"
          title={holidayName ?? "Lễ"}
        >
          🎉
        </span>
      ) : null}
      {hasTraces ? (
        <TracePopoverOverlay schedules={workingSchedules} shifts={shifts} source={highestSource ?? "none"} traces={allTraces} />
      ) : null}
    </>
  );
}

interface RosterCellContentProps {
  complianceStatus?: "ok" | "warning" | "violation";
  daySchedules: import("@/lib/api/types").EffectiveSchedule[];
  shifts?: Shift[];
  status?: "saving" | "saved" | "error";
}

export function RosterCellContent({
  complianceStatus,
  daySchedules,
  shifts,
  status,
}: RosterCellContentProps) {
  const workingSchedules = daySchedules.filter((s) => s.shift_id);
  const hasShift = workingSchedules.length > 0;
  const isHoliday = daySchedules.some((s) => s.is_holiday);
  const holidayName = daySchedules.find((s) => s.holiday_name)?.holiday_name ?? undefined;
  const highestSource = getHighestPrioritySource(daySchedules);
  const allTraces = daySchedules.flatMap((s) => s.traces ?? []);
  const hasTraces = allTraces.length > 0;

  return (
    <div className="py-2.5 px-1 min-h-[44px] flex flex-col justify-center items-center gap-1">
      <ShiftContent hasShift={hasShift} shifts={shifts} workingSchedules={workingSchedules} />

      <SourceIndicator source={highestSource} />

      <CellOverlays
        allTraces={allTraces}
        complianceStatus={complianceStatus}
        hasTraces={hasTraces}
        highestSource={highestSource}
        holidayName={holidayName}
        isHoliday={isHoliday}
        shifts={shifts}
        status={status}
        workingSchedules={workingSchedules}
      />
    </div>
  );
}

const RosterCell = memo(function RosterCell({
  complianceStatus,
  dateStr,
  employeeId,
  isSelected,
  daySchedules,
  shifts,
  status,
}: RosterCellProps) {
  const highestSource = getHighestPrioritySource(daySchedules);
  const statusStyles =
    status === "saving"
      ? " opacity-60 animate-pulse"
      : status === "error"
      ? " ring-1 ring-inset ring-rose-400 dark:ring-rose-500"
      : "";
  const cellStyles = getCellStyles(highestSource, isSelected) + statusStyles;

  return (
    <td
      className={cellStyles}
      data-date={dateStr}
      data-employee-id={employeeId}
    >
      <RosterCellContent
        complianceStatus={complianceStatus}
        daySchedules={daySchedules}
        shifts={shifts}
        status={status}
      />
    </td>
  );
});
RosterCell.displayName = "RosterCell";

export { RosterCell };

// ── Internal ──────────────────────────────────────────────────────────

function ShiftContent({
  hasShift,
  shifts,
  workingSchedules,
}: {
  hasShift: boolean;
  shifts: Shift[] | undefined;
  workingSchedules: import("@/lib/api/types").EffectiveSchedule[];
}) {
  if (!hasShift) return <span className="text-[10px] text-slate-350 dark:text-slate-600 font-bold">—</span>;
  return (
    <div className="flex flex-col gap-1 items-center">
      {workingSchedules.slice(0, 3).map((s, idx) => {
        const shiftConfig = shifts?.find((sh) => sh.id === s.shift_id);
        const shiftSymbol = getShiftSymbol(s.shift_name, shiftConfig?.report_symbol);
        const shiftBgClass = s.source === "roster"
          ? "bg-teal-100/60 dark:bg-teal-950/60 text-teal-800 dark:text-teal-200"
          : "bg-slate-100/50 dark:bg-slate-800 text-slate-700 dark:text-slate-300";
        return (
          <span
            className={`text-[9px] font-extrabold tracking-tight px-1.5 py-0.5 rounded-md ${shiftBgClass}`}
            key={idx}
            title={`${s.shift_name ?? ""} (${s.start_time?.substring(0, 5) ?? ""} - ${s.end_time?.substring(0, 5) ?? ""})`}
          >
            {shiftSymbol}
          </span>
        );
      })}
      {workingSchedules.length > 3 ? (
        <span className="text-[8px] font-bold text-slate-400 dark:text-slate-500">+{workingSchedules.length - 3} more</span>
      ) : null}
    </div>
  );
}

function TracePopoverOverlay({
  schedules,
  shifts,
  source,
  traces,
}: {
  schedules: import("@/lib/api/types").EffectiveSchedule[];
  shifts: Shift[] | undefined;
  source: string;
  traces: import("@/lib/api/types").ScheduleTrace[];
}) {
  return (
    <div className="absolute top-0.5 right-0.5 z-30 opacity-0 hover:opacity-100 group-hover:opacity-60 transition-opacity">
      <ScheduleTracePopover
        schedules={schedules.map((s) => {
          const shiftConfig = shifts?.find((sh) => sh.id === s.shift_id);
          const shiftSymbol = getShiftSymbol(s.shift_name, shiftConfig?.report_symbol);
          return { shiftName: s.shift_name, shiftSymbol, startTime: s.start_time, endTime: s.end_time };
        })}
        source={source}
        traces={traces}
      />
    </div>
  );
}

const SourceIndicator = memo(function SourceIndicator({
  source,
}: {
  source?: string;
}) {
  switch (source) {
    case "roster":
      return (
        <span className="absolute bottom-1 right-1 w-1 h-1 rounded-full bg-teal-500" />
      );
    case "pattern":
      return (
        <span className="absolute bottom-1 right-1 w-1 h-1 rounded-full bg-blue-500" />
      );
    case "assignment_rule":
      return (
        <span className="absolute bottom-1 right-1 w-1 h-1 rounded-full bg-violet-500" />
      );
    case "department_pattern":
      return (
        <span className="absolute bottom-1 right-1 w-1 h-1 rounded-full bg-slate-400" />
      );
    case "fixed":
    case "department_fixed":
      return (
        <span className="absolute bottom-1 right-1 w-1 h-1 rounded-full bg-emerald-500" />
      );
    default:
      return null;
  }
});
SourceIndicator.displayName = "SourceIndicator";

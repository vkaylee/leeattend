import { Building2, UserX } from "lucide-react";

import { ScheduleTracePopover } from "@/components/shifts/schedule-trace-popover";
import { cn } from "@/lib/utils";

import type { NoShowRowSharedProps } from "./no-show-mobile-card";

import { formatDateTime, formatShiftTime, zoneLabel, ZoneIcon } from "./utils";

function zoneRowBg(zone: string): string {
  switch (zone) {
    case "b": return "bg-amber-50/60 dark:bg-amber-950/25 border-l-2 border-l-amber-400 dark:border-l-amber-500";
    case "a": return "bg-slate-50/60 dark:bg-slate-800/40";
    default: return "bg-rose-50/40 dark:bg-rose-950/15";
  }
}

function zoneChipBg(zone: string): string {
  return zone === "b"
    ? "bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-300"
    : zone === "a"
      ? "bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400"
      : "bg-rose-100 dark:bg-rose-900/40 text-rose-700 dark:text-rose-300";
}

interface NoShowDesktopRowProps extends NoShowRowSharedProps {
  idx: number;
}

export function NoShowDesktopRow({
  row,
  idx,
  tracesByKey,
  t,
}: NoShowDesktopRowProps) {
  const { key, hint } = zoneLabel(row, t);
  return (
    <tr
      className={cn(
        "transition-colors group",
        zoneRowBg(row.zone),
        row.zone !== "b" && "hover:bg-rose-50/40 dark:hover:bg-rose-950/20",
        idx % 2 === 0 ? "" : "bg-opacity-50",
      )}
    >
      <td className="px-5 py-3.5">
        <span
          className={cn(
            "inline-flex items-center gap-1.5 px-2 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest",
            zoneChipBg(row.zone),
          )}
        >
          <ZoneIcon zone={row.zone} />
          <span>{key}</span>
          {!!hint && <span className="opacity-70 normal-case tracking-normal">· {hint}</span>}
          <span className="sr-only">
            {key}{hint ? `, ${hint}` : ""}
          </span>
        </span>
      </td>
      <td className="px-5 py-3.5">
        <span className="font-mono font-bold text-[10px] text-primary-600 dark:text-primary-400 uppercase tracking-tight bg-primary-50 dark:bg-primary-950 px-2 py-1 rounded-lg border border-primary-100 dark:border-primary-800">
          {row.employee_id_number}
        </span>
      </td>
      <td className="px-5 py-3.5">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-xl bg-rose-100 dark:bg-rose-900/40 flex items-center justify-center shrink-0">
            <UserX className="text-rose-500 dark:text-rose-400" size={14} />
          </div>
          <span className="font-bold text-sm text-slate-800 dark:text-slate-100 tracking-tight">
            {row.employee_name}
          </span>
        </div>
      </td>
      <td className="px-5 py-3.5">
        <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 rounded-xl text-[10px] font-bold uppercase tracking-widest border border-blue-100 dark:border-blue-800">
          <Building2 size={10} />
          {row.department_name}
        </span>
      </td>
      <td className="px-5 py-3.5">
        <div className="relative flex items-center gap-1 w-fit">
          <span className="font-bold text-slate-700 dark:text-slate-200 text-xs tracking-tight">
            {row.shift_name ?? "—"}
          </span>
          <ScheduleTracePopover
            endTime={row.scheduled_end}
            shiftName={row.shift_name}
            shiftSymbol={row.shift_symbol}
            source="system"
            startTime={row.scheduled_start}
            traces={tracesByKey.get(`${row.employment_id}-${row.date}`) ?? []}
          />
        </div>
      </td>
      <td className="px-5 py-3.5 text-center">
        <span className="font-mono font-bold text-sm text-slate-700 dark:text-slate-200 tabular-nums bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-lg">
          {formatShiftTime(row.scheduled_start)}
        </span>
      </td>
      <td className="px-5 py-3.5 text-center">
        <span className="font-mono font-bold text-sm text-slate-700 dark:text-slate-200 tabular-nums bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-lg">
          {formatShiftTime(row.scheduled_end)}
        </span>
      </td>
      <td className="px-5 py-3.5 text-center font-mono text-xs">
        {formatDateTime(row.actual_punch_in)}
      </td>
      <td className="px-5 py-3.5 text-center font-mono text-xs">
        {formatDateTime(row.actual_punch_out)}
      </td>
    </tr>
  );
}

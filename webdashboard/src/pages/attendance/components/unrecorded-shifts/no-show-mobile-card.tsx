import { Building2, UserX } from "lucide-react";

import { ScheduleTracePopover } from "@/components/shifts/schedule-trace-popover";
import { ScheduleTrace } from "@/lib/api";
import { cn } from "@/lib/utils";

import type { ZonedNoShowRow } from "../../hooks/use-no-show-rows";

import { formatDateTime, formatShiftTime, zoneLabel, ZoneIcon } from "./utils";

export interface NoShowRowSharedProps {
  columns: Record<string, string>;
  row: ZonedNoShowRow;
  t: (k: string, o?: Record<string, unknown>) => string;
  tracesByKey: Map<string, ScheduleTrace[]>;
}

function zoneChipBg(zone: string): string {
  return zone === "b"
    ? "bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-300"
    : zone === "a"
      ? "bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400"
      : "bg-rose-100 dark:bg-rose-900/40 text-rose-700 dark:text-rose-300";
}

export function NoShowMobileCard({
  row,
  columns,
  tracesByKey,
  t,
}: NoShowRowSharedProps) {
  const { key } = zoneLabel(row, t);
  return (
    <div className={cn(
      "rounded-2xl border bg-white p-5 shadow-sm dark:bg-slate-900 border-slate-100 dark:border-slate-800",
      row.zone !== "a" && "border-l-4",
      row.zone === "b" && "border-l-amber-400 dark:border-l-amber-500 bg-amber-50/60 dark:bg-amber-950/25",
      row.zone === "c" && "border-l-rose-400 dark:border-l-rose-500 bg-rose-50/40 dark:bg-rose-950/15",
    )}>
      {/* Header: Zone + Employee code */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-3 min-w-0">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-rose-100 dark:bg-rose-900/40">
            <UserX className="text-rose-500 dark:text-rose-400" size={18} />
          </div>
          <div className="min-w-0">
            <div className="text-sm font-bold text-slate-800 dark:text-slate-100 tracking-tight">
              {row.employee_name}
            </div>
            <div className="mt-0.5 font-mono text-[10px] font-bold uppercase tracking-tight text-primary-600 dark:text-primary-400">
              {row.employee_id_number}
            </div>
          </div>
        </div>
        <span className={cn(
          "inline-flex shrink-0 items-center gap-1.5 px-2 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest",
          zoneChipBg(row.zone),
        )}>
          <ZoneIcon zone={row.zone} />
          <span>{key}</span>
        </span>
      </div>

      {/* Detail grid */}
      <div className="mt-4 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{columns.department}</div>
          <div className="mt-1 flex items-center gap-1.5">
            <Building2 className="text-blue-500" size={10} />
            <span className="text-[10px] font-bold uppercase tracking-wider text-blue-700 dark:text-blue-300">
              {row.department_name}
            </span>
          </div>
        </div>
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{columns.shift_type}</div>
          <div className="mt-1 flex items-center gap-1">
            <span className="text-xs font-bold text-slate-700 dark:text-slate-200 tracking-tight">
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
        </div>
      </div>

      {/* Punch times */}
      <div className="mt-3 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{columns.punch_in}</div>
          <div className="mt-1 font-mono text-sm font-bold tabular-nums text-slate-700 dark:text-slate-200">
            {formatDateTime(row.actual_punch_in)}
          </div>
        </div>
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{columns.punch_out}</div>
          <div className="mt-1 font-mono text-sm font-bold tabular-nums text-slate-700 dark:text-slate-200">
            {formatDateTime(row.actual_punch_out)}
          </div>
        </div>
      </div>

      {/* Shift times */}
      <div className="mt-3 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{columns.shift_in}</div>
          <div className="mt-1 font-mono text-sm font-bold tabular-nums text-slate-700 dark:text-slate-200">
            {formatShiftTime(row.scheduled_start)}
          </div>
        </div>
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{columns.shift_out}</div>
          <div className="mt-1 font-mono text-sm font-bold tabular-nums text-slate-700 dark:text-slate-200">
            {formatShiftTime(row.scheduled_end)}
          </div>
        </div>
      </div>
    </div>
  );
}

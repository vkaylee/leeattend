/* eslint-disable @typescript-eslint/no-unnecessary-type-conversion, complexity, max-lines-per-function, react/forbid-elements, unicorn/filename-case, unused-imports/no-unused-vars */
import {
  Building2,
  UserX,
} from "lucide-react";
import React from "react";

import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import { ScheduleTracePopover } from "@/components/shifts/schedule-trace-popover";
import { ScheduleTrace } from "@/lib/api";
import { cn } from "@/lib/utils";

import type { ZonedNoShowRow } from "../../hooks/use-no-show-rows";

import { formatShiftTime, zoneLabel, ZoneIcon } from "./utils";

interface NoShowTableProps {
  filteredRows: ZonedNoShowRow[];
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  loadMoreRef: React.RefObject<HTMLTableRowElement | null>;
  onlyZoneB: boolean;
  t: (k: string, o?: Record<string, unknown>) => string;
  tracesByKey: Map<string, ScheduleTrace[]>;
}

function TableSkeleton() {
  return (
    <div className="space-y-2 animate-pulse">
      {Array.from({ length: 6 }).map((_, i) => (
        <div
          className="h-14 rounded-xl bg-slate-100 dark:bg-slate-800"
          key={i}
        />
      ))}
    </div>
  );
}

function EmptyStateRow({ empty, hint }: { empty: string; hint: string }) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-20 text-center">
      <div className="w-16 h-16 rounded-2xl bg-emerald-50 dark:bg-emerald-950 flex items-center justify-center shadow-inner">
        <UserX className="text-emerald-500 dark:text-emerald-400" size={28} />
      </div>
      <p className="font-bold text-sm text-slate-700 dark:text-slate-200">{empty}</p>
      <p className="text-xs text-slate-400 dark:text-slate-500 max-w-xs">{hint}</p>
    </div>
  );
}

/**
 * Renders the no-show report table with zone-aware row styling,
 * employee details, shift schedule, and infinite-scroll loading sentinel.
 */
export function NoShowTable({
  filteredRows,
  hasNextPage,
  isFetchingNextPage,
  loadMoreRef,
  tracesByKey,
  onlyZoneB,
  t,
}: NoShowTableProps) {
  const columns = {
    zone: t("unrecorded_shifts.zone_header"),
    employee_code: t("unrecorded_shifts.columns.employee_code"),
    employee_name: t("unrecorded_shifts.columns.employee_name"),
    department: t("unrecorded_shifts.columns.department"),
    shift_type: t("unrecorded_shifts.columns.shift_type"),
    shift_in: t("unrecorded_shifts.columns.shift_in"),
    shift_out: t("unrecorded_shifts.columns.shift_out"),
  };

  if (filteredRows.length === 0) {
    return (
      <EmptyStateRow
        empty={
          onlyZoneB
            ? t("unrecorded_shifts.empty_zone_b")
            : t("unrecorded_shifts.empty")
        }
        hint={
          onlyZoneB
            ? t("unrecorded_shifts.empty_zone_b_hint")
            : t("unrecorded_shifts.empty_hint")
        }
      />
    );
  }

  return (
    <div className="rounded-2xl border border-slate-200 dark:border-slate-700 overflow-hidden shadow-sm">
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="bg-slate-50 dark:bg-slate-800/60 border-b border-slate-200 dark:border-slate-700">
              <th className="px-5 py-3 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                {columns.zone}
              </th>
              <th className="px-5 py-3 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                {columns.employee_code}
              </th>
              <th className="px-5 py-3 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                {columns.employee_name}
              </th>
              <th className="px-5 py-3 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                {columns.department}
              </th>
              <th className="px-5 py-3 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                {columns.shift_type}
              </th>
              <th className="px-5 py-3 text-center text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                {columns.shift_in}
              </th>
              <th className="px-5 py-3 text-center text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                {columns.shift_out}
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100 dark:divide-slate-800">

            {filteredRows.map((row, idx) => {
              const { key, hint } = zoneLabel(row, t);
              const rowBg =
                row.zone === "b"
                  ? "bg-amber-50/60 dark:bg-amber-950/25 border-l-2 border-l-amber-400 dark:border-l-amber-500"
                  : row.zone === "a"
                    ? "bg-slate-50/60 dark:bg-slate-800/40"
                    : "bg-rose-50/40 dark:bg-rose-950/15";
              const chip =
                row.zone === "b"
                  ? "bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-300"
                  : row.zone === "a"
                    ? "bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400"
                    : "bg-rose-100 dark:bg-rose-900/40 text-rose-700 dark:text-rose-300";
              return (
                <tr
                  className={cn(
                    "transition-colors group",
                    rowBg,
                    row.zone !== "b" && "hover:bg-rose-50/40 dark:hover:bg-rose-950/20",
                    idx % 2 === 0 ? "" : "bg-opacity-50",
                  )}
                  key={`${row.employment_id}-${row.date}`}
                >
                  {/* Zone — never color-only: icon + text + aria-label */}
                  <td className="px-5 py-3.5">
                    <span
                      className={cn(
                        "inline-flex items-center gap-1.5 px-2 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest",
                        chip,
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

                  {/* Employee Code */}
                  <td className="px-5 py-3.5">
                    <span className="font-mono font-bold text-[10px] text-primary-600 dark:text-primary-400 uppercase tracking-tight bg-primary-50 dark:bg-primary-950 px-2 py-1 rounded-lg border border-primary-100 dark:border-primary-800">
                      {row.employee_id_number}
                    </span>
                  </td>

                  {/* Name */}
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

                  {/* Department */}
                  <td className="px-5 py-3.5">
                    <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 rounded-xl text-[10px] font-bold uppercase tracking-widest border border-blue-100 dark:border-blue-800">
                      <Building2 size={10} />
                      {row.department_name}
                    </span>
                  </td>

                  {/* Shift */}
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

                  {/* Shift In */}
                  <td className="px-5 py-3.5 text-center">
                    <span className="font-mono font-bold text-sm text-slate-700 dark:text-slate-200 tabular-nums bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-lg">
                      {formatShiftTime(row.scheduled_start)}
                    </span>
                  </td>

                  {/* Shift Out */}
                  <td className="px-5 py-3.5 text-center">
                    <span className="font-mono font-bold text-sm text-slate-700 dark:text-slate-200 tabular-nums bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-lg">
                      {formatShiftTime(row.scheduled_end)}
                    </span>
                  </td>
                </tr>
              );
            })}
            {!!hasNextPage && (
              <InfiniteScrollSentinel
                colSpan={7}
                isFetchingNextPage={!!isFetchingNextPage}
                sentinelRef={loadMoreRef}
              />
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}

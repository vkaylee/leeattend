import { ChevronRight, Clock, Zap, History, Pencil, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";

import type { ShiftListTableProps } from "./types";

import { isNightShift } from "../shift-utils";
import { ScopeBadges } from "./scope-badges";
import { ShiftListEmptyState } from "./shift-list-empty-state";

// ── ShiftListTable ─────────────────────────────────────────────────────

/**
 * Renders the desktop table view of shifts with full columns.
 */
export function ShiftListTable({
  shifts,
  onEdit,
  onDelete,
  onViewHistory,
  isFiltered,
  formatTime,
}: ShiftListTableProps) {
  const { t } = useTranslation();

  return (
    <Table className="rounded-none border-none shadow-none">
      <TableHeader>
        <TableRow className="bg-slate-50 dark:bg-slate-800/10">
          <TableHead className="px-8 py-5 border-b border-slate-50 dark:border-slate-800">
            {t("shifts.table.shift_name")}
          </TableHead>
          <TableHead className="px-8 py-5 border-b border-slate-50 dark:border-slate-800 text-center">
            {t("shifts.table.time_range")}
          </TableHead>
          <TableHead className="px-8 py-5 border-b border-slate-50 dark:border-slate-800 text-center">
            {t("shifts.table.break_time")}
          </TableHead>
          <TableHead className="px-8 py-5 border-b border-slate-50 dark:border-slate-800 text-center">
            {t("common.global")}
          </TableHead>
          <TableHead className="px-8 py-5 border-b border-slate-50 dark:border-slate-800" />
        </TableRow>
      </TableHeader>
      <TableBody>
        {shifts.map((shift) => {
          const firstInterval = shift.intervals;
          const iv = firstInterval?.[0];
          const startTime = iv?.start_time ?? null;
          const endTime = iv?.end_time ?? null;
          const breakStart = iv?.break_start ?? null;
          const breakEnd = iv?.break_end ?? null;
          const breakDuration = iv?.break_duration_minutes ?? 0;
          const isAutoDeduct = iv?.is_break_auto_deduct ?? true;
          const hasFixedBreak = !!(breakStart && breakEnd);

          return (
            <TableRow
              className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group"
              key={shift.id}
            >
              <TableCell className="px-8 py-6">
                <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">
                  {shift.name}
                </div>
                <div className="flex items-center gap-2 mt-1">
                  {!!shift.report_symbol && (
                    <span className="text-[9px] font-mono font-bold text-slate-400 dark:text-slate-500">
                      {shift.report_symbol}
                    </span>
                  )}
                  {isNightShift(shift) && (
                    <span className="px-1.5 py-0.5 rounded-full text-[8px] font-bold uppercase tracking-wider bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400 border border-indigo-100 dark:border-indigo-800">
                      {t("shifts.picker.filter_night")}
                    </span>
                  )}
                </div>
                <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold tracking-tighter mt-1">
                  {shift.id}
                </div>
              </TableCell>
              <TableCell className="px-8 py-6 text-center">
                <div className="flex items-center justify-center gap-3">
                  <div className="bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 px-4 py-2 rounded-xl font-mono font-bold text-xs border border-primary-100 dark:border-primary-800 shadow-sm">
                    {startTime ? formatTime(startTime) : "—"}
                  </div>
                  <ChevronRight className="w-4 h-4 text-slate-300 dark:text-slate-600" />
                  <div className="bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 px-4 py-2 rounded-xl font-mono font-bold text-xs border border-primary-100 dark:border-primary-800 shadow-sm">
                    {endTime ? formatTime(endTime) : "—"}
                  </div>
                </div>
              </TableCell>
              <TableCell className="px-8 py-6 text-center">
                {hasFixedBreak ? (
                  <div className="flex flex-col items-center gap-1.5">
                    <div className="flex items-center gap-2 px-3 py-1 bg-slate-50 dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-800 shadow-inner">
                      <span className="text-[10px] text-slate-600 dark:text-slate-300 font-bold font-mono">
                        {breakStart.substring(0, 5)}
                      </span>
                      <span className="text-slate-300 dark:text-slate-600 font-bold">-</span>
                      <span className="text-[10px] text-slate-600 dark:text-slate-300 font-bold font-mono">
                        {breakEnd.substring(0, 5)}
                      </span>
                    </div>
                    {isAutoDeduct ? (
                      <div className="text-[8px] font-bold text-blue-500 uppercase tracking-widest flex items-center gap-1 opacity-80">
                        <Zap className="fill-blue-500" size={10} /> {t("shifts.auto_deduct")}
                      </div>
                    ) : null}
                  </div>
                ) : (
                  <div className="flex items-center justify-center gap-2 font-bold text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    <Clock className="text-slate-200 dark:text-slate-700" size={14} />
                    {breakDuration} {t("common.minutes")}
                  </div>
                )}
              </TableCell>
              <TableCell className="px-8 py-6 text-center">
                <ScopeBadges shift={shift} />
              </TableCell>
              <TableCell className="px-8 py-6 text-right">
                <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-all duration-300">
                  {onViewHistory ? (
                    <Button
                      className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-amber-600 dark:hover:text-amber-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90"
                      onClick={() => { onViewHistory(shift); }}
                      title={t("shifts.revisions.title")}
                      variant="ghost"
                    >
                      <History size={18} />
                    </Button>
                  ) : null}
                  <Button
                    className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90"
                    onClick={() => { onEdit(shift); }}
                    title={t("common.edit")}
                    variant="ghost"
                  >
                    <Pencil size={18} />
                  </Button>
                  {!!onDelete && (
                    <Button
                      className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90"
                      onClick={() => { onDelete(shift); }}
                      title={t("common.delete")}
                      variant="ghost"
                    >
                      <Trash2 size={18} />
                    </Button>
                  )}
                </div>
              </TableCell>
            </TableRow>
          );
        })}
        {shifts.length === 0 && (
          <TableRow>
            <TableCell colSpan={5}>
              <ShiftListEmptyState isFiltered={isFiltered} />
            </TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  );
}

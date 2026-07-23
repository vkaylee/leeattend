/* eslint-disable complexity */
import { ChevronRight, Clock, History, Zap, Pencil, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import ResponsiveTable from "@/components/ui/responsive-table";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Typography } from "@/components/ui/typography";

import { useDateTimeFormatter } from "../../hooks/use-datetime";
import { Shift } from "../../lib/api";

// ── Types ──────────────────────────────────────────────────────────

interface ShiftListProps {
  onDelete?: (shift: Shift) => void;
  onEdit: (shift: Shift) => void;
  onViewHistory?: (shift: Shift) => void;
  shifts: Shift[] | undefined;
}

// ── Mobile Card ────────────────────────────────────────────────────

function ShiftMobileCard({
  shift,
  onDelete,
  onEdit,
  onViewHistory,
}: {
  onDelete?: (shift: Shift) => void;
  onEdit: (shift: Shift) => void;
  onViewHistory?: (shift: Shift) => void;
  shift: Shift;
}) {
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();

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
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900">
      {/* Header: Name + ID */}
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="text-sm font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200">{shift.name}</div>
          <div className="mt-1 font-mono text-[10px] font-bold tracking-tighter text-slate-500 dark:text-slate-400">{shift.id}</div>
        </div>
      </div>

      {/* Time range */}
      <div className="mt-4 flex items-center gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div className="rounded-xl border border-primary-100 bg-white px-4 py-2 font-mono text-xs font-bold text-primary-600 shadow-sm dark:border-primary-800 dark:bg-slate-900 dark:text-primary-400">
          {startTime ? formatTime(startTime) : "—"}
        </div>
        <ChevronRight className="h-4 w-4 text-slate-300 dark:text-slate-600" />
        <div className="rounded-xl border border-primary-100 bg-white px-4 py-2 font-mono text-xs font-bold text-primary-600 shadow-sm dark:border-primary-800 dark:bg-slate-900 dark:text-primary-400">
          {endTime ? formatTime(endTime) : "—"}
        </div>
      </div>

      {/* Break info */}
      <div className="mt-3">
        <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">{t("shifts.table.break_time")}</div>
        <div className="mt-1">
          {hasFixedBreak ? (
            <div className="flex items-center gap-2">
              <span className="rounded-lg border border-slate-100 bg-slate-50 px-3 py-1 font-mono text-[10px] font-bold text-slate-600 shadow-inner dark:border-slate-800 dark:bg-slate-800 dark:text-slate-300">{breakStart.substring(0, 5)}</span>
              <span className="font-bold text-slate-300 dark:text-slate-600">-</span>
              <span className="rounded-lg border border-slate-100 bg-slate-50 px-3 py-1 font-mono text-[10px] font-bold text-slate-600 shadow-inner dark:border-slate-800 dark:bg-slate-800 dark:text-slate-300">{breakEnd.substring(0, 5)}</span>
              {isAutoDeduct ? (
                <span className="flex items-center gap-1 text-[8px] font-bold uppercase tracking-widest text-blue-500 opacity-80">
                  <Zap className="fill-blue-500" size={10} /> {t("shifts.auto_deduct")}
                </span>
              ) : null}
            </div>
          ) : (
            <span className="inline-flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
              <Clock className="text-slate-300 dark:text-slate-600" size={12} />
              {breakDuration} {t("common.minutes")}
            </span>
          )}
        </div>
      </div>

      {/* Actions */}
      <div className="mt-4 flex justify-end gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
        {onViewHistory ? (
          <Button
            className="rounded-lg border border-slate-200 bg-white p-2 text-slate-500 transition-colors hover:bg-amber-50 hover:text-amber-600 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-amber-950/30 dark:hover:text-amber-400"
            onClick={() => { onViewHistory(shift); }}
            title={t("shifts.revisions.title")}
            variant="ghost"
          >
            <History size={16} />
          </Button>
        ) : null}
        <Button
          className="rounded-lg border border-slate-200 bg-white p-2 text-slate-500 transition-colors hover:bg-primary-50 hover:text-primary-600 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-primary-950/30 dark:hover:text-primary-400"
          onClick={() => { onEdit(shift); }}
          title={t("common.edit")}
          variant="ghost"
        >
          <Pencil size={16} />
        </Button>
        {!!onDelete && (
          <Button
            className="rounded-lg border border-slate-200 bg-white p-2 text-slate-500 transition-colors hover:bg-rose-50 hover:text-rose-600 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-rose-950/30 dark:hover:text-rose-400"
            onClick={() => { onDelete(shift); }}
            title={t("common.delete")}
            variant="ghost"
          >
            <Trash2 size={16} />
          </Button>
        )}
      </div>
    </div>
  );
}

// ── Component ──────────────────────────────────────────────────────

export function ShiftList({ shifts, onEdit, onDelete, onViewHistory }: ShiftListProps) {
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm animate-in fade-in slide-in-from-bottom-4 duration-700">
      <div className="p-8 border-b border-slate-50 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 flex items-center justify-between">
        <Typography className="font-bold text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]" variant="h2">{t("shifts.directory_title")}</Typography>
        <span className="text-[10px] font-bold text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-3 py-1 rounded-full border border-primary-100 dark:border-primary-800 shadow-sm">{t("common.total").toUpperCase()}: {shifts?.length ?? 0}</span>
      </div>
      <ResponsiveTable<Shift>
        data={shifts ?? []}
        emptyState={{
          icon: <Clock className="h-10 w-10" />,
          title: t("shifts.messages.no_shifts"),
          description: `${t("shifts.directory_title")} is empty`,
        }}
        getCardKey={(shift) => shift.id}
        renderCard={(shift) => (
          <ShiftMobileCard
            onDelete={onDelete}
            onEdit={onEdit}
            onViewHistory={onViewHistory}
            shift={shift}
          />
        )}
        renderDesktopTable={() => (
          <Table className="rounded-none border-none shadow-none">
            <TableHeader>
              <TableRow className="bg-slate-50 dark:bg-slate-800/10">
                <TableHead className="px-8 py-5 border-b border-slate-50 dark:border-slate-800">{t("shifts.table.shift_name")}</TableHead>
                <TableHead className="px-8 py-5 border-b border-slate-50 dark:border-slate-800 text-center">{t("shifts.table.time_range")}</TableHead>
                <TableHead className="px-8 py-5 border-b border-slate-50 dark:border-slate-800 text-center">{t("shifts.table.break_time")}</TableHead>
                <TableHead className="px-8 py-5 border-b border-slate-50 dark:border-slate-800" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {shifts?.map(shift => {
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
                  <TableRow className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group" key={shift.id}>
                    <TableCell className="px-8 py-6">
                      <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">{shift.name}</div>
                      <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold tracking-tighter mt-1">{shift.id}</div>
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
                            <span className="text-[10px] text-slate-600 dark:text-slate-300 font-bold font-mono">{breakStart.substring(0, 5)}</span>
                            <span className="text-slate-300 dark:text-slate-600 font-bold">-</span>
                            <span className="text-[10px] text-slate-600 dark:text-slate-300 font-bold font-mono">{breakEnd.substring(0, 5)}</span>
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
                    <TableCell className="px-8 py-6 text-right">
                      <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-all duration-300">
                        {onViewHistory ? (
                          <Button className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-amber-600 dark:hover:text-amber-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { onViewHistory(shift); }} title={t("shifts.revisions.title")} variant="ghost"><History size={18} /></Button>
                        ) : null}
                        <Button className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { onEdit(shift); }} title={t("common.edit")} variant="ghost"><Pencil size={18} /></Button>
                        {!!onDelete && <Button className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { onDelete(shift); }} title={t("common.delete")} variant="ghost"><Trash2 size={18} /></Button>}
                      </div>
                    </TableCell>
                  </TableRow>
                );
              })}
              {shifts?.length === 0 && (
                <TableRow>
                  <TableCell className="py-32 text-center bg-slate-50 dark:bg-slate-800/30" colSpan={4}>
                    <div className="bg-white dark:bg-slate-900 w-20 h-20 rounded-[2rem] flex items-center justify-center mx-auto mb-6 shadow-sm border border-slate-100 dark:border-slate-800">
                      <Clock className="text-slate-200 dark:text-slate-700" size={32} />
                    </div>
                    <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">{t("shifts.messages.no_shifts")}</Typography>
                    <Typography className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-1 opacity-50" variant="p">{t("shifts.directory_title")} is empty</Typography>
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        )}
      />
    </div>
  );
}


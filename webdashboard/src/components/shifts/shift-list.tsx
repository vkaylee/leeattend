/* eslint-disable complexity */
import { ChevronRight, Clock, History, Zap, Pencil, Trash2, Search, Building2, MapPin } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import SearchBox from "@/components/common/search-box";
import Button from "@/components/ui/button";
import ResponsiveTable from "@/components/ui/responsive-table";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import { useDateTimeFormatter } from "../../hooks/use-datetime";
import { Shift } from "../../lib/api";
import { isNightShift, shiftMatchesQuery } from "./shift-utils";

// ── Types ──────────────────────────────────────────────────────────

type ListFilter = "all" | "active" | "inactive" | "night";

interface ShiftListProps {
  onDelete?: (shift: Shift) => void;
  onEdit: (shift: Shift) => void;
  onViewHistory?: (shift: Shift) => void;
  shifts: Shift[] | undefined;
}

// ── Filter chip definitions ────────────────────────────────────────

interface ChipDef {
  key: ListFilter;
  labelKey: string;
}

const FILTER_CHIPS: ChipDef[] = [
  { key: "all", labelKey: "shifts.picker.filter_all" },
  { key: "active", labelKey: "common.status_active" },
  { key: "inactive", labelKey: "common.status_inactive" },
  { key: "night", labelKey: "shifts.picker.filter_night" },
];

// ── Scope Badge ────────────────────────────────────────────────────

function ScopeBadges({ shift }: { shift: Shift }) {
  const { t } = useTranslation();
  const hasLocation = !!shift.location_id;
  const hasEmployer = !!shift.employer_id;

  if (!hasLocation && !hasEmployer) {
    return (
      <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-lg bg-slate-100 dark:bg-slate-800 text-[9px] font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500">
        <Building2 size={10} /> {t("common.global")}
      </span>
    );
  }

  return (
    <div className="flex items-center gap-1.5">
      {hasLocation ? (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-lg bg-blue-50 dark:bg-blue-950/40 text-[9px] font-bold uppercase tracking-wider text-blue-600 dark:text-blue-400">
          <MapPin size={10} /> {t("common.office_branch")}
        </span>
      ) : null}
      {hasEmployer ? (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-lg bg-violet-50 dark:bg-violet-950/40 text-[9px] font-bold uppercase tracking-wider text-violet-600 dark:text-violet-400">
          <Building2 size={10} /> {t("common.employer")}
        </span>
      ) : null}
    </div>
  );
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
          {!!shift.report_symbol && <span className="text-[9px] font-mono font-bold text-slate-400 dark:text-slate-500">{shift.report_symbol}</span>}
          <div className="mt-1 font-mono text-[10px] font-bold tracking-tighter text-slate-500 dark:text-slate-400">{shift.id}</div>
        </div>
        {/* Night shift badge on card */}
        {isNightShift(shift) && (
          <span className="shrink-0 px-2 py-0.5 rounded-full text-[8px] font-bold uppercase tracking-wider bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400 border border-indigo-100 dark:border-indigo-800">
            {t("shifts.picker.filter_night")}
          </span>
        )}
      </div>

      {/* Scope badges */}
      <div className="mt-2">
        <ScopeBadges shift={shift} />
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

  const [searchQuery, setSearchQuery] = useState("");
  const [activeFilter, setActiveFilter] = useState<ListFilter>("all");

  const filteredShifts = useMemo(() => {
    const all = shifts ?? [];
    let result = all;

    // Apply search
    if (searchQuery.trim()) {
      result = result.filter((s) => shiftMatchesQuery(s, searchQuery));
    }

    // Apply filter chip
    switch (activeFilter) {
      case "active":
        result = result.filter((s) => s.is_active);
        break;
      case "inactive":
        result = result.filter((s) => !s.is_active);
        break;
      case "night":
        result = result.filter((s) => isNightShift(s));
        break;
      default:
        break;
    }

    return result;
  }, [shifts, searchQuery, activeFilter]);

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm animate-in fade-in slide-in-from-bottom-4 duration-700">
      {/* ── Header ────────────────────────────────────────────────── */}
      <div className="p-8 border-b border-slate-50 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30">
        <div className="flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <Typography className="font-bold text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]" variant="h2">{t("shifts.directory_title")}</Typography>
            <span className="text-[10px] font-bold text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-3 py-1 rounded-full border border-primary-100 dark:border-primary-800 shadow-sm">{t("common.total").toUpperCase()}: {filteredShifts.length}</span>
          </div>

          {/* ── Search + Filter Toolbar ────────────────────────────── */}
          <div className="flex flex-col sm:flex-row gap-3 items-start sm:items-center">
            <SearchBox
              className="w-full sm:max-w-xs"
              onChange={setSearchQuery}
              placeholder={t("shifts.picker.search_placeholder")}
              value={searchQuery}
            />
            <div className="flex gap-2 overflow-x-auto scrollbar-hide">
              {FILTER_CHIPS.map((chip) => (
                <Button
                  aria-pressed={activeFilter === chip.key}
                  className={cn(
                    "shrink-0 px-3 py-1.5 rounded-xl text-[10px] font-bold uppercase tracking-wider transition-all duration-200 h-auto",
                    "border-2 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/50",
                    activeFilter === chip.key
                      ? "bg-primary-600 text-white border-primary-600 shadow-md shadow-primary-200 dark:shadow-primary-900/30"
                      : "bg-white dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-primary-300 dark:hover:border-primary-700 hover:text-primary-600 dark:hover:text-primary-400"
                  )}
                  key={chip.key}
                  onClick={() => { setActiveFilter(chip.key); }}
                  type="button"
                  variant="ghost"
                >
                  {t(chip.labelKey)}
                </Button>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* ── Table / Mobile Cards ──────────────────────────────────── */}
      <ResponsiveTable<Shift>
        data={filteredShifts}
        emptyState={{
          icon: searchQuery || activeFilter !== "all"
            ? <Search className="h-10 w-10" />
            : <Clock className="h-10 w-10" />,
          title: searchQuery || activeFilter !== "all"
            ? t("shifts.picker.no_results_title")
            : t("shifts.messages.no_shifts"),
          description: searchQuery || activeFilter !== "all"
            ? t("shifts.picker.no_results_desc")
            : `${t("shifts.directory_title")} is empty`,
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
                <TableHead className="px-8 py-5 border-b border-slate-50 dark:border-slate-800 text-center">{t("common.global")}</TableHead>
                <TableHead className="px-8 py-5 border-b border-slate-50 dark:border-slate-800" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredShifts.map(shift => {
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
                      <div className="flex items-center gap-2 mt-1">
                        {!!shift.report_symbol && <span className="text-[9px] font-mono font-bold text-slate-400 dark:text-slate-500">{shift.report_symbol}</span>}
                        {isNightShift(shift) && (
                          <span className="px-1.5 py-0.5 rounded-full text-[8px] font-bold uppercase tracking-wider bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400 border border-indigo-100 dark:border-indigo-800">
                            {t("shifts.picker.filter_night")}
                          </span>
                        )}
                      </div>
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
                    <TableCell className="px-8 py-6 text-center">
                      <ScopeBadges shift={shift} />
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
              {filteredShifts.length === 0 && (
                <TableRow>
                  <TableCell className="py-32 text-center bg-slate-50 dark:bg-slate-800/30" colSpan={5}>
                    <div className="bg-white dark:bg-slate-900 w-20 h-20 rounded-[2rem] flex items-center justify-center mx-auto mb-6 shadow-sm border border-slate-100 dark:border-slate-800">
                      {searchQuery || activeFilter !== "all"
                        ? <Search className="text-slate-200 dark:text-slate-700" size={32} />
                        : <Clock className="text-slate-200 dark:text-slate-700" size={32} />
                      }
                    </div>
                    <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">
                      {searchQuery || activeFilter !== "all"
                        ? t("shifts.picker.no_results_title")
                        : t("shifts.messages.no_shifts")
                      }
                    </Typography>
                    <Typography className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-1 opacity-50" variant="p">
                      {searchQuery || activeFilter !== "all"
                        ? t("shifts.picker.no_results_desc")
                        : `${t("shifts.directory_title")} is empty`
                      }
                    </Typography>
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

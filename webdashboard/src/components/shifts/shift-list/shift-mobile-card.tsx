import { ChevronRight, Clock, History, Pencil, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { Shift } from "@/lib/api";

import Button from "@/components/ui/button";

import { isNightShift } from "../shift-utils";
import { ScopeBadges } from "./scope-badges";
import { parseShiftInterval } from "./shift-interval-utils";

// ── ShiftCardTimeRange ──────────────────────────────────────────────────────

function ShiftCardTimeRange({ startTime, endTime }: { startTime: string | null; endTime: string | null }) {
  const fmtTime = (tStr: string | null) => (tStr ? tStr.substring(0, 5) : "—");
  return (
    <div className="mt-4 flex items-center gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
      <div className="rounded-xl border border-primary-100 bg-white px-4 py-2 font-mono text-xs font-bold text-primary-600 shadow-sm dark:border-primary-800 dark:bg-slate-900 dark:text-primary-400">
        {fmtTime(startTime)}
      </div>
      <ChevronRight className="h-4 w-4 text-slate-300 dark:text-slate-600" />
      <div className="rounded-xl border border-primary-100 bg-white px-4 py-2 font-mono text-xs font-bold text-primary-600 shadow-sm dark:border-primary-800 dark:bg-slate-900 dark:text-primary-400">
        {fmtTime(endTime)}
      </div>
    </div>
  );
}

// ── BreakDisplay ──────────────────────────────────────────────────────────

interface BreakDisplayProps {
  breakDuration: number;
  breakEnd: string | null;
  breakStart: string | null;
}

/** Shows either fixed break times or a simple duration. */
function BreakDisplay({ breakStart, breakEnd, breakDuration }: BreakDisplayProps) {
  const { t } = useTranslation();
  if (breakStart && breakEnd) {
    return (
      <div className="flex items-center gap-2">
        <span className="rounded-lg border border-slate-100 bg-slate-50 px-3 py-1 font-mono text-[10px] font-bold text-slate-600 shadow-inner dark:border-slate-800 dark:bg-slate-800 dark:text-slate-300">
          {breakStart.substring(0, 5)}
        </span>
        <span className="font-bold text-slate-300 dark:text-slate-600">-</span>
        <span className="rounded-lg border border-slate-100 bg-slate-50 px-3 py-1 font-mono text-[10px] font-bold text-slate-600 shadow-inner dark:border-slate-800 dark:bg-slate-800 dark:text-slate-300">
          {breakEnd.substring(0, 5)}
        </span>
      </div>
    );
  }
  return (
    <span className="inline-flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
      <Clock className="text-slate-300 dark:text-slate-600" size={12} />
      {breakDuration} {t("common.minutes")}
    </span>
  );
}

// ── ShiftCardHeader ───────────────────────────────────────────────────────

function ShiftCardHeader({ shift }: { shift: Shift }) {
  const { t } = useTranslation();
  return (
    <div className="flex items-start justify-between gap-3">
      <div className="min-w-0 flex-1">
        <div className="text-sm font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200">
          {shift.name}
        </div>
        {!!shift.report_symbol && (
          <span className="text-[9px] font-mono font-bold text-slate-400 dark:text-slate-500">
            {shift.report_symbol}
          </span>
        )}
        <div className="mt-1 font-mono text-[10px] font-bold tracking-tighter text-slate-500 dark:text-slate-400">
          {shift.id}
        </div>
      </div>
      {isNightShift(shift) && (
        <span className="shrink-0 px-2 py-0.5 rounded-full text-[8px] font-bold uppercase tracking-wider bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400 border border-indigo-100 dark:border-indigo-800">
          {t("shifts.picker.filter_night")}
        </span>
      )}
    </div>
  );
}

// ── ShiftCardBreakInfo ─────────────────────────────────────────────────────

function ShiftCardBreakInfo({ breakStart, breakEnd, breakDuration }: BreakDisplayProps) {
  const { t } = useTranslation();
  return (
    <div className="mt-3">
      <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">
        {t("shifts.table.break_time")}
      </div>
      <div className="mt-1">
        <BreakDisplay breakDuration={breakDuration} breakEnd={breakEnd} breakStart={breakStart} />
      </div>
    </div>
  );
}

// ── ShiftCardActions ──────────────────────────────────────────────────────

function ShiftCardActions({ shift, onEdit, onDelete, onViewHistory }: {
  shift: Shift;
  onEdit: (s: Shift) => void;
  onDelete?: (s: Shift) => void;
  onViewHistory?: (s: Shift) => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="mt-4 flex justify-end gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
      {onViewHistory ? (
        <Button className="rounded-lg border border-slate-200 bg-white p-2 text-slate-500 transition-colors hover:bg-amber-50 hover:text-amber-600 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-amber-950/30 dark:hover:text-amber-400" onClick={() => { onViewHistory(shift); }} title={t("shifts.revisions.title")} variant="ghost">
          <History size={16} />
        </Button>
      ) : null}
      <Button className="rounded-lg border border-slate-200 bg-white p-2 text-slate-500 transition-colors hover:bg-primary-50 hover:text-primary-600 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-primary-950/30 dark:hover:text-primary-400" onClick={() => { onEdit(shift); }} title={t("common.edit")} variant="ghost">
        <Pencil size={16} />
      </Button>
      {!!onDelete && (
        <Button className="rounded-lg border border-slate-200 bg-white p-2 text-slate-500 transition-colors hover:bg-rose-50 hover:text-rose-600 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-rose-950/30 dark:hover:text-rose-400" onClick={() => { onDelete(shift); }} title={t("common.delete")} variant="ghost">
          <Trash2 size={16} />
        </Button>
      )}
    </div>
  );
}

// ── ShiftMobileCard ────────────────────────────────────────────────────

interface ShiftMobileCardProps {
  onDelete?: (shift: Shift) => void;
  onEdit: (shift: Shift) => void;
  onViewHistory?: (shift: Shift) => void;
  shift: Shift;
}

/**
 * Mobile card view for a single shift, shown in ResponsiveTable's card mode.
 */
export function ShiftMobileCard({
  shift,
  onDelete,
  onEdit,
  onViewHistory,
}: ShiftMobileCardProps) {
  const { startTime, endTime, breakStart, breakEnd, breakDuration } = parseShiftInterval(shift);

  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900">
      <ShiftCardHeader shift={shift} />

      {/* Scope badges */}
      <div className="mt-2">
        <ScopeBadges shift={shift} />
      </div>

      <ShiftCardTimeRange endTime={endTime} startTime={startTime} />

      <ShiftCardBreakInfo breakDuration={breakDuration} breakEnd={breakEnd} breakStart={breakStart} />

      <ShiftCardActions onDelete={onDelete} onEdit={onEdit} onViewHistory={onViewHistory} shift={shift} />
    </div>
  );
}

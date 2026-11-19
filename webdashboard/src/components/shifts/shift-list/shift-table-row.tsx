import { History, Pencil, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { Shift } from "@/lib/api";

import Button from "@/components/ui/button";
import { TableRow, TableCell } from "@/components/ui/table";

import { isNightShift } from "../shift-utils";
import { BreakDisplayDesktop } from "./break-display-desktop";
import { ScopeBadges } from "./scope-badges";
import { ShiftTimeCell } from "./shift-time-cell";

// ── ShiftRowActions ────────────────────────────────────────────────────────

function ShiftRowActions({ shift, onEdit, onDelete, onViewHistory }: {
  shift: Shift;
  onEdit: (s: Shift) => void;
  onDelete?: (s: Shift) => void;
  onViewHistory?: (s: Shift) => void;
}) {
  const { t } = useTranslation();
  return (
    <TableCell className="px-8 py-6 text-right">
      <div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-all duration-300">
        {onViewHistory ? (
          <Button className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-amber-600 dark:hover:text-amber-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { onViewHistory(shift); }} title={t("shifts.revisions.title")} variant="ghost">
            <History size={18} />
          </Button>
        ) : null}
        <Button className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { onEdit(shift); }} title={t("common.edit")} variant="ghost">
          <Pencil size={18} />
        </Button>
        {!!onDelete && (
          <Button className="p-2.5 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { onDelete(shift); }} title={t("common.delete")} variant="ghost">
            <Trash2 size={18} />
          </Button>
        )}
      </div>
    </TableCell>
  );
}

// ── ShiftTableRow ──────────────────────────────────────────────────────────

interface ShiftTableRowProps {
  formatTime: (t: string) => string;
  onDelete?: (shift: Shift) => void;
  onEdit: (shift: Shift) => void;
  onViewHistory?: (shift: Shift) => void;
  shift: Shift;
}

/** Individual table row for a shift in the desktop ShiftListTable. */
export function ShiftTableRow({ shift, onEdit, onDelete, onViewHistory, formatTime }: ShiftTableRowProps) {
  const { t } = useTranslation();
  const iv = shift.intervals?.[0];
  const startTime = iv?.start_time ?? null;
  const endTime = iv?.end_time ?? null;
  return (
    <TableRow className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group" key={shift.id}>
      <TableCell className="px-8 py-6">
        <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">{shift.name}</div>
        <div className="flex items-center gap-2 mt-1">
          {!!shift.report_symbol && <span className="text-[9px] font-mono font-bold text-slate-400 dark:text-slate-500">{shift.report_symbol}</span>}
          {isNightShift(shift) && <span className="px-1.5 py-0.5 rounded-full text-[8px] font-bold uppercase tracking-wider bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400 border border-indigo-100 dark:border-indigo-800">{t("shifts.picker.filter_night")}</span>}
        </div>
        <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold tracking-tighter mt-1">{shift.id}</div>
      </TableCell>
      <ShiftTimeCell endTime={endTime} formatTime={formatTime} startTime={startTime} />
      <BreakDisplayDesktop endTime={endTime} interval={iv} startTime={startTime} />
      <TableCell className="px-8 py-6 text-center"><ScopeBadges shift={shift} /></TableCell>
      <ShiftRowActions onDelete={onDelete} onEdit={onEdit} onViewHistory={onViewHistory} shift={shift} />
    </TableRow>
  );
}

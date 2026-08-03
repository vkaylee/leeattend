import React from "react";
import { useTranslation } from "react-i18next";

import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";

import type { ShiftListTableProps } from "./types";

import { ShiftListEmptyState } from "./shift-list-empty-state";
import { ShiftTableRow } from "./shift-table-row";

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
        {shifts.map((shift) => (
          <ShiftTableRow
            formatTime={formatTime}
            key={shift.id}
            onDelete={onDelete}
            onEdit={onEdit}
            onViewHistory={onViewHistory}
            shift={shift}
          />
        ))}
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

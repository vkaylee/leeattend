import { ChevronRight } from "lucide-react";
import React from "react";

import { TableCell } from "@/components/ui/table";

interface ShiftTimeCellProps {
  endTime: string | null;
  formatTime: (t: string) => string;
  startTime: string | null;
}

/** Renders the time-range table cell with start/end times. */
export function ShiftTimeCell({ startTime, endTime, formatTime }: ShiftTimeCellProps) {
  return (
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
  );
}

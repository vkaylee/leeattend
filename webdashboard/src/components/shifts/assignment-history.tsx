import { History, Zap, ShieldCheck } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Typography } from "@/components/ui/typography";

import { useDateTimeFormatter } from "../../hooks/use-datetime";
import { AssignmentHistory as AssignmentHistoryType, Employee } from "../../lib/api";

interface AssignmentHistoryProps {
  history: AssignmentHistoryType[] | undefined;
  selectedEmployee: Employee | null;
}

export function AssignmentHistory({ history, selectedEmployee }: AssignmentHistoryProps) {
  const { t } = useTranslation();
  const { formatDateTime } = useDateTimeFormatter();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-[40px] p-8 shadow-sm animate-in fade-in slide-in-from-bottom-8 duration-500">
      <div className="flex items-center gap-3 mb-6">
        <History className="w-5 h-5 text-slate-500 dark:text-slate-400" />
        <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest" variant="h3">{t("shifts.history_title")}</Typography>
      </div>
      
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="p-4">{t("common.time")}</TableHead>
            <TableHead className="p-4">{t("common.employee")}</TableHead>
            <TableHead className="p-4">{t("shifts.table.assignment_type")}</TableHead>
            <TableHead className="p-4">{t("common.shift")}</TableHead>
            <TableHead className="p-4">{t("shifts.table.range")}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {history?.filter(h => h.employee_id === selectedEmployee?.id || !selectedEmployee).map(h => (
            <TableRow key={h.id}>
              <TableCell className="p-4 text-xs font-bold text-slate-500 dark:text-slate-400">
                {formatDateTime(h.created_at)}
              </TableCell>
              <TableCell className="p-4 text-sm font-bold text-slate-700 dark:text-slate-200">{h.employee_name}</TableCell>
              <TableCell className="p-4">
                {h.assignment_type === 'roster' ? (
                  <span className="inline-flex items-center gap-1 px-2 py-1 rounded-lg bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-300 text-[10px] font-bold uppercase tracking-tighter border border-teal-100">
                    <Zap size={10} /> {t("shifts.types.flexible")}
                  </span>
                ) : h.assignment_type === 'pattern' ? (
                  <span className="inline-flex items-center gap-1 px-2 py-1 rounded-lg bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 text-[10px] font-bold uppercase tracking-tighter border border-blue-100 dark:border-blue-800">
                    <ShieldCheck size={10} /> {t("patterns.title")}
                  </span>
                ) : h.assignment_type === 'department_pattern' ? (
                  <span className="inline-flex items-center gap-1 px-2 py-1 rounded-lg bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 text-[10px] font-bold uppercase tracking-tighter border border-slate-200 dark:border-slate-700">
                    <ShieldCheck size={10} /> {t("patterns.title")} (Dept)
                  </span>
                ) : h.assignment_type === 'department_fixed' ? (
                  <span className="inline-flex items-center gap-1 px-2 py-1 rounded-lg bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-tighter border border-slate-100 dark:border-slate-800">
                    <ShieldCheck size={10} /> {t("shifts.types.fixed")} (Dept)
                  </span>
                ) : (
                  <span className="inline-flex items-center gap-1 px-2 py-1 rounded-lg bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 text-[10px] font-bold uppercase tracking-tighter border border-emerald-100 dark:border-emerald-800">
                    <ShieldCheck size={10} /> {t("shifts.types.fixed")}
                  </span>
                )}
              </TableCell>
              <TableCell className="p-4 font-bold text-slate-700 dark:text-slate-200 text-xs">{h.shift_name ?? t("shifts.off_day")}</TableCell>
              <TableCell className="p-4 font-mono text-xs text-slate-500 dark:text-slate-400">{h.date_range}</TableCell>
            </TableRow>
          ))}
          {(!history || history.length === 0) && (
            <TableRow>
              <TableCell className="p-8 text-center text-xs font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" colSpan={5}>{t("common.no_history")}</TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}


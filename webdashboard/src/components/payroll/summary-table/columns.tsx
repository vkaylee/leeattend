import { type ColumnDef } from "@tanstack/react-table";
import { Check, ChevronRight, Calculator, CheckCircle2, AlertCircle } from "lucide-react";
import React, { useMemo } from "react";

import { type TFunction } from "i18next";

import { Typography } from "@/components/ui/typography";
import { Employee, Payslip, IncentiveEvaluationResult } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface UsePayrollColumnsProps {
  canReadSensitive: boolean;
  canWriteSensitive: boolean;
  currentPageIds: string[];
  handleSelectPage: (pageIds: string[], isPageSelected: boolean) => void;
  isPageSelected: boolean;
  onToggleSelect: (id: string) => void;
  payslipMap: Record<string, Payslip | undefined>;
  selectedIds: Set<string>;
  t: TFunction;
}

interface GetSelectColumnProps {
  currentPageIds: string[];
  handleSelectPage: (pageIds: string[], isPageSelected: boolean) => void;
  isPageSelected: boolean;
  onToggleSelect: (id: string) => void;
  selectedIds: Set<string>;
}

const getSelectColumn = ({
  isPageSelected,
  currentPageIds,
  handleSelectPage,
  selectedIds,
  onToggleSelect
}: GetSelectColumnProps): ColumnDef<Employee> => ({
  id: "select",
  accessorKey: "id",
  header: () => (
    <div className="flex items-center justify-center w-full" onClick={(e) => { e.stopPropagation(); }} onKeyDown={(e) => { e.stopPropagation(); }} role="presentation">
      <div
        aria-checked={isPageSelected}
        className={cn(
          "w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all cursor-pointer",
          isPageSelected ? "bg-primary-600 border-primary-600 text-white shadow-lg shadow-primary-100" : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 group-hover:border-primary-300"
        )}
        onClick={() => { handleSelectPage(currentPageIds, isPageSelected); }}
        onKeyDown={(e) => {
          if (e.key === " " || e.key === "Enter") {
            e.preventDefault();
            handleSelectPage(currentPageIds, isPageSelected);
          }
        }}
        role="checkbox"
        tabIndex={0}
      >
        {isPageSelected ? <Check className="w-3.5 h-3.5" strokeWidth={4} /> : null}
      </div>
    </div>
  ),
  cell: ({ row }) => {
    const emp = row.original;
    const isSelected = selectedIds.has(emp.id);
    return (
      <div className="flex items-center justify-center w-full" onClick={(e) => { e.stopPropagation(); }} onKeyDown={(e) => { e.stopPropagation(); }} role="presentation">
        <div
          aria-checked={isSelected}
          className={cn(
            "w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all cursor-pointer",
            isSelected ? "bg-primary-600 border-primary-600 text-white shadow-lg shadow-primary-100" : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 group-hover:border-primary-300"
          )}
          data-testid={`select-employee-${emp.id}`}
          onClick={() => { onToggleSelect(emp.id); }}
          onKeyDown={(e) => {
            if (e.key === " " || e.key === "Enter") {
              e.preventDefault();
              onToggleSelect(emp.id);
            }
          }}
          role="checkbox"
          tabIndex={0}
        >
          {isSelected ? <Check className="w-3.5 h-3.5" strokeWidth={4} /> : null}
        </div>
      </div>
    );
  },
  size: 70,
  meta: { sticky: "left" as const, headerClassName: "text-center", cellClassName: "text-center" }
});

const getEmployeeColumn = (t: TFunction): ColumnDef<Employee> => ({
  id: "employee",
  header: t("employees.table.name"),
  cell: ({ row }) => {
    const emp = row.original;
    return (
      <div className="flex items-center gap-4">
        <div className="w-10 h-10 rounded-2xl bg-slate-100 dark:bg-slate-700 border border-slate-100 dark:border-slate-800 flex items-center justify-center font-bold text-slate-500 dark:text-slate-400 group-hover:bg-primary-50 dark:group-hover:bg-primary-950 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800 group-hover:shadow-md transition-all shadow-inner uppercase">
          {emp.full_name[0]}
        </div>
        <div className="truncate">
          <Typography className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-tight truncate" variant="p">
            {emp.full_name}
          </Typography>
          <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1 opacity-70 tracking-tighter font-mono leading-none" variant="p">
            {emp.employee_id_number}
          </Typography>
        </div>
      </div>
    );
  },
  size: 260,
  meta: { sticky: "left" as const }
});

const getBasePayColumn = (
  t: TFunction,
  payslipMap: Record<string, Payslip | undefined>,
  canReadSensitive: boolean,
  canWriteSensitive: boolean
): ColumnDef<Employee> => ({
  id: "base_pay",
  header: () => <div className="text-right w-full">{t("common.payroll.table.base_pay")}</div>,
  cell: ({ row }) => {
    const emp = row.original;
    const slip = payslipMap[emp.id];
    if (!canReadSensitive) {
      return <div className="text-right font-mono text-sm font-bold text-slate-400">***</div>;
    }
    return (
      <div className="text-right font-mono text-sm font-bold text-slate-600 dark:text-slate-350">
        {slip ? new Intl.NumberFormat("vi-VN").format(parseFloat(slip.base_pay_amount)) : "—"}
      </div>
    );
  },
  size: 150,
  meta: { editable: canReadSensitive && canWriteSensitive, editType: "number", cellClassName: "text-right" }
});

const getOvertimeColumn = (
  t: TFunction,
  payslipMap: Record<string, Payslip | undefined>,
  canReadSensitive: boolean
): ColumnDef<Employee> => ({
  id: "overtime",
  header: () => <div className="text-right w-full">{t("common.payroll.table.overtime")}</div>,
  cell: ({ row }) => {
    const emp = row.original;
    const slip = payslipMap[emp.id];
    if (!canReadSensitive) {
      return <div className="text-right font-mono text-sm font-bold text-slate-400">***</div>;
    }
    return (
      <div className="text-right font-mono text-sm text-emerald-600 dark:text-emerald-400 font-bold">
        {slip && parseFloat(slip.overtime_pay_amount) > 0 
          ? `+${new Intl.NumberFormat("vi-VN").format(parseFloat(slip.overtime_pay_amount))}` 
          : "—"
        }
      </div>
    );
  },
  size: 150,
  meta: { cellClassName: "text-right" }
});

const getBonusColumn = (
  t: TFunction,
  payslipMap: Record<string, Payslip | undefined>,
  canReadSensitive: boolean
): ColumnDef<Employee> => ({
  id: "bonus",
  header: () => <div className="text-right w-full">{t("common.payroll.table.bonus")}</div>,
  cell: ({ row }) => {
    const emp = row.original;
    const slip = payslipMap[emp.id];
    if (!canReadSensitive) {
      return <div className="text-right font-mono text-sm font-bold text-slate-400">***</div>;
    }
    const bonusAmount = slip?.incentives && slip.incentives.length > 0 
      ? slip.incentives.reduce((acc: number, inc: IncentiveEvaluationResult) => acc + parseFloat(inc.amount), 0)
      : 0;

    return (
      <div className="text-right font-mono text-sm text-blue-600 dark:text-blue-400 font-bold">
        {bonusAmount > 0 ? `+${new Intl.NumberFormat("vi-VN").format(bonusAmount)}` : "—"}
      </div>
    );
  },
  size: 150,
  meta: { cellClassName: "text-right" }
});

const getNetPayColumn = (
  t: TFunction,
  payslipMap: Record<string, Payslip | undefined>,
  canReadSensitive: boolean
): ColumnDef<Employee> => ({
  id: "net_pay",
  header: () => <div className="text-right w-full">{t("common.payroll.table.net_pay")}</div>,
  cell: ({ row }) => {
    const emp = row.original;
    const slip = payslipMap[emp.id];
    if (!canReadSensitive) {
      return <div className="text-right font-mono text-base font-bold text-slate-400">***</div>;
    }
    return (
      <div className="text-right font-bold text-primary-600 dark:text-primary-400 text-base font-mono">
        {slip ? new Intl.NumberFormat("vi-VN").format(parseFloat(slip.net_pay_amount)) : "—"}
      </div>
    );
  },
  size: 160,
  meta: { cellClassName: "text-right" }
});

const getStatusColumn = (
  t: TFunction,
  payslipMap: Record<string, Payslip | undefined>,
  canReadSensitive: boolean,
  canWriteSensitive: boolean
): ColumnDef<Employee> => ({
  id: "status",
  header: () => <div className="text-center w-full">{t("common.payroll.table.status")}</div>,
  cell: ({ row }) => {
    const emp = row.original;
    const slip = payslipMap[emp.id];

    if (!slip) {
      return (
        <div className="flex justify-center w-full">
          <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 rounded-full text-[9px] font-bold uppercase tracking-widest border border-slate-200 dark:border-slate-700 opacity-60">
            <Calculator size={12} /> {t("common.pending")}
          </span>
        </div>
      );
    }

    return (
      <div className="flex justify-center w-full">
        <span className={cn(
          "inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm", 
          slip.status === "paid" 
            ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" 
            : "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800"
        )}>
          {slip.status === "paid" ? <CheckCircle2 size={12} /> : <AlertCircle size={12} />}
          {slip.status === "paid" ? t("common.payroll.status.paid") : t("common.payroll.status.finalized")}
        </span>
      </div>
    );
  },
  size: 160,
  meta: {
    editable: canReadSensitive && canWriteSensitive,
    editType: "select",
    editOptions: [
      { value: "draft", label: t("common.pending") },
      { value: "finalized", label: t("common.payroll.status.finalized") },
      { value: "paid", label: t("common.payroll.status.paid") }
    ],
    headerClassName: "text-center", cellClassName: "text-center"
  }
});

const getActionsColumn = (): ColumnDef<Employee> => ({
  id: "actions",
  header: "",
  cell: () => (
    <div className="flex items-center justify-end w-full pr-4 text-slate-350 dark:text-slate-650 group-hover:text-primary-500">
      <ChevronRight className="w-4.5 h-4.5 transition-all group-hover:translate-x-1" />
    </div>
  ),
  size: 60,
  meta: { cellClassName: "text-right" }
});

export function usePayrollColumns({
  t,
  payslipMap,
  canReadSensitive,
  canWriteSensitive,
  selectedIds,
  isPageSelected,
  currentPageIds,
  handleSelectPage,
  onToggleSelect
}: UsePayrollColumnsProps): ColumnDef<Employee>[] {
  return useMemo(() => [
    getSelectColumn({ isPageSelected, currentPageIds, handleSelectPage, selectedIds, onToggleSelect }),
    getEmployeeColumn(t),
    getBasePayColumn(t, payslipMap, canReadSensitive, canWriteSensitive),
    getOvertimeColumn(t, payslipMap, canReadSensitive),
    getBonusColumn(t, payslipMap, canReadSensitive),
    getNetPayColumn(t, payslipMap, canReadSensitive),
    getStatusColumn(t, payslipMap, canReadSensitive, canWriteSensitive),
    getActionsColumn()
  ], [t, payslipMap, canReadSensitive, canWriteSensitive, selectedIds, isPageSelected, currentPageIds, handleSelectPage, onToggleSelect]);
}

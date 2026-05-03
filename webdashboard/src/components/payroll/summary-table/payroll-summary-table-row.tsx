import { CheckCircle2, AlertCircle, Calculator, ChevronRight } from "lucide-react";
import React from "react";

import { Employee, Payslip } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface CellProps {
  canReadSensitive: boolean;
  slip: Payslip | undefined;
}

function BasePayCell({ canReadSensitive, slip }: CellProps) {
  if (!canReadSensitive) return <td className="px-4 py-6 text-right font-mono text-sm font-bold text-slate-500 dark:text-slate-400">***</td>;
  return (
    <td className="px-4 py-6 text-right font-mono text-sm font-bold text-slate-500 dark:text-slate-400">
      {slip ? new Intl.NumberFormat('vi-VN').format(parseFloat(slip.base_pay_amount)) : "—"}
    </td>
  );
}

function OvertimePayCell({ canReadSensitive, slip }: CellProps) {
  if (!canReadSensitive) return <td className="px-4 py-6 text-right font-mono text-sm text-emerald-600 dark:text-emerald-300 font-bold">***</td>;
  return (
    <td className="px-4 py-6 text-right font-mono text-sm text-emerald-600 dark:text-emerald-300 font-bold">
      {slip && parseFloat(slip.overtime_pay_amount) > 0 ? `+${new Intl.NumberFormat('vi-VN').format(parseFloat(slip.overtime_pay_amount))}` : "—"}
    </td>
  );
}

function BonusPayCell({ canReadSensitive, slip }: CellProps) {
  if (!canReadSensitive) return <td className="px-4 py-6 text-right font-mono text-sm text-blue-600 dark:text-blue-300 font-bold">***</td>;
  return (
    <td className="px-4 py-6 text-right font-mono text-sm text-blue-600 dark:text-blue-300 font-bold">
      {slip?.incentives && slip.incentives.length > 0 
        ? `+${new Intl.NumberFormat('vi-VN').format(slip.incentives.reduce((accumulator, inc) => accumulator + parseFloat(inc.amount), 0))}` 
        : "—"}
    </td>
  );
}

function NetPayCell({ canReadSensitive, slip }: CellProps) {
  if (!canReadSensitive) return <td className="px-4 py-6 text-right"><div className="font-bold text-primary-600 dark:text-primary-400 text-base tracking-tighter font-mono">***</div></td>;
  return (
    <td className="px-4 py-6 text-right">
      <div className="font-bold text-primary-600 dark:text-primary-400 text-base tracking-tighter font-mono">
        {slip ? new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount)) : "—"}
      </div>
    </td>
  );
}

function StatusCell({ slip, t }: { slip: Payslip | undefined; t: (key: string) => string }) {
  if (!slip) {
    return (
      <td className="px-8 py-6 text-center">
        <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 rounded-full text-[9px] font-bold uppercase tracking-widest border border-slate-200 dark:border-slate-700 opacity-60">
          <Calculator size={12} /> {t("common.pending")}
        </span>
      </td>
    );
  }

  return (
    <td className="px-8 py-6 text-center">
      <span className={cn("inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm", slip.status === 'paid' ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800")}>
        {slip.status === 'paid' ? <CheckCircle2 size={12} /> : <AlertCircle size={12} />}
        {slip.status === 'paid' ? t("common.payroll.status.paid") : t("common.payroll.status.finalized")}
      </span>
    </td>
  );
}

function Check({ size, strokeWidth }: { size: number, strokeWidth: number }) {
  return (
    <svg fill="none" height={size} stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth={strokeWidth} viewBox="0 0 24 24" width={size}>
      <polyline points="20 6 9 17 4 12" />
    </svg>
  );
}

interface PayrollSummaryTableRowProps {
  canReadSensitive: boolean;
  emp: Employee;
  isSelected: boolean;
  onToggleSelect: (id: string) => void;
  onViewDetail: (emp: Employee) => void;
  slip: Payslip | undefined;
  t: (key: string) => string;
}

export function PayrollSummaryTableRow({
  emp,
  slip,
  isSelected,
  canReadSensitive,
  onToggleSelect,
  onViewDetail,
  t
}: PayrollSummaryTableRowProps) {
  return (
    <tr className={cn("group transition-all hover:bg-slate-50 dark:hover:bg-slate-800/50 cursor-pointer", isSelected ? "bg-primary-50 dark:bg-primary-950/30" : "")} onClick={() => { onViewDetail(emp); }}>
      <td className="px-10 py-6" onClick={(e) => { e.stopPropagation(); }}>
        <button 
          className={cn("w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all", isSelected ? "bg-primary-600 border-primary-600 text-white shadow-lg shadow-primary-100" : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 group-hover:border-primary-300")}
          data-testid={`select-employee-${emp.id}`} 
          onClick={() => { onToggleSelect(emp.id); }}
        >
          {isSelected ? <Check size={14} strokeWidth={4} /> : null}
        </button>
      </td>
      <td className="px-4 py-6">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-2xl bg-slate-100 dark:bg-slate-700 border border-slate-100 dark:border-slate-800 flex items-center justify-center font-bold text-slate-500 dark:text-slate-400 group-hover:bg-primary-50 dark:group-hover:bg-primary-950 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800 group-hover:shadow-md transition-all shadow-inner uppercase">{emp.full_name[0]}</div>
          <div>
            <p className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-tight">{emp.full_name}</p>
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1 opacity-70 tracking-tighter font-mono leading-none">{emp.employee_id_number}</p>
          </div>
        </div>
      </td>
      <BasePayCell canReadSensitive={canReadSensitive} slip={slip} />
      <OvertimePayCell canReadSensitive={canReadSensitive} slip={slip} />
      <BonusPayCell canReadSensitive={canReadSensitive} slip={slip} />
      <NetPayCell canReadSensitive={canReadSensitive} slip={slip} />
      <StatusCell slip={slip} t={t} />
      <td className="px-4 py-6 text-right pr-8"><ChevronRight className="ml-auto text-slate-200 dark:text-slate-700 transition-all group-hover:text-primary-500 dark:group-hover:text-primary-400 group-hover:translate-x-1" size={18} /></td>
    </tr>
  );
}

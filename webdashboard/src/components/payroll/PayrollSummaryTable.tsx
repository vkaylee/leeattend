import React from "react";
import { useTranslation } from "react-i18next";
import { 
  CheckCircle2, 
  AlertCircle, 
  ChevronRight,
  User,
  Calculator,
  Search,
  ArrowRight
} from "lucide-react";
import { Employee, Payslip } from "@/lib/api/types";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

interface PayrollSummaryTableProps {
  employees: Employee[];
  payslips: Payslip[];
  isLoading: boolean;
  selectedIds: Set<string>;
  onToggleSelect: (id: string) => void;
  onToggleSelectAll: () => void;
  onViewDetail: (employee: Employee) => void;
  searchTerm: string;
  onSearchChange: (val: string) => void;
}

export default function PayrollSummaryTable({
  employees,
  payslips,
  isLoading,
  selectedIds,
  onToggleSelect,
  onToggleSelectAll,
  onViewDetail,
  searchTerm,
  onSearchChange
}: PayrollSummaryTableProps) {
  const { t } = useTranslation();

  const payslipMap = React.useMemo(() => {
    const map: Record<string, Payslip> = {};
    payslips.forEach(p => {
      map[p.employee_id] = p;
    });
    return map;
  }, [payslips]);

  const filteredEmployees = employees.filter(emp => 
    `${emp.first_name} ${emp.last_name}`.toLowerCase().includes(searchTerm.toLowerCase()) ||
    emp.employee_id_number.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const isAllSelected = filteredEmployees.length > 0 && selectedIds.size === filteredEmployees.length;

  if (isLoading) {
    return (
      <div className="bg-white border border-slate-200 rounded-[40px] p-20 text-center text-slate-400 italic shadow-sm">
        {t("common.loading")}
      </div>
    );
  }

  return (
    <div className="bg-white border border-slate-200 rounded-[40px] overflow-hidden shadow-sm flex flex-col h-full">
      <div className="p-6 border-b border-slate-100 bg-slate-50/50 flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="relative group flex-1 max-w-md">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 w-4 h-4 group-focus-within:text-primary-500" />
          <input 
            type="text" 
            placeholder={t("payroll.search_placeholder")}
            className="w-full pl-10 pr-4 py-2.5 bg-white border border-slate-200 rounded-2xl outline-none focus:ring-4 focus:ring-primary-500/10 transition-all text-sm"
            value={searchTerm}
            onChange={(e) => onSearchChange(e.target.value)}
          />
        </div>

        <div className="flex items-center gap-2">
           <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest bg-white px-3 py-1.5 rounded-xl border border-slate-100">
             {filteredEmployees.length} {t("common.people")}
           </span>
        </div>
      </div>

      <div className="flex-1 overflow-auto">
        <table className="w-full text-left border-collapse">
          <thead className="bg-slate-50/30 sticky top-0 backdrop-blur-md border-b border-slate-100 z-10">
            <tr>
              <th className="px-8 py-5 w-12">
                <button 
                  onClick={onToggleSelectAll}
                  className={cn(
                    "w-5 h-5 rounded-lg border flex items-center justify-center transition-all",
                    isAllSelected ? "bg-primary-600 border-primary-600 text-white shadow-md shadow-primary-100" : "border-slate-200 bg-white hover:border-primary-300"
                  )}
                >
                  {isAllSelected && <Check size={12} strokeWidth={4} />}
                </button>
              </th>
              <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("employees.table.name")}</th>
              <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.base_pay")}</th>
              <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.overtime")}</th>
              <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.net_pay")}</th>
              <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("common.payroll.table.status")}</th>
              <th className="px-4 py-5 w-12"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50">
            {filteredEmployees.map(emp => {
              const slip = payslipMap[emp.id];
              const isSelected = selectedIds.has(emp.id);

              return (
                <tr 
                  key={emp.id}
                  className={cn(
                    "group transition-colors hover:bg-slate-50/50 cursor-pointer",
                    isSelected ? "bg-primary-50/30" : ""
                  )}
                  onClick={() => onViewDetail(emp)}
                >
                  <td className="px-8 py-5" onClick={(e) => e.stopPropagation()}>
                    <button 
                      onClick={() => onToggleSelect(emp.id)}
                      className={cn(
                        "w-5 h-5 rounded-lg border flex items-center justify-center transition-all",
                        isSelected ? "bg-primary-600 border-primary-600 text-white shadow-md shadow-primary-100" : "border-slate-200 bg-white group-hover:border-primary-300"
                      )}
                    >
                      {isSelected && <Check size={12} strokeWidth={4} />}
                    </button>
                  </td>
                  <td className="px-4 py-5">
                    <div className="flex items-center gap-3">
                      <div className="w-10 h-10 rounded-2xl bg-slate-100 flex items-center justify-center font-bold text-slate-400 group-hover:bg-primary-600 group-hover:text-white transition-colors">
                        {emp.last_name[0]}
                      </div>
                      <div>
                        <p className="font-bold text-slate-900 text-sm">{emp.first_name} {emp.last_name}</p>
                        <p className="text-[10px] text-slate-400 font-black tracking-widest uppercase">{emp.employee_id_number}</p>
                      </div>
                    </div>
                  </td>
                  <td className="px-4 py-5 text-right font-mono text-sm text-slate-600">
                    {slip ? new Intl.NumberFormat('vi-VN').format(parseFloat(slip.base_pay_amount)) : "—"}
                  </td>
                  <td className="px-4 py-5 text-right font-mono text-sm text-emerald-600 font-bold">
                    {slip && parseFloat(slip.overtime_pay_amount) > 0 ? `+${new Intl.NumberFormat('vi-VN').format(parseFloat(slip.overtime_pay_amount))}` : "—"}
                  </td>
                  <td className="px-4 py-5 text-right">
                    {slip ? (
                      <div className="font-black text-primary-600 text-base tracking-tight">
                        {new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount))}
                      </div>
                    ) : "—"}
                  </td>
                  <td className="px-8 py-5 text-center">
                    {slip ? (
                      <span className={cn(
                        "inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-tighter",
                        slip.status === 'paid' ? "bg-emerald-100 text-emerald-700" : "bg-blue-100 text-blue-700"
                      )}>
                        {slip.status === 'paid' ? <CheckCircle2 size={12} /> : <AlertCircle size={12} />}
                        {slip.status === 'paid' ? t("common.payroll.status.paid") : t("common.payroll.status.finalized")}
                      </span>
                    ) : (
                      <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-slate-100 text-slate-400 rounded-full text-[10px] font-black uppercase tracking-tighter">
                        <Calculator size={12} /> {t("common.pending")}
                      </span>
                    )}
                  </td>
                  <td className="px-4 py-5 text-center text-slate-300 group-hover:text-primary-400 transition-colors">
                    <ChevronRight size={16} />
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
        
        {filteredEmployees.length === 0 && (
          <div className="p-20 text-center">
            <div className="bg-slate-50 w-16 h-16 rounded-3xl flex items-center justify-center mx-auto mb-6 shadow-sm border border-slate-100">
              <User className="text-slate-300 w-8 h-8" />
            </div>
            <p className="text-sm font-bold text-slate-400 uppercase tracking-widest">
              {t("employees.search.no_results")}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

function Check({ size, strokeWidth }: { size: number, strokeWidth: number }) {
  return (
    <svg 
      width={size} 
      height={size} 
      viewBox="0 0 24 24" 
      fill="none" 
      stroke="currentColor" 
      strokeWidth={strokeWidth} 
      strokeLinecap="round" 
      strokeLinejoin="round"
    >
      <polyline points="20 6 9 17 4 12" />
    </svg>
  );
}

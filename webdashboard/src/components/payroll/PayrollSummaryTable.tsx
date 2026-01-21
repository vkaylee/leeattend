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
import { cn } from "@/lib/utils";

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
    payslips.forEach(p => { map[p.employee_id] = p; });
    return map;
  }, [payslips]);

  const filteredEmployees = employees.filter(emp => 
    `${emp.first_name} ${emp.last_name}`.toLowerCase().includes(searchTerm.toLowerCase()) ||
    emp.employee_id_number.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const isAllSelected = filteredEmployees.length > 0 && selectedIds.size === filteredEmployees.length;

  if (isLoading) return <div className="bg-white border border-slate-100 rounded-[2.5rem] p-24 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 shadow-sm">{t("common.loading")}</div>;

  return (
    <div className="bg-white border border-slate-100 rounded-[2.5rem] overflow-hidden shadow-sm flex flex-col h-full animate-in fade-in duration-700">
      <div className="p-6 border-b border-slate-100 bg-slate-50/30 flex flex-col md:flex-row md:items-center justify-between gap-6 px-8">
        <div className="relative group flex-1 max-w-lg">
          <Search className="absolute left-5 top-1/2 -translate-y-1/2 text-slate-400 w-4.5 h-4.5 group-focus-within:text-primary-500 transition-colors" />
          <input type="text" placeholder={t("payroll.search_placeholder")} className="w-full pl-14 pr-6 py-3.5 bg-white border border-slate-200 rounded-[1.5rem] outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm shadow-sm" value={searchTerm} onChange={(e) => onSearchChange(e.target.value)} />
        </div>
        <div className="flex items-center gap-3"><span className="text-[9px] font-black text-slate-400 uppercase tracking-[0.2em] bg-white px-4 py-2 rounded-xl border border-slate-100 shadow-sm">{filteredEmployees.length} {t("common.employees")}</span></div>
      </div>

      <div className="flex-1 overflow-auto scroll-smooth">
        <table className="w-full text-left border-collapse">
          <thead className="bg-slate-50/50 sticky top-0 backdrop-blur-md z-10 border-b border-slate-100">
            <tr>
              <th className="px-10 py-5 w-16">
                <button onClick={onToggleSelectAll} className={cn("w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all", isAllSelected ? "bg-primary-600 border-primary-600 text-white shadow-lg shadow-primary-100" : "border-slate-200 bg-white hover:border-primary-300")}>
                  {isAllSelected && <Check size={14} strokeWidth={4} />}
                </button>
              </th>
              <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("employees.table.name")}</th>
              <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.base_pay")}</th>
              <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.overtime")}</th>
              <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.net_pay")}</th>
              <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("common.payroll.table.status")}</th>
              <th className="px-4 py-5 w-12 text-right pr-8"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50">
            {filteredEmployees.map(emp => {
              const slip = payslipMap[emp.id];
              const isSelected = selectedIds.has(emp.id);
              return (
                <tr key={emp.id} className={cn("group transition-all hover:bg-slate-50/50 cursor-pointer", isSelected ? "bg-primary-50/30" : "")} onClick={() => onViewDetail(emp)}>
                  <td className="px-10 py-6" onClick={(e) => e.stopPropagation()}>
                    <button 
                      data-testid={`select-employee-${emp.id}`}
                      onClick={() => onToggleSelect(emp.id)} 
                      className={cn("w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all", isSelected ? "bg-primary-600 border-primary-600 text-white shadow-lg shadow-primary-100" : "border-slate-200 bg-white group-hover:border-primary-300")}
                    >
                      {isSelected && <Check size={14} strokeWidth={4} />}
                    </button>
                  </td>
                  <td className="px-4 py-6">
                    <div className="flex items-center gap-4">
                      <div className="w-12 h-12 rounded-2xl bg-slate-100 border border-slate-100 flex items-center justify-center font-black text-slate-400 group-hover:bg-primary-50 group-hover:text-primary-600 group-hover:border-primary-100 group-hover:shadow-md transition-all shadow-inner uppercase">{emp.last_name[0]}</div>
                      <div>
                        <p className="font-black text-slate-900 text-sm tracking-tight uppercase leading-tight">{emp.first_name} {emp.last_name}</p>
                        <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest mt-1 opacity-70 tracking-tighter font-mono leading-none">{emp.employee_id_number}</p>
                      </div>
                    </div>
                  </td>
                  <td className="px-4 py-6 text-right font-mono text-sm font-bold text-slate-500">{slip ? new Intl.NumberFormat('vi-VN').format(parseFloat(slip.base_pay_amount)) : "—"}</td>
                  <td className="px-4 py-6 text-right font-mono text-sm text-emerald-600 font-black">{slip && parseFloat(slip.overtime_pay_amount) > 0 ? `+${new Intl.NumberFormat('vi-VN').format(parseFloat(slip.overtime_pay_amount))}` : "—"}</td>
                  <td className="px-4 py-6 text-right"><div className="font-black text-primary-600 text-base tracking-tighter font-mono">{slip ? new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount)) : "—"}</div></td>
                  <td className="px-8 py-6 text-center">
                    {slip ? (
                      <span className={cn("inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-black uppercase tracking-widest border shadow-sm", slip.status === 'paid' ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-blue-50 text-blue-600 border-blue-100")}>
                        {slip.status === 'paid' ? <CheckCircle2 size={12} /> : <AlertCircle size={12} />}
                        {slip.status === 'paid' ? t("common.payroll.status.paid") : t("common.payroll.status.finalized")}
                      </span>
                    ) : (
                      <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-slate-100 text-slate-400 rounded-full text-[9px] font-black uppercase tracking-widest border border-slate-200 opacity-60"><Calculator size={12} /> {t("common.pending")}</span>
                    )}
                  </td>
                  <td className="px-4 py-6 text-right pr-8"><ChevronRight size={18} className="ml-auto text-slate-200 transition-all group-hover:text-primary-500 group-hover:translate-x-1" /></td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {filteredEmployees.length === 0 && (
          <div className="py-32 text-center border-2 border-dashed border-slate-100 m-8 rounded-[3rem] bg-slate-50/30 flex flex-col items-center">
            <div className="w-20 h-20 bg-white rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100"><User size={32} className="text-slate-200" /></div>
            <h3 className="text-lg font-bold text-slate-900">{t("employees.search.no_results")}</h3>
            <p className="text-slate-400 text-sm mt-1 uppercase font-black tracking-widest opacity-50">{t("common.no_records_found")}</p>
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

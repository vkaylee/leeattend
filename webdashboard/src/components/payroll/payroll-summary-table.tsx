import { 
  CheckCircle2, 
  AlertCircle, 
  ChevronRight,
  User,
  Calculator,
  Search,
  Info
} from "lucide-react";
import React, { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";

import PaginationControls from "@/components/common/pagination-controls";
import { useMyPermissions } from "@/hooks/use-permissions";
import { Employee, Payslip } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface PayrollSummaryTableProps {
  employees: Employee[];
  isLoading: boolean;
  onSearchChange: (value: string) => void;
  onToggleSelect: (id: string) => void;
  onToggleSelectAll: (ids?: string[]) => void;
  onViewDetail: (employee: Employee) => void;
  payslips: Payslip[];
  searchTerm: string;
  selectedIds: Set<string>;
}

function PayrollSummaryTableRow({
  emp,
  slip,
  isSelected,
  canReadSensitive,
  onToggleSelect,
  onViewDetail,
  t
}: {
  canReadSensitive: boolean;
  emp: Employee;
  isSelected: boolean;
  onToggleSelect: (id: string) => void;
  onViewDetail: (emp: Employee) => void;
  slip: Payslip | undefined;
  t: (key: string) => string;
}) {
  return (
    <tr className={cn("group transition-all hover:bg-slate-50 dark:hover:bg-slate-800/50 cursor-pointer", isSelected ? "bg-primary-50 dark:bg-primary-950/30" : "")} onClick={() => { onViewDetail(emp); }}>
      <td className="px-10 py-6" onClick={(e) => { e.stopPropagation(); }}>
        <button 
          className={cn("w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all", isSelected ? "bg-primary-600 border-primary-600 text-white shadow-lg shadow-primary-100" : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 group-hover:border-primary-300")}
          data-testid={`select-employee-${emp.id}`} 
          onClick={() => { onToggleSelect(emp.id); }}
        >
          {!!isSelected && <Check size={14} strokeWidth={4} />}
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
      <td className="px-4 py-6 text-right font-mono text-sm font-bold text-slate-500 dark:text-slate-400">
        {!canReadSensitive ? "***" : (slip ? new Intl.NumberFormat('vi-VN').format(parseFloat(slip.base_pay_amount)) : "—")}
      </td>
      <td className="px-4 py-6 text-right font-mono text-sm text-emerald-600 dark:text-emerald-300 font-bold">
        {!canReadSensitive ? "***" : (slip && parseFloat(slip.overtime_pay_amount) > 0 ? `+${new Intl.NumberFormat('vi-VN').format(parseFloat(slip.overtime_pay_amount))}` : "—")}
      </td>
      <td className="px-4 py-6 text-right font-mono text-sm text-blue-600 dark:text-blue-300 font-bold">
        {!canReadSensitive ? "***" : (slip?.incentives && slip.incentives.length > 0 
          ? `+${new Intl.NumberFormat('vi-VN').format(slip.incentives.reduce((accumulator, index) => accumulator + parseFloat(index.amount), 0))}` 
          : "—")}
      </td>
      <td className="px-4 py-6 text-right">
        <div className="font-bold text-primary-600 dark:text-primary-400 text-base tracking-tighter font-mono">
          {!canReadSensitive ? "***" : (slip ? new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount)) : "—")}
        </div>
      </td>
      <td className="px-8 py-6 text-center">
        {slip ? (
          <span className={cn("inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm", slip.status === 'paid' ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800")}>
            {slip.status === 'paid' ? <CheckCircle2 size={12} /> : <AlertCircle size={12} />}
            {slip.status === 'paid' ? t("common.payroll.status.paid") : t("common.payroll.status.finalized")}
          </span>
        ) : (
          <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 rounded-full text-[9px] font-bold uppercase tracking-widest border border-slate-200 dark:border-slate-700 opacity-60"><Calculator size={12} /> {t("common.pending")}</span>
        )}
      </td>
      <td className="px-4 py-6 text-right pr-8"><ChevronRight className="ml-auto text-slate-200 dark:text-slate-700 transition-all group-hover:text-primary-500 dark:group-hover:text-primary-400 group-hover:translate-x-1" size={18} /></td>
    </tr>
  );
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
  const { hasPermission } = useMyPermissions();
  const canReadSensitive = hasPermission("payroll:sensitive:read");
  
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  const payslipMap = useMemo(() => {
    const map: Record<string, Payslip> = {};
    payslips.forEach(p => { map[p.employee_id] = p; });
    return map;
  }, [payslips]);

  const filteredEmployees = useMemo(() => {
    return employees.filter(emp => 
      emp.full_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      emp.employee_id_number.toLowerCase().includes(searchTerm.toLowerCase())
    );
  }, [employees, searchTerm]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- Derived state synced from props; intentional pattern
    setCurrentPage(1);
  }, [searchTerm]);

  const totalPages = Math.ceil(filteredEmployees.length / pageSize);
  const paginatedEmployees = filteredEmployees.slice(
    (currentPage - 1) * pageSize,
    currentPage * pageSize
  );

  const currentPageIds = paginatedEmployees.map(e => e.id);
  const isPageSelected = currentPageIds.length > 0 && currentPageIds.every(id => selectedIds.has(id));
  const isAllFilteredSelected = filteredEmployees.length > 0 && filteredEmployees.every(e => selectedIds.has(e.id));

  const handleSelectPage = () => {
    if (isPageSelected) {
      currentPageIds.forEach(id => {
        if (selectedIds.has(id)) onToggleSelect(id);
      });
    } else {
      currentPageIds.forEach(id => {
        if (!selectedIds.has(id)) onToggleSelect(id);
      });
    }
  };

  const selectAllFiltered = () => {
    onToggleSelectAll();
  };

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm flex flex-col h-full animate-in fade-in duration-700">
      <div className="p-6 border-b border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 flex flex-col xl:flex-row xl:items-center justify-between gap-6 px-8">
        <div className="flex flex-col md:flex-row md:items-center gap-6 flex-1">
          <div className="relative group flex-1 max-w-lg">
            <Search className="absolute left-5 top-1/2 -translate-y-1/2 text-slate-500 dark:text-slate-400 w-4.5 h-4.5 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" />
            <input aria-label={t("common.search")} className="w-full pl-14 pr-6 py-3.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-[1.5rem] outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 transition-all font-bold text-sm shadow-sm" onChange={(e) => { onSearchChange(e.target.value); }} placeholder={t("payroll.search_placeholder")} type="text" value={searchTerm} />
          </div>
          <div className="flex items-center gap-3"><span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] bg-white dark:bg-slate-900 px-4 py-2 rounded-xl border border-slate-100 dark:border-slate-800 shadow-sm">{filteredEmployees.length} {t("common.employees")}</span></div>
        </div>
      </div>
      {isLoading ? (
        <div className="p-24 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.loading")}</div>
      ) : (
        <>
      {selectedIds.size > 0 && (
        <div className="bg-primary-50 dark:bg-primary-950 px-8 py-3 border-b border-primary-100 dark:border-primary-800 flex items-center justify-between animate-in slide-in-from-top-2">
          <div className="flex items-center gap-3">
             <Info className="text-primary-600 dark:text-primary-400" size={16} />
             <p className="text-xs font-bold text-primary-900">
               {selectedIds.size === filteredEmployees.length 
                 ? t("payroll.selection.all_filtered_selected", { count: selectedIds.size, defaultValue: `All ${selectedIds.size} employees in this list are selected.` })
                 : t("payroll.selection.count_selected", { count: selectedIds.size, defaultValue: `${selectedIds.size} employees selected.` })
               }
             </p>
          </div>
          {!isAllFilteredSelected && filteredEmployees.length > pageSize && (
            <button 
              className="text-xs font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest hover:underline decoration-2 underline-offset-4"
              onClick={selectAllFiltered}
            >
              {t("payroll.selection.select_all_filtered", { count: filteredEmployees.length, defaultValue: `Select all ${filteredEmployees.length} employees` })}
            </button>
          )}
        </div>
      )}

      <div className="flex-1 overflow-auto scroll-smooth">
        <table className="w-full text-left border-collapse">
          <thead className="bg-slate-50 dark:bg-slate-800/50 sticky top-0 backdrop-blur-md z-10 border-b border-slate-100 dark:border-slate-800">
            <tr>
              <th className="px-10 py-5 w-16">
                <button className={cn("w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all", isPageSelected ? "bg-primary-600 border-primary-600 text-white shadow-lg shadow-primary-100" : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 hover:border-primary-300")} onClick={handleSelectPage}>
                  {!!isPageSelected && <Check size={14} strokeWidth={4} />}
                </button>
              </th>
              <th className="px-4 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("employees.table.name")}</th>
              <th className="px-4 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.base_pay")}</th>
              <th className="px-4 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.overtime")}</th>
              <th className="px-4 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.bonus")}</th>
              <th className="px-4 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.net_pay")}</th>
              <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">{t("common.payroll.table.status")}</th>
              <th className="px-4 py-5 w-12 text-right pr-8"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
            {paginatedEmployees.map(emp => (
              <PayrollSummaryTableRow 
                canReadSensitive={canReadSensitive}
                emp={emp}
                isSelected={selectedIds.has(emp.id)}
                key={emp.id}
                onToggleSelect={onToggleSelect}
                onViewDetail={onViewDetail}
                slip={payslipMap[emp.id]}
                t={t as any}
              />
            ))}
          </tbody>
        </table>
        {filteredEmployees.length === 0 && (
          <div className="py-32 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 m-8 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
            <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-4 border border-slate-100 dark:border-slate-800"><User className="text-slate-200 dark:text-slate-700" size={32} /></div>
            <h3 className="text-lg font-bold text-slate-700 dark:text-slate-200">{t("employees.search.no_results")}</h3>
            <p className="text-slate-500 dark:text-slate-400 text-sm mt-1 uppercase font-bold tracking-widest opacity-50">{t("common.no_records_found")}</p>
          </div>
        )}
      </div>

      <div className="p-4 border-t border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 flex justify-center">
        <PaginationControls 
          currentPage={currentPage}
          onPageChange={setCurrentPage}
          onPageSizeChange={setPageSize}
          pageSize={pageSize}
          totalItems={filteredEmployees.length}
          totalPages={totalPages}
        />
      </div>
        </>
      )}
    </div>
  );
}

function Check({ size, strokeWidth }: { size: number, strokeWidth: number }) {
  return (
    <svg 
      fill="none" 
      height={size} 
      stroke="currentColor" 
      strokeLinecap="round" 
      strokeLinejoin="round" 
      strokeWidth={strokeWidth} 
      viewBox="0 0 24 24" 
      width={size}
    >
      <polyline points="20 6 9 17 4 12" />
    </svg>
  );
}

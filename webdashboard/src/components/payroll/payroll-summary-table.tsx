import { 
  User
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import PaginationControls from "@/components/common/pagination-controls";
import { useMyPermissions } from "@/hooks/use-permissions";
import { Employee, Payslip } from "@/lib/api/types";

import { PayrollSummaryTableRow } from "./summary-table/payroll-summary-table-row";
import { SelectionBanner } from "./summary-table/selection-banner";
import { SummarySearchHeader } from "./summary-table/summary-search-header";
import { TableHead } from "./summary-table/table-head";
import { usePayrollSummaryLogic } from "./summary-table/use-payroll-summary-logic";

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

  const {
    payslipMap,
    filteredEmployees,
    handleSelectPage
  } = usePayrollSummaryLogic({ employees, payslips, searchTerm, selectedIds, onToggleSelect });

  const [prevSearchTerm, setPrevSearchTerm] = useState(searchTerm);
  if (searchTerm !== prevSearchTerm) {
    setPrevSearchTerm(searchTerm);
    setCurrentPage(1);
  }

  const totalPages = Math.ceil(filteredEmployees.length / pageSize);
  const paginatedEmployees = filteredEmployees.slice(
    (currentPage - 1) * pageSize,
    currentPage * pageSize
  );

  const currentPageIds = paginatedEmployees.map(e => e.id);
  const isPageSelected = currentPageIds.length > 0 && currentPageIds.every(id => selectedIds.has(id));
  const isAllFilteredSelected = filteredEmployees.length > 0 && filteredEmployees.every(e => selectedIds.has(e.id));

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm flex flex-col h-full animate-in fade-in duration-700">
      <SummarySearchHeader 
        filteredCount={filteredEmployees.length}
        onSearchChange={onSearchChange}
        searchTerm={searchTerm}
        t={t}
      />

      {isLoading ? (
        <div className="p-24 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.loading")}</div>
      ) : (
        <>
          <SelectionBanner 
            filteredCount={filteredEmployees.length}
            isAllFilteredSelected={isAllFilteredSelected}
            pageSize={pageSize}
            selectAllFiltered={onToggleSelectAll}
            selectedCount={selectedIds.size}
            t={t}
          />

          <div className="flex-1 overflow-auto scroll-smooth">
            <table className="w-full text-left border-collapse">
              <TableHead 
                isPageSelected={isPageSelected}
                onSelectPage={() => { handleSelectPage(currentPageIds, isPageSelected); }}
                t={t}
              />
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
                    t={t}
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

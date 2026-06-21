import { 
  User,
  Check
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import PaginationControls from "@/components/common/pagination-controls";
import SmartTable from "@/components/ui/smart-table";
import { useToast } from "@/components/ui/toast";
import { useMyPermissions } from "@/hooks/use-permissions";
import { Employee, Payslip } from "@/lib/api/types";

import { usePayrollColumns } from "./summary-table/columns";
import { SelectionBanner } from "./summary-table/selection-banner";
import { SummarySearchHeader } from "./summary-table/summary-search-header";
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
  const toast = useToast();
  const { hasPermission } = useMyPermissions();
  const canReadSensitive = hasPermission("payroll:sensitive:read");
  const canWriteSensitive = hasPermission("payroll:write");
  
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

  // Handle local cell updates (Base Pay & Status)
  const handleCellSave = (employeeId: string, columnId: string, value: unknown) => {
    // Typically this would fire an API mutation, here we display a success toast feedback
    toast.success(
      t("common.saved", "Saved change successfully"),
      `Updated ${columnId} for employee: ${String(value)}`
    );
  };

  const handleBulkCellSave = (changes: { rowId: string; columnId: string; value: unknown }[]) => {
    toast.success(
      t("common.saved", "Saved change successfully"),
      `Updated ${changes.length} cells successfully`
    );
  };

  const validateCell = (rowId: string, columnId: string, value: unknown) => {
    if (columnId === "base_pay") {
      const num = Number(value);
      if (isNaN(num) || num <= 0) {
        return t("common.payroll.validation.base_pay_positive", "Base pay must be a positive number");
      }
    }
    if (columnId === "status") {
      const allowed = ["draft", "finalized", "paid"];
      if (typeof value !== "string" || !allowed.includes(value)) {
        return t("common.payroll.validation.status_invalid", "Status must be draft, finalized, or paid");
      }
    }
    return null;
  };

  const columns = usePayrollColumns({
    t,
    payslipMap,
    canReadSensitive,
    canWriteSensitive,
    selectedIds,
    isPageSelected,
    currentPageIds,
    handleSelectPage,
    onToggleSelect
  });

  // Context Menu setup
  const contextMenuItems = (emp: Employee) => [
    {
      label: t("common.view", "View Details"),
      onClick: () => { onViewDetail(emp); },
      icon: <User className="w-3.5 h-3.5" />,
    },
    {
      label: selectedIds.has(emp.id) ? t("common.deselect", "Deselect") : t("common.select", "Select"),
      onClick: () => { onToggleSelect(emp.id); },
      icon: <Check className="w-3.5 h-3.5" />,
    }
  ];

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm flex flex-col h-full animate-in fade-in duration-700">
      <SummarySearchHeader 
        filteredCount={filteredEmployees.length}
        onSearchChange={onSearchChange}
        searchTerm={searchTerm}
        t={t}
      />

      {isLoading ? (
        <div className="p-24 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
          {t("common.loading")}
        </div>
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

          <div className="flex-1 min-h-0 px-6 py-2 overflow-hidden">
            <SmartTable<Employee>
              columns={columns}
              contextMenuItems={contextMenuItems}
              data={paginatedEmployees}
              emptyMessage={t("common.no_records_found")}
              enableDragSelect
              getRowId={(emp) => emp.id}
              onBulkCellSave={handleBulkCellSave}
              onCellSave={handleCellSave}
              onRowClick={(emp) => { onViewDetail(emp); }}
              onSelectionChange={(ids) => { onToggleSelectAll(Array.from(ids)); }}
              selectedRowIds={selectedIds}
              validateCell={validateCell}
            />
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

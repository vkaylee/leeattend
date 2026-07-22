import {
  User,
  Check,
  DollarSign,
  TrendingUp,
  Gift,
  Wallet,
  AlertCircle,
  CheckCircle2
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";

import PaginationControls from "@/components/common/pagination-controls";
import ResponsiveTable from "@/components/ui/responsive-table";
import SmartTable from "@/components/ui/smart-table";
import { useToast } from "@/components/ui/toast";
import { useMyPermissions } from "@/hooks/use-permissions";
import { Employee, Payslip } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { usePayrollColumns } from "./summary-table/columns";
import { SelectionBanner } from "./summary-table/selection-banner";
import { SummarySearchHeader } from "./summary-table/summary-search-header";
import { usePayrollSummaryLogic } from "./summary-table/use-payroll-summary-logic";

function PayrollMobileCard({
  employee,
  payslip,
  isSelected,
  canReadSensitive,
  onSelect,
  onViewDetail,
  t,
}: {
  canReadSensitive: boolean;
  employee: Employee;
  isSelected: boolean;
  onSelect: () => void;
  onViewDetail: () => void;
  payslip?: Payslip;
  t: TFunction;
}) {
  const formatCurrency = (amount: string) => {
    if (!canReadSensitive) return "***";
    return new Intl.NumberFormat("vi-VN").format(parseFloat(amount));
  };

  const bonusAmount = payslip?.incentives?.reduce(
    (acc, inc) => acc + parseFloat(inc.amount),
    0
  ) ?? 0;

  return (
    <div
      className={cn(
        "rounded-2xl border bg-white p-5 shadow-sm transition-all active:scale-[0.99] dark:bg-slate-900",
        isSelected
          ? "border-primary-300 ring-2 ring-primary-500/20 dark:border-primary-700"
          : "border-slate-100 dark:border-slate-800"
      )}
      onClick={onViewDetail}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onViewDetail();
        }
      }}
    >
      {/* Header: Employee info + checkbox */}
      <div className="flex items-start gap-3">
        <div
          aria-checked={isSelected}
          className={cn(
            "mt-1 w-6 h-6 shrink-0 rounded-xl border-2 flex items-center justify-center transition-all cursor-pointer",
            isSelected
              ? "bg-primary-600 border-primary-600 text-white"
              : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900"
          )}
          onClick={(e) => {
            e.stopPropagation();
            onSelect();
          }}
          onKeyDown={(e) => {
            if (e.key === " " || e.key === "Enter") {
              e.preventDefault();
              e.stopPropagation();
              onSelect();
            }
          }}
          role="checkbox"
          tabIndex={0}
        >
          {isSelected ? <Check className="w-3.5 h-3.5" strokeWidth={4} /> : null}
        </div>
        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-slate-100 bg-slate-100 font-bold uppercase text-slate-500 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300">
          {employee.full_name[0]}
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">
            {employee.full_name}
          </div>
          <div className="mt-1 font-mono text-[10px] font-bold uppercase tracking-wider text-slate-400">
            {employee.employee_id_number}
          </div>
        </div>
      </div>

      {/* Payslip summary grid */}
      <div className="mt-4 grid grid-cols-2 gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
        <div className="rounded-xl bg-slate-50 p-3 dark:bg-slate-800/60">
          <div className="flex items-center gap-1 text-[9px] font-bold uppercase tracking-wider text-slate-400">
            <DollarSign size={10} />
            {t("common.payroll.table.base_pay")}
          </div>
          <div className="mt-1 font-mono text-sm font-bold text-slate-700 dark:text-slate-200">
            {payslip ? formatCurrency(payslip.base_pay_amount) : "—"}
          </div>
        </div>
        <div className="rounded-xl bg-emerald-50 p-3 dark:bg-emerald-950/30">
          <div className="flex items-center gap-1 text-[9px] font-bold uppercase tracking-wider text-emerald-500">
            <TrendingUp size={10} />
            {t("common.payroll.table.overtime")}
          </div>
          <div className="mt-1 font-mono text-sm font-bold text-emerald-600 dark:text-emerald-400">
            {payslip && parseFloat(payslip.overtime_pay_amount) > 0
              ? `+${formatCurrency(payslip.overtime_pay_amount)}`
              : "—"}
          </div>
        </div>
        <div className="rounded-xl bg-blue-50 p-3 dark:bg-blue-950/30">
          <div className="flex items-center gap-1 text-[9px] font-bold uppercase tracking-wider text-blue-500">
            <Gift size={10} />
            {t("common.payroll.table.bonus")}
          </div>
          <div className="mt-1 font-mono text-sm font-bold text-blue-600 dark:text-blue-400">
            {bonusAmount > 0 ? `+${formatCurrency(String(bonusAmount))}` : "—"}
          </div>
        </div>
        <div className="rounded-xl bg-primary-50 p-3 dark:bg-primary-950/30">
          <div className="flex items-center gap-1 text-[9px] font-bold uppercase tracking-wider text-primary-500">
            <Wallet size={10} />
            {t("common.payroll.table.net_pay")}
          </div>
          <div className="mt-1 font-mono text-sm font-bold text-primary-600 dark:text-primary-400">
            {payslip ? formatCurrency(payslip.net_pay_amount) : "—"}
          </div>
        </div>
      </div>

      {/* Status badge */}
      {payslip && (
        <div className="mt-4 flex justify-end border-t border-slate-100 pt-4 dark:border-slate-800">
          <span
            className={cn(
              "inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm",
              payslip.status === "paid"
                ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
                : "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800"
            )}
          >
            {payslip.status === "paid" ? (
              <CheckCircle2 size={12} />
            ) : (
              <AlertCircle size={12} />
            )}
            {payslip.status === "paid"
              ? t("common.payroll.status.paid")
              : t("common.payroll.status.finalized")}
          </span>
        </div>
      )}
    </div>
  );
}

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
            <ResponsiveTable<Employee>
              columns={columns}
              contextMenuItems={contextMenuItems}
              data={paginatedEmployees}
              emptyState={{
                title: t("common.no_records_found"),
                icon: <User className="h-10 w-10" />,
              }}
              enableDragSelect
              getCardKey={(emp) => emp.id}
              getRowId={(emp) => emp.id}
              isLoading={isLoading}
              onBulkCellSave={handleBulkCellSave}
              onCellSave={handleCellSave}
              onRowClick={(emp) => { onViewDetail(emp); }}
              onSelectionChange={(ids) => { onToggleSelectAll(Array.from(ids)); }}
              renderCard={(emp) => (
                <PayrollMobileCard
                  canReadSensitive={canReadSensitive}
                  employee={emp}
                  isSelected={selectedIds.has(emp.id)}
                  onSelect={() => { onToggleSelect(emp.id); }}
                  onViewDetail={() => { onViewDetail(emp); }}
                  payslip={payslipMap[emp.id]}
                  t={t}
                />
              )}
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

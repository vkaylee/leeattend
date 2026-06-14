import { 
  User,
  Check,
  ChevronRight,
  Calculator,
  CheckCircle2,
  AlertCircle
} from "lucide-react";
import React, { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";

import PaginationControls from "@/components/common/pagination-controls";
import SmartTable, { type SmartTableColumnMeta } from "@/components/ui/smart-table";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useMyPermissions } from "@/hooks/use-permissions";
import { Employee, Payslip, IncentiveEvaluationResult } from "@/lib/api/types";
import { cn } from "@/lib/utils";

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
  const handleCellSave = async (employeeId: string, columnId: string, value: any) => {
    // Typically this would fire an API mutation, here we display a success toast feedback
    toast.success(
      t("common.saved", "Saved change successfully"),
      `Updated ${columnId} for employee: ${value}`
    );
  };

  const handleBulkCellSave = async (changes: { rowId: string; columnId: string; value: any }[]) => {
    toast.success(
      t("common.saved", "Saved change successfully"),
      `Updated ${changes.length} cells successfully`
    );
  };

  const validateCell = (rowId: string, columnId: string, value: any) => {
    if (columnId === "base_pay") {
      const num = Number(value);
      if (isNaN(num) || num <= 0) {
        return t("common.payroll.validation.base_pay_positive", "Base pay must be a positive number");
      }
    }
    if (columnId === "status") {
      const allowed = ["draft", "finalized", "paid"];
      if (!allowed.includes(value)) {
        return t("common.payroll.validation.status_invalid", "Status must be draft, finalized, or paid");
      }
    }
    return null;
  };

  // Define Columns for SmartTable
  const columns = useMemo(() => [
    {
      id: "select",
      accessorKey: "id",
      header: () => (
        <div className="flex items-center justify-center w-full" onClick={(e) => e.stopPropagation()}>
          <button
            className={cn(
              "w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all cursor-pointer",
              isPageSelected
                ? "bg-primary-600 border-primary-600 text-white shadow-lg shadow-primary-100"
                : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 group-hover:border-primary-300"
            )}
            onClick={() => handleSelectPage(currentPageIds, isPageSelected)}
            type="button"
          >
            {isPageSelected ? <Check className="w-3.5 h-3.5" strokeWidth={4} /> : null}
          </button>
        </div>
      ),
      cell: ({ row }: any) => {
        const emp = row.original;
        const isSelected = selectedIds.has(emp.id);
        return (
          <div className="flex items-center justify-center w-full" onClick={(e) => e.stopPropagation()}>
            <button
              className={cn(
                "w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all cursor-pointer",
                isSelected
                  ? "bg-primary-600 border-primary-600 text-white shadow-lg shadow-primary-100"
                  : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 group-hover:border-primary-300"
              )}
              data-testid={`select-employee-${emp.id}`}
              onClick={() => onToggleSelect(emp.id)}
              type="button"
            >
              {isSelected ? <Check className="w-3.5 h-3.5" strokeWidth={4} /> : null}
            </button>
          </div>
        );
      },
      size: 70,
      meta: {
        sticky: "left",
        headerClassName: "text-center",
        cellClassName: "text-center"
      } as SmartTableColumnMeta
    },
    {
      id: "employee",
      header: t("employees.table.name"),
      cell: ({ row }: any) => {
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
      meta: {
        sticky: "left"
      } as SmartTableColumnMeta
    },
    {
      id: "base_pay",
      header: () => <div className="text-right w-full">{t("common.payroll.table.base_pay")}</div>,
      cell: ({ row }: any) => {
        const emp = row.original;
        const slip = payslipMap[emp.id];
        if (!canReadSensitive) {
          return <div className="text-right font-mono text-sm font-bold text-slate-400">***</div>;
        }
        return (
          <div className="text-right font-mono text-sm font-bold text-slate-600 dark:text-slate-350">
            {slip ? new Intl.NumberFormat('vi-VN').format(parseFloat(slip.base_pay_amount)) : "—"}
          </div>
        );
      },
      size: 150,
      meta: {
        editable: canReadSensitive && canWriteSensitive,
        editType: "number",
        cellClassName: "text-right"
      } as SmartTableColumnMeta
    },
    {
      id: "overtime",
      header: () => <div className="text-right w-full">{t("common.payroll.table.overtime")}</div>,
      cell: ({ row }: any) => {
        const emp = row.original;
        const slip = payslipMap[emp.id];
        if (!canReadSensitive) {
          return <div className="text-right font-mono text-sm font-bold text-slate-400">***</div>;
        }
        return (
          <div className="text-right font-mono text-sm text-emerald-600 dark:text-emerald-400 font-bold">
            {slip && parseFloat(slip.overtime_pay_amount) > 0 
              ? `+${new Intl.NumberFormat('vi-VN').format(parseFloat(slip.overtime_pay_amount))}` 
              : "—"
            }
          </div>
        );
      },
      size: 150,
      meta: {
        cellClassName: "text-right"
      } as SmartTableColumnMeta
    },
    {
      id: "bonus",
      header: () => <div className="text-right w-full">{t("common.payroll.table.bonus")}</div>,
      cell: ({ row }: any) => {
        const emp = row.original;
        const slip = payslipMap[emp.id];
        if (!canReadSensitive) {
          return <div className="text-right font-mono text-sm font-bold text-slate-400">***</div>;
        }
        const bonusAmount = slip?.incentives && slip.incentives.length > 0 
          ? slip.incentives.reduce((accumulator: number, inc: IncentiveEvaluationResult) => accumulator + parseFloat(inc.amount), 0)
          : 0;

        return (
          <div className="text-right font-mono text-sm text-blue-600 dark:text-blue-400 font-bold">
            {bonusAmount > 0 ? `+${new Intl.NumberFormat('vi-VN').format(bonusAmount)}` : "—"}
          </div>
        );
      },
      size: 150,
      meta: {
        cellClassName: "text-right"
      } as SmartTableColumnMeta
    },
    {
      id: "net_pay",
      header: () => <div className="text-right w-full">{t("common.payroll.table.net_pay")}</div>,
      cell: ({ row }: any) => {
        const emp = row.original;
        const slip = payslipMap[emp.id];
        if (!canReadSensitive) {
          return <div className="text-right font-mono text-base font-bold text-slate-400">***</div>;
        }
        return (
          <div className="text-right font-bold text-primary-600 dark:text-primary-400 text-base font-mono">
            {slip ? new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount)) : "—"}
          </div>
        );
      },
      size: 160,
      meta: {
        cellClassName: "text-right"
      } as SmartTableColumnMeta
    },
    {
      id: "status",
      header: () => <div className="text-center w-full">{t("common.payroll.table.status")}</div>,
      cell: ({ row }: any) => {
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
              slip.status === 'paid' 
                ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" 
                : "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800"
            )}>
              {slip.status === 'paid' ? <CheckCircle2 size={12} /> : <AlertCircle size={12} />}
              {slip.status === 'paid' ? t("common.payroll.status.paid") : t("common.payroll.status.finalized")}
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
        headerClassName: "text-center",
        cellClassName: "text-center"
      } as SmartTableColumnMeta
    },
    {
      id: "actions",
      header: "",
      cell: () => (
        <div className="flex items-center justify-end w-full pr-4 text-slate-350 dark:text-slate-650 group-hover:text-primary-500">
          <ChevronRight className="w-4.5 h-4.5 transition-all group-hover:translate-x-1" />
        </div>
      ),
      size: 60,
      meta: {
        cellClassName: "text-right"
      } as SmartTableColumnMeta
    }
  ], [t, payslipMap, canReadSensitive, canWriteSensitive, selectedIds, isPageSelected, currentPageIds, handleSelectPage, onToggleSelect]);

  // Context Menu setup
  const contextMenuItems = (emp: Employee) => [
    {
      label: t("common.view", "View Details"),
      onClick: () => onViewDetail(emp),
      icon: <User className="w-3.5 h-3.5" />,
    },
    {
      label: selectedIds.has(emp.id) ? t("common.deselect", "Deselect") : t("common.select", "Select"),
      onClick: () => onToggleSelect(emp.id),
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
              data={paginatedEmployees}
              columns={columns}
              getRowId={(emp) => emp.id}
              selectedRowIds={selectedIds}
              onSelectionChange={onToggleSelectAll ? (ids) => onToggleSelectAll(Array.from(ids)) : undefined}
              onCellSave={handleCellSave}
              onBulkCellSave={handleBulkCellSave}
              validateCell={validateCell}
              onRowClick={(emp) => onViewDetail(emp)}
              contextMenuItems={contextMenuItems}
              emptyMessage={t("common.no_records_found")}
              enableDragSelect
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

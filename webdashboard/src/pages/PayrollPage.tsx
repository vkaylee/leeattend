import { useQuery, useQueryClient, keepPreviousData } from "@tanstack/react-query";
import { 
  Calculator, 
  ArrowLeft, 
  ArrowRight
} from "lucide-react";
import React, { useState, useMemo, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { MonthYearPicker } from "@/components/common/MonthYearPicker";
import AdjustmentDrawer from "@/components/payroll/AdjustmentDrawer";
import BatchCalculationModal from "@/components/payroll/BatchCalculationModal";
import _EmployeeSalaryConfig from "@/components/payroll/EmployeeSalaryConfig";
import PayrollDetailDrawer from "@/components/payroll/PayrollDetailDrawer";
import PayrollSummaryTable from "@/components/payroll/PayrollSummaryTable";
import _PayrollTable from "@/components/payroll/PayrollTable";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import _Modal from "@/components/ui/Modal";
import { cn } from "@/lib/utils";

import { useToast } from "../components/ui/Toast";
import {
  fetchEmployees,
  searchEmployees,
  fetchEmployeePayslips,
  fetchWorkspacePayslips,
  getCurrentWorkspaceId,
  fetchWorkspaceAttendanceAdjustments,
  Employee,
} from "../lib/api";

export default function PayrollPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const _toast = useToast();
  
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [searchTerm, setSearchTerm] = useState("");
  const [isBatchCalcModalOpen, setIsBatchCalcModalOpen] = useState(false);
  const [isAdjustmentDrawerOpen, setIsAdjustmentDrawerOpen] = useState(false);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);

  // Debounce search term for server-side search
  const [debouncedSearchTerm, setDebouncedSearchTerm] = useState("");
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedSearchTerm(searchTerm);
    }, 300);
    return () => { clearTimeout(timer); };
  }, [searchTerm]);
  
  const [calcYear, setCalcYear] = useState(() => {
    const d = new Date();
    d.setMonth(d.getMonth() - 1);
    return d.getFullYear();
  });
  const [calcMonth, setCalcMonth] = useState(() => {
 
    const d = new Date();
    d.setMonth(d.getMonth() - 1);
    return d.getMonth() + 1;
  });

  // Use server-side search when search term exists, otherwise fetch all employees
  const { data: employeesData, isLoading: empsLoading } = useQuery({
    queryKey: ["employees", wsId, debouncedSearchTerm],
    queryFn: () => {
      if (debouncedSearchTerm.trim()) {
        // Server-side search with higher limit to show more results
        return searchEmployees(wsId, debouncedSearchTerm, 1, 100);
      }
      // Fetch all employees (up to 3000)
      return fetchEmployees(wsId, 1, 3000);
    },
    enabled: !!wsId,
    placeholderData: keepPreviousData,
  });

  const employees = useMemo(() => employeesData?.data ?? [], [employeesData?.data]);

  const { data: workspacePayslipsData = [], isLoading: workspacePayslipsLoading } = useQuery({
    queryKey: ["workspace-payslips", wsId, calcYear, calcMonth],
    queryFn: () => fetchWorkspacePayslips(wsId, calcYear, calcMonth),
    enabled: !!wsId,
    placeholderData: keepPreviousData,
  });

  const workspacePayslips = workspacePayslipsData;
 

  const { data: employeePayslips = [], isLoading: employeePayslipsLoading } = useQuery({
    queryKey: ["payslips", selectedEmployee?.id],
    queryFn: () => fetchEmployeePayslips(wsId, selectedEmployee!.id),
    enabled: !!selectedEmployee && isDetailDrawerOpen,
  });

  const { data: allAdjustments = [] } = useQuery({
    queryKey: ["adjustments", wsId, "pending"],
    queryFn: () => fetchWorkspaceAttendanceAdjustments(wsId, "pending"),
    enabled: !!wsId,
    placeholderData: keepPreviousData,
  });

  const employeeAdjustments = selectedEmployee 
    ? allAdjustments.filter(adj => adj.employee_id === selectedEmployee.id)
    : [];

  const toggleSelectAll = (ids?: string[]) => {
    if (ids) {
      const next = new Set(selectedIds);
      const allIncluded = ids.every(id => next.has(id));
      if (allIncluded) {
        ids.forEach(id => next.delete(id));
      } else {
        ids.forEach(id => next.add(id));
      }
      setSelectedIds(next);
      return;
    }

    // Server-side search already filters employees, so use all visible employees
    if (selectedIds.size === employees.length) setSelectedIds(new Set());
    else setSelectedIds(new Set(employees.map(e => e.id)));
  };

  const toggleSelect = (id: string) => {
    const next = new Set(selectedIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setSelectedIds(next);
  };

  const handleViewDetail = (employee: Employee) => {
    setSelectedEmployee(employee);
    setIsDetailDrawerOpen(true);
  };

  const employeeNames = useMemo(() => {
    const map: Record<string, string> = {};
    employees.forEach(e => { map[e.id] = e.full_name; });
    return map;
  }, [employees]);

  const handleAdjustmentResolved = () => {
    queryClient.invalidateQueries({ queryKey: ["payslips", selectedEmployee?.id] });
    queryClient.invalidateQueries({ queryKey: ["workspace-payslips"] });
    queryClient.invalidateQueries({ queryKey: ["adjustments"] });
  };

  const changeMonth = (delta: number) => {
 
    let nextMonth = calcMonth + delta;
    let nextYear = calcYear;
    if (nextMonth > 12) { nextMonth = 1; nextYear++; }
    else if (nextMonth < 1) { nextMonth = 12; nextYear--; }
    setCalcMonth(nextMonth);
    setCalcYear(nextYear);
  };

  if (!wsId) return <div className="p-8 text-center">{t("common.select_workspace")}</div>;

  return (
    <div className="flex flex-col h-[calc(100vh-8rem)] gap-8 overflow-hidden animate-in fade-in duration-500">
      {/* Header with Stats and Actions */}
      <header className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200" data-testid="payroll-title">{t("payroll.title")}</h1>
          <p className="text-slate-500 dark:text-slate-400 mt-1 font-medium">{t("payroll.subtitle")}</p>
        </div>

        <div className="flex items-center gap-4 bg-white dark:bg-slate-900 p-2 rounded-[2rem] border border-slate-200 dark:border-slate-700 shadow-sm">
           <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-2xl p-1 shadow-inner border border-slate-200 dark:border-slate-700/50">
              <button className="w-10 h-10 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-sm rounded-xl transition-all" onClick={() => { changeMonth(-1); }}><ArrowLeft size={18} /></button>
              
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <button className="px-6 flex flex-col items-center min-w-[160px] cursor-pointer hover:bg-white dark:hover:bg-slate-900 hover:shadow-sm rounded-xl transition-all py-1.5 group outline-none border border-transparent hover:border-slate-100 dark:border-slate-800">
                    <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest leading-none mb-1 group-hover:text-primary-500 dark:group-hover:text-primary-400 transition-colors">{t("payroll.table.period")}</span>
                    <span className="text-sm font-bold text-slate-700 dark:text-slate-200 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors uppercase tracking-tighter">
                      {calcMonth} / {calcYear}
                    </span>
                  </button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="center" className="w-80 p-4 rounded-[2rem] shadow-2xl border-slate-100 dark:border-slate-800" sideOffset={8}>
                    <DropdownMenuLabel className="text-center text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 mb-4">{t("common.select_month", { defaultValue: "SELECT PAY PERIOD" })}</DropdownMenuLabel>
                    <MonthYearPicker month={calcMonth} onChange={(m, y) => { setCalcMonth(m); setCalcYear(y); }} year={calcYear} />
                </DropdownMenuContent>
              </DropdownMenu>

              <button className="w-10 h-10 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-sm rounded-xl transition-all" onClick={() => { changeMonth(1); }}><ArrowRight size={18} /></button>
           </div>

           <div className="h-10 w-px bg-slate-100 dark:bg-slate-700" />

           <button 
            className={cn(
              "flex items-center gap-3 px-8 py-3 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all shadow-lg active:scale-95",
              selectedIds.size > 0 
                ? "bg-primary-600 text-white shadow-primary-100 hover:bg-primary-700" 
                : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 shadow-none cursor-not-allowed"
            )}
            data-testid="calculate-batch-btn"
            disabled={selectedIds.size === 0}
            onClick={() => {
              setIsBatchCalcModalOpen(true);
            }}
           >
             <Calculator size={18} />
             {t("payroll.calculate_btn")} {selectedIds.size > 0 && <span className="ml-1 bg-white dark:bg-slate-900/20 px-2 py-0.5 rounded-lg text-white">{selectedIds.size}</span>}
           </button>
        </div>
      </header>

      {/* Main Grid Section */}
      <div className="flex-1 min-h-0">
        <PayrollSummaryTable 
          employees={employees}
          isLoading={empsLoading || workspacePayslipsLoading}
          onSearchChange={setSearchTerm}
          onToggleSelect={toggleSelect}
          onToggleSelectAll={toggleSelectAll}
          onViewDetail={handleViewDetail}
          payslips={workspacePayslips}
          searchTerm={searchTerm}
          selectedIds={selectedIds}
        />
      </div>

      {/* Detail Drawer */}
      <PayrollDetailDrawer 
        adjustments={employeeAdjustments}
        employee={selectedEmployee}
        isLoadingPayslips={employeePayslipsLoading}
        isOpen={isDetailDrawerOpen}
        onClose={() => { setIsDetailDrawerOpen(false); }}
        onResolveAdjustments={() => { setIsAdjustmentDrawerOpen(true); }}
        payslips={employeePayslips}
        workspaceId={wsId}
      />

      {/* Adjustment Drawer */}
      {!!selectedEmployee && (
        <AdjustmentDrawer
          employeeId={selectedEmployee.id}
          employeeName={selectedEmployee.full_name}
          isOpen={isAdjustmentDrawerOpen}
          onAdjustmentResolved={handleAdjustmentResolved}
          onClose={() => { setIsAdjustmentDrawerOpen(false); }}
        />
      )}

      {/* Batch Calculate Modal */}
      <BatchCalculationModal
        employeeIds={Array.from(selectedIds)}
        employeeNames={employeeNames}
        isOpen={isBatchCalcModalOpen}
        onClose={() => { setIsBatchCalcModalOpen(false); }}
        workspaceId={wsId}
      />
    </div>
  );
}
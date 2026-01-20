import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Banknote, 
  Calculator, 
  ChevronRight, 
  History, 
  Search, 
  UserCircle,
  X,
  Check,
  CalendarDays,
  ArrowRight,
  ArrowLeft,
  Filter
} from "lucide-react";
import { 
  fetchEmployees, 
  fetchEmployeePayslips, 
  fetchWorkspacePayslips,
  calculate_payroll, 
  getCurrentWorkspaceId,
  fetchWorkspaceAttendanceAdjustments,
  Employee,
} from "../lib/api";
import { useToast } from "../components/Toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import Modal from "@/components/ui/Modal";
import EmployeeSalaryConfig from "@/components/payroll/EmployeeSalaryConfig";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import PayrollTable from "@/components/payroll/PayrollTable";
import AdjustmentDrawer from "@/components/payroll/AdjustmentDrawer";
import BatchCalculationModal from "@/components/payroll/BatchCalculationModal";
import PayrollSummaryTable from "@/components/payroll/PayrollSummaryTable";
import PayrollDetailDrawer from "@/components/payroll/PayrollDetailDrawer";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export default function PayrollPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [searchTerm, setSearchTerm] = useState("");
  const [isBatchCalcModalOpen, setIsBatchCalcModalOpen] = useState(false);
  const [isAdjustmentDrawerOpen, setIsAdjustmentDrawerOpen] = useState(false);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  
  // Default to last month
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

  const { data: employees = [], isLoading: empsLoading } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId),
    enabled: !!wsId,
  });

  const { data: workspacePayslips = [], isLoading: workspacePayslipsLoading } = useQuery({
    queryKey: ["workspace-payslips", wsId, calcYear, calcMonth],
    queryFn: () => fetchWorkspacePayslips(wsId, calcYear, calcMonth),
    enabled: !!wsId,
  });

  const { data: employeePayslips = [], isLoading: employeePayslipsLoading } = useQuery({
    queryKey: ["payslips", selectedEmployee?.id],
    queryFn: () => fetchEmployeePayslips(wsId, selectedEmployee!.id),
    enabled: !!selectedEmployee && isDetailDrawerOpen,
  });

  const { data: allAdjustments = [] } = useQuery({
    queryKey: ["adjustments", wsId, "pending"],
    queryFn: () => fetchWorkspaceAttendanceAdjustments(wsId, "pending"),
    enabled: !!wsId,
  });

  const employeeAdjustments = selectedEmployee 
    ? allAdjustments.filter(adj => adj.employee_id === selectedEmployee.id)
    : [];

  const toggleSelectAll = () => {
    const employeesToSelect = employees.filter(emp => 
      `${emp.first_name} ${emp.last_name}`.toLowerCase().includes(searchTerm.toLowerCase()) ||
      emp.employee_id_number.toLowerCase().includes(searchTerm.toLowerCase())
    );

    if (selectedIds.size === employeesToSelect.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(employeesToSelect.map(e => e.id)));
    }
  };

  const toggleSelect = (id: string) => {
    const next = new Set(selectedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    setSelectedIds(next);
  };

  const handleViewDetail = (employee: Employee) => {
    setSelectedEmployee(employee);
    setIsDetailDrawerOpen(true);
  };

  const employeeNames = React.useMemo(() => {
    const map: Record<string, string> = {};
    employees.forEach(e => {
      map[e.id] = `${e.first_name} ${e.last_name}`;
    });
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
    if (nextMonth > 12) {
      nextMonth = 1;
      nextYear++;
    } else if (nextMonth < 1) {
      nextMonth = 12;
      nextYear--;
    }
    setCalcMonth(nextMonth);
    setCalcYear(nextYear);
  };

  if (!wsId) return <div className="p-8 text-center">{t("common.select_workspace")}</div>;

  return (
    <div className="flex flex-col h-[calc(100vh-8rem)] gap-8 overflow-hidden animate-in fade-in duration-500">
      {/* Header with Stats and Actions */}
      <header className="flex flex-col md:flex-row md:items-end justify-between gap-6">
        <div>
          <h1 className="text-4xl font-black text-slate-900 tracking-tight">{t("payroll.title")}</h1>
          <p className="text-slate-500 mt-2 text-lg">{t("payroll.subtitle")}</p>
        </div>

        <div className="flex items-center gap-4 bg-white p-2 rounded-[2rem] border border-slate-100 shadow-premium-sm">
           <div className="flex items-center bg-slate-50 rounded-2xl p-1 shadow-inner border border-slate-100">
              <button 
                onClick={() => changeMonth(-1)}
                className="w-10 h-10 flex items-center justify-center text-slate-400 hover:text-primary-600 hover:bg-white hover:shadow-sm rounded-xl transition-all"
              >
                <ArrowLeft size={18} />
              </button>
              
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <button className="px-6 flex flex-col items-center min-w-[140px] cursor-pointer hover:bg-slate-50 rounded-xl transition-colors py-1 group outline-none">
                    <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest leading-none mb-1 group-hover:text-primary-500 transition-colors">Kỳ lương</span>
                    <span className="text-sm font-bold text-slate-900 group-hover:text-primary-600 transition-colors">
                      Tháng {calcMonth}, {calcYear}
                    </span>
                  </button>
                </DropdownMenuTrigger>
                <DropdownMenuContent className="w-72 p-4" align="center">
                    <DropdownMenuLabel className="text-center text-xs text-slate-400 mb-4">Chọn kỳ lương</DropdownMenuLabel>
                    <MonthYearPicker 
                        month={calcMonth}
                        year={calcYear}
                        onChange={(m, y) => {
                            setCalcMonth(m);
                            setCalcYear(y);
                        }}
                    />
                </DropdownMenuContent>
              </DropdownMenu>

              <button 
                onClick={() => changeMonth(1)}
                className="w-10 h-10 flex items-center justify-center text-slate-400 hover:text-primary-600 hover:bg-white hover:shadow-sm rounded-xl transition-all"
              >
                <ArrowRight size={18} />
              </button>
           </div>

           <div className="h-10 w-px bg-slate-100" />

           <button 
            onClick={() => setIsBatchCalcModalOpen(true)}
            disabled={selectedIds.size === 0}
            className={cn(
              "flex items-center gap-2 px-8 py-3 rounded-2xl font-black text-sm transition-all shadow-lg active:scale-95",
              selectedIds.size > 0 
                ? "bg-slate-900 text-white shadow-slate-200 hover:bg-black" 
                : "bg-slate-100 text-slate-400 shadow-none cursor-not-allowed"
            )}
           >
             <Calculator size={18} />
             {t("payroll.calculate_btn")} {selectedIds.size > 0 ? `(${selectedIds.size})` : ""}
           </button>
        </div>
      </header>

      {/* Main Grid Section */}
      <div className="flex-1 min-h-0">
        <PayrollSummaryTable 
          employees={employees}
          payslips={workspacePayslips}
          isLoading={empsLoading || workspacePayslipsLoading}
          selectedIds={selectedIds}
          onToggleSelect={toggleSelect}
          onToggleSelectAll={toggleSelectAll}
          onViewDetail={handleViewDetail}
          searchTerm={searchTerm}
          onSearchChange={setSearchTerm}
        />
      </div>

      {/* Detail Drawer */}
      <PayrollDetailDrawer 
        isOpen={isDetailDrawerOpen}
        onClose={() => setIsDetailDrawerOpen(false)}
        employee={selectedEmployee}
        workspaceId={wsId}
        payslips={employeePayslips}
        isLoadingPayslips={employeePayslipsLoading}
        adjustments={employeeAdjustments}
        onResolveAdjustments={() => setIsAdjustmentDrawerOpen(true)}
      />

      {/* Adjustment Drawer */}
      {selectedEmployee && (
        <AdjustmentDrawer
          isOpen={isAdjustmentDrawerOpen}
          onClose={() => setIsAdjustmentDrawerOpen(false)}
          employeeId={selectedEmployee.id}
          employeeName={`${selectedEmployee.first_name} ${selectedEmployee.last_name}`}
          onAdjustmentResolved={handleAdjustmentResolved}
        />
      )}

      {/* Batch Calculate Modal */}
      <BatchCalculationModal
        isOpen={isBatchCalcModalOpen}
        onClose={() => setIsBatchCalcModalOpen(false)}
        workspaceId={wsId}
        employeeIds={Array.from(selectedIds)}
        employeeNames={employeeNames}
      />
    </div>
  );
}

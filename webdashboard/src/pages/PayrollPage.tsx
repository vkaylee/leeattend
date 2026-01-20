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
    if (selectedIds.size === employeesToSelect.length) setSelectedIds(new Set());
    else setSelectedIds(new Set(employeesToSelect.map(e => e.id)));
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

  const employeeNames = React.useMemo(() => {
    const map: Record<string, string> = {};
    employees.forEach(e => { map[e.id] = `${e.first_name} ${e.last_name}`; });
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
          <h1 className="text-3xl font-bold tracking-tight text-slate-900">{t("payroll.title")}</h1>
          <p className="text-slate-500 mt-1 font-medium">{t("payroll.subtitle")}</p>
        </div>

        <div className="flex items-center gap-4 bg-white p-2 rounded-[2rem] border border-slate-200 shadow-sm">
           <div className="flex items-center bg-slate-100/50 rounded-2xl p-1 shadow-inner border border-slate-200/50">
              <button onClick={() => changeMonth(-1)} className="w-10 h-10 flex items-center justify-center text-slate-400 hover:text-primary-600 hover:bg-white hover:shadow-sm rounded-xl transition-all"><ArrowLeft size={18} /></button>
              
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <button className="px-6 flex flex-col items-center min-w-[160px] cursor-pointer hover:bg-white hover:shadow-sm rounded-xl transition-all py-1.5 group outline-none border border-transparent hover:border-slate-100">
                    <span className="text-[9px] font-black text-slate-400 uppercase tracking-widest leading-none mb-1 group-hover:text-primary-500 transition-colors">{t("payroll.table.period")}</span>
                    <span className="text-sm font-black text-slate-900 group-hover:text-primary-600 transition-colors uppercase tracking-tighter">
                      {t("common.month")} {calcMonth}, {calcYear}
                    </span>
                  </button>
                </DropdownMenuTrigger>
                <DropdownMenuContent className="w-80 p-4 rounded-[2rem] shadow-2xl border-slate-100" align="center" sideOffset={8}>
                    <DropdownMenuLabel className="text-center text-[10px] font-black uppercase tracking-widest text-slate-400 mb-4">{t("common.select_month", { defaultValue: "SELECT PAY PERIOD" })}</DropdownMenuLabel>
                    <MonthYearPicker month={calcMonth} year={calcYear} onChange={(m, y) => { setCalcMonth(m); setCalcYear(y); }} />
                </DropdownMenuContent>
              </DropdownMenu>

              <button onClick={() => changeMonth(1)} className="w-10 h-10 flex items-center justify-center text-slate-400 hover:text-primary-600 hover:bg-white hover:shadow-sm rounded-xl transition-all"><ArrowRight size={18} /></button>
           </div>

           <div className="h-10 w-px bg-slate-100" />

           <button 
            onClick={() => setIsBatchCalcModalOpen(true)}
            disabled={selectedIds.size === 0}
            className={cn(
              "flex items-center gap-3 px-8 py-3 rounded-2xl font-black text-[10px] uppercase tracking-widest transition-all shadow-lg active:scale-95",
              selectedIds.size > 0 
                ? "bg-primary-600 text-white shadow-primary-100 hover:bg-primary-700" 
                : "bg-slate-100 text-slate-400 shadow-none cursor-not-allowed"
            )}
           >
             <Calculator size={18} />
             {t("payroll.calculate_btn")} {selectedIds.size > 0 && <span className="ml-1 bg-white/20 px-2 py-0.5 rounded-lg text-white">{selectedIds.size}</span>}
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
      {/* Drawers and Modals... */}

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

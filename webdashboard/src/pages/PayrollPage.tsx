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
} from "lucide-react";
import { 
  fetchEmployees, 
  fetchEmployeePayslips, 
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

export default function PayrollPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [isCalcModalOpen, setIsCalcModalOpen] = useState(false);
  const [isAdjustmentDrawerOpen, setIsAdjustmentDrawerOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<"payslips" | "config">("payslips");
  
  const [calcYear, setCalcYear] = useState(new Date().getFullYear());
  const [calcMonth, setCalcMonth] = useState(new Date().getMonth() + 1);

  const { data: employees = [], isLoading: empsLoading } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId),
    enabled: !!wsId,
  });

  const { data: payslips = [], isLoading: payslipsLoading } = useQuery({
    queryKey: ["payslips", selectedEmployee?.id],
    queryFn: () => fetchEmployeePayslips(wsId, selectedEmployee!.id),
    enabled: !!selectedEmployee,
  });

  const { data: allAdjustments = [] } = useQuery({
    queryKey: ["adjustments", wsId, "pending"],
    queryFn: () => fetchWorkspaceAttendanceAdjustments(wsId, "pending"),
    enabled: !!wsId,
  });

  const employeeAdjustments = selectedEmployee 
    ? allAdjustments.filter(adj => adj.employee_id === selectedEmployee.id)
    : [];

  const calcMutation = useMutation({
    mutationFn: (data: { year: number; month: number }) => 
      calculate_payroll(wsId, selectedEmployee!.id, data.year, data.month),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["payslips", selectedEmployee?.id] });
      toast.success(t("payroll.messages.calculate_success"));
      setIsCalcModalOpen(false);
    },
    onError: (e: any) => toast.error(e.message, t("payroll.messages.calculate_error"))
  });

  const handleCalculate = () => {
    calcMutation.mutate({ year: calcYear, month: calcMonth });
  };

  const handleAdjustmentResolved = () => {
    // Optionally trigger a recalculation for the affected month? 
    // For now, just invalidate payslips so user sees they can re-run payroll
    queryClient.invalidateQueries({ queryKey: ["payslips", selectedEmployee?.id] });
    // Also invalidate adjustments
    queryClient.invalidateQueries({ queryKey: ["adjustments"] });
  };

  const filteredEmployees = employees.filter(emp => 
    `${emp.first_name} ${emp.last_name}`.toLowerCase().includes(searchTerm.toLowerCase()) ||
    emp.employee_id_number.toLowerCase().includes(searchTerm.toLowerCase())
  );

  if (!wsId) return <div className="p-8 text-center">{t("common.select_workspace")}</div>;

  return (
    <div className="flex h-[calc(100vh-8rem)] gap-6 overflow-hidden animate-in fade-in duration-500">
      {/* Left Sidebar: Employee List */}
      <div className={`flex-1 flex flex-col gap-6 overflow-hidden transition-all ${selectedEmployee ? 'max-w-[350px]' : 'max-w-full'}`}>
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900">{t("payroll.title")}</h1>
          <p className="text-slate-500">{t("payroll.subtitle")}</p>
        </div>

        <div className="relative">
          <EmployeeSearch
            workspaceId={wsId}
            value={selectedEmployee?.id}
            onChange={(id) => {
              const emp = employees.find(e => e.id === id);
              if (emp) setSelectedEmployee(emp);
            }}
            placeholder={t("payroll.search_placeholder")}
          />
        </div>

        <div className="relative mt-2 group">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 w-4 h-4 group-focus-within:text-primary-500" />
          <input 
            type="text" 
            placeholder={t("common.filter_by_name", "Filter sidebar...")}
            className="w-full pl-10 pr-4 py-2.5 bg-white border border-slate-100 rounded-2xl outline-none focus:ring-4 focus:ring-primary-500/10 transition-all text-sm"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>

        <div className="bg-white border border-slate-200 rounded-3xl flex-1 overflow-auto shadow-sm">
          {empsLoading ? (
            <div className="p-8 text-center text-slate-400 italic">{t("common.loading")}</div>
          ) : (
            <div className="divide-y divide-slate-50">
              {filteredEmployees.map(emp => (
                <button
                  key={emp.id}
                  onClick={() => setSelectedEmployee(emp)}
                  className={`w-full flex items-center gap-4 p-4 hover:bg-slate-50 transition-all text-left group ${selectedEmployee?.id === emp.id ? 'bg-primary-50/50' : ''}`}
                >
                  <div className={`w-10 h-10 rounded-2xl flex items-center justify-center font-bold text-base transition-colors ${selectedEmployee?.id === emp.id ? 'bg-primary-600 text-white' : 'bg-slate-100 text-slate-400'}`}>
                    {emp.last_name[0]}
                  </div>
                  <div className="flex-1">
                    <div className={`font-bold text-sm ${selectedEmployee?.id === emp.id ? 'text-blue-700' : 'text-slate-900'}`}>
                      {emp.first_name} {emp.last_name}
                    </div>
                    <div className="text-[10px] text-slate-400 font-black tracking-widest uppercase">{emp.employee_id_number}</div>
                  </div>
                  <ChevronRight className={`w-4 h-4 transition-transform ${selectedEmployee?.id === emp.id ? 'text-blue-400 translate-x-1' : 'text-slate-200'}`} />
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Main Content: Payslips */}
      <div className="flex-1 flex flex-col gap-6 overflow-hidden">
        {!selectedEmployee ? (
          <div className="bg-white border border-slate-200 rounded-[40px] flex-1 flex flex-col items-center justify-center text-center p-12 shadow-sm">
            <div className="bg-slate-50 p-6 rounded-[32px] mb-6">
              <Banknote className="w-12 h-12 text-slate-300" />
            </div>
            <h3 className="text-xl font-bold text-slate-900">{t("payroll.empty_state.title")}</h3>
            <p className="text-slate-500 mt-2 max-w-xs mx-auto text-sm">
              {t("payroll.empty_state.desc")}
            </p>
          </div>
        ) : (
          <>
            <div className="bg-white p-6 rounded-[32px] border border-slate-200 shadow-sm flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className="w-14 h-14 bg-primary-600 rounded-[22px] flex items-center justify-center text-white text-xl font-black shadow-lg shadow-primary-100">
                  {selectedEmployee.last_name[0]}
                </div>
                <div>
                  <h2 className="text-xl font-bold text-slate-900">{selectedEmployee.first_name} {selectedEmployee.last_name}</h2>
                  <div className="flex items-center gap-4 mt-1">
                    <button 
                      onClick={() => setActiveTab("payslips")}
                      className={`text-[10px] font-black uppercase tracking-widest transition-colors ${activeTab === 'payslips' ? 'text-primary-600' : 'text-slate-400 hover:text-slate-600'}`}
                    >
                      {t("payroll.history_title")}
                    </button>
                    <div className="w-1 h-1 bg-slate-200 rounded-full" />
                    <button 
                      onClick={() => setActiveTab("config")}
                      className={`text-[10px] font-black uppercase tracking-widest transition-colors ${activeTab === 'config' ? 'text-primary-600' : 'text-slate-400 hover:text-slate-600'}`}
                    >
                      {t("common.payroll.employee_config.tab_title")}
                    </button>
                  </div>
                </div>
              </div>
              <button 
                onClick={() => setIsCalcModalOpen(true)}
                className="bg-slate-900 text-white px-6 py-3 rounded-2xl font-bold flex items-center gap-2 hover:bg-black transition-all shadow-lg shadow-slate-200 active:scale-95"
              >
                <Calculator className="w-5 h-5" />
                {t("payroll.calculate_btn")}
              </button>
            </div>

            <div className="flex-1 overflow-y-auto">
              {activeTab === "payslips" ? (
                <div className="bg-white border border-slate-200 rounded-[40px] overflow-hidden shadow-sm flex flex-col h-full">
                  <div className="p-6 border-b border-slate-100 bg-slate-50/50 flex items-center gap-2">
                    <History className="w-4 h-4 text-slate-400" />
                    <h3 className="font-black text-[10px] text-slate-400 uppercase tracking-widest">{t("payroll.history_title")}</h3>
                  </div>
                  
                  <div className="flex-1 overflow-auto">
                    <PayrollTable 
                      payslips={payslips} 
                      isLoading={payslipsLoading} 
                      adjustments={employeeAdjustments}
                      onResolveAdjustments={() => setIsAdjustmentDrawerOpen(true)}
                    />
                  </div>
                </div>
              ) : (
                <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
                  <EmployeeSalaryConfig wsId={wsId} employeeId={selectedEmployee.id} />
                </div>
              )}
            </div>
          </>
        )}
      </div>

      {/* Calculate Payroll Modal */}
      <Modal
        isOpen={isCalcModalOpen && !!selectedEmployee}
        onClose={() => setIsCalcModalOpen(false)}
        title={t("payroll.modal.title")}
        subtitle={t("payroll.modal.subtitle")}
        icon={<Calculator className="w-6 h-6" />}
      >
        <div className="space-y-8">
          <div className="bg-slate-50 p-6 rounded-3xl border border-slate-100 flex items-center gap-4">
            <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center text-slate-400 shadow-sm border border-slate-100">
              <UserCircle className="w-7 h-7" />
            </div>
            <div>
              <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest leading-none mb-1">{t("common.employee")}</p>
              <p className="text-lg font-black text-slate-900 tracking-tight">{selectedEmployee?.first_name} {selectedEmployee?.last_name}</p>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.year")}</label>
              <input type="number" value={calcYear} onChange={e => setCalcYear(parseInt(e.target.value))} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-bold" />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.month")}</label>
              <input type="number" min="1" max="12" value={calcMonth} onChange={e => setCalcMonth(parseInt(e.target.value))} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 outline-none transition-all font-bold" />
            </div>
          </div>

          <div className="pt-4 grid grid-cols-2 gap-5">
            <button type="button" onClick={() => setIsCalcModalOpen(false)} className="px-8 py-4 border border-slate-100 rounded-2xl font-bold text-slate-500 hover:bg-slate-50 transition-all">{t("common.cancel")}</button>
            <button 
              onClick={handleCalculate}
              disabled={calcMutation.isPending}
              className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black hover:bg-slate-800 transition-all shadow-xl shadow-slate-200 flex items-center justify-center gap-2"
            >
              {calcMutation.isPending ? t("common.processing") : t("payroll.modal.submit")}
            </button>
          </div>
        </div>
      </Modal>

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
    </div>
  );
}


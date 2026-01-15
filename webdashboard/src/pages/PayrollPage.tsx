import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Banknote, 
  Calculator, 
  Calendar, 
  ChevronRight, 
  Download, 
  FileText, 
  History, 
  Plus, 
  Search, 
  UserCircle,
  X,
  CreditCard,
  CheckCircle2,
  AlertCircle
} from "lucide-react";
import { 
  fetchEmployees, 
  fetchEmployeePayslips, 
  calculatePayroll, 
  getCurrentWorkspaceId,
  Employee,
  Payslip
} from "../lib/api";
import { useToast } from "../components/Toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Combobox } from "@/components/ui/Combobox";
import Modal from "@/components/ui/Modal";

export default function PayrollPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatDate } = useDateTimeFormatter();
  
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [isCalcModalOpen, setIsCalcModalOpen] = useState(false);
  
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

  const calcMutation = useMutation({
    mutationFn: (data: { year: number; month: number }) => 
      calculatePayroll(wsId, selectedEmployee!.id, data.year, data.month),
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
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 w-5 h-5" />
          <input 
            type="text" 
            placeholder={t("payroll.search_placeholder")}
            className="w-full pl-12 pr-4 py-3.5 bg-white border border-slate-200 rounded-2xl outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-blue-500 transition-all"
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
                  <p className="text-[10px] font-black uppercase tracking-widest text-slate-400">{t("payroll.employee_profile")}</p>
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

            <div className="flex-1 bg-white border border-slate-200 rounded-[40px] overflow-hidden shadow-sm flex flex-col">
              <div className="p-6 border-b border-slate-100 bg-slate-50/50 flex items-center gap-2">
                <History className="w-4 h-4 text-slate-400" />
                <h3 className="font-black text-[10px] text-slate-400 uppercase tracking-widest">{t("payroll.history_title")}</h3>
              </div>
              
              <div className="flex-1 overflow-auto">
                <table className="w-full text-left">
                  <thead className="bg-slate-50/30 sticky top-0 backdrop-blur-md border-b border-slate-100">
                    <tr>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("payroll.table.period")}</th>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("payroll.table.base_pay")}</th>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("payroll.table.overtime")}</th>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("payroll.table.deductions")}</th>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("payroll.table.net_pay")}</th>
                      <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("payroll.table.status")}</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-50">
                    {payslipsLoading ? (
                      <tr><td colSpan={6} className="px-8 py-12 text-center text-slate-400 italic">{t("payroll.table.loading_payslips")}</td></tr>
                    ) : payslips.length === 0 ? (
                      <tr>
                        <td colSpan={6} className="px-8 py-20 text-center">
                          <div className="bg-slate-50 w-12 h-12 rounded-2xl flex items-center justify-center mx-auto mb-4">
                            <FileText className="text-slate-200 w-6 h-6" />
                          </div>
                          <p className="text-xs font-bold text-slate-300 uppercase tracking-widest">{t("payroll.table.no_payslips")}</p>
                        </td>
                      </tr>
                    ) : payslips.map(slip => (
                      <tr key={slip.id} className="hover:bg-slate-50/50 transition-colors group">
                        <td className="px-8 py-6">
                          <div className="font-bold text-slate-900">{t("common.month")} {new Date(slip.period_start).getMonth() + 1}/{new Date(slip.period_start).getFullYear()}</div>
                          <div className="text-[10px] text-slate-400 font-bold mt-1">
                            {formatDate(slip.period_start)} - {formatDate(slip.period_end)}
                          </div>
                        </td>
                        <td className="px-8 py-6 text-right font-mono text-sm text-slate-600">
                          {new Intl.NumberFormat('vi-VN').format(parseFloat(slip.base_pay_amount))}
                        </td>
                        <td className="px-8 py-6 text-right font-mono text-sm text-emerald-600 font-bold">
                          +{new Intl.NumberFormat('vi-VN').format(parseFloat(slip.overtime_pay_amount))}
                        </td>
                        <td className="px-8 py-6 text-right font-mono text-sm text-rose-600 font-bold">
                          -{new Intl.NumberFormat('vi-VN').format(parseFloat(slip.deductions_amount))}
                        </td>
                        <td className="px-8 py-6 text-right">
                          <div className="font-black text-primary-600 text-lg">
                            {new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount))}
                          </div>
                        </td>
                        <td className="px-8 py-6 text-center">
                          {slip.status === 'paid' ? (
                            <span className="inline-flex items-center gap-1 px-3 py-1 bg-emerald-100 text-emerald-700 rounded-full text-[10px] font-black uppercase tracking-tighter">
                              <CheckCircle2 size={10} /> {t("payroll.status.paid")}
                            </span>
                          ) : (
                            <span className="inline-flex items-center gap-1 px-3 py-1 bg-blue-100 text-blue-700 rounded-full text-[10px] font-black uppercase tracking-tighter">
                              <AlertCircle size={10} /> {t("payroll.status.finalized")}
                            </span>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
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


    </div>
  );
}

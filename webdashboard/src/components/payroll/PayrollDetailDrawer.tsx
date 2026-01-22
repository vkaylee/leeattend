import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import * as Dialog from "@radix-ui/react-dialog";
import { 
  X, 
  History, 
  Settings2, 
  UserCircle,
  Banknote,
  CalendarDays
} from "lucide-react";
import { Employee, Payslip, AttendanceAdjustmentRequest } from "@/lib/api/types";
import PayrollTable from "./PayrollTable";
import EmployeeSalaryConfig from "./EmployeeSalaryConfig";
import { cn } from "@/lib/utils";

interface PayrollDetailDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  employee: Employee | null;
  workspaceId: string;
  payslips: Payslip[];
  isLoadingPayslips: boolean;
  adjustments: AttendanceAdjustmentRequest[];
  onResolveAdjustments: () => void;
}

export default function PayrollDetailDrawer({
  isOpen,
  onClose,
  employee,
  workspaceId,
  payslips,
  isLoadingPayslips,
  adjustments,
  onResolveAdjustments
}: PayrollDetailDrawerProps) {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<"history" | "config">("history");

  if (!employee) return null;

  return (
    <Dialog.Root open={isOpen} onOpenChange={onClose}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-slate-900/20 backdrop-blur-lg z-50 animate-in fade-in duration-500" />
        <Dialog.Content className="fixed top-0 right-0 h-full w-full max-w-3xl bg-white shadow-premium-2xl z-50 animate-in slide-in-from-right duration-500 flex flex-col outline-none">
          {/* Header */}
          <header className="p-10 border-b border-slate-50 flex items-center justify-between bg-slate-50/30">
            <div className="flex items-center gap-6">
              <div className="w-20 h-20 bg-primary-600 rounded-[2rem] flex items-center justify-center text-white text-3xl font-black shadow-xl shadow-primary-100 rotate-3 hover:rotate-0 transition-transform duration-500">
                {employee.full_name[0]}
              </div>
              <div className="space-y-1">
                <Dialog.Title className="text-3xl font-black text-slate-900 tracking-tight uppercase">
                  {employee.full_name}
                </Dialog.Title>
                <div className="flex items-center gap-3">
                  <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest bg-white px-3 py-1.5 rounded-xl border border-slate-100 shadow-sm">
                    {employee.employee_id_number}
                  </span>
                  <div className="w-1.5 h-1.5 bg-primary-200 rounded-full" />
                  <span className="text-[10px] font-black text-primary-600 uppercase tracking-widest">
                    {employee.department_name || t("departments.not_assigned")}
                  </span>
                </div>
              </div>
            </div>
            <Dialog.Close asChild>
              <button className="w-12 h-12 rounded-2xl bg-white border border-slate-100 flex items-center justify-center text-slate-300 hover:text-rose-600 hover:border-rose-100 hover:shadow-md transition-all active:scale-90">
                <X size={24} />
              </button>
            </Dialog.Close>
          </header>

          {/* Premium Tab Switcher */}
          <nav className="px-10 py-6 bg-white border-b border-slate-50">
            <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">
              <button
                onClick={() => setActiveTab("history")}
                className={cn(
                  "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all",
                  activeTab === "history" ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"
                )}
              >
                <History size={14} /> {t("payroll.history_title")}
              </button>
              <button
                onClick={() => setActiveTab("config")}
                className={cn(
                  "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all",
                  activeTab === "config" ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"
                )}
              >
                <Settings2 size={14} /> {t("common.payroll.employee_config.tab_title")}
              </button>
            </div>
          </nav>

          {/* Content */}
          <main className="flex-1 overflow-y-auto p-10 bg-white custom-scrollbar scroll-smooth">
            {activeTab === "history" ? (
              <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-700">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                   <div className="bg-slate-50/50 p-8 rounded-[2.5rem] border border-slate-100 flex items-center gap-5 shadow-inner">
                      <div className="w-14 h-14 bg-white rounded-2xl flex items-center justify-center text-primary-600 shadow-sm border border-primary-50">
                        <Banknote size={24} />
                      </div>
                      <div>
                        <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] mb-1 opacity-70">{t("common.payroll.table.net_pay")} (Latest)</p>
                        <p className="text-2xl font-black text-slate-900 font-mono tracking-tighter">
                          {payslips.length > 0 
                            ? new Intl.NumberFormat('vi-VN').format(parseFloat(payslips[0].net_pay_amount))
                            : "—"}
                        </p>
                      </div>
                   </div>
                   <div className="bg-slate-50/50 p-8 rounded-[2.5rem] border border-slate-100 flex items-center gap-5 shadow-inner">
                      <div className="w-14 h-14 bg-white rounded-2xl flex items-center justify-center text-emerald-600 shadow-sm border border-emerald-50">
                        <CalendarDays size={24} />
                      </div>
                      <div>
                        <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] mb-1 opacity-70">{t("employees.hire_date")}</p>
                        <p className="text-2xl font-black text-slate-900 font-mono tracking-tighter">{employee.hire_date}</p>
                      </div>
                   </div>
                </div>

                <div className="space-y-6">
                  <div className="flex items-center gap-3 px-2">
                    <div className="w-1.5 h-6 bg-primary-600 rounded-full" />
                    <span className="text-xs font-black text-slate-800 uppercase tracking-[0.25em]">{t("payroll.history_title")} Detail</span>
                  </div>
                  <PayrollTable 
                    payslips={payslips} 
                    isLoading={isLoadingPayslips} 
                    adjustments={adjustments}
                    onResolveAdjustments={onResolveAdjustments}
                  />
                </div>
              </div>
            ) : (
              <div className="animate-in fade-in slide-in-from-bottom-4 duration-700">
                <EmployeeSalaryConfig wsId={workspaceId} employeeId={employee.id} />
              </div>
            )}
          </main>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

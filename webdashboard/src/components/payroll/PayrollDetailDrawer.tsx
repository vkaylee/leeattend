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
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

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
        <Dialog.Overlay className="fixed inset-0 bg-slate-900/40 backdrop-blur-sm z-50 animate-in fade-in duration-300" />
        <Dialog.Content className="fixed top-0 right-0 h-full w-full max-w-3xl bg-white shadow-2xl z-50 animate-in slide-in-from-right duration-500 flex flex-col outline-none">
          {/* Header */}
          <header className="p-8 border-b border-slate-100 flex items-center justify-between bg-slate-50/30">
            <div className="flex items-center gap-5">
              <div className="w-16 h-16 bg-primary-600 rounded-[24px] flex items-center justify-center text-white text-2xl font-black shadow-lg shadow-primary-100">
                {employee.last_name[0]}
              </div>
              <div>
                <Dialog.Title className="text-2xl font-bold text-slate-900 tracking-tight">
                  {employee.first_name} {employee.last_name}
                </Dialog.Title>
                <div className="flex items-center gap-3 mt-1.5">
                  <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest bg-white px-2 py-1 rounded-lg border border-slate-100">
                    {employee.employee_id_number}
                  </span>
                  <div className="w-1 h-1 bg-slate-200 rounded-full" />
                  <span className="text-[10px] font-black text-primary-600 uppercase tracking-widest">
                    {employee.department_name || t("departments.not_assigned")}
                  </span>
                </div>
              </div>
            </div>
            <Dialog.Close asChild>
              <button className="w-12 h-12 rounded-2xl bg-white border border-slate-100 flex items-center justify-center text-slate-400 hover:text-slate-600 hover:shadow-md transition-all">
                <X size={20} />
              </button>
            </Dialog.Close>
          </header>

          {/* Tab Navigation */}
          <nav className="flex px-8 py-4 bg-slate-50/30 border-b border-slate-100 gap-8">
            <button
              onClick={() => setActiveTab("history")}
              className={cn(
                "flex items-center gap-2 py-2 text-sm font-bold transition-all relative",
                activeTab === "history" ? "text-primary-600" : "text-slate-400 hover:text-slate-600"
              )}
            >
              <History size={16} />
              {t("payroll.history_title")}
              {activeTab === "history" && (
                <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 rounded-full animate-in fade-in zoom-in duration-300" />
              )}
            </button>
            <button
              onClick={() => setActiveTab("config")}
              className={cn(
                "flex items-center gap-2 py-2 text-sm font-bold transition-all relative",
                activeTab === "config" ? "text-primary-600" : "text-slate-400 hover:text-slate-600"
              )}
            >
              <Settings2 size={16} />
              {t("common.payroll.employee_config.tab_title")}
              {activeTab === "config" && (
                <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary-600 rounded-full animate-in fade-in zoom-in duration-300" />
              )}
            </button>
          </nav>

          {/* Content */}
          <main className="flex-1 overflow-y-auto p-8 bg-white custom-scrollbar">
            {activeTab === "history" ? (
              <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                   <div className="bg-slate-50 p-6 rounded-3xl border border-slate-100 flex items-center gap-4">
                      <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center text-primary-600 shadow-sm">
                        <Banknote size={20} />
                      </div>
                      <div>
                        <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.payroll.table.net_pay")} (Lần cuối)</p>
                        <p className="text-xl font-black text-slate-900">
                          {payslips.length > 0 
                            ? new Intl.NumberFormat('vi-VN').format(parseFloat(payslips[0].net_pay_amount))
                            : "—"}
                        </p>
                      </div>
                   </div>
                   <div className="bg-slate-50 p-6 rounded-3xl border border-slate-100 flex items-center gap-4">
                      <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center text-emerald-600 shadow-sm">
                        <CalendarDays size={20} />
                      </div>
                      <div>
                        <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("employees.hire_date")}</p>
                        <p className="text-xl font-black text-slate-900">{employee.hire_date}</p>
                      </div>
                   </div>
                </div>

                <div className="bg-white border border-slate-100 rounded-[32px] overflow-hidden shadow-sm">
                  <div className="px-6 py-4 bg-slate-50/50 border-b border-slate-100 flex items-center gap-2">
                    <History size={14} className="text-slate-400" />
                    <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest">Chi tiết phiếu lương</span>
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
              <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
                <EmployeeSalaryConfig wsId={workspaceId} employeeId={employee.id} />
              </div>
            )}
          </main>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

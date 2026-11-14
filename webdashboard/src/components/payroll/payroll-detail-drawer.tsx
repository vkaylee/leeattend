/* eslint-disable complexity */
import * as Dialog from "@radix-ui/react-dialog";
import { useQuery } from "@tanstack/react-query";
import { 
  X, 
  History, 
  Settings2, 
  Banknote,
  CalendarDays
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useOrganization } from "@/hooks/use-organization";
import { useMyPermissions } from "@/hooks/use-permissions";
import { fetchEmployeeEmployments } from "@/lib/api";
import { Employee, Payslip, AttendanceAdjustmentRequest } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import { maybeNull } from "@/types/api-overrides";

import EmployeeSalaryConfig from "./employee-salary-config";
import PayrollTable from "./payroll-table";
import { CalculationTrace } from "./calculation-trace";

interface PayrollDetailDrawerProps {
  adjustments: AttendanceAdjustmentRequest[];
  employee: Employee | null;
  isLoadingPayslips: boolean;
  isOpen: boolean;
  onClose: () => void;
  onResolveAdjustments: () => void;
  payslips: Payslip[];
}

function PayrollDetailHeader({ employee }: { employee: Employee; }) {
  const { t } = useTranslation();
  return (
    <header className="p-10 border-b border-slate-50 dark:border-slate-800 flex items-center justify-between bg-slate-50 dark:bg-slate-800/30">
      <div className="flex items-center gap-6">
        <div className="w-20 h-20 bg-primary-600 rounded-[2rem] flex items-center justify-center text-white text-3xl font-bold shadow-xl shadow-primary-100 dark:shadow-black/30 rotate-3 hover:rotate-0 transition-transform duration-500">
          {employee.full_name[0]}
        </div>
        <div className="space-y-1">
          <Dialog.Title className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tight uppercase">
            {employee.full_name}
          </Dialog.Title>
          <Dialog.Description className="sr-only">
            {`Salary profile and payroll history for ${employee.full_name}`}
          </Dialog.Description>
          <div className="flex items-center gap-3">
            <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest bg-white dark:bg-slate-900 px-3 py-1.5 rounded-xl border border-slate-100 dark:border-slate-800 shadow-sm">
              {employee.employee_id_number}
            </span>
            <div className="w-1.5 h-1.5 bg-primary-200 rounded-full" />
            <span className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest">
              {employee.department_name ?? t("departments.not_assigned")}
            </span>
          </div>
        </div>
      </div>
      <Dialog.Close asChild>
        <Button className="w-12 h-12 rounded-2xl bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 hover:border-rose-100 dark:border-rose-800 hover:shadow-md transition-all active:scale-90" variant="ghost">
          <X size={24} />
        </Button>
      </Dialog.Close>
    </header>
  );
}

function PayrollDetailSummary({ payslips, canReadSensitive, hireDate }: { employee: Employee; payslips: Payslip[]; canReadSensitive: boolean; hireDate: string; }) {
  const { t } = useTranslation();
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
       <div className="bg-slate-50 dark:bg-slate-800/50 p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 flex items-center gap-5 shadow-inner">
          <div className="w-14 h-14 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-50">
            <Banknote size={24} />
          </div>
          <div>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mb-1 opacity-70" variant="p">{t("common.payroll.table.net_pay")} (Latest)</Typography>
            <Typography className="text-2xl font-bold text-slate-700 dark:text-slate-200 font-mono tracking-tighter" variant="p">
              {!canReadSensitive ? "***" : (payslips.length > 0 
                ? new Intl.NumberFormat('vi-VN').format(parseFloat(payslips[0].net_pay_amount))
                : "—")}
            </Typography>
          </div>
       </div>
       <div className="bg-slate-50 dark:bg-slate-800/50 p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 flex items-center gap-5 shadow-inner">
          <div className="w-14 h-14 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center text-emerald-600 dark:text-emerald-300 shadow-sm border border-emerald-50">
            <CalendarDays size={24} />
          </div>
          <div>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mb-1 opacity-70" variant="p">{t("employees.hire_date")}</Typography>
            <Typography className="text-2xl font-bold text-slate-700 dark:text-slate-200 font-mono tracking-tighter" variant="p">{hireDate}</Typography>
          </div>
       </div>
    </div>
  );
}

export default function PayrollDetailDrawer({
  isOpen,
  onClose,
  employee,payslips,
  isLoadingPayslips,
  adjustments,
  onResolveAdjustments
}: PayrollDetailDrawerProps) {
  const { organization } = useOrganization();
  const { t } = useTranslation();
  const { hasPermission } = useMyPermissions();
  const canReadSensitive = hasPermission("payroll:sensitive:read");
  const [activeTab, setActiveTab] = useState<"history" | "config">("history");

  const { data: employments = [] } = useQuery({ queryKey: ["employments", employee?.id], queryFn: () => employee ? fetchEmployeeEmployments(employee.id) : Promise.resolve([]),
  enabled: isOpen && !!employee , });

  if (!employee) return null;

  const activeEmployment = employments.find(emp => emp.id === employee.active_employment_id) ?? employments[0];
  const hireDate = maybeNull(activeEmployment)?.start_date ?? "—";
  const latestPayslip = payslips[0];
  const tracePeriod = latestPayslip ? new Date(latestPayslip.period_start) : null;

  return (
    <Dialog.Root onOpenChange={onClose} open={isOpen}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-slate-900/20 backdrop-blur-lg z-50 animate-in fade-in duration-500" />
        <Dialog.Content
          className="fixed top-0 right-0 h-full w-full max-w-3xl bg-white dark:bg-slate-900 shadow-premium-2xl z-50 animate-in slide-in-from-right duration-500 flex min-h-0 flex-col outline-none"
        >
          {/* Header */}
          <PayrollDetailHeader employee={employee} />

          {/* Premium Tab Switcher */}
          <nav className="px-10 py-6 bg-white dark:bg-slate-900 border-b border-slate-50 dark:border-slate-800">
            <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
              <Button className={cn(
                  "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
                  activeTab === "history" ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700"
                )}
                onClick={() => { setActiveTab("history"); }}
                variant="ghost"
              >
                <History size={14} /> {t("payroll.history_title")}
              </Button>
              <Button className={cn(
                  "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
                  activeTab === "config" ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700"
                )}
                onClick={() => { setActiveTab("config"); }}
                variant="ghost"
              >
                <Settings2 size={14} /> {t("common.payroll.employee_config.tab_title")}
              </Button>
            </div>
          </nav>

          {/* Content */}
          <main className="min-h-0 flex-1 overflow-y-auto p-10 bg-white dark:bg-slate-900 custom-scrollbar scroll-smooth">
            {activeTab === "history" ? (
              <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-700">
                <PayrollDetailSummary 
                  canReadSensitive={canReadSensitive}
                  employee={employee}
                  hireDate={hireDate}
                  payslips={payslips}
                />

                <div className="space-y-6">
                  <div className="flex items-center gap-3 px-2">
                    <div className="w-1.5 h-6 bg-primary-600 rounded-full" />
                    <span className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-[0.25em]">{t("payroll.history_title")} Detail</span>
                  </div>
                  <PayrollTable
                    adjustments={adjustments}
                    employee={employee}
                    isLoading={isLoadingPayslips}
                    onResolveAdjustments={onResolveAdjustments}
                    organization={organization}
                    payslips={payslips}
                  />
                  {activeEmployment && tracePeriod && !Number.isNaN(tracePeriod.getTime()) ? (
                    <CalculationTrace
                      employmentId={activeEmployment.id}
                      month={tracePeriod.getUTCMonth() + 1}
                      year={tracePeriod.getUTCFullYear()}
                    />
                  ) : null}
                </div>
              </div>
            ) : (
              <div className="animate-in fade-in slide-in-from-bottom-4 duration-700">
                <EmployeeSalaryConfig employeeId={employee.id} employmentId={maybeNull(activeEmployment)?.id ?? ""}  />
              </div>
            )}
          </main>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

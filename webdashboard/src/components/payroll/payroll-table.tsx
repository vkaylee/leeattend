import * as Popover from '@radix-ui/react-popover';
import { 
  ChevronDown, 
  History,
  Info,
  Banknote,
  Target,
  Clock,
  Briefcase,
  Calculator,
  ArrowDownRight,
  Calendar,
  AlertTriangle,
  Sparkles, FileText 
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMyPermissions } from "@/hooks/use-permissions";
import { Employee, Workspace, Payslip, AttendanceAdjustmentRequest, PayrollSegment  } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { PayrollPriorityLegend } from "./payroll-priority-legend";
import PayslipModal from "./payslip-modal";

interface PayrollTableProps {
  adjustments: AttendanceAdjustmentRequest[];
  employee: Employee | null;
  isLoading: boolean;
  onResolveAdjustments: () => void;
  payslips: Payslip[];
  workspace: Workspace | null;
}

function SegmentCard({ segment, index, canReadSensitive }: { segment: PayrollSegment, index: number, isLast: boolean, canReadSensitive: boolean }) {
  const { t } = useTranslation();
  const { formatDate } = useDateTimeFormatter();

  return (
    <div className="relative pl-16">
      <div className={cn("absolute left-5 top-2 w-4 h-4 rounded-full border-[3px] border-white z-10 shadow-md ring-4", index === 0 ? "bg-primary-600 ring-primary-50" : "bg-slate-200 dark:bg-slate-600 ring-slate-50")} />

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] p-8 hover:shadow-2xl hover:shadow-primary-100/10 transition-all group/card relative">
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 mb-8">
          <div className="flex items-center gap-4">
             <div className="w-10 h-10 bg-slate-50 dark:bg-slate-800 rounded-xl flex items-center justify-center text-slate-300 dark:text-slate-600 group-hover/card:text-primary-600 dark:text-primary-400 group-hover/card:bg-primary-50 dark:bg-primary-950 transition-all border border-slate-100 dark:border-slate-800 shadow-inner">
               <ArrowDownRight size={18} />
             </div>
             <div>
               <p className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] leading-none mb-1.5">{t("common.segment")} {index + 1}</p>
               <p className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">{formatDate(segment.start_date)} — {formatDate(segment.end_date)}</p>
             </div>
          </div>
          <span className={cn("text-[9px] font-bold uppercase tracking-[0.2em] px-3 py-1 rounded-lg border shadow-sm", segment.reason === 'Standard' ? "bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800" : "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800")}>{segment.reason || "Proration"}</span>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
          <div className="space-y-6">
            <div className="flex items-center gap-3 mb-2 px-1 opacity-60"><Target className="text-slate-500 dark:text-slate-400" size={14} /><span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">Configuration Applied</span></div>
            <div className="grid grid-cols-2 gap-6">
              <div className="bg-slate-50 dark:bg-slate-800/50 p-5 rounded-[1.5rem] border border-slate-100 dark:border-slate-800 shadow-inner">
                <p className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 opacity-70">Base Salary</p>
                <div className="flex items-center gap-2 text-slate-700 dark:text-slate-200 font-bold font-mono text-sm tracking-tighter">
                  <Banknote className="text-primary-500 opacity-50" size={14} /> {!canReadSensitive ? "***" : new Intl.NumberFormat('vi-VN').format(segment.applied_base_salary)}
                </div>
              </div>
              <div className="bg-slate-50 dark:bg-slate-800/50 p-5 rounded-[1.5rem] border border-slate-100 dark:border-slate-800 shadow-inner">
                <p className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 opacity-70">Std. Period</p>
                <div className="flex items-center gap-3">
                  <span className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tighter">{segment.applied_std_days} Days</span>
                  {!!segment.std_days_source && (
                    <Popover.Root>
                      <Popover.Trigger asChild><button className="p-1.5 rounded-lg bg-white dark:bg-slate-900 shadow-sm border border-slate-100 dark:border-slate-800 text-slate-300 dark:text-slate-600 hover:text-primary-600 dark:text-primary-400 transition-all"><Info size={12} /></button></Popover.Trigger>
                      <Popover.Portal><Popover.Content align="center" className="z-50 w-80 bg-transparent animate-in zoom-in-95 duration-300 origin-bottom" side="top" sideOffset={12}><PayrollPriorityLegend activeSource={segment.std_days_source} /></Popover.Content></Popover.Portal>
                    </Popover.Root>
                  )}
                </div>
              </div>
            </div>
            <div className="flex items-center gap-4 px-2">
              <div className="flex items-center gap-2"><Clock className="text-slate-300 dark:text-slate-600" size={12} /><span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{segment.applied_hours_per_day}h / day</span></div>
              <div className="w-1 h-1 bg-slate-200 dark:bg-slate-600 rounded-full" />
              <div className="flex items-center gap-2"><Briefcase className="text-slate-300 dark:text-slate-600" size={12} /><span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest italic opacity-60">Dept: Global</span></div>
            </div>
          </div>

          <div className="space-y-6">
            <div className="flex items-center gap-3 mb-2 px-1 opacity-60"><Calculator className="text-slate-500 dark:text-slate-400" size={14} /><span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">Calculation Outcome</span></div>
            <div className="bg-primary-600 rounded-[1.5rem] p-6 text-white shadow-xl shadow-primary-100 dark:shadow-black/30 relative overflow-hidden group/out">
              <div className="flex items-center justify-between relative z-10">
                <div>
                  <p className="text-[9px] font-bold text-primary-200 uppercase tracking-widest mb-1 opacity-80">Paid Days</p>
                  <p className="text-xl font-bold tracking-tighter">{segment.paid_work_days} <span className="text-xs text-primary-300 font-bold">/ {segment.actual_work_days}</span></p>
                </div>
                <div className="h-10 w-px bg-white dark:bg-slate-900/10" />
                <div className="text-right">
                  <p className="text-[9px] font-bold text-primary-200 uppercase tracking-widest mb-1 opacity-80">Segment Net</p>
                  <p className="text-xl font-bold font-mono tracking-tighter">{!canReadSensitive ? "***" : new Intl.NumberFormat('vi-VN').format(segment.amount)}</p>
                </div>
              </div>
              <Calculator className="absolute -right-4 -bottom-4 w-20 h-20 text-white opacity-5 -rotate-12 transition-transform group-hover/out:rotate-0 duration-700" />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function PayslipCardHeader({ slip, isExpanded, setExpandedId, t, formatMonthYear, canReadSensitive, setSelectedPayslip, setIsModalOpen }: any) {
  return (
    <div 
      className={cn("p-8 flex items-center justify-between cursor-pointer transition-colors", isExpanded ? "bg-primary-50 dark:bg-primary-950/30 dark:bg-primary-950/30" : "hover:bg-slate-50 dark:hover:bg-slate-800/50 dark:hover:bg-slate-800/50")} 
      data-testid={`toggle-card-${slip.id}`} 
      onClick={() => { setExpandedId(isExpanded ? null : slip.id); }}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); setExpandedId(isExpanded ? null : slip.id); } }}
      role="button"
      tabIndex={0}
    >
      <div className="flex items-center gap-8">
        <div className={cn("w-14 h-14 rounded-2xl flex items-center justify-center transition-all shadow-inner border", isExpanded ? "bg-primary-600 text-white border-primary-500" : "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 group-hover:bg-white dark:group-hover:bg-slate-900 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800")}>
          <Calendar size={24} />
        </div>
        <div>
          <p className="text-lg font-bold text-slate-700 dark:text-slate-200 tracking-tight leading-none mb-2 uppercase">{t("common.month")} {formatMonthYear(slip.period_start)}</p>
          <div className="flex items-center gap-3">
            <span className={cn("text-[9px] font-bold uppercase tracking-widest px-2.5 py-1 rounded-full border shadow-sm", slip.status === 'paid' ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800")}>{slip.status === 'paid' ? t("common.payroll.status.paid") : t("common.payroll.status.finalized")}</span>
            <div className="w-1 h-1 bg-slate-200 dark:bg-slate-600 rounded-full" />
            <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-70">{slip.standard_units} Units • {slip.overtime_units} OT</span>
          </div>
        </div>
      </div>

      <button 
        className="hidden md:flex items-center gap-2 px-4 py-2 bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 rounded-xl border border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900 hover:text-primary-600 dark:text-primary-400 hover:border-primary-200 dark:border-primary-800 hover:shadow-md transition-all active:scale-95 group/btn" 
        onClick={(e) => { 
          e.stopPropagation(); 
          setSelectedPayslip(slip); 
          setIsModalOpen(true); 
        }}
      >
        <FileText className="text-slate-300 dark:text-slate-600 group-hover/btn:text-primary-500 transition-colors" size={14} />
        <span className="text-[9px] font-bold uppercase tracking-widest">Official Payslip</span>
      </button>
      <div className="flex items-center gap-10 pr-2">
        <div className="text-right">
          <p className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest leading-none mb-1.5 opacity-60">{t("common.payroll.table.net_pay")}</p>
          <p className="text-2xl font-bold text-primary-600 dark:text-primary-400 tracking-tighter font-mono">{!canReadSensitive ? "***" : new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount))}</p>
        </div>
        <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center border transition-all shadow-sm", isExpanded ? "bg-white dark:bg-slate-900 border-primary-200 dark:border-primary-800 text-primary-600 dark:text-primary-400 rotate-180" : "bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-800 text-slate-300 dark:text-slate-600 group-hover:text-primary-400")}>
          <ChevronDown size={20} />
        </div>
      </div>
    </div>
  );
}

function PayslipCardBody({ slip, slipAdjustments, onResolveAdjustments, canReadSensitive, t }: any) {
  return (
    <div className="px-8 pb-10 animate-in fade-in slide-in-from-top-4 duration-500">
      <div className="h-px bg-gradient-to-r from-transparent via-slate-100 to-transparent mb-10" />
      
      {slipAdjustments.length > 0 && (
        <div className="px-10 py-6 bg-amber-50 dark:bg-amber-950 border-y border-amber-100 dark:border-amber-800/50 flex items-center justify-between animate-in slide-in-from-top-4 duration-500 mb-8 rounded-2xl" data-testid={`pending-adj-banner-${slip.id}`}>
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center text-amber-500 shadow-sm border border-amber-100 dark:border-amber-800"><AlertTriangle size={24} /></div>
            <div>
              <p className="text-sm font-bold text-amber-900 uppercase tracking-tight">{t("payroll.traceability.pending_adjustments")}</p>
              <p className="text-[10px] text-amber-700 dark:text-amber-300 font-bold uppercase tracking-widest opacity-70 mt-0.5">{t("payroll.traceability.recalc_needed")}</p>
            </div>
          </div>
          <button className="px-6 py-2.5 bg-amber-50 dark:bg-amber-950 text-white rounded-xl text-[10px] font-bold uppercase tracking-widest hover:bg-amber-600 transition-all shadow-lg shadow-amber-200 active:scale-95" onClick={onResolveAdjustments}>{t("payroll.traceability.view_adjustments")}</button>
        </div>
      )}

      <div className="space-y-8 relative">
        <div className="absolute left-[27px] top-6 bottom-6 w-0.5 bg-slate-100 dark:bg-slate-700" />
        {slip.segments?.map((seg: any, index: number) => (
          <SegmentCard 
            canReadSensitive={canReadSensitive} 
            index={index} 
            isLast={index === (slip.segments?.length ?? 0) - 1} 
            key={index} 
            segment={seg}
          />
        ))}
      </div>

      {!!slip.incentives && slip.incentives.length > 0 && (
        <div className="mt-12 space-y-6">
          <div className="flex items-center gap-3 px-2">
            <div className="w-1.5 h-6 bg-blue-600 rounded-full" />
            <span className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-[0.25em]">{t("common.payroll.table.bonus")} Detail</span>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {slip.incentives.map((inc: any, index: number) => (
              <div className="flex items-center justify-between p-6 bg-blue-50 dark:bg-blue-950/30 border border-blue-100 dark:border-blue-800/50 rounded-2xl" key={index}>
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 rounded-xl bg-white dark:bg-slate-900 flex items-center justify-center text-blue-600 dark:text-blue-300 shadow-sm border border-blue-100 dark:border-blue-800">
                    <Sparkles size={18} />
                  </div>
                  <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">{inc.policy_name}</span>
                </div>
                <span className="text-sm font-bold text-blue-600 dark:text-blue-300 font-mono tracking-tighter">{!canReadSensitive ? "***" : `+${new Intl.NumberFormat('vi-VN').format(parseFloat(inc.amount))}`}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="mt-12 grid grid-cols-1 md:grid-cols-4 gap-6 pt-10 border-t border-slate-50 dark:border-slate-800">
        {[
          { label: t("common.payroll.table.base_pay"), value: slip.base_pay_amount, color: "text-slate-700 dark:text-slate-200" },
          { label: t("common.payroll.table.overtime"), value: slip.overtime_pay_amount, color: "text-emerald-600 dark:text-emerald-300", prefix: "+" },
          { 
            label: t("common.payroll.table.bonus"), 
            value: slip.incentives?.reduce((accumulator: number, index: any) => accumulator + parseFloat(index.amount), 0).toString() ?? "0", 
            color: "text-blue-600 dark:text-blue-300", 
            prefix: "+" 
          },
          { label: "Deductions", value: slip.deductions_amount, color: "text-rose-600 dark:text-rose-300", prefix: "-" }
        ].map((stat, index) => (
          <div className="p-6 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[2rem] shadow-inner flex flex-col gap-1" key={index}>
             <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">{stat.label}</p>
             <p className={cn("text-xl font-bold font-mono tracking-tighter", stat.color)}>{!canReadSensitive ? "***" : `${stat.prefix ?? ''}${new Intl.NumberFormat('vi-VN').format(parseFloat(stat.value))}`}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

function PayslipCard({ slip, isExpanded, setExpandedId, slipAdjustments, onResolveAdjustments, t, formatMonthYear, canReadSensitive, setSelectedPayslip, setIsModalOpen }: any) {
  return (
    <div className={cn("group bg-white dark:bg-slate-900 border transition-all duration-500 overflow-hidden", isExpanded ? "border-primary-200 dark:border-primary-800 rounded-[2.5rem] shadow-2xl shadow-primary-100/20 dark:shadow-black/20" : "border-slate-100 dark:border-slate-800 hover:border-primary-100 dark:border-primary-800 rounded-[2rem] shadow-sm dark:shadow-black/10 hover:shadow-lg hover:shadow-primary-100/10")} data-testid={`payslip-card-${slip.id}`}>
      <PayslipCardHeader canReadSensitive={canReadSensitive} formatMonthYear={formatMonthYear} isExpanded={isExpanded} setExpandedId={setExpandedId} setIsModalOpen={setIsModalOpen} setSelectedPayslip={setSelectedPayslip} slip={slip} t={t} />

      {!!isExpanded && (
        <PayslipCardBody canReadSensitive={canReadSensitive} onResolveAdjustments={onResolveAdjustments} slip={slip} slipAdjustments={slipAdjustments} t={t} />
      )}
    </div>
  );
}

export default function PayrollTable({
  employee,
  workspace, 
  payslips, 
  isLoading, 
  adjustments,
  onResolveAdjustments
}: PayrollTableProps) {
  const [selectedPayslip, setSelectedPayslip] = useState<Payslip | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const { t } = useTranslation();
  const { formatMonthYear } = useDateTimeFormatter();
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const { hasPermission } = useMyPermissions();
  const canReadSensitive = hasPermission("payroll:sensitive:read");

  if (isLoading) return <div className="p-20 text-center text-slate-500 dark:text-slate-400 font-bold italic animate-pulse uppercase tracking-widest text-xs opacity-50">{t("common.loading")}</div>;

  if (payslips.length === 0) {
    return (
      <div className="py-24 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[2.5rem] bg-slate-50 dark:bg-slate-800/30 dark:bg-slate-800/30 flex flex-col items-center">
        <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm dark:shadow-black/10 mb-6 border border-slate-100 dark:border-slate-800">
          <History className="text-slate-200 dark:text-slate-600" size={32} />
        </div>
        <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">{t("common.payroll.table.no_payslips")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 animate-in fade-in duration-700">
      {payslips.map((slip) => {
        const isExpanded = expandedId === slip.id;
        const slipAdjustments = adjustments.filter(adj => 
          adj.employee_id === slip.employee_id && 
          new Date(adj.requested_date) >= new Date(slip.period_start) &&
          new Date(adj.requested_date) <= new Date(slip.period_end)
        );

        return (
          <PayslipCard
            canReadSensitive={canReadSensitive}
            formatMonthYear={formatMonthYear}
            isExpanded={isExpanded}
            key={slip.id}
            onResolveAdjustments={onResolveAdjustments}
            setExpandedId={setExpandedId}
            setIsModalOpen={setIsModalOpen}
            setSelectedPayslip={setSelectedPayslip}
            slip={slip}
            slipAdjustments={slipAdjustments}
            t={t}
          />
        );
      })}

      {!!employee && !!selectedPayslip && (
        <PayslipModal
          employee={employee}
          isOpen={isModalOpen}
          onClose={() => {
            setIsModalOpen(false);
            setSelectedPayslip(null);
          }}
          payslip={selectedPayslip}
          workspace={workspace}
        />
      )}
    </div>
  );
}
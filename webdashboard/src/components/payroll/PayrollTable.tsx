import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import * as Popover from '@radix-ui/react-popover';
import { 
  CheckCircle2, 
  AlertCircle, 
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
  Sparkles
} from "lucide-react";
import { Payslip, AttendanceAdjustmentRequest, PayrollSegment } from "@/lib/api/types";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { PayrollPriorityLegend } from "./PayrollPriorityLegend";
import { cn } from "@/lib/utils";

interface PayrollTableProps {
  payslips: Payslip[];
  isLoading: boolean;
  adjustments: AttendanceAdjustmentRequest[];
  onResolveAdjustments: () => void;
}

export default function PayrollTable({ 
  payslips, 
  isLoading, 
  adjustments,
  onResolveAdjustments
}: PayrollTableProps) {
  const { t } = useTranslation();
  const { formatMonthYear } = useDateTimeFormatter();
  const [expandedId, setExpandedId] = useState<string | null>(null);

  if (isLoading) return <div className="p-20 text-center text-slate-400 font-bold italic animate-pulse uppercase tracking-widest text-xs opacity-50">{t("common.loading")}</div>;

  if (payslips.length === 0) {
    return (
      <div className="py-24 text-center border-2 border-dashed border-slate-100 rounded-[2.5rem] bg-slate-50/30 flex flex-col items-center">
        <div className="w-20 h-20 bg-white rounded-full flex items-center justify-center shadow-sm mb-6 border border-slate-100">
          <History size={32} className="text-slate-200" />
        </div>
        <p className="text-[10px] font-black text-slate-300 uppercase tracking-widest">{t("common.payroll.table.no_payslips")}</p>
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
          <div key={slip.id} data-testid={`payslip-card-${slip.id}`} className={cn("group bg-white border transition-all duration-500 overflow-hidden", isExpanded ? "border-primary-200 rounded-[2.5rem] shadow-2xl shadow-primary-100/20" : "border-slate-100 hover:border-primary-100 rounded-[2rem] shadow-sm hover:shadow-lg hover:shadow-primary-100/10")}>
            <div data-testid={`toggle-card-${slip.id}`} className={cn("p-8 flex items-center justify-between cursor-pointer transition-colors", isExpanded ? "bg-primary-50/30" : "hover:bg-slate-50/50")} onClick={() => setExpandedId(isExpanded ? null : slip.id)}>
              <div className="flex items-center gap-8">
                <div className={cn("w-14 h-14 rounded-2xl flex items-center justify-center transition-all shadow-inner border", isExpanded ? "bg-primary-600 text-white border-primary-500" : "bg-slate-50 text-slate-400 border-slate-100 group-hover:bg-white group-hover:text-primary-600 group-hover:border-primary-100")}>
                  <Calendar size={24} />
                </div>
                <div>
                  <p className="text-lg font-black text-slate-900 tracking-tight leading-none mb-2 uppercase">{t("common.month")} {formatMonthYear(slip.period_start)}</p>
                  <div className="flex items-center gap-3">
                    <span className={cn("text-[9px] font-black uppercase tracking-widest px-2.5 py-1 rounded-full border shadow-sm", slip.status === 'paid' ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-blue-50 text-blue-600 border-blue-100")}>{slip.status === 'paid' ? t("common.payroll.status.paid") : t("common.payroll.status.finalized")}</span>
                    <div className="w-1 h-1 bg-slate-200 rounded-full" />
                    <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest opacity-70">{slip.standard_units} Units • {slip.overtime_units} OT</span>
                  </div>
                </div>
              </div>

              <div className="flex items-center gap-10 pr-2">
                <div className="text-right">
                  <p className="text-[9px] font-black text-slate-400 uppercase tracking-widest leading-none mb-1.5 opacity-60">{t("common.payroll.table.net_pay")}</p>
                  <p className="text-2xl font-black text-primary-600 tracking-tighter font-mono">{new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount))}</p>
                </div>
                <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center border transition-all shadow-sm", isExpanded ? "bg-white border-primary-200 text-primary-600 rotate-180" : "bg-white border-slate-100 text-slate-300 group-hover:text-primary-400")}>
                  <ChevronDown size={20} />
                </div>
              </div>
            </div>

            {isExpanded && (
              <div className="px-8 pb-10 animate-in fade-in slide-in-from-top-4 duration-500">
                <div className="h-px bg-gradient-to-r from-transparent via-slate-100 to-transparent mb-10" />
                
                {slipAdjustments.length > 0 && (
                  <div data-testid={`pending-adj-banner-${slip.id}`} className="px-10 py-6 bg-amber-50 border-y border-amber-100/50 flex items-center justify-between animate-in slide-in-from-top-4 duration-500 mb-8 rounded-2xl">
                    <div className="flex items-center gap-4">
                      <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center text-amber-500 shadow-sm border border-amber-100"><AlertTriangle size={24} /></div>
                      <div>
                        <p className="text-sm font-black text-amber-900 uppercase tracking-tight">{t("payroll.traceability.pending_adjustments")}</p>
                        <p className="text-[10px] text-amber-700 font-black uppercase tracking-widest opacity-70 mt-0.5">{t("payroll.traceability.recalc_needed")}</p>
                      </div>
                    </div>
                    <button onClick={onResolveAdjustments} className="px-6 py-2.5 bg-amber-500 text-white rounded-xl text-[10px] font-black uppercase tracking-widest hover:bg-amber-600 transition-all shadow-lg shadow-amber-200 active:scale-95">{t("payroll.traceability.view_adjustments")}</button>
                  </div>
                )}

                <div className="space-y-8 relative">
                  <div className="absolute left-[27px] top-6 bottom-6 w-0.5 bg-slate-100" />
                  {slip.segments?.map((seg, idx) => (
                    <SegmentCard 
                      key={idx} 
                      segment={seg} 
                      index={idx} 
                      isLast={idx === (slip.segments?.length || 0) - 1} 
                    />
                  ))}
                </div>

                {slip.incentives && slip.incentives.length > 0 && (
                  <div className="mt-12 space-y-6">
                    <div className="flex items-center gap-3 px-2">
                      <div className="w-1.5 h-6 bg-blue-600 rounded-full" />
                      <span className="text-xs font-black text-slate-800 uppercase tracking-[0.25em]">{t("common.payroll.table.bonus")} Detail</span>
                    </div>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      {slip.incentives.map((inc, i) => (
                        <div key={i} className="flex items-center justify-between p-6 bg-blue-50/30 border border-blue-100/50 rounded-2xl">
                          <div className="flex items-center gap-4">
                            <div className="w-10 h-10 rounded-xl bg-white flex items-center justify-center text-blue-600 shadow-sm border border-blue-100">
                              <Sparkles size={18} />
                            </div>
                            <span className="text-xs font-black text-slate-700 uppercase tracking-tight">{inc.policy_name}</span>
                          </div>
                          <span className="text-sm font-black text-blue-600 font-mono tracking-tighter">+{new Intl.NumberFormat('vi-VN').format(parseFloat(inc.amount))}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                <div className="mt-12 grid grid-cols-1 md:grid-cols-4 gap-6 pt-10 border-t border-slate-50">
                  {[
                    { label: t("common.payroll.table.base_pay"), value: slip.base_pay_amount, color: "text-slate-900" },
                    { label: t("common.payroll.table.overtime"), value: slip.overtime_pay_amount, color: "text-emerald-600", prefix: "+" },
                    { 
                      label: t("common.payroll.table.bonus"), 
                      value: slip.incentives?.reduce((acc, i) => acc + parseFloat(i.amount), 0).toString() || "0", 
                      color: "text-blue-600", 
                      prefix: "+" 
                    },
                    { label: "Deductions", value: slip.deductions_amount, color: "text-rose-600", prefix: "-" }
                  ].map((stat, i) => (
                    <div key={i} className="p-6 bg-slate-50 border border-slate-100 rounded-[2rem] shadow-inner flex flex-col gap-1">
                       <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em]">{stat.label}</p>
                       <p className={cn("text-xl font-black font-mono tracking-tighter", stat.color)}>{stat.prefix}{new Intl.NumberFormat('vi-VN').format(parseFloat(stat.value))}</p>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

function SegmentCard({ segment, index }: { segment: PayrollSegment, index: number, isLast: boolean }) {
  const { t } = useTranslation();
  const { formatDate } = useDateTimeFormatter();

  return (
    <div className="relative pl-16">
      <div className={cn("absolute left-5 top-2 w-4 h-4 rounded-full border-[3px] border-white z-10 shadow-md ring-4", index === 0 ? "bg-primary-600 ring-primary-50" : "bg-slate-200 ring-slate-50")} />

      <div className="bg-white border border-slate-100 rounded-[2rem] p-8 hover:shadow-2xl hover:shadow-primary-100/10 transition-all group/card relative">
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 mb-8">
          <div className="flex items-center gap-4">
             <div className="w-10 h-10 bg-slate-50 rounded-xl flex items-center justify-center text-slate-300 group-hover/card:text-primary-600 group-hover/card:bg-primary-50 transition-all border border-slate-100 shadow-inner">
               <ArrowDownRight size={18} />
             </div>
             <div>
               <p className="text-[9px] font-black text-slate-400 uppercase tracking-[0.25em] leading-none mb-1.5">{t("common.segment")} {index + 1}</p>
               <p className="text-sm font-black text-slate-900 uppercase tracking-tight">{formatDate(segment.start_date)} — {formatDate(segment.end_date)}</p>
             </div>
          </div>
          <span className={cn("text-[9px] font-black uppercase tracking-[0.2em] px-3 py-1 rounded-lg border shadow-sm", segment.reason === 'Standard' ? "bg-white text-slate-400 border-slate-100" : "bg-primary-50 text-primary-600 border-primary-100")}>{segment.reason || "Proration"}</span>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
          <div className="space-y-6">
            <div className="flex items-center gap-3 mb-2 px-1 opacity-60"><Target size={14} className="text-slate-400" /><span className="text-[10px] font-black text-slate-500 uppercase tracking-[0.25em]">Configuration Applied</span></div>
            <div className="grid grid-cols-2 gap-6">
              <div className="bg-slate-50/50 p-5 rounded-[1.5rem] border border-slate-100 shadow-inner">
                <p className="text-[9px] font-black text-slate-400 uppercase tracking-widest mb-2 opacity-70">Base Salary</p>
                <div className="flex items-center gap-2 text-slate-900 font-black font-mono text-sm tracking-tighter">
                  <Banknote size={14} className="text-primary-500 opacity-50" /> {new Intl.NumberFormat('vi-VN').format(segment.applied_base_salary)}
                </div>
              </div>
              <div className="bg-slate-50/50 p-5 rounded-[1.5rem] border border-slate-100 shadow-inner">
                <p className="text-[9px] font-black text-slate-400 uppercase tracking-widest mb-2 opacity-70">Std. Period</p>
                <div className="flex items-center gap-3">
                  <span className="text-sm font-black text-slate-900 uppercase tracking-tighter">{segment.applied_std_days} Days</span>
                  {segment.std_days_source && (
                    <Popover.Root>
                      <Popover.Trigger asChild><button className="p-1.5 rounded-lg bg-white shadow-sm border border-slate-100 text-slate-300 hover:text-primary-600 transition-all"><Info size={12} /></button></Popover.Trigger>
                      <Popover.Portal><Popover.Content className="z-50 w-80 bg-transparent animate-in zoom-in-95 duration-300 origin-bottom" side="top" sideOffset={12} align="center"><PayrollPriorityLegend activeSource={segment.std_days_source} /></Popover.Content></Popover.Portal>
                    </Popover.Root>
                  )}
                </div>
              </div>
            </div>
            <div className="flex items-center gap-4 px-2">
              <div className="flex items-center gap-2"><Clock size={12} className="text-slate-300" /><span className="text-[10px] font-black text-slate-500 uppercase tracking-widest">{segment.applied_hours_per_day}h / day</span></div>
              <div className="w-1 h-1 bg-slate-200 rounded-full" />
              <div className="flex items-center gap-2"><Briefcase size={12} className="text-slate-300" /><span className="text-[10px] font-black text-slate-500 uppercase tracking-widest italic opacity-60">Dept: Global</span></div>
            </div>
          </div>

          <div className="space-y-6">
            <div className="flex items-center gap-3 mb-2 px-1 opacity-60"><Calculator size={14} className="text-slate-400" /><span className="text-[10px] font-black text-slate-500 uppercase tracking-[0.25em]">Calculation Outcome</span></div>
            <div className="bg-primary-600 rounded-[1.5rem] p-6 text-white shadow-xl shadow-primary-100 relative overflow-hidden group/out">
              <div className="flex items-center justify-between relative z-10">
                <div>
                  <p className="text-[9px] font-black text-primary-200 uppercase tracking-widest mb-1 opacity-80">Paid Days</p>
                  <p className="text-xl font-black tracking-tighter">{segment.paid_work_days} <span className="text-xs text-primary-300 font-bold">/ {segment.actual_work_days}</span></p>
                </div>
                <div className="h-10 w-px bg-white/10" />
                <div className="text-right">
                  <p className="text-[9px] font-black text-primary-200 uppercase tracking-widest mb-1 opacity-80">Segment Net</p>
                  <p className="text-xl font-black font-mono tracking-tighter">{new Intl.NumberFormat('vi-VN').format(segment.amount)}</p>
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
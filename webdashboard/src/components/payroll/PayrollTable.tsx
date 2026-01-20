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
  AlertTriangle
} from "lucide-react";
import { Payslip, AttendanceAdjustmentRequest, PayrollSegment } from "@/lib/api/types";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { PayrollPriorityLegend } from "./PayrollPriorityLegend";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

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
  const { formatMonthYear, formatDate } = useDateTimeFormatter();
  const [expandedId, setExpandedId] = useState<string | null>(null);

  if (isLoading) {
    return (
      <div className="p-12 text-center text-slate-400 italic animate-pulse">
        {t("common.loading")}
      </div>
    );
  }

  if (payslips.length === 0) {
    return (
      <div className="p-20 text-center bg-slate-50/50 rounded-[40px] m-4 border border-dashed border-slate-200">
        <div className="bg-white w-16 h-16 rounded-3xl flex items-center justify-center mx-auto mb-4 shadow-sm border border-slate-100">
          <History className="text-slate-300 w-8 h-8" />
        </div>
        <p className="text-sm font-bold text-slate-400 uppercase tracking-widest">{t("common.payroll.table.no_payslips")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {payslips.map((slip) => {
        const isExpanded = expandedId === slip.id;
        
        // Check for pending adjustments in this period
        const slipStart = new Date(slip.period_start);
        const slipEnd = new Date(slip.period_end);
        const hasPendingAdjustments = adjustments.some(adj => {
          const adjDate = new Date(adj.requested_punch_in_time);
          return adjDate >= slipStart && adjDate <= slipEnd;
        });

        return (
          <div 
            key={slip.id} 
            data-testid={`payslip-card-${slip.id}`}
            className={cn(
              "group bg-white border rounded-[32px] transition-all duration-300 overflow-hidden",
              isExpanded ? "border-primary-200 shadow-xl shadow-primary-50/50" : "border-slate-100 hover:border-slate-200 shadow-sm"
            )}
          >
            {/* Summary Bar */}
            <div 
              data-testid={`toggle-card-${slip.id}`}
              className={cn(
                "p-6 flex items-center justify-between cursor-pointer transition-colors",
                isExpanded ? "bg-primary-50/20" : "hover:bg-slate-50/50"
              )}
              onClick={() => setExpandedId(isExpanded ? null : slip.id)}
            >
              <div className="flex items-center gap-6">
                <div className={cn(
                  "w-12 h-12 rounded-2xl flex items-center justify-center transition-colors",
                  isExpanded ? "bg-primary-600 text-white" : "bg-slate-100 text-slate-400 group-hover:bg-slate-200"
                )}>
                  <Calendar size={20} />
                </div>
                <div>
                  <p className="text-sm font-black text-slate-900 tracking-tight">
                    {t("common.month")} {formatMonthYear(slip.period_start)}
                  </p>
                  <div className="flex items-center gap-2 mt-1">
                    <span className={cn(
                      "text-[10px] font-black uppercase tracking-tighter px-2 py-0.5 rounded-md",
                      slip.status === 'paid' ? "bg-emerald-100 text-emerald-700" : "bg-blue-100 text-blue-700"
                    )}>
                      {slip.status === 'paid' ? t("common.payroll.status.paid") : t("common.payroll.status.finalized")}
                    </span>
                    <div className="w-1 h-1 bg-slate-200 rounded-full" />
                    <span className="text-[10px] font-bold text-slate-400">
                      {slip.standard_units} Units • {slip.overtime_units} OT
                    </span>
                  </div>
                </div>
              </div>

              <div className="flex items-center gap-8">
                <div className="text-right">
                  <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest leading-none mb-1">{t("common.payroll.table.net_pay")}</p>
                  <p className="text-xl font-black text-primary-600 tracking-tight">
                    {new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount))}
                  </p>
                </div>
                <div className={cn(
                  "w-10 h-10 rounded-xl flex items-center justify-center border transition-all",
                  isExpanded ? "bg-white border-primary-200 text-primary-600 rotate-180" : "border-slate-100 text-slate-300"
                )}>
                  <ChevronDown size={20} />
                </div>
              </div>
            </div>

            {/* Details Section */}
            {isExpanded && (
              <div className="px-6 pb-8 animate-in fade-in slide-in-from-top-2 duration-300">
                <div className="h-px bg-slate-100 mb-8" />
                
                {/* Warnings for adjustments */}
                {hasPendingAdjustments && (
                  <div 
                    data-testid={`pending-adj-banner-${slip.id}`}
                    className="mb-8 p-4 bg-amber-50 border border-amber-100 rounded-2xl flex items-center justify-between"
                  >
                    <div className="flex items-center gap-3">
                      <div className="w-10 h-10 bg-white rounded-xl flex items-center justify-center text-amber-500 shadow-sm">
                        <AlertTriangle size={20} />
                      </div>
                      <div>
                        <p className="text-sm font-bold text-amber-900">{t("payroll.traceability.pending_adjustments")}</p>
                        <p className="text-[10px] text-amber-700 font-medium">{t("payroll.traceability.recalc_needed")}</p>
                      </div>
                    </div>
                    <button 
                      onClick={onResolveAdjustments}
                      className="px-4 py-2 bg-amber-500 text-white rounded-xl text-[10px] font-black uppercase tracking-widest hover:bg-amber-600 transition-all shadow-md shadow-amber-100"
                    >
                      {t("payroll.traceability.view_adjustments")}
                    </button>
                  </div>
                )}

                {/* Segments Display - Vertical Timeline */}
                <div className="space-y-6 relative">
                  {/* Vertical Line */}
                  <div className="absolute left-[23px] top-4 bottom-4 w-0.5 bg-slate-100" />

                  {slip.segments?.map((seg, idx) => (
                    <SegmentCard key={idx} segment={seg} index={idx} isLast={idx === (slip.segments?.length || 0) - 1} />
                  ))}
                </div>

                {/* Total Calculation breakdown footer */}
                <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div className="p-4 bg-slate-50 rounded-2xl border border-slate-100">
                     <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest mb-1">{t("common.payroll.table.base_pay")}</p>
                     <p className="text-lg font-black text-slate-900">{new Intl.NumberFormat('vi-VN').format(parseFloat(slip.base_pay_amount))}</p>
                  </div>
                  <div className="p-4 bg-slate-50 rounded-2xl border border-slate-100">
                     <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest mb-1">{t("common.payroll.table.overtime")}</p>
                     <p className="text-lg font-black text-emerald-600">+{new Intl.NumberFormat('vi-VN').format(parseFloat(slip.overtime_pay_amount))}</p>
                  </div>
                  <div className="p-4 bg-slate-50 rounded-2xl border border-slate-100">
                     <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest mb-1">Deductions</p>
                     <p className="text-lg font-black text-rose-600">-{new Intl.NumberFormat('vi-VN').format(parseFloat(slip.deductions_amount))}</p>
                  </div>
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
    <div className="relative pl-12">
      {/* Node Dot */}
      <div className={cn(
        "absolute left-4 top-2 w-5 h-5 rounded-full border-4 border-white z-10 shadow-sm",
        index === 0 ? "bg-primary-600 ring-4 ring-primary-50" : "bg-slate-300"
      )} />

      <div className="bg-slate-50/50 border border-slate-100 rounded-[24px] p-5 hover:bg-white hover:border-primary-100 hover:shadow-premium-sm transition-all group/card">
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6">
          <div className="flex items-center gap-3">
             <div className="w-8 h-8 bg-white rounded-xl flex items-center justify-center text-slate-400 group-hover/card:text-primary-600 shadow-sm transition-colors">
               <ArrowDownRight size={14} />
             </div>
             <div>
               <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest leading-none mb-1">Giai đoạn {index + 1}</p>
               <p className="text-sm font-bold text-slate-900">{formatDate(segment.start_date)} - {formatDate(segment.end_date)}</p>
             </div>
          </div>

          <div className="flex items-center gap-2">
            <span className={cn(
              "text-[9px] font-black uppercase tracking-widest px-2 py-1 rounded-lg border",
              segment.reason === 'Standard' ? "bg-white text-slate-400 border-slate-100" : "bg-blue-50 text-blue-600 border-blue-100"
            )}>
               {segment.reason || "Proration"}
            </span>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          {/* Policy Column */}
          <div className="space-y-4">
            <div className="flex items-center gap-2 mb-2">
               <Target size={12} className="text-slate-300" />
               <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest">Cấu hình áp dụng</span>
            </div>
            
            <div className="grid grid-cols-2 gap-4">
              <div className="bg-white p-3 rounded-2xl border border-slate-50">
                <p className="text-[9px] font-bold text-slate-400 uppercase tracking-tighter mb-1">Lương Cơ Bản</p>
                <div className="flex items-center gap-1">
                  <Banknote size={10} className="text-primary-500" />
                  <span className="text-xs font-black text-slate-900">{new Intl.NumberFormat('vi-VN').format(segment.applied_base_salary)}</span>
                </div>
              </div>
              <div className="bg-white p-3 rounded-2xl border border-slate-50">
                <p className="text-[9px] font-bold text-slate-400 uppercase tracking-tighter mb-1">Công chuẩn</p>
                <div className="flex items-center gap-2">
                  <span className="text-xs font-black text-slate-900">{segment.applied_std_days} Ngày</span>
                  {segment.std_days_source && (
                    <Popover.Root>
                      <Popover.Trigger asChild>
                        <button className="p-1 rounded-full hover:bg-slate-100 text-slate-400 transition-colors">
                          <Info size={10} />
                        </button>
                      </Popover.Trigger>
                      <Popover.Portal>
                        <Popover.Content 
                          className="z-50 w-72 bg-transparent animate-in zoom-in-95 duration-200" 
                          side="top"
                          sideOffset={5}
                        >
                          <PayrollPriorityLegend activeSource={segment.std_days_source} />
                        </Popover.Content>
                      </Popover.Portal>
                    </Popover.Root>
                  )}
                </div>
              </div>
            </div>

            <div className="flex items-center gap-3 px-3">
              <div className="flex items-center gap-1.5">
                 <Clock size={10} className="text-slate-300" />
                 <span className="text-[10px] font-bold text-slate-500">{segment.applied_hours_per_day}h/ngày</span>
              </div>
              <div className="w-1 h-1 bg-slate-200 rounded-full" />
              <div className="flex items-center gap-1.5">
                 <Briefcase size={10} className="text-slate-300" />
                 <span className="text-[10px] font-bold text-slate-500">Phòng ban: —</span>
              </div>
            </div>
          </div>

          {/* Outcome Column */}
          <div className="space-y-4">
            <div className="flex items-center gap-2 mb-2">
               <Calculator size={12} className="text-slate-300" />
               <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest">Kết quả tính toán</span>
            </div>

            <div className="bg-white border border-slate-100 rounded-2xl p-4 flex items-center justify-between">
              <div>
                <p className="text-[9px] font-bold text-slate-400 uppercase tracking-tighter mb-1">Công tính lương</p>
                <p className="text-base font-black text-slate-900">{segment.paid_work_days} <span className="text-[10px] text-slate-400">/ {segment.actual_work_days}</span></p>
              </div>
              <div className="h-8 w-px bg-slate-50" />
              <div className="text-right">
                <p className="text-[9px] font-bold text-primary-400 uppercase tracking-tighter mb-1">Thành tiền segment</p>
                <p className="text-lg font-black text-primary-600 tracking-tight">{new Intl.NumberFormat('vi-VN').format(segment.amount)}</p>
              </div>
            </div>

            <div className="flex items-center gap-1.5 px-3">
               <CheckCircle2 size={10} className="text-emerald-500" />
               <span className="text-[9px] font-black text-slate-400 uppercase tracking-widest">Logic: (Lương / Công chuẩn) * Công thực tế</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

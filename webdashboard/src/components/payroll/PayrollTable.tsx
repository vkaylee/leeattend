import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import * as Popover from '@radix-ui/react-popover';
import { 
  CheckCircle2, 
  AlertCircle, 
  ChevronDown, 
  ChevronRight,
  FileText,
  AlertTriangle,
  Info
} from "lucide-react";
import { Payslip, AttendanceAdjustmentRequest } from "@/lib/api/types";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import SegmentVisualizer from "./SegmentVisualizer";
import { PayrollPriorityLegend } from "./PayrollPriorityLegend";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

interface PayrollTableProps {
  payslips: Payslip[];
  isLoading: boolean;
  adjustments?: AttendanceAdjustmentRequest[]; // Pass all pending adjustments for this employee
  onResolveAdjustments?: () => void;
}

export default function PayrollTable({ payslips, isLoading, adjustments = [], onResolveAdjustments }: PayrollTableProps) {
  const { t } = useTranslation();
  const { formatDate, formatMonthYear } = useDateTimeFormatter();
  const [expandedRows, setExpandedRows] = useState<Record<string, boolean>>({});

  const toggleRow = (id: string) => {
    setExpandedRows(prev => ({ ...prev, [id]: !prev[id] }));
  };

  const hasSegments = (slip: Payslip) => slip.segments && slip.segments.length > 0;

  if (isLoading) {
    return (
      <div className="p-12 text-center text-slate-400 italic">
        {t("common.payroll.table.loading_payslips")}
      </div>
    );
  }

  if (payslips.length === 0) {
    return (
      <div className="p-20 text-center">
        <div className="bg-slate-50 w-16 h-16 rounded-3xl flex items-center justify-center mx-auto mb-6 shadow-sm border border-slate-100">
          <FileText className="text-slate-300 w-8 h-8" />
        </div>
        <p className="text-sm font-bold text-slate-400 uppercase tracking-widest">
          {t("common.payroll.table.no_payslips")}
        </p>
      </div>
    );
  }

  return (
    <table className="w-full text-left border-collapse" data-testid="payroll-table">
      <thead className="bg-slate-50/50 sticky top-0 backdrop-blur-md border-b border-slate-100 z-10">
        <tr>
          <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest w-12"></th>
          <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.payroll.table.period")}</th>
          <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.base_pay")}</th>
          <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.overtime")}</th>
          <th className="px-4 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.net_pay")}</th>
          <th className="px-8 py-5 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("common.payroll.table.status")}</th>
        </tr>
      </thead>
      <tbody className="divide-y divide-slate-50">
        {payslips.map(slip => {
          // Check for pending adjustments in this period
          const slipStart = new Date(slip.period_start);
          const slipEnd = new Date(slip.period_end);
          const hasPendingAdjustments = adjustments.some(adj => {
            const adjDate = new Date(adj.requested_punch_in_time);
            return adjDate >= slipStart && adjDate <= slipEnd;
          });

          const isExpanded = expandedRows[slip.id];
          const showExpand = hasSegments(slip);

          return (
            <React.Fragment key={slip.id}>
              <tr 
                data-testid={`payslip-row-${slip.id}`}
                className={cn(
                  "group transition-colors",
                  isExpanded ? "bg-slate-50/80" : "hover:bg-slate-50/50"
                )}
              >
                <td className="px-8 py-6 text-center">
                  {showExpand && (
                    <button 
                      data-testid={`toggle-segments-${slip.id}`}
                      onClick={() => toggleRow(slip.id)}
                      className="w-8 h-8 rounded-full flex items-center justify-center hover:bg-white hover:shadow-md transition-all text-slate-400 hover:text-primary-600"
                    >
                      {isExpanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                    </button>
                  )}
                </td>
                <td className="px-4 py-6">
                  <div className="flex flex-col">
                    <span className="font-bold text-slate-900 text-sm">
                      {t("common.month")} {formatMonthYear(slip.period_start)}
                    </span>
                    <span className="text-[10px] text-slate-400 font-bold mt-1">
                      {formatDate(slip.period_start)} - {formatDate(slip.period_end)}
                    </span>
                    {/* Mini Visualizer in Master Row if collapsed */}
                    {showExpand && !isExpanded && (
                      <div className="mt-3 w-32 opacity-60 grayscale group-hover:grayscale-0 transition-all">
                        <SegmentVisualizer 
                          segments={slip.segments!} 
                          periodStart={slip.period_start} 
                          periodEnd={slip.period_end} 
                          className="h-1.5"
                        />
                      </div>
                    )}
                  </div>
                </td>
                <td className="px-4 py-6 text-right font-mono text-sm text-slate-600">
                  {new Intl.NumberFormat('vi-VN').format(parseFloat(slip.base_pay_amount))}
                </td>
                <td className="px-4 py-6 text-right font-mono text-sm text-emerald-600 font-bold">
                  +{new Intl.NumberFormat('vi-VN').format(parseFloat(slip.overtime_pay_amount))}
                </td>
                <td className="px-4 py-6 text-right">
                  <div className="font-black text-primary-600 text-lg tracking-tight">
                    {new Intl.NumberFormat('vi-VN').format(parseFloat(slip.net_pay_amount))}
                  </div>
                </td>
                <td className="px-8 py-6 text-center">
                  <div className="flex flex-col items-center gap-2">
                    {slip.status === 'paid' ? (
                      <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-emerald-100 text-emerald-700 rounded-full text-[10px] font-black uppercase tracking-tighter">
                        <CheckCircle2 size={12} /> {t("common.payroll.status.paid")}
                      </span>
                    ) : (
                      <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-blue-100 text-blue-700 rounded-full text-[10px] font-black uppercase tracking-tighter">
                        <AlertCircle size={12} /> {t("common.payroll.status.finalized")}
                      </span>
                    )}

                    {hasPendingAdjustments && slip.status !== 'paid' && (
                      <button 
                        data-testid={`pending-adj-badge-${slip.id}`}
                        onClick={onResolveAdjustments}
                        className="inline-flex items-center gap-1.5 px-3 py-1 bg-amber-100 text-amber-700 rounded-full text-[10px] font-black uppercase tracking-tighter animate-pulse hover:bg-amber-200 transition-colors shadow-sm cursor-pointer"
                      >
                        <AlertTriangle size={12} /> Pending Adj.
                      </button>
                    )}
                  </div>
                </td>
              </tr>
              
              {/* Detail Row */}
              {isExpanded && showExpand && (
                <tr className="bg-slate-50/50 animate-in fade-in slide-in-from-top-2 duration-300" data-testid={`segments-detail-${slip.id}`}>
                  <td colSpan={6} className="px-8 pb-8 pt-2">
                    <div className="bg-white rounded-2xl p-6 border border-slate-200/60 shadow-sm ml-12">
                      <div className="flex items-center justify-between mb-6">
                        <h4 className="text-xs font-black text-slate-400 uppercase tracking-widest">
                          {t("common.payroll.segments.title")}
                        </h4>
                        <div className="w-64">
                          <SegmentVisualizer 
                            segments={slip.segments!} 
                            periodStart={slip.period_start} 
                            periodEnd={slip.period_end} 
                          />
                        </div>
                      </div>

                      <div className="overflow-x-auto">
                        <table className="w-full text-left" data-testid="segments-table">
                          <thead>
                            <tr className="border-b border-slate-100 text-[10px] font-black text-slate-400 uppercase tracking-widest">
                              <th className="py-3 px-4">{t("common.payroll.table.period")}</th>
                              <th className="py-3 px-4">{t("common.payroll.table.reason")}</th>
                              <th className="py-3 px-4 text-right">{t("common.payroll.table.basis")}</th>
                              <th className="py-3 px-4 text-right">{t("common.payroll.table.std_days")}</th>
                              <th className="py-3 px-4 text-right">{t("common.payroll.table.actual")}</th>
                              <th className="py-3 px-4 text-right">{t("common.payroll.table.amount")}</th>
                            </tr>
                          </thead>
                          <tbody className="divide-y divide-slate-50 text-xs text-slate-600 font-medium">
                            {slip.segments!.map((seg, idx) => (
                              <tr key={idx} data-testid="segment-row">
                                <td className="py-3 px-4">
                                  {formatDate(seg.start_date)} - {formatDate(seg.end_date)}
                                </td>
                                <td className="py-3 px-4">
                                  <span className={cn(
                                    "px-2 py-0.5 rounded-md text-[10px] font-bold uppercase",
                                    seg.reason === 'Standard' ? "bg-slate-100 text-slate-500" : "bg-blue-100 text-blue-600"
                                  )}>
                                    {seg.reason}
                                  </span>
                                </td>
                                <td className="py-3 px-4 text-right font-mono">
                                  {new Intl.NumberFormat('vi-VN').format(seg.applied_base_salary)}
                                </td>
                                <td className="py-3 px-4 text-right font-mono">
                                  <div className="flex items-center justify-end gap-2">
                                    {seg.applied_std_days}
                                    <Popover.Root>
                                      <Popover.Trigger asChild>
                                        <button 
                                          className="p-1 rounded-full hover:bg-slate-100 text-slate-400 transition-colors"
                                          onClick={(e) => e.stopPropagation()}
                                        >
                                          <Info size={12} />
                                        </button>
                                      </Popover.Trigger>
                                      <Popover.Portal>
                                        <Popover.Content 
                                          className="z-50 w-72 bg-transparent animate-in zoom-in-95 duration-200" 
                                          side="left"
                                          sideOffset={5}
                                          onClick={(e) => e.stopPropagation()}
                                        >
                                          <PayrollPriorityLegend activeSource={seg.std_days_source} />
                                        </Popover.Content>
                                      </Popover.Portal>
                                    </Popover.Root>
                                  </div>
                                </td>
                                <td className="py-3 px-4 text-right font-mono">{seg.actual_work_days}</td>
                                <td className="py-3 px-4 text-right font-bold text-slate-900 font-mono">
                                  {new Intl.NumberFormat('vi-VN').format(seg.amount)}
                                </td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      </div>
                    </div>
                  </td>
                </tr>
              )}
            </React.Fragment>
          );
        })}
      </tbody>
    </table>
  );
}
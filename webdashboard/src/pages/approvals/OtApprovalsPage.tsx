import { useState } from "react";

import * as Popover from "@radix-ui/react-popover";
import { startOfMonth, endOfMonth, format, subMonths, addMonths } from "date-fns";
import {
  CheckCircle,
  XCircle,
  ChevronLeft,
  ChevronRight,
  FileText,
  Filter,
  Upload
} from "lucide-react";
import { useTranslation } from "react-i18next";

import { BulkUploadOtDialog } from "@/components/approvals/BulkUploadOtDialog";
import { EnterPaperOtDialog } from "@/components/approvals/EnterPaperOtDialog";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";

import { useAttendanceSummaries, useApproveOt } from "@/hooks/use-attendance";
import { cn } from "@/lib/utils";

type SourceFilter = "all" | "paper_form" | "self_service";

export default function OtApprovalsPage() {
  const { t } = useTranslation();
  const [currentMonth, setCurrentMonth] = useState(new Date());
  const [paperOtDialogOpen, setPaperOtDialogOpen] = useState(false);
  const [bulkUploadDialogOpen, setBulkUploadDialogOpen] = useState(false);
  const [sourceFilter, setSourceFilter] = useState<SourceFilter>("all");

  const startDate = format(startOfMonth(currentMonth), "yyyy-MM-dd");
  const endDate = format(endOfMonth(currentMonth), "yyyy-MM-dd");

  const { summaries, isLoading, refetch } = useAttendanceSummaries(startDate, endDate);
  const { approveOt, isProcessing } = useApproveOt();

  const pendingSummaries = summaries
    .filter(s => s.ot_status === "pending_manager")
    .filter(s => {
      if (sourceFilter === "all") return true;
      return s.entered_via === sourceFilter;
    });

  const handleApprove = async (summary: { employee_id: string; date: string; claimed_ot_minutes: number; }, approved: boolean) => {
    try {
 
        await approveOt({
            employeeId: summary.employee_id,
            date: summary.date,
            approved_minutes: approved ? summary.claimed_ot_minutes : 0,
            status: approved ? "approved" : "rejected",
            manager_note: approved ? "Approved via Manager Portal" : "Rejected via Manager Portal"
        });
        refetch();
    } catch (e) {
        console.error(e);
        alert("Action failed");
    }
  };

  const handleMonthYearChange = (month: number, year: number) => { setCurrentMonth(new Date(year, month - 1, 1)); };
  const handlePrevMonth = () => setCurrentMonth(subMonths(currentMonth, 1));
  const handleNextMonth = () => setCurrentMonth(addMonths(currentMonth, 1));

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20 p-8">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8">
        <div className="space-y-1">
          <h1 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">{t("ot_approvals.title")}</h1>
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("ot_approvals.subtitle")}</p>
        </div>

        <div className="flex items-center gap-4">
          <button
            onClick={() => setPaperOtDialogOpen(true)}
            className="px-6 py-3 rounded-[1.5rem] bg-amber-50 dark:bg-amber-950 text-white text-xs font-bold uppercase tracking-wider shadow-lg hover:bg-amber-600 transition-all flex items-center gap-2"
          >
            <FileText className="w-4 h-4" />
            {t("approvals.paper_ot.btn_enter") || "Enter Paper Form"}
          </button>

          <button
            onClick={() => setBulkUploadDialogOpen(true)}
            className="px-6 py-3 rounded-[1.5rem] bg-blue-50 dark:bg-blue-950 text-white text-xs font-bold uppercase tracking-wider shadow-lg hover:bg-blue-600 transition-all flex items-center gap-2"
          >
            <Upload className="w-4 h-4" />
            {t("approvals.bulk_upload.btn") || "Bulk Upload"}
          </button>

          <div className="flex items-center gap-4 bg-white dark:bg-slate-900 p-2.5 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-900/5">
            <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-[1.5rem] p-1.5 shadow-inner border border-slate-200 dark:border-slate-700/50">
              <button onClick={handlePrevMonth} className="w-12 h-12 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all active:scale-90"><ChevronLeft size={24} /></button>

              <Popover.Root>
                <Popover.Trigger asChild>
                  <button className="px-10 flex flex-col items-center min-w-[200px] cursor-pointer hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all py-2 group outline-none border border-transparent hover:border-slate-100 dark:border-slate-800">
                    <span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] leading-none mb-1.5 group-hover:text-primary-500 dark:group-hover:text-primary-400 transition-colors">{t("payroll.table.period").toUpperCase()}</span>
                    <span className="text-sm font-bold text-slate-700 dark:text-slate-200 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors uppercase tracking-tight">
                      {format(currentMonth, "MMMM yyyy")}
                    </span>
                  </button>
                </Popover.Trigger>
                <Popover.Portal>
                  <Popover.Content className="z-50 w-80 p-4 rounded-[2.5rem] shadow-premium-2xl border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 animate-in zoom-in-95 slide-in-from-top-4 origin-top" sideOffset={12} align="center">
                    <MonthYearPicker month={currentMonth.getMonth() + 1} year={currentMonth.getFullYear()} onChange={handleMonthYearChange} />
                  </Popover.Content>
                </Popover.Portal>
              </Popover.Root>

              <button onClick={handleNextMonth} className="w-12 h-12 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-md rounded-xl transition-all active:scale-90"><ChevronRight size={24} /></button>
            </div>
          </div>
        </div>
      </div>

      <div className="flex items-center gap-4">
        <Filter size={16} className="text-slate-400 dark:text-slate-500" />
        <div className="flex gap-2">
          {(["all", "paper_form", "self_service"] as SourceFilter[]).map((filter) => (
            <button
              key={filter}
              onClick={() => setSourceFilter(filter)}
              className={cn(
                "px-4 py-2 rounded-xl text-[10px] font-bold uppercase tracking-wider transition-all",
                sourceFilter === filter
                  ? "bg-primary-600 text-white shadow-lg"
                  : "bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:bg-slate-600"
              )}
            >
              {filter === "all" ? t("common.all") : filter === "paper_form" ? t("approvals.filter.paper_forms") : t("approvals.filter.self_service")}
            </button>
          ))}
        </div>
      </div>

      <EnterPaperOtDialog
        open={paperOtDialogOpen}
        onOpenChange={setPaperOtDialogOpen}
        onSuccess={() => refetch()}
      />

      <BulkUploadOtDialog
        open={bulkUploadDialogOpen}
        onOpenChange={setBulkUploadDialogOpen}
        onSuccess={() => refetch()}
      />

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
        {isLoading ? (
          <div className="p-32 flex flex-col items-center gap-6 animate-pulse">
            <div className="w-12 h-12 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
            <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em]">{t("common.loading")}</span>
          </div>
        ) : pendingSummaries.length === 0 ? (
          <div className="py-40 text-center flex flex-col items-center border-4 border-dashed border-slate-50 dark:border-slate-800 m-8 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30">
            <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-xl border border-slate-100 dark:border-slate-800 mb-8"><CheckCircle size={40} className="text-emerald-200" /></div>
            <h3 className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">{t("ot_approvals.empty_state.title")}</h3>
            <p className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-2 opacity-50 italic">{t("ot_approvals.empty_state.description")}</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
                  <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("ot_approvals.table.date")}</th>
                  <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("ot_approvals.table.employee")}</th>
                  <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("ot_approvals.table.source")}</th>
                  <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">{t("ot_approvals.table.system")}</th>
                  <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">{t("ot_approvals.table.claimed")}</th>
                  <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">{t("ot_approvals.table.reason")}</th>
                  <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-right">{t("ot_approvals.table.actions")}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                {pendingSummaries.map((summary) => (
                  <tr key={summary.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group">
                    <td className="px-8 py-6 font-bold text-slate-700 dark:text-slate-200">{summary.date}</td>
                    <td className="px-8 py-6 text-sm text-slate-600 dark:text-slate-300">{summary.employee_id}</td>
                    <td className="px-8 py-6">
                      <SourceBadge enteredVia={summary.entered_via} />
                    </td>
                    <td className="px-8 py-6 text-center text-sm font-mono text-slate-500 dark:text-slate-400">{summary.system_detected_ot_minutes}m</td>
                    <td className="px-8 py-6 text-center text-sm font-bold font-mono text-amber-600 dark:text-amber-300">{summary.claimed_ot_minutes}m</td>
                    <td className="px-8 py-6 text-sm text-slate-600 dark:text-slate-300 italic">"{summary.ot_reason}"</td>
                    <td className="px-8 py-6 text-right">
                      <div className="flex items-center justify-end gap-2">
                        <button
                          onClick={() => handleApprove(summary, true)}
                          disabled={isProcessing}
                          className="p-2 rounded-full hover:bg-emerald-50 dark:bg-emerald-950 text-slate-400 dark:text-slate-500 hover:text-emerald-600 dark:text-emerald-300 transition-all disabled:opacity-50"
                          title={t("ot_approvals.actions.approve")}
                        >
                          <CheckCircle className="w-5 h-5" />
                        </button>
                        <button
                          onClick={() => handleApprove(summary, false)}
                          disabled={isProcessing}
                          className="p-2 rounded-full hover:bg-rose-50 dark:bg-rose-950 text-slate-400 dark:text-slate-500 hover:text-rose-600 dark:text-rose-300 transition-all disabled:opacity-50"
                          title={t("ot_approvals.actions.reject")}
                        >
                          <XCircle className="w-5 h-5" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}

function SourceBadge({ enteredVia }: { enteredVia?: string }) {
  const { t } = useTranslation();
  if (enteredVia === "paper_form") {
    return (
      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-amber-100 dark:bg-amber-900 text-amber-700 dark:text-amber-300 border border-amber-200">
        {t("ot_approvals.badges.paper")}
      </span>
    );
  }
  return (
    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-slate-700">
      {t("ot_approvals.badges.self")}
    </span>
  );
}

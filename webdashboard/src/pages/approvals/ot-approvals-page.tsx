import { CheckCircle } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { addMonths, endOfMonth, format, startOfMonth, subMonths } from "date-fns";

import { BulkUploadOtDialog } from "@/components/approvals/bulk-upload-ot-dialog";
import { EnterPaperOtDialog } from "@/components/approvals/enter-paper-ot-dialog";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useApproveOt, useAttendanceSummaries } from "@/hooks/use-attendance";

import { OtApprovalsFilters, SourceFilter } from "./components/ot-approvals-filters";
import { OtApprovalsHeader } from "./components/ot-approvals-header";
import { OtApprovalsTable } from "./components/ot-approvals-table";

export default function OtApprovalsPage() {
  const toast = useToast();
  const [currentMonth, setCurrentMonth] = useState(new Date());
  const [paperOtDialogOpen, setPaperOtDialogOpen] = useState(false);
  const [bulkUploadDialogOpen, setBulkUploadDialogOpen] = useState(false);
  const [sourceFilter, setSourceFilter] = useState<SourceFilter>("all");

  const startDate = format(startOfMonth(currentMonth), "yyyy-MM-dd");
  const endDate = format(endOfMonth(currentMonth), "yyyy-MM-dd");

  const { isLoading, refetch, summaries } = useAttendanceSummaries(startDate, endDate);
  const { approveOt, isProcessing } = useApproveOt();

  const pendingSummaries = summaries
    .filter((s) => s.ot_status === "pending_manager")
    .filter((s) => {
      if (sourceFilter === "all") return true;
      return s.entered_via === sourceFilter;
    });

  const handleApprove = async (
    summary: { claimed_ot_minutes: number; date: string; employee_id: string },
    approved: boolean
  ) => {
    try {
      await approveOt({
        approved_minutes: approved ? summary.claimed_ot_minutes : 0,
        date: summary.date,
        employeeId: summary.employee_id,
        manager_note: approved ? "Approved via Manager Portal" : "Rejected via Manager Portal",
        status: approved ? "approved" : "rejected",
      });
      toast.success(
        approved ? "Overtime approved successfully" : "Overtime rejected successfully"
      );
      void refetch();
    } catch (e) {
      console.error(e);
      toast.error("Action failed");
    }
  };

  const handleMonthYearChange = (month: number, year: number) => {
    setCurrentMonth(new Date(year, month - 1, 1));
  };
  const handlePreviousMonth = () => {
    setCurrentMonth(subMonths(currentMonth, 1));
  };
  const handleNextMonth = () => {
    setCurrentMonth(addMonths(currentMonth, 1));
  };

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20 p-8">
      <OtApprovalsHeader
        currentMonth={currentMonth}
        handleMonthYearChange={handleMonthYearChange}
        handleNextMonth={handleNextMonth}
        handlePreviousMonth={handlePreviousMonth}
        setBulkUploadDialogOpen={setBulkUploadDialogOpen}
        setPaperOtDialogOpen={setPaperOtDialogOpen}
      />

      <OtApprovalsFilters setSourceFilter={setSourceFilter} sourceFilter={sourceFilter} />

      <EnterPaperOtDialog
        onOpenChange={setPaperOtDialogOpen}
        onSuccess={() => {
          void refetch();
        }}
        open={paperOtDialogOpen}
      />

      <BulkUploadOtDialog
        onOpenChange={setBulkUploadDialogOpen}
        onSuccess={() => {
          void refetch();
        }}
        open={bulkUploadDialogOpen}
      />

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
        {isLoading ? (
          <LoadingState />
        ) : pendingSummaries.length === 0 ? (
          <EmptyState />
        ) : (
          <OtApprovalsTable
            handleApprove={handleApprove}
            isProcessing={isProcessing}
            pendingSummaries={pendingSummaries}
          />
        )}
      </div>
    </div>
  );
}

function LoadingState() {
  const { t } = useTranslation();
  return (
    <div className="p-32 flex flex-col items-center gap-6 animate-pulse">
      <div className="w-12 h-12 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
      <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em]">
        {t("common.loading")}
      </span>
    </div>
  );
}

function EmptyState() {
  const { t } = useTranslation();
  return (
    <div className="py-40 text-center flex flex-col items-center border-4 border-dashed border-slate-50 dark:border-slate-800 m-8 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30">
      <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-xl border border-slate-100 dark:border-slate-800 mb-8">
        <CheckCircle className="text-emerald-200" size={40} />
      </div>
      <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">
        {t("ot_approvals.empty_state.title")}
      </Typography>
      <Typography className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-2 opacity-50 italic" variant="p">
        {t("ot_approvals.empty_state.description")}
      </Typography>
    </div>
  );
}

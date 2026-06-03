import { useQuery } from "@tanstack/react-query";
import { Check, CheckCircle, Loader2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { addMonths, endOfMonth, format, startOfMonth, subMonths } from "date-fns";

import { BulkUploadOtDialog } from "@/components/approvals/bulk-upload-ot-dialog";
import { EnterPaperOtDialog } from "@/components/approvals/enter-paper-ot-dialog";
import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useApproveOt, useAttendanceSummaries, useClaimOt } from "@/hooks/use-attendance";
import { useMyPermissions } from "@/hooks/use-permissions";
import { useWorkspace } from "@/hooks/use-workspace";
import { fetchAttendanceReport } from "@/lib/api";
import { cn } from "@/lib/utils";

import { BehalfAssignmentsTab } from "./components/behalf-assignments-tab";
import { OtApprovalsFilters, SourceFilter } from "./components/ot-approvals-filters";
import { OtApprovalsHeader } from "./components/ot-approvals-header";
import { OtApprovalsTable } from "./components/ot-approvals-table";
import { PreOtApprovalsTab } from "./components/pre-ot-approvals-tab";

export default function OtApprovalsPage() {
  const toast = useToast();
  const { t } = useTranslation(["common", "attendance"]);
  const { workspace } = useWorkspace();
  const { hasPermission, isLoading: permissionsLoading } = useMyPermissions();

  const canApprove = hasPermission("overtime:approve");
  const canClaimOnBehalf = hasPermission("overtime:claim_on_behalf");
  const canManagePolicies = hasPermission("overtime:policies:manage");

  const [currentMonth, setCurrentMonth] = useState(new Date());
  const [paperOtDialogOpen, setPaperOtDialogOpen] = useState(false);
  const [bulkUploadDialogOpen, setBulkUploadDialogOpen] = useState(false);
  const [sourceFilter, setSourceFilter] = useState<SourceFilter>("all");
  const [activeTab, setActiveTab] = useState<"pre" | "post" | "delegated_claim" | "behalf_assignments">("pre");
  const [tabInitialized, setTabInitialized] = useState(false);

  if (!tabInitialized && !permissionsLoading) {
    if (!canApprove && canClaimOnBehalf) {
      setActiveTab("delegated_claim");
    } else if (!canApprove && !canClaimOnBehalf && canManagePolicies) {
      setActiveTab("behalf_assignments");
    }
    setTabInitialized(true);
  }

  const startDate = format(startOfMonth(currentMonth), "yyyy-MM-dd");
  const endDate = format(endOfMonth(currentMonth), "yyyy-MM-dd");

  const { isLoading, refetch, summaries } = useAttendanceSummaries(startDate, endDate);
  const { approveOt, isProcessing } = useApproveOt();

  if (permissionsLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingState />
      </div>
    );
  }

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

      <div className="flex bg-slate-100 dark:bg-slate-800/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 mb-8 overflow-x-auto custom-scrollbar">
        {canApprove ? (
          <Button
            className={cn(
              "flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold transition-all duration-300 whitespace-nowrap normal-case tracking-normal shadow-none",
              activeTab === "pre"
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            )}
            onClick={() => { setActiveTab("pre"); }}
            variant="ghost"
          >
            {t("ot_approvals.tabs.pre_ot", { defaultValue: "Pre-OT Approvals (Requests)" })}
          </Button>
        ) : null}
        {canApprove ? (
          <Button
            className={cn(
              "flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold transition-all duration-300 whitespace-nowrap normal-case tracking-normal shadow-none",
              activeTab === "post"
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            )}
            onClick={() => { setActiveTab("post"); }}
            variant="ghost"
          >
            {t("ot_approvals.tabs.post_ot", { defaultValue: "Post-OT Approvals (Actuals)" })}
          </Button>
        ) : null}
        {(canApprove || canClaimOnBehalf) ? (
          <Button
            className={cn(
              "flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold transition-all duration-300 whitespace-nowrap normal-case tracking-normal shadow-none",
              activeTab === "delegated_claim"
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            )}
            onClick={() => { setActiveTab("delegated_claim"); }}
            variant="ghost"
          >
            {t("ot_approvals.tabs.delegated_claim", { defaultValue: "Delegated Claims" })}
          </Button>
        ) : null}
        {canManagePolicies ? (
          <Button
            className={cn(
              "flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold transition-all duration-300 whitespace-nowrap normal-case tracking-normal shadow-none",
              activeTab === "behalf_assignments"
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            )}
            onClick={() => { setActiveTab("behalf_assignments"); }}
            variant="ghost"
          >
            {t("ot_approvals.tabs.behalf_assignments", { defaultValue: "Behalf Assignments" })}
          </Button>
        ) : null}
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700 min-h-[400px]">
        {activeTab === "pre" && workspace && canApprove ? (
          <PreOtApprovalsTab wsId={workspace.id} />
        ) : activeTab === "post" && canApprove ? (
          <>
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
          </>
        ) : activeTab === "delegated_claim" && workspace ? (
          <DelegatedClaimsTab
            endDate={endDate}
            startDate={startDate}
            wsId={workspace.id}
          />
        ) : activeTab === "behalf_assignments" && workspace ? (
          <BehalfAssignmentsTab wsId={workspace.id} />
        ) : null}
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

function DelegatedClaimsTab({ wsId, startDate, endDate }: { wsId: string; startDate: string; endDate: string }) {
  const { t } = useTranslation(["common", "attendance"]);
  const toast = useToast();
  const { claimOt, isSubmitting: isClaiming } = useClaimOt();

  const { data: reportResponse, isLoading, refetch } = useQuery({
    queryKey: ["attendance-report", wsId, startDate, endDate, "daily"],
    queryFn: () =>
      fetchAttendanceReport({
        wsId,
        start: startDate,
        end: endDate,
        type: "daily",
        page: 1,
        limit: 1000,
      }),
    enabled: !!wsId,
  });

  const reportData = (reportResponse?.data ?? []) as any[];
  
  // Filter for records with pending employee OT claim and non-zero OT hours
  const claimableRows = reportData.filter(
    (r) => r.ot_claim_status === "pending_employee" && r.computed_ot_hours > 0
  );

  const [selectedRows, setSelectedRows] = useState<Set<string>>(new Set());
  const [reasons, setReasons] = useState<Record<string, string>>({});

  const toggleSelectRow = (key: string) => {
    const next = new Set(selectedRows);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
    }
    setSelectedRows(next);
  };

  const toggleSelectAll = () => {
    if (selectedRows.size === claimableRows.length) {
      setSelectedRows(new Set());
    } else {
      setSelectedRows(new Set(claimableRows.map((r) => `${r.employee_id}-${r.date}`)));
    }
  };

  const handleReasonChange = (key: string, val: string) => {
    setReasons({ ...reasons, [key]: val });
  };

  const handleClaim = async (row: any) => {
    const key = `${row.employee_id}-${row.date}`;
    const reason = reasons[key] || "Delegated OT claim";
    const minutes = Math.round(row.computed_ot_hours * 60);

    try {
      await claimOt({
        employeeId: row.employee_id,
        date: row.date,
        claimed_minutes: minutes,
        reason,
      });
      toast.success(t("ot_approvals.delegated_claim.messages.success", { defaultValue: "Overtime claimed successfully" }));
      setSelectedRows((prev) => {
        const next = new Set(prev);
        next.delete(key);
        return next;
      });
      void refetch();
    } catch (e) {
      console.error(e);
      toast.error(t("ot_approvals.delegated_claim.messages.error", { defaultValue: "Failed to claim overtime" }));
    }
  };

  const handleBulkClaim = async () => {
    if (selectedRows.size === 0) {
      toast.error(t("ot_approvals.delegated_claim.messages.select_at_least_one", { defaultValue: "Please select at least one entry" }));
      return;
    }

    const selectedList = claimableRows.filter((r) =>
      selectedRows.has(`${r.employee_id}-${r.date}`)
    );

    const promises = selectedList.map((row) => {
      const key = `${row.employee_id}-${row.date}`;
      const reason = reasons[key] || "Delegated OT claim";
      const minutes = Math.round(row.computed_ot_hours * 60);
      return claimOt({
        employeeId: row.employee_id,
        date: row.date,
        claimed_minutes: minutes,
        reason,
      });
    });

    try {
      await Promise.allSettled(promises);
      toast.success(t("ot_approvals.delegated_claim.messages.success", { defaultValue: "Overtime claimed successfully" }));
      setSelectedRows(new Set());
      void refetch();
    } catch (e) {
      console.error(e);
      toast.error(t("ot_approvals.delegated_claim.messages.error", { defaultValue: "Failed to claim overtime" }));
    }
  };

  if (isLoading) {
    return <LoadingState />;
  }

  if (claimableRows.length === 0) {
    return (
      <div className="py-40 text-center flex flex-col items-center border-4 border-dashed border-slate-50 dark:border-slate-800 m-8 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30">
        <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-xl border border-slate-100 dark:border-slate-800 mb-8">
          <CheckCircle className="text-emerald-200" size={40} />
        </div>
        <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">
          {t("ot_approvals.delegated_claim.empty_state.title", { defaultValue: "No pending entries to claim" })}
        </Typography>
        <Typography className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-2 opacity-50 italic px-6 max-w-md" variant="p">
          {t("ot_approvals.delegated_claim.empty_state.description", { defaultValue: "All overtime has been claimed, or no overtime hours were computed for the selected period." })}
        </Typography>
      </div>
    );
  }

  return (
    <div className="p-8 space-y-6">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 border-b border-slate-50 dark:border-slate-800 pb-6">
        <div>
          <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200" variant="h3">
            {t("ot_approvals.delegated_claim.title", { defaultValue: "Delegated Claims" })}
          </Typography>
          <Typography className="text-xs text-slate-500 mt-1" variant="p">
            {t("ot_approvals.delegated_claim.subtitle", { defaultValue: "Claim overtime hours on behalf of workshop/production personnel who lack system accounts" })}
          </Typography>
        </div>
        {selectedRows.size > 0 && (
          <Button
            className="normal-case tracking-normal gap-2 h-10 px-6 bg-primary-600 hover:bg-primary-700 text-white rounded-xl font-bold text-sm shadow-md transition-all duration-300 transform active:scale-95 disabled:opacity-50"
            disabled={isClaiming}
            onClick={handleBulkClaim}
            variant="primary"
          >
            {isClaiming ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Check className="w-4 h-4" />
            )}
            {t("ot_approvals.delegated_claim.actions.claim_selected", {
              count: selectedRows.size,
              defaultValue: `Claim ${selectedRows.size} Selected`
            })}
          </Button>
        )}
      </div>

      <div className="overflow-x-auto rounded-3xl border border-slate-100 dark:border-slate-800">
        <table className="w-full border-collapse text-left">
          <thead>
            <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
              <th className="p-4 pl-6 w-12">
                <label className={cn(
                  "relative inline-flex items-center justify-center w-5 h-5 rounded border-2 transition-all cursor-pointer hover:border-primary-400",
                  selectedRows.size === claimableRows.length
                    ? "bg-primary-500 border-primary-500 text-white"
                    : "bg-transparent border-slate-200 dark:border-slate-700 text-transparent"
                )}>
                  <input
                    checked={selectedRows.size === claimableRows.length}
                    className="sr-only"
                    onChange={toggleSelectAll}
                    type="checkbox"
                  />
                  <Check className="w-3.5 h-3.5" strokeWidth={3} />
                </label>
              </th>
              <th className="p-4 font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.delegated_claim.table.employee", { defaultValue: "Employee" })}
              </th>
              <th className="p-4 font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.delegated_claim.table.date", { defaultValue: "Date" })}
              </th>
              <th className="p-4 font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.delegated_claim.table.actual_hours", { defaultValue: "Actual Hours" })}
              </th>
              <th className="p-4 font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.delegated_claim.table.computed_ot", { defaultValue: "System OT" })}
              </th>
              <th className="p-4 font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.delegated_claim.table.reason", { defaultValue: "Claim Reason" })}
              </th>
              <th className="p-4 pr-6 text-right font-bold text-xs text-slate-500 uppercase tracking-wider">
                {t("ot_approvals.table.actions", { defaultValue: "Actions" })}
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
            {claimableRows.map((row) => {
              const key = `${row.employee_id}-${row.date}`;
              const isChecked = selectedRows.has(key);
              return (
                <tr className="hover:bg-slate-50/50 dark:hover:bg-slate-800/20 transition-colors" key={key}>
                  <td className="p-4 pl-6">
                    <label className={cn(
                      "relative inline-flex items-center justify-center w-5 h-5 rounded border-2 transition-all cursor-pointer hover:border-primary-400",
                      isChecked
                        ? "bg-primary-500 border-primary-500 text-white"
                        : "bg-transparent border-slate-200 dark:border-slate-700 text-transparent"
                    )}>
                      <input
                        checked={isChecked}
                        className="sr-only"
                        onChange={() => { toggleSelectRow(key); }}
                        type="checkbox"
                      />
                      <Check className="w-3.5 h-3.5" strokeWidth={3} />
                    </label>
                  </td>
                  <td className="p-4">
                    <div className="flex flex-col">
                      <span className="font-bold text-sm text-slate-800 dark:text-slate-200">
                        {row.employee_name}
                      </span>
                      {!!row.employee_id_number && (
                        <span className="text-[10px] font-mono text-slate-400 dark:text-slate-500 uppercase tracking-tight">
                          {row.employee_id_number}
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="p-4 text-sm text-slate-600 dark:text-slate-400">
                    {row.date}
                  </td>
                  <td className="p-4 text-sm text-slate-600 dark:text-slate-400">
                    {row.computed_std_hours ? `${row.computed_std_hours} hrs` : "-"}
                  </td>
                  <td className="p-4">
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-amber-50 dark:bg-amber-900/20 text-amber-800 dark:text-amber-300">
                      {row.computed_ot_hours} hrs
                    </span>
                  </td>
                  <td className="p-4">
                    <input
                      className="w-full max-w-xs px-3 py-1.5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-lg text-xs text-slate-800 dark:text-slate-200 placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-primary-500 focus:border-primary-500 transition-all"
                      onChange={(e) => { handleReasonChange(key, e.target.value); }}
                      placeholder={t("ot_approvals.delegated_claim.table.reason", { defaultValue: "Enter reason..." })}
                      type="text"
                      value={reasons[key] ?? ""}
                    />
                  </td>
                  <td className="p-4 pr-6 text-right">
                    <Button
                      className="normal-case tracking-normal bg-slate-100 dark:bg-slate-800 hover:bg-primary-50 dark:hover:bg-primary-950/30 hover:text-primary-600 dark:hover:text-primary-400 text-slate-700 dark:text-slate-300 font-bold text-xs transition-all duration-300 disabled:opacity-50 shadow-none border-none"
                      disabled={isClaiming}
                      onClick={() => { void handleClaim(row); }}
                      size="sm"
                      variant="ghost"
                    >
                      {t("ot_approvals.delegated_claim.actions.claim", { defaultValue: "Claim" })}
                    </Button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

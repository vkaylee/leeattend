import { CheckCircle, Clock, FileText, ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";

import { BulkUploadOtDialog } from "@/components/approvals/bulk-upload-ot-dialog";
import { PDFPreviewModal } from "@/components/files/pdf-preview-modal";
import { PageTabs, PageTab } from "@/components/ui/page-tabs";
import { invalidateAttendanceQueries } from "@/lib/utils/query-invalidation";

import { AddPhotoDialog } from "./components/add-photo-dialog";
import { OtApprovalsHeader } from "./components/ot-approvals-header";
import { TabContentRenderer } from "./components/tab-content-renderer";
import { useOtApprovals, getPendingSummaries } from "./hooks/use-ot-approvals";

// ── Loading State ────────────────────────────────────────────────────

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

// ── Page Tabs Config ─────────────────────────────────────────────────

function buildTabs(opts: {
  t: (key: string, opts?: Record<string, unknown>) => string;
  canApprove: boolean;
  canCreateOnBehalf: boolean;
  canClaimOnBehalf: boolean;
  canManagePolicies: boolean;
}): PageTab[] {
  const { t, canApprove, canCreateOnBehalf, canClaimOnBehalf, canManagePolicies } = opts;
  return [
    {
      id: "pre",
      label: t("ot_approvals.tabs.pre_ot", { defaultValue: "Pre-OT Approvals (Requests)" }),
      icon: <Clock size={14} />,
      colorScheme: "primary",
      hidden: !(canApprove || canCreateOnBehalf),
    },
    {
      id: "post",
      label: t("ot_approvals.tabs.post_ot", { defaultValue: "Post-OT Approvals (Actuals)" }),
      icon: <CheckCircle size={14} />,
      colorScheme: "amber",
      hidden: !canApprove,
    },
    {
      id: "delegated_claim",
      label: t("ot_approvals.tabs.delegated_claim", { defaultValue: "Delegated Claims" }),
      icon: <FileText size={14} />,
      colorScheme: "blue",
      hidden: !(canApprove || canClaimOnBehalf),
    },
    {
      id: "behalf_assignments",
      label: t("ot_approvals.tabs.behalf_assignments", { defaultValue: "Behalf Assignments" }),
      icon: <ShieldCheck size={14} />,
      colorScheme: "violet",
      hidden: !canManagePolicies,
    },
  ];
}

// ── Main Page ────────────────────────────────────────────────────────

export default function OtApprovalsPage() {
  const approvals = useOtApprovals();
  const {
    canApprove,
    canClaimOnBehalf,
    canManagePolicies,
    canCreateOnBehalf,
    permissionsLoading,
    currentMonth,
    startDate,
    endDate,
    bulkUploadDialogOpen,
    setBulkUploadDialogOpen,
    photoTargetSummary,
    setPhotoTargetSummary,
    sourceFilter,
    setSourceFilter,
    activeTab,
    setActiveTab,
    previewFileUrl,
    setPreviewFileUrl,
    previewTitle,
    setPreviewTitle,
    isPreviewOpen,
    setIsPreviewOpen,
    summaries,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    isProcessing,
    employmentToEmployeeMap,
    employeeNameMap,
    handleApprove,
    handleMonthYearChange,
    handlePreviousMonth,
    handleNextMonth,
    queryClient,
    organization,
    t,
  } = approvals;

  const tabs = buildTabs({ t, canApprove, canCreateOnBehalf, canClaimOnBehalf, canManagePolicies });

  if (permissionsLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingState />
      </div>
    );
  }

  const pendingSummaries = getPendingSummaries(summaries, sourceFilter, employeeNameMap);

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20 p-8">
      <OtApprovalsHeader
        activeTab={activeTab}
        canApprove={canApprove}
        canCreateOnBehalf={canCreateOnBehalf}
        currentMonth={currentMonth}
        handleMonthYearChange={handleMonthYearChange}
        handleNextMonth={handleNextMonth}
        handlePreviousMonth={handlePreviousMonth}
        setBulkUploadDialogOpen={setBulkUploadDialogOpen}
      />

      <AddPhotoDialog
        employeeId={photoTargetSummary ? employmentToEmployeeMap.get(photoTargetSummary.employment_id) : undefined}
        onOpenChange={(open) => { if (!open) setPhotoTargetSummary(null); }}
        onSuccess={() => { invalidateAttendanceQueries(queryClient); }}
        open={!!photoTargetSummary}
        summary={photoTargetSummary}
      />

      <BulkUploadOtDialog
        onOpenChange={setBulkUploadDialogOpen}
        onSuccess={() => { invalidateAttendanceQueries(queryClient); }}
        open={bulkUploadDialogOpen}
      />

      <PageTabs
        activeTab={activeTab}
        className="mb-8"
        onTabChange={(tabId) => {
          setActiveTab(tabId as "pre" | "post" | "delegated_claim" | "behalf_assignments");
        }}
        tabs={tabs}
      />

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-3xl sm:rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700 min-h-[400px]">
        <TabContentRenderer
          activeTab={activeTab}
          canApprove={canApprove}
          canCreateOnBehalf={canCreateOnBehalf}
          endDate={endDate}
          fetchNextPage={() => { void fetchNextPage(); }}
          handleApprove={handleApprove}
          hasNextPage={hasNextPage}
          isFetchingNextPage={isFetchingNextPage}
          isLoading={isLoading}
          isProcessing={isProcessing}
          onPreviewAttachment={(url, title) => {
            setPreviewFileUrl(url);
            setPreviewTitle(title);
            setIsPreviewOpen(true);
          }}
          pendingSummaries={pendingSummaries}
          setPhotoTargetSummary={setPhotoTargetSummary}
          setSourceFilter={setSourceFilter}
          sourceFilter={sourceFilter}
          startDate={startDate}
          organization={organization?.id ? organization : null}
        />
      </div>

      <PDFPreviewModal
        fileUrl={previewFileUrl}
        isOpen={isPreviewOpen}
        onClose={() => { setIsPreviewOpen(false); }}
        title={previewTitle}
      />
    </div>
  );
}

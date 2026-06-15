import * as Dialog from "@radix-ui/react-dialog";
import { useQuery } from "@tanstack/react-query";
import { Check, CheckCircle, Clock, FileText, Search, ShieldCheck, Upload, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { addMonths, endOfMonth, format, startOfMonth, subMonths } from "date-fns";

import type { AttendanceDailySummary, DailyAttendanceReport, Employee, PaginatedResponse } from "@/lib/api";

import { BulkUploadOtDialog } from "@/components/approvals/bulk-upload-ot-dialog";
import { ConfirmClaimsDialog } from "@/components/approvals/confirm-claims-dialog";
import { ErrorBoundary } from "@/components/common/error-boundary";
import Button from "@/components/ui/button";
import { PageTabs, PageTab } from "@/components/ui/page-tabs";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useApproveOt, useAttendanceSummaries, useEnterPaperOt } from "@/hooks/use-attendance";
import { useMyPermissions } from "@/hooks/use-permissions";
import { useWorkspace } from "@/hooks/use-workspace";
import { fetchAttendanceReport, fetchEmployees } from "@/lib/api";
import { cn } from "@/lib/utils";
import { FileUpload } from "@/components/files/FileUpload";
import { PDFPreviewModal } from "@/components/files/PDFPreviewModal";

import type { OtSummary } from "./components/ot-approvals-table";

import { BehalfAssignmentsTab } from "./components/behalf-assignments-tab";
import { OtApprovalsFilters, SourceFilter } from "./components/ot-approvals-filters";
import { OtApprovalsHeader } from "./components/ot-approvals-header";
import { OtApprovalsTable } from "./components/ot-approvals-table";
import { PreOtApprovalsTab } from "./components/pre-ot-approvals-tab";

type ReportRow = DailyAttendanceReport & { employee_id_number?: string };

function getEmployeeMap(reportData: ReportRow[]): Map<string, { idNumber: string; name: string }> {
  const map = new Map<string, { idNumber: string; name: string }>();
  for (const r of reportData) {
    if (r.employment_id) {
      map.set(r.employment_id, {
        name: r.employee_name || "",
        idNumber: r.employee_id_number ?? "",
      });
    }
  }
  return map;
}

function getPendingSummaries(
  summaries: AttendanceDailySummary[],
  sourceFilter: SourceFilter,
  employeeMap: Map<string, { idNumber: string; name: string }>
): OtSummary[] {
  return summaries
    .filter((s) => s.ot_status === "pending_manager")
    .filter((s) => sourceFilter === "all" || s.entered_via === sourceFilter)
    .map((s) => {
      const emp = employeeMap.get(s.employment_id);
      return {
        ...s,
        employee_name: emp?.name ?? s.employment_id,
        employee_id_number: emp?.idNumber ?? "",
      };
    });
}



export default function OtApprovalsPage() {
  const toast = useToast();
  const { t } = useTranslation(["common", "attendance"]);
  const { workspace } = useWorkspace();
  const { hasPermission, isLoading: permissionsLoading } = useMyPermissions();

  const canApprove = hasPermission("overtime:approve");
  const canClaimOnBehalf = hasPermission("overtime:claim_on_behalf");
  const canManagePolicies = hasPermission("overtime:policies:manage");
  const canCreateOnBehalf = hasPermission("overtime:request:create_on_behalf");

  const tabs: PageTab[] = [
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

  const [currentMonth, setCurrentMonth] = useState(new Date());
  const [bulkUploadDialogOpen, setBulkUploadDialogOpen] = useState(false);
  const [photoTargetSummary, setPhotoTargetSummary] = useState<OtSummary | null>(null);
  const [sourceFilter, setSourceFilter] = useState<SourceFilter>("all");
  const [activeTab, setActiveTab] = useState<"pre" | "post" | "delegated_claim" | "behalf_assignments">("pre");
  const [tabInitialized, setTabInitialized] = useState(false);

  const [previewFileUrl, setPreviewFileUrl] = useState<string | null>(null);
  const [previewTitle, setPreviewTitle] = useState<string>("");
  const [isPreviewOpen, setIsPreviewOpen] = useState<boolean>(false);

  useEffect(() => {
    if (!tabInitialized && !permissionsLoading) {
      setTimeout(() => {
        if (!canApprove && !canCreateOnBehalf && canClaimOnBehalf) {
          setActiveTab("delegated_claim");
        } else if (!canApprove && !canCreateOnBehalf && !canClaimOnBehalf && canManagePolicies) {
          setActiveTab("behalf_assignments");
        }
        setTabInitialized(true);
      }, 0);
    }
  }, [tabInitialized, permissionsLoading, canApprove, canCreateOnBehalf, canClaimOnBehalf, canManagePolicies]);

  const startDate = format(startOfMonth(currentMonth), "yyyy-MM-dd");
  const endDate = format(endOfMonth(currentMonth), "yyyy-MM-dd");

  const { isLoading, refetch, summaries } = useAttendanceSummaries(startDate, endDate);
  const { approveOt, isProcessing } = useApproveOt();

  const { data: employeesData } = useQuery<PaginatedResponse<Employee>>({
    queryKey: ["employees-list-all", workspace?.id],
    queryFn: () => workspace?.id ? fetchEmployees(workspace.id, 1, 1000) : Promise.resolve({ data: [], limit: 0, page: 0, total: 0 }),
    enabled: !!workspace?.id,
  });

  const employmentToEmployeeMap = useMemo(() => {
    const map = new Map<string, string>();
    employeesData?.data.forEach((emp: Employee) => {
      if (emp.active_employment_id) {
        map.set(emp.active_employment_id, emp.id);
      }
    });
    return map;
  }, [employeesData]);

  const { data: reportResponse, isLoading: isReportLoading, refetch: refetchReport } = useQuery({
    queryKey: ["attendance-report", workspace?.id, startDate, endDate, "daily"],
    queryFn: () => {
      if (!workspace?.id) throw new Error("No workspace ID");
      return fetchAttendanceReport({
        wsId: workspace.id,
        start: startDate,
        end: endDate,
        type: "daily",
        page: 1,
        limit: 1000,
      });
    },
    enabled: !!workspace?.id,
  });

  const reportData = useMemo(() => (reportResponse?.data ?? []) as ReportRow[], [reportResponse]);
  const employeeMap = useMemo(() => getEmployeeMap(reportData), [reportData]);

  if (permissionsLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingState />
      </div>
    );
  }

  const pendingSummaries = getPendingSummaries(summaries, sourceFilter, employeeMap);

  const handleApprove = async (
    summary: OtSummary,
    approved: boolean
  ) => {
    const employeeId = employmentToEmployeeMap.get(summary.employment_id);
    if (!employeeId) {
      toast.error("Không tìm thấy thông tin nhân viên hoạt động");
      return;
    }
    try {
      await approveOt({
        approved_minutes: approved ? summary.claimed_ot_minutes : 0,
        date: summary.date,
        employeeId,
        manager_note: approved ? "Approved via Manager Portal" : "Rejected via Manager Portal",
        status: approved ? "approved" : "rejected",
      });
      toast.success(
        approved ? "Overtime approved successfully" : "Overtime rejected successfully"
      );
      void refetch();
      void refetchReport();
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
        onSuccess={() => {
          void refetch();
          void refetchReport();
        }}
        open={!!photoTargetSummary}
        summary={photoTargetSummary}
      />

      <BulkUploadOtDialog
        onOpenChange={setBulkUploadDialogOpen}
        onSuccess={() => {
          void refetch();
        }}
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

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700 min-h-[400px]">
        <TabContentRenderer
          activeTab={activeTab}
          canApprove={canApprove}
          canCreateOnBehalf={canCreateOnBehalf}
          handleApprove={handleApprove}
          isLoading={isLoading}
          isProcessing={isProcessing}
          isReportLoading={isReportLoading}
          pendingSummaries={pendingSummaries}
          refetch={refetch}
          refetchReport={refetchReport}
          reportData={reportData}
          setPhotoTargetSummary={setPhotoTargetSummary}
          setSourceFilter={setSourceFilter}
          sourceFilter={sourceFilter}
          workspace={workspace}
          onPreviewAttachment={(url, title) => {
            setPreviewFileUrl(url);
            setPreviewTitle(title);
            setIsPreviewOpen(true);
          }}
        />
      </div>

      <PDFPreviewModal
        isOpen={isPreviewOpen}
        onClose={() => setIsPreviewOpen(false)}
        fileUrl={previewFileUrl}
        title={previewTitle}
      />
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

interface TabContentRendererProps {
  activeTab: "pre" | "post" | "delegated_claim" | "behalf_assignments";
  canApprove: boolean;
  canCreateOnBehalf: boolean;
  handleApprove: (
    summary: OtSummary,
    approved: boolean
  ) => Promise<void>;
  isLoading: boolean;
  isProcessing: boolean;
  isReportLoading: boolean;
  pendingSummaries: OtSummary[];
  refetch: () => Promise<unknown>;
  refetchReport: () => Promise<unknown>;
  reportData: ReportRow[];
  setPhotoTargetSummary: (summary: OtSummary) => void;
  setSourceFilter: (filter: SourceFilter) => void;
  sourceFilter: SourceFilter;
  workspace: { id: string } | null | undefined;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
}

interface PostOtTabContentProps {
  canApprove: boolean;
  handleApprove: (
    summary: OtSummary,
    approved: boolean
  ) => Promise<void>;
  isLoading: boolean;
  isProcessing: boolean;
  pendingSummaries: OtSummary[];
  setPhotoTargetSummary: (summary: OtSummary) => void;
  setSourceFilter: (filter: SourceFilter) => void;
  sourceFilter: SourceFilter;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
}

function PostOtTabContent({
  canApprove,
  isLoading,
  pendingSummaries,
  sourceFilter,
  setSourceFilter,
  handleApprove,
  isProcessing,
  setPhotoTargetSummary,
  onPreviewAttachment,
}: PostOtTabContentProps) {
  if (!canApprove) return null;
  if (isLoading) return <LoadingState />;
  return (
    <div className="flex flex-col">
      <div className="px-8 pt-8 pb-4 border-b border-slate-50 dark:border-slate-800/60">
        <OtApprovalsFilters setSourceFilter={setSourceFilter} sourceFilter={sourceFilter} />
      </div>
      {pendingSummaries.length === 0 ? (
        <EmptyState />
      ) : (
        <OtApprovalsTable
          handleApprove={handleApprove}
          isProcessing={isProcessing}
          onAddPhoto={setPhotoTargetSummary}
          pendingSummaries={pendingSummaries}
          onPreviewAttachment={onPreviewAttachment}
        />
      )}
    </div>
  );
}

function TabContentRenderer({
  activeTab,
  workspace,
  canApprove,
  canCreateOnBehalf,
  isLoading,
  isReportLoading,
  pendingSummaries,
  reportData,
  handleApprove,
  isProcessing,
  setPhotoTargetSummary,
  setSourceFilter,
  sourceFilter,
  refetch,
  refetchReport,
  onPreviewAttachment,
}: TabContentRendererProps) {
  if (!workspace) return null;

  switch (activeTab) {
    case "pre":
      return (canApprove || canCreateOnBehalf) ? (
        <ErrorBoundary variant="compact">
          <PreOtApprovalsTab 
            canApprove={canApprove} 
            wsId={workspace.id} 
            onPreviewAttachment={onPreviewAttachment}
          />
        </ErrorBoundary>
      ) : null;
    case "post":
      return (
        <ErrorBoundary variant="compact">
          <PostOtTabContent
            canApprove={canApprove}
            handleApprove={handleApprove}
            isLoading={isLoading}
            isProcessing={isProcessing}
            pendingSummaries={pendingSummaries}
            setPhotoTargetSummary={setPhotoTargetSummary}
            setSourceFilter={setSourceFilter}
            sourceFilter={sourceFilter}
            onPreviewAttachment={onPreviewAttachment}
          />
        </ErrorBoundary>
      );
    case "delegated_claim":
      return (
        <ErrorBoundary variant="compact">
          <DelegatedClaimsTab
            isLoading={isReportLoading}
            refetch={async () => {
              await refetch();
              await refetchReport();
            }}
            reportData={reportData}
          />
        </ErrorBoundary>
      );
    case "behalf_assignments":
      return (
        <ErrorBoundary variant="compact">
          <div className="divide-y divide-slate-100 dark:divide-slate-800">
            <BehalfAssignmentsTab assignmentType="ot" wsId={workspace.id} />
            <BehalfAssignmentsTab assignmentType="overtime_request" wsId={workspace.id} />
          </div>
        </ErrorBoundary>
      );
    default:
      return null;
  }
}

function AddPhotoDialog({
  open,
  onOpenChange,
  summary,
  onSuccess,
  employeeId,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  summary: OtSummary | null;
  onSuccess?: () => void;
  employeeId?: string;
}) {
  const { t } = useTranslation();
  const toast = useToast();
  const { workspace } = useWorkspace();
  const { enterPaperOt, isSubmitting } = useEnterPaperOt();
  const [photoUrl, setPhotoUrl] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const wsId = workspace?.id ?? "";

  const handleSubmit = async (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!summary) return;
    if (!photoUrl) {
      setError("Please select and upload a PDF document");
      return;
    }
    if (!employeeId) {
      setError("Không tìm thấy thông tin nhân viên");
      return;
    }
    setError(null);
    try {
      await enterPaperOt({
        employeeId,
        date: summary.date,
        claimed_minutes: summary.claimed_ot_minutes,
        reason: summary.ot_reason ?? "Paper OT Claim",
        paper_form_photo_url: photoUrl,
      });
      setPhotoUrl(null);
      setFileName(null);
      onOpenChange(false);
      onSuccess?.();
      toast.success("Document evidence attached successfully");
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to upload document");
    }
  };

  return (
    <Dialog.Root onOpenChange={(o) => { if (!o) { setPhotoUrl(null); setFileName(null); setError(null); } onOpenChange(o); }} open={open}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40 z-50 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 p-6 shadow-xl rounded-[2rem]">
          <Dialog.Title className="text-xl font-bold text-slate-800 dark:text-slate-100 uppercase tracking-tight flex items-center gap-2">
            <Upload className="w-5 h-5" />
            {t("ot_approvals.evidence.add", { defaultValue: "Add Document Evidence (PDF)" })}
          </Dialog.Title>
          <Dialog.Description className="text-sm text-slate-500 dark:text-slate-400 mt-1">
            {t("ot_approvals.evidence.description", { defaultValue: "Upload a PDF of the signed paper form for this claim." })}
          </Dialog.Description>

          <Button
            className="absolute right-4 top-4 h-auto w-auto p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-700 shadow-none"
            icon={<X className="h-4 w-4" />}
            onClick={() => { onOpenChange(false); }}
            size="icon"
            variant="ghost"
          />

          <form className="mt-6 space-y-5" onSubmit={handleSubmit}>
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
                {t("approvals.paper_ot.photo") || "Paper Form Document (PDF)"}
              </label>
              {photoUrl ? (
                <div className="border border-emerald-100 dark:border-emerald-800/50 bg-emerald-50/50 dark:bg-emerald-950/20 rounded-xl p-4 text-center relative group">
                  <FileText className="w-8 h-8 text-emerald-500 mx-auto mb-2" />
                  <Typography className="text-xs font-semibold text-emerald-800 dark:text-emerald-300 animate-in fade-in" variant="p">
                    {fileName ?? "paper_form_evidence.pdf"}
                  </Typography>
                  <Button
                    className="absolute top-2 right-2 p-1 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-full h-auto w-auto shadow-none"
                    onClick={() => {
                      setPhotoUrl(null);
                      setFileName(null);
                    }}
                    type="button"
                    variant="ghost"
                  >
                    <X size={14} />
                  </Button>
                </div>
              ) : (
                <div className="w-full">
                  <FileUpload
                    workspaceId={wsId}
                    category="OtClaimAttachment"
                    ownerType="AttendanceDailySummary"
                    ownerId={summary?.id ?? ""}
                    accept="application/pdf"
                    maxSizeMB={20}
                    onUploadSuccess={(file) => {
                      setFileName(file.file_name);
                      setPhotoUrl(`/api/v1/workspaces/${wsId}/files/${file.id}/download`);
                    }}
                  />
                </div>
              )}
            </div>

            {!!error && (
              <div className="p-4 bg-rose-50 dark:bg-rose-950/30 border border-rose-100 dark:border-rose-800 rounded-2xl flex items-center gap-3">
                <Typography className="text-[10px] font-bold text-rose-600 dark:text-rose-300 uppercase tracking-widest" variant="p">{error}</Typography>
              </div>
            )}

            <div className="flex justify-end gap-3 pt-2">
              <Button
                className="h-auto px-4 py-2 rounded-xl text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 shadow-none"
                onClick={() => { onOpenChange(false); }}
                type="button"
                variant="ghost"
              >
                {t("common.cancel") || "Cancel"}
              </Button>
              <Button
                className="h-auto px-6 py-2 rounded-xl text-xs font-bold uppercase tracking-wider shadow-lg"
                disabled={isSubmitting}
                loading={isSubmitting}
                type="submit"
                variant="primary"
              >
                {t("common.submit") || "Submit"}
              </Button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

function DelegatedClaimsTab({
  reportData,
  isLoading,
  refetch,
}: {
  reportData: ReportRow[];
  isLoading: boolean;
  refetch: () => Promise<unknown>;
}) {
  const { t } = useTranslation(["common", "attendance"]);
  const toast = useToast();
  const [confirmClaimsOpen, setConfirmClaimsOpen] = useState(false);
  const [activeRowsToClaim, setActiveRowsToClaim] = useState<ReportRow[]>([]);

  const claimableRows = reportData.filter(
    (r) => r.ot_claim_status === "pending_employee" && r.computed_ot_hours > 0
  );

  const [searchQuery, setSearchQuery] = useState("");
  const [selectedRows, setSelectedRows] = useState<Set<string>>(new Set());
  const [reasons, setReasons] = useState<Record<string, string>>({});

  const filteredRows = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    if (!query) return claimableRows;
    return claimableRows.filter((row) => {
      const nameMatch = row.employee_name.toLowerCase().includes(query);
      const idMatch = row.employee_id_number?.toLowerCase().includes(query);
      return nameMatch || idMatch;
    });
  }, [claimableRows, searchQuery]);

  const toggleSelectRow = (key: string) => {
    const next = new Set(selectedRows);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
    }
    setSelectedRows(next);
  };

  const isAllFilteredSelected = useMemo(() => {
    if (filteredRows.length === 0) return false;
    return filteredRows.every((r) => selectedRows.has(`${r.employment_id}-${r.date}`));
  }, [filteredRows, selectedRows]);

  const toggleSelectAll = () => {
    const next = new Set(selectedRows);
    if (isAllFilteredSelected) {
      for (const r of filteredRows) {
        next.delete(`${r.employment_id}-${r.date}`);
      }
    } else {
      for (const r of filteredRows) {
        next.add(`${r.employment_id}-${r.date}`);
      }
    }
    setSelectedRows(next);
  };

  const handleReasonChange = (key: string, val: string) => {
    setReasons({ ...reasons, [key]: val });
  };

  const handleClaim = (row: ReportRow) => {
    setActiveRowsToClaim([row]);
    setConfirmClaimsOpen(true);
  };

  const handleBulkClaim = () => {
    if (selectedRows.size === 0) {
      toast.error(t("ot_approvals.delegated_claim.messages.select_at_least_one", { defaultValue: "Please select at least one entry" }));
      return;
    }

    const selectedList = claimableRows.filter((r) =>
      selectedRows.has(`${r.employment_id}-${r.date}`)
    );
    setActiveRowsToClaim(selectedList);
    setConfirmClaimsOpen(true);
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
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 border-b border-slate-50 dark:border-slate-800 pb-6 w-full">
        <div>
          <Typography className="text-lg font-bold text-slate-800 dark:text-slate-200" variant="h3">
            {t("ot_approvals.delegated_claim.title", { defaultValue: "Delegated Claims" })}
          </Typography>
          <Typography className="text-xs text-slate-500 mt-1" variant="p">
            {t("ot_approvals.delegated_claim.subtitle", { defaultValue: "Claim overtime hours on behalf of workshop/production personnel who lack system accounts" })}
          </Typography>
        </div>
        <div className="flex items-center gap-3">
          {selectedRows.size > 0 && (
            <Button
              className="normal-case tracking-normal gap-2 h-10 px-6 bg-primary-600 hover:bg-primary-700 text-white rounded-xl font-bold text-sm shadow-md transition-all duration-300 transform active:scale-95 disabled:opacity-50 animate-in fade-in"
              onClick={handleBulkClaim}
              variant="primary"
            >
              <Check className="w-4 h-4" />
              {t("ot_approvals.delegated_claim.actions.claim_selected", {
                count: selectedRows.size,
                defaultValue: `Claim ${selectedRows.size} Selected`
              })}
            </Button>
          )}
        </div>
      </div>

      <div className="flex flex-col sm:flex-row gap-4 items-stretch sm:items-center justify-between">
        <div className="relative group max-w-md w-full">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-450 dark:text-slate-500 w-4.5 h-4.5 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" />
          <input
            aria-label={t("common.search", { defaultValue: "Search..." })}
            className="w-full pl-11 pr-4 py-2.5 bg-slate-50/50 dark:bg-slate-800/30 border border-slate-200 dark:border-slate-700/50 rounded-xl outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-500/50 dark:focus:border-primary-400/50 transition-all text-xs font-semibold shadow-sm placeholder-slate-400"
            onChange={(e) => { setSearchQuery(e.target.value); }}
            placeholder={t("ot_approvals.delegated_claim.search_placeholder", { defaultValue: "Search by employee name or ID..." })}
            type="text"
            value={searchQuery}
          />
        </div>
        {!!searchQuery && (
          <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider bg-slate-100/80 dark:bg-slate-800/80 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-800/50 w-fit">
            {t("ot_approvals.delegated_claim.table.filtered_count", {
              count: filteredRows.length,
              defaultValue: `Found ${filteredRows.length} entries`
            })}
          </span>
        )}
      </div>

      <div className="overflow-x-auto rounded-3xl border border-slate-100 dark:border-slate-800">
        <table className="w-full border-collapse text-left">
          <thead>
            <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
              <th className="p-4 pl-6 w-12">
                <label className={cn(
                  "relative inline-flex items-center justify-center w-5 h-5 rounded border-2 transition-all cursor-pointer hover:border-primary-400",
                  isAllFilteredSelected
                    ? "bg-primary-500 border-primary-500 text-white"
                    : "bg-transparent border-slate-200 dark:border-slate-700 text-transparent"
                )}>
                  <input
                    checked={isAllFilteredSelected}
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
            {filteredRows.length === 0 ? (
              <tr>
                <td className="p-8 text-center text-slate-450 dark:text-slate-500 text-xs font-semibold" colSpan={7}>
                  {t("ot_approvals.delegated_claim.no_search_results", { defaultValue: "No matching entries found" })}
                </td>
              </tr>
            ) : (
              filteredRows.map((row) => {
                const key = `${row.employment_id}-${row.date}`;
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
                        onClick={() => { handleClaim(row); }}
                        size="sm"
                        variant="ghost"
                      >
                        {t("ot_approvals.delegated_claim.actions.claim", { defaultValue: "Claim" })}
                      </Button>
                    </td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>

      <ConfirmClaimsDialog
        initialReasons={reasons}
        onOpenChange={setConfirmClaimsOpen}
        onSuccess={() => {
          const claimedKeys = new Set(activeRowsToClaim.map((r) => `${r.employment_id}-${r.date}`));
          setSelectedRows((prev) => {
            const next = new Set(prev);
            for (const key of claimedKeys) {
              next.delete(key);
            }
            return next;
          });
          refetch().catch(console.error);
        }}
        open={confirmClaimsOpen}
        selectedRows={activeRowsToClaim}
      />
    </div>
  );
}

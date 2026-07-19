import { useQuery, useQueryClient } from "@tanstack/react-query";
import { CheckCircle, Clock, FileText, ShieldCheck } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { addMonths, endOfMonth, format, startOfMonth, subMonths } from "date-fns";

import type { AttendanceDailySummary, Employee, PaginatedResponse } from "@/lib/api";

import { BulkUploadOtDialog } from "@/components/approvals/bulk-upload-ot-dialog";
import { ErrorBoundary } from "@/components/common/error-boundary";
import SearchBox from "@/components/common/search-box";
import { PDFPreviewModal } from "@/components/files/pdf-preview-modal";
import { PageTabs, PageTab } from "@/components/ui/page-tabs";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useApproveOt, useAttendanceSummariesInfinite } from "@/hooks/use-attendance";
import { useMyPermissions } from "@/hooks/use-permissions";
import { useWorkspace } from "@/hooks/use-workspace";
import { fetchEmployees } from "@/lib/api";
import { logError } from "@/lib/logger";

import type { OtSummary } from "./components/ot-approvals-table";

import { AddPhotoDialog } from "./components/add-photo-dialog";
import { BehalfAssignmentsTab } from "./components/behalf-assignments-tab";
import { DelegatedClaimsTab } from "./components/delegated-claims-tab";
import { OtApprovalsFilters, SourceFilter } from "./components/ot-approvals-filters";
import { OtApprovalsHeader } from "./components/ot-approvals-header";
import { OtApprovalsTable } from "./components/ot-approvals-table";
import { PreOtApprovalsTab } from "./components/pre-ot-approvals-tab";

function getPendingSummaries(
  summaries: AttendanceDailySummary[],
  sourceFilter: SourceFilter,
  employeeNameMap: Map<string, { idNumber: string; name: string }>
): OtSummary[] {
  return summaries
    .filter((s) => sourceFilter === "all" || s.entered_via === sourceFilter)
    .map((s) => {
      const emp = employeeNameMap.get(s.employment_id);
      return {
        ...s,
        employee_name: emp?.name ?? s.employment_id,
        employee_id_number: emp?.idNumber ?? "",
      };
    });
}

  // eslint-disable-next-line max-lines-per-function
export default function OtApprovalsPage() {
  const toast = useToast();
  const { t } = useTranslation(["common", "attendance"]);
  const { workspace } = useWorkspace();
  const queryClient = useQueryClient();
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

  const {
    summaries,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useAttendanceSummariesInfinite(startDate, endDate, { otStatus: "pending_manager" });
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

  const employeeNameMap = useMemo(() => {
    const map = new Map<string, { idNumber: string; name: string }>();
    employeesData?.data.forEach((emp: Employee) => {
      if (emp.active_employment_id) {
        map.set(emp.active_employment_id, {
          name: emp.full_name,
          idNumber: emp.employee_id_number,
        });
      }
    });
    return map;
  }, [employeesData]);

  if (permissionsLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingState />
      </div>
    );
  }

  const pendingSummaries = getPendingSummaries(summaries, sourceFilter, employeeNameMap);

  const handleApprove = async (
    summary: OtSummary,
    approved: boolean
  ) => {
    const employeeId = employmentToEmployeeMap.get(summary.employment_id);
    if (!employeeId) {
      toast.error(t("ot_approvals.no_active_employee", "Không tìm thấy thông tin nhân viên hoạt động"));
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
      void queryClient.invalidateQueries({ queryKey: ["attendance"] });
      void queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
      void queryClient.invalidateQueries({ queryKey: ["attendance-report-delegated"] });
    } catch (e) {
      logError(e, { component: "OtApprovalsPage" });
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
          void queryClient.invalidateQueries({ queryKey: ["attendance"] });
          void queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
        }}
        open={!!photoTargetSummary}
        summary={photoTargetSummary}
      />

      <BulkUploadOtDialog
        onOpenChange={setBulkUploadDialogOpen}
        onSuccess={() => {
          void queryClient.invalidateQueries({ queryKey: ["attendance"] });
          void queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
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
          workspace={workspace}
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
  endDate: string;
  fetchNextPage: () => void;
  handleApprove: (
    summary: OtSummary,
    approved: boolean
  ) => Promise<void>;
  hasNextPage: boolean | undefined;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  isProcessing: boolean;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
  pendingSummaries: OtSummary[];
  setPhotoTargetSummary: (summary: OtSummary) => void;
  setSourceFilter: (filter: SourceFilter) => void;
  sourceFilter: SourceFilter;
  startDate: string;
  workspace: { id: string } | null | undefined;
}

interface PostOtTabContentProps {
  canApprove: boolean;
  fetchNextPage: () => void;
  handleApprove: (
    summary: OtSummary,
    approved: boolean
  ) => Promise<void>;
  hasNextPage: boolean | undefined;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  isProcessing: boolean;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
  pendingSummaries: OtSummary[];
  setPhotoTargetSummary: (summary: OtSummary) => void;
  setSourceFilter: (filter: SourceFilter) => void;
  sourceFilter: SourceFilter;
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
  fetchNextPage,
  hasNextPage,
  isFetchingNextPage,
}: PostOtTabContentProps) {
  const { t } = useTranslation(["common", "attendance"]);
  const [searchQuery, setSearchQuery] = useState("");

  const filteredSummaries = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return pendingSummaries;
    return pendingSummaries.filter((s) => {
      const name = (s.employee_name ?? "").toLowerCase();
      const id = (s.employee_id_number ?? "").toLowerCase();
      const reason = (s.ot_reason ?? "").toLowerCase();
      return name.includes(q) || id.includes(q) || reason.includes(q);
    });
  }, [pendingSummaries, searchQuery]);

  if (!canApprove) return null;
  if (isLoading) return <LoadingState />;

  return (
    <div className="flex flex-col">
      <div className="px-8 pt-8 pb-4 border-b border-slate-50 dark:border-slate-800/60 space-y-4">
        <div className="flex flex-col sm:flex-row gap-4 items-stretch sm:items-center justify-between">
          <SearchBox
            className="max-w-md w-full"
            onChange={setSearchQuery}
            placeholder={t("ot_approvals.post.search_placeholder", { defaultValue: "Search by employee name, ID or reason..." })}
            value={searchQuery}
          />
          {!!searchQuery && (
            <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider bg-slate-100/80 dark:bg-slate-800/80 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-800/50 w-fit shrink-0">
              {t("ot_approvals.delegated_claim.table.filtered_count", {
                count: filteredSummaries.length,
                defaultValue: `Found ${filteredSummaries.length} entries`
              })}
            </span>
          )}
        </div>
        <OtApprovalsFilters setSourceFilter={setSourceFilter} sourceFilter={sourceFilter} />
      </div>
      {!isFetchingNextPage && pendingSummaries.length === 0 ? (
        <EmptyState />
      ) : pendingSummaries.length > 0 && filteredSummaries.length === 0 ? (
        <div className="py-20 text-center flex flex-col items-center">
          <Typography className="text-sm font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider" variant="p">
            {t("ot_approvals.delegated_claim.no_search_results", { defaultValue: "No matching entries found" })}
          </Typography>
        </div>
      ) : (
        <OtApprovalsTable
          fetchNextPage={fetchNextPage}
          handleApprove={handleApprove}
          hasNextPage={hasNextPage}
          isFetchingNextPage={isFetchingNextPage}
          isProcessing={isProcessing}
          onAddPhoto={setPhotoTargetSummary}
          onPreviewAttachment={onPreviewAttachment}
          pendingSummaries={filteredSummaries}
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
  pendingSummaries,
  handleApprove,
  isProcessing,
  setPhotoTargetSummary,
  setSourceFilter,
  sourceFilter,
  startDate,
  endDate,
  fetchNextPage,
  hasNextPage,
  isFetchingNextPage,
  onPreviewAttachment,
}: TabContentRendererProps) {
  if (!workspace) return null;

  switch (activeTab) {
    case "pre":
      return (canApprove || canCreateOnBehalf) ? (
        <ErrorBoundary variant="compact">
          <PreOtApprovalsTab
            canApprove={canApprove}
            onPreviewAttachment={onPreviewAttachment}
            wsId={workspace.id}
          />
        </ErrorBoundary>
      ) : null;
    case "post":
      return (
        <ErrorBoundary variant="compact">
          <PostOtTabContent
            canApprove={canApprove}
            fetchNextPage={fetchNextPage}
            handleApprove={handleApprove}
            hasNextPage={hasNextPage}
            isFetchingNextPage={isFetchingNextPage}
            isLoading={isLoading}
            isProcessing={isProcessing}
            onPreviewAttachment={onPreviewAttachment}
            pendingSummaries={pendingSummaries}
            setPhotoTargetSummary={setPhotoTargetSummary}
            setSourceFilter={setSourceFilter}
            sourceFilter={sourceFilter}
          />
        </ErrorBoundary>
      );
    case "delegated_claim":
      return (
        <ErrorBoundary variant="compact">
          <DelegatedClaimsTab
            endDate={endDate}
            startDate={startDate}
            wsId={workspace.id}
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

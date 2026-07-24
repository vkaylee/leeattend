import { ErrorBoundary } from "@/components/common/error-boundary";

import { BehalfAssignmentsTab } from "./behalf-assignments-tab";
import { DelegatedClaimsTab } from "./delegated-claims-tab";
import { PostOtTabContent } from "./post-ot-tab-content";
import { PreOtApprovalsTab } from "./pre-ot-approvals-tab";
import type { SourceFilter } from "./ot-approvals-filters";
import type { OtSummary } from "./ot-approvals-table";

interface TabContentRendererProps {
  activeTab: "pre" | "post" | "delegated_claim" | "behalf_assignments";
  canApprove: boolean;
  canCreateOnBehalf: boolean;
  endDate: string;
  fetchNextPage: () => void;
  handleApprove: (summary: OtSummary, approved: boolean) => Promise<void>;
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

export function TabContentRenderer({
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

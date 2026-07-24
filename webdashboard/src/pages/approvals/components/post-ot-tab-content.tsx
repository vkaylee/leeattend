import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import SearchBox from "@/components/common/search-box";
import { Typography } from "@/components/ui/typography";

import type { OtSummary } from "./ot-approvals-table";

import { OtApprovalsFilters, SourceFilter } from "./ot-approvals-filters";
import { OtApprovalsTable } from "./ot-approvals-table";

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

// ── Empty State ──────────────────────────────────────────────────────

import { CheckCircle } from "lucide-react";

function EmptyState() {
  const { t } = useTranslation();
  return (
    <div className="py-40 text-center flex flex-col items-center border-4 border-dashed border-slate-50 dark:border-slate-800 mx-2 sm:m-8 rounded-3xl sm:rounded-[3rem] bg-slate-50 dark:bg-slate-800/30">
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

// ── Post OT Tab Content ──────────────────────────────────────────────

interface PostOtTabContentProps {
  canApprove: boolean;
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
}

export function PostOtTabContent({
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

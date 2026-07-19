import { useInfiniteQuery, useQueryClient } from "@tanstack/react-query";
import { CheckCircle, ArrowRight, Check, X, Paperclip } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type { PaginatedResponse } from "@/lib/api";
import type { OvertimeRequest } from "@/lib/api/overtime-requests";

import SearchBox from "@/components/common/search-box";
import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useProcessApproval } from "@/hooks/use-approval-workflows";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { overtimeRequestsApi } from "@/lib/api/overtime-requests";

const PRE_OT_PAGE_SIZE = 20;

  // eslint-disable-next-line max-lines-per-function
export function PreOtApprovalsTab({
  wsId,
  canApprove,
  onPreviewAttachment,
}: {
  wsId: string;
  canApprove: boolean;
  onPreviewAttachment?: (fileUrl: string, title: string) => void;
}) {
  const { t } = useTranslation(["common", "attendance"]);
  const toast = useToast();
  const queryClient = useQueryClient();

  const {
    data: paginatedData,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery<PaginatedResponse<OvertimeRequest>>({
    queryKey: ["overtime-requests", wsId, "list", { limit: PRE_OT_PAGE_SIZE }],
    queryFn: ({ pageParam }) => {
      return overtimeRequestsApi.listRequests(wsId, {
        page: pageParam as number,
        limit: PRE_OT_PAGE_SIZE,
      });
    },
    getNextPageParam: (lastPage, allPages) => {
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data?.length || 0), 0);
      return totalLoaded >= (lastPage.total || 0) ? undefined : allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId,
  });

  const allRequests = useMemo(
    () => paginatedData?.pages.flatMap((p) => p.data) ?? [],
    [paginatedData]
  );

  const { mutate: processApproval, isPending: isProcessing } = useProcessApproval();

  const [searchQuery, setSearchQuery] = useState("");

  // Filter for pending status, then apply client-side search
  const pendingRequests = useMemo(
    () => allRequests.filter(req => req.status === "pending_approval" || req.status === "draft"),
    [allRequests]
  );

  const filteredRequests = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return pendingRequests;
    return pendingRequests.filter((req) => {
      const employeeNames = (req.employees ?? [])
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        .map(e => `${e.full_name ?? ""} ${e.employee_id ?? ""}`.toLowerCase())
        .join(" ");
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
      const reason = (req.reason ?? "").toLowerCase();
      return employeeNames.includes(q) || reason.includes(q);
    });
  }, [pendingRequests, searchQuery]);

  const loadMoreRef = useInfiniteScroll<HTMLDivElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage: () => { void fetchNextPage(); },
  });

  const handleAction = (id: string, action: "approve" | "reject") => {
    processApproval(
      {
        entity_type: "ot_request",
        entity_id: id,
        action,
        note: action === "approve" ? t("ot_approvals.pre.approved_note") : t("ot_approvals.pre.rejected_note"),
      },
      {
        onSuccess: () => {
          toast.success(action === "approve" ? t("ot_approvals.pre.success_approve") : t("ot_approvals.pre.success_reject"));
          void queryClient.invalidateQueries({ queryKey: ["overtime-requests", wsId, "list"] });
        },
        onError: () => {
          toast.error(t("ot_approvals.pre.fail_process"));
        }
      }
    );
  };

  if (isLoading) {
    return (
      <div className="p-32 flex flex-col items-center gap-6 animate-pulse">
        <div className="w-12 h-12 border-4 border-indigo-600 border-t-transparent rounded-full animate-spin"></div>
        <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em]">
          {t("common.loading")}
        </span>
      </div>
    );
  }

  if (!isFetchingNextPage && pendingRequests.length === 0) {
    return (
      <div className="py-40 text-center flex flex-col items-center border-4 border-dashed border-slate-50 dark:border-slate-800 m-8 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30">
        <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-xl border border-slate-100 dark:border-slate-800 mb-8">
          <CheckCircle className="text-emerald-200" size={40} />
        </div>
        <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">
          {t("ot_approvals.pre.no_pending_title")}
        </Typography>
        <Typography className="text-slate-500 dark:text-slate-400 text-[10px] font-bold uppercase tracking-widest mt-2 opacity-50 italic" variant="p">
          {t("ot_approvals.pre.no_pending_desc")}
        </Typography>
      </div>
    );
  }

  return (
    <div className="space-y-4 m-8">
      <div className="flex flex-col sm:flex-row gap-4 items-stretch sm:items-center justify-between">
        <SearchBox
          className="max-w-md w-full"
          onChange={setSearchQuery}
          placeholder={t("ot_approvals.pre.search_placeholder", { defaultValue: "Search by employee name or reason..." })}
          value={searchQuery}
        />
        {!!searchQuery && (
          <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider bg-slate-100/80 dark:bg-slate-800/80 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-800/50 w-fit shrink-0">
            {t("ot_approvals.delegated_claim.table.filtered_count", {
              count: filteredRequests.length,
              defaultValue: `Found ${filteredRequests.length} entries`
            })}
          </span>
        )}
      </div>

      {filteredRequests.length === 0 ? (
        <div className="py-20 text-center flex flex-col items-center">
          <Typography className="text-sm font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider">
            {t("ot_approvals.delegated_claim.no_search_results", { defaultValue: "No matching entries found" })}
          </Typography>
        </div>
      ) : (
        filteredRequests.map((req) => (
        <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-6 flex flex-col md:flex-row gap-6 justify-between shadow-sm hover:shadow-md transition-shadow" key={req.id}>
          <div className="space-y-3 flex-1">
            <div className="flex items-center gap-3">
              <span className="px-3 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400">
                {t("ot_approvals.pre.pending_status")}
              </span>
              <Typography className="font-bold text-lg text-slate-800 dark:text-slate-100">
                {req.employees && req.employees.length > 0 ? req.employees.map(emp => emp.full_name ?? emp.employee_id).join(", ") : t("ot_approvals.pre.employees")}
              </Typography>
            </div>
 
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 bg-slate-50 dark:bg-slate-800/50 p-4 rounded-2xl">
              <div>
                <span className="block text-[10px] font-bold text-slate-500 uppercase">{t("common.date")}</span>
                <span className="text-sm font-bold text-slate-700 dark:text-slate-300">{new Date(req.planned_date).toLocaleDateString()}</span>
              </div>
              <div>
                <span className="block text-[10px] font-bold text-slate-500 uppercase">{t("common.time")}</span>
                <span className="text-sm font-bold text-slate-700 dark:text-slate-300">{req.planned_start_time} - {req.planned_end_time}</span>
              </div>
              <div>
                <span className="block text-[10px] font-bold text-slate-500 uppercase">{t("common.duration")}</span>
                <span className="text-sm font-bold text-slate-700 dark:text-slate-300">{Math.round(req.planned_minutes / 60 * 10) / 10} hrs</span>
              </div>
              <div>
                <span className="block text-[10px] font-bold text-slate-500 uppercase">{t("common.reason")}</span>
                <span className="text-sm font-bold text-slate-700 dark:text-slate-300 truncate max-w-[150px] inline-block" title={req.reason || ""}>
                  {req.reason || t("ot_approvals.pre.no_reason")}
                </span>
              </div>
            </div>
            {!!req.attachment_file_ids && req.attachment_file_ids.length > 0 && (
              <div className="flex flex-wrap gap-2 mt-3">
                {req.attachment_file_ids.map((fileId: string | null, index: number) => {
                  if (!fileId) return null;
                  return (
                    <Button
                      className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[10px] font-bold bg-indigo-50 hover:bg-indigo-100 text-indigo-750 dark:bg-indigo-950/30 dark:hover:bg-indigo-900/40 dark:text-indigo-400 border border-indigo-150 dark:border-indigo-900/50 transition-colors cursor-pointer"
                      icon={<Paperclip className="w-3 h-3" />}
                      key={fileId}
                      onClick={() => { onPreviewAttachment?.(`/api/v1/workspaces/${wsId}/files/${fileId}/download`, `Pre-OT Attachment #${index + 1}`); }}
                      type="button"
                    >
                      Doc #{index + 1}
                    </Button>
                  );
                })}
              </div>
            )}
          </div>
          
          <div className="flex flex-col items-end justify-center gap-2 shrink-0">
            {canApprove ? (
              <div className="flex gap-2">
                <Button
                  className="normal-case tracking-normal px-4 py-2 bg-emerald-50 text-emerald-600 hover:bg-emerald-100 hover:text-emerald-700 dark:bg-emerald-950/20 dark:text-emerald-300 dark:hover:bg-emerald-900/40 dark:hover:text-emerald-200 border-none flex items-center gap-1.5 rounded-xl font-bold text-xs shadow-none h-auto transition-colors"
                  disabled={isProcessing}
                  onClick={() => { handleAction(req.id, "approve"); }}
                  variant="ghost"
                >
                  <Check className="w-3.5 h-3.5" />
                  {t("common.approve")}
                </Button>
                <Button
                  className="normal-case tracking-normal px-4 py-2 bg-rose-50 text-rose-600 hover:bg-rose-100 hover:text-rose-700 dark:bg-rose-950/20 dark:text-rose-300 dark:hover:bg-rose-900/40 dark:hover:text-rose-200 border-none flex items-center gap-1.5 rounded-xl font-bold text-xs shadow-none h-auto transition-colors"
                  disabled={isProcessing}
                  onClick={() => { handleAction(req.id, "reject"); }}
                  variant="ghost"
                >
                  <X className="w-3.5 h-3.5" />
                  {t("common.reject")}
                </Button>
              </div>
            ) : null}
            {!!req.workflow_id && (
              <span className="text-[10px] text-slate-400 font-medium flex items-center gap-1">
                {t("ot_approvals.pre.workflow_configured")} <ArrowRight className="w-3 h-3" />
              </span>
            )}
          </div>
        </div>
      )))}

      {/* Infinite scroll sentinel */}
      <div ref={loadMoreRef}>
        {isFetchingNextPage ? (
          <div className="flex items-center justify-center gap-3 py-6">
            <div className="w-5 h-5 border-2 border-indigo-600 border-t-transparent rounded-full animate-spin" />
            <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
              {t("common.loading_more", { defaultValue: "Loading more..." })}
            </span>
          </div>
        ) : null}
      </div>
    </div>
  );
}

import { useQuery, useInfiniteQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, useCallback, useRef, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";

import type { LeaveRequestsSearch } from "@/components/features/leave/search-schema";
import type { LeaveImportJob } from "@/lib/api/leaves";

import { useToast } from "@/components/ui/toast";
import { useAllEmployees } from "@/hooks/use-employees";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import {
  fetchWorkspaceLeaveRequests,
  approveLeaveRequest,
  fetchLeaveTypes,
  fetchLatestLeaveImport,
  PaginatedResponse,
  LeaveRequest,
} from "@/lib/api";
import { logError } from "@/lib/logger";
import { useSearchPatch } from "@/lib/state";
import { Route } from "@/routes/_authenticated/_workspace/leave-requests";

export type TabType = LeaveRequestsSearch["tab"];
export type LeaveStatusFilter = LeaveRequestsSearch["status"];

const PAGE_SIZE = 20;

export function useLeaveRequests(wsId: string | null) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  const search = Route.useSearch();
  const patchSearch = useSearchPatch<LeaveRequestsSearch>();

  const activeTab = search.tab;
  const statusFilter = search.status;
  const employeeId = search.employee ?? "all";

  const setActiveTab = useCallback((tab: TabType) => { patchSearch({ tab }); }, [patchSearch]);
  const setStatusFilter = useCallback((s: LeaveStatusFilter) => { patchSearch({ status: s }); }, [patchSearch]);
  const setEmployeeId = useCallback((id: string) => { patchSearch({ employee: id === "all" ? undefined : id }); }, [patchSearch]);

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isImportModalOpen, setIsImportModalOpen] = useState(false);

  const { employees } = useAllEmployees(wsId);

  const {
    data: paginatedRequests,
    isLoading: requestsLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery<PaginatedResponse<LeaveRequest>>({
    queryKey: ["leave-requests", wsId, activeTab, employeeId, statusFilter],
    queryFn: ({ pageParam }) => {
      return fetchWorkspaceLeaveRequests(wsId ?? "", {
        status: activeTab !== "behalf_assignments" && statusFilter !== "all" ? statusFilter : undefined,
        employee_id: employeeId,
        page: pageParam as number,
        limit: PAGE_SIZE,
      });
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
      return totalLoaded >= (lastPage.total || 0) ? undefined : allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId && activeTab !== "behalf_assignments",
  });

  const requests = useMemo(
    () => paginatedRequests?.pages.flatMap((p) => p.data) ?? [],
    [paginatedRequests]
  );
  const requestsTotal = paginatedRequests?.pages[0]?.total ?? 0;

  const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage: () => { void fetchNextPage(); },
  });

  const { data: leaveTypes = [] } = useQuery({ queryKey: ["leave-types", wsId], queryFn: () => fetchLeaveTypes(wsId ?? ""), enabled: !!wsId });

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: "cancelled" | "pending" | "approved" | "rejected" | "pre_approved" }) => approveLeaveRequest(wsId ?? "", data.id, { status: data.status }),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: ["leave-requests"] }); toast.success(t("common.success")); },
  });

  const [importJob, setImportJob] = useState<LeaveImportJob | null>(null);
  const [importBannerDismissed, setImportBannerDismissedState] = useState(false);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const setImportBannerDismissed = useCallback((dismissed: boolean) => {
    if (dismissed && importJob && wsId) {
      localStorage.setItem(`dismissed_leave_import_job_${wsId}`, importJob.id);
    }
    setImportBannerDismissedState(dismissed);
  }, [importJob, wsId]);

  useEffect(() => {
    if (!wsId) return;
    void fetchLatestLeaveImport(wsId).then((job) => {
      if (job) {
        const created = new Date(job.created_at);
        const now = new Date();
        if (now.getTime() - created.getTime() < 24 * 60 * 60 * 1000) {
          setImportJob(job);
          const dismissedId = localStorage.getItem(`dismissed_leave_import_job_${wsId}`);
          setImportBannerDismissedState(dismissedId === job.id);
        }
      }
    }).catch((e: unknown) => { logError(e, { component: 'useLeaveRequests.fetchLatest' }); });
  }, [wsId]);

  const startPolling = useCallback((wsIdValue: string) => {
    if (pollingRef.current) clearInterval(pollingRef.current);
    pollingRef.current = setInterval(() => {
      void (async () => {
        try {
          const job = await fetchLatestLeaveImport(wsIdValue);
          if (job) {
            setImportJob(job);
            if (job.status !== "processing" && job.status !== "pending") {
              if (pollingRef.current) clearInterval(pollingRef.current);
              pollingRef.current = null;
              void queryClient.invalidateQueries({ queryKey: ["leave-requests"] });
            }
          }
        } catch (e: unknown) {
          // Network error — keep polling; log at debug level for observability
          logError(e, { component: 'useLeaveRequests.polling' });
        }
      })();
    }, 3000);
  }, [queryClient]);

  useEffect(() => {
    if (importJob?.status === "processing" || importJob?.status === "pending") {
      if (wsId) startPolling(wsId);
    }
    return () => {
      if (pollingRef.current) clearInterval(pollingRef.current);
    };
  }, [importJob?.status, wsId, startPolling]);

  const handleImportComplete = useCallback(() => {
    if (wsId) {
      void fetchLatestLeaveImport(wsId).then((job) => {
        if (job) {
          setImportJob(job);
          setImportBannerDismissedState(false);
        }
      }).catch((e: unknown) => { logError(e, { component: 'useLeaveRequests.importComplete' }); });
      void queryClient.invalidateQueries({ queryKey: ["leave-requests"] });
    }
  }, [wsId, queryClient]);

  return {
    isModalOpen, setIsModalOpen,
    isImportModalOpen, setIsImportModalOpen,
    activeTab, setActiveTab,
    statusFilter, setStatusFilter,
    employeeId, setEmployeeId,
    employees, requests, requestsLoading, requestsTotal, leaveTypes,
    hasNextPage, isFetchingNextPage, loadMoreRef,
    approveMutation,
    importJob, setImportJob,
    importBannerDismissed, setImportBannerDismissed,
    handleImportComplete,
    t
  };
}

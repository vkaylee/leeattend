import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useDebounce } from "@/hooks/use-debounce";
import { usePagination } from "@/hooks/use-pagination";
import { fetchWorkspaceLeaveBalances, getCurrentWorkspaceId, triggerAccrualRun, fetchJobStatus } from "@/lib/api";
import { WorkspaceLeaveBalance } from "@/lib/api/types";
import { logError } from "@/lib/logger";

export function useLeaveBalances() {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId();
  const [searchTerm, setSearchTerm] = useState("");
  const { page, setPage, reset } = usePagination({ initialPageSize: 20 });
  const [selectedYear, setSelectedYear] = useState(new Date().getFullYear());
  const { confirm, ConfirmationDialog } = useConfirm();
  const limit = 20;
  const debouncedSearch = useDebounce(searchTerm, 500);

  const [selectedEmployee, setSelectedEmployee] = useState<{ id: string, name: string, leaveTypeId?: string, leaveTypeName?: string } | null>(null);
  const [selectedBalance, setSelectedBalance] = useState<WorkspaceLeaveBalance | null>(null);

  const { data: paginatedData, isLoading } = useQuery({
    queryKey: ["workspace-leave-balances", wsId, page, debouncedSearch, selectedYear],
    queryFn: () => {
      if (!wsId) throw new Error("Workspace ID is required");
      return fetchWorkspaceLeaveBalances(wsId, { page, limit, q: debouncedSearch, year: selectedYear });
    },
    enabled: !!wsId,
    placeholderData: (previousData) => previousData
  });

  const pollJobStatus = useCallback(async (jobId: string) => {
    const POLL_INTERVAL = 2000;
    const MAX_POLLS = 60;

    for (let index = 0; index < MAX_POLLS; index++) {
      await new Promise(resolve => setTimeout(resolve, POLL_INTERVAL));
      try {
        if (!wsId) return;
        const status = await fetchJobStatus(wsId, jobId);
        if (status.status === 'completed') {
          void queryClient.invalidateQueries({ queryKey: ["workspace-leave-balances"] });
          toast.success(t("leave_balances.accrual_completed"));
          return;
        }
        if (status.status === 'failed') {
          toast.error(status.error_message ?? t("common.error"));
          return;
        }
      } catch (e: unknown) {
        logError(e, { component: 'useLeaveBalances.pollJobStatus' });
        toast.error(t("common.error"));
        return;
      }
    }
    toast.error(t("leave_balances.accrual_timeout"));
  }, [wsId, queryClient, toast, t]);

  const accrualMutation = useMutation({
    mutationFn: () => {
      if (!wsId) throw new Error("Workspace ID is required");
      return triggerAccrualRun(wsId, { date: new Date().toISOString().split('T')[0] });
    },
    onSuccess: (data) => {
      if (data.status === 'queued' && data.job_id) {
        toast.success(t("leave_balances.accrual_queued"));
        void pollJobStatus(data.job_id);
      } else {
        void queryClient.invalidateQueries({ queryKey: ["workspace-leave-balances"] });
        toast.success(data.message || t("common.success"));
      }
    },
    onError: () => { toast.error(t("common.error")); }
  });

  const handleRunAccrual = async () => {
    const ok = await confirm({
      title: t("leave_balances.run_accrual"),
      message: t("leave_balances.confirm_accrual"),
      variant: "warning"
    });
    if (ok) accrualMutation.mutate();
  };

  const balances = paginatedData?.data ?? [];
  const totalItems = paginatedData?.total ?? 0;
  const totalPages = Math.ceil(totalItems / limit);

  return {
    t,
    wsId,
    searchTerm,
    setSearchTerm,
    page,
    setPage,
    resetPagination: reset,
    selectedYear,
    setSelectedYear,
    selectedEmployee,
    setSelectedEmployee,
    selectedBalance,
    setSelectedBalance,
    isLoading,
    balances,
    totalItems,
    totalPages,
    limit,
    accrualMutation,
    handleRunAccrual,
    ConfirmationDialog
  };
}

import { useState, useCallback, useRef, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import type { LeaveImportJob } from "@/lib/api/leaves";

import { useToast } from "@/components/ui/toast";
import { useAllEmployees } from "@/hooks/use-employees";
import {
  fetchWorkspaceLeaveRequests,
  approveLeaveRequest,
  fetchLeaveTypes,
  fetchLatestLeaveImport,
} from "@/lib/api";

type TabType = "pending" | "approved" | "rejected";

export function useLeaveRequests(wsId: string | null) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isImportModalOpen, setIsImportModalOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>("pending");

  const { employees } = useAllEmployees(wsId);
  const { data: requests = [], isLoading: requestsLoading } = useQuery({ queryKey: ["leave-requests", wsId, activeTab], queryFn: () => fetchWorkspaceLeaveRequests(wsId!, activeTab), enabled: !!wsId });
  const { data: leaveTypes = [] } = useQuery({ queryKey: ["leave-types", wsId], queryFn: () => fetchLeaveTypes(wsId!), enabled: !!wsId });

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: string }) => approveLeaveRequest(wsId!, data.id, { status: data.status }),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: ["leave-requests"] }); toast.success(t("common.success")); },
  });

  const [importJob, setImportJob] = useState<LeaveImportJob | null>(null);
  const [importBannerDismissed, setImportBannerDismissed] = useState(false);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (!wsId) return;
    void fetchLatestLeaveImport(wsId).then((job) => {
      if (job) {
        const created = new Date(job.created_at);
        const now = new Date();
        if (now.getTime() - created.getTime() < 24 * 60 * 60 * 1000) {
          setImportJob(job);
          setImportBannerDismissed(false);
        }
      }
    }).catch(() => { /* empty */ });
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
        } catch {
          // Network error — keep polling
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
          setImportBannerDismissed(false);
        }
      }).catch(() => { /* empty */ });
      void queryClient.invalidateQueries({ queryKey: ["leave-requests"] });
    }
  }, [wsId, queryClient]);

  return {
    isModalOpen, setIsModalOpen,
    isImportModalOpen, setIsImportModalOpen,
    activeTab, setActiveTab,
    employees, requests, requestsLoading, leaveTypes,
    approveMutation,
    importJob, setImportJob,
    importBannerDismissed, setImportBannerDismissed,
    handleImportComplete,
    t
  };
}

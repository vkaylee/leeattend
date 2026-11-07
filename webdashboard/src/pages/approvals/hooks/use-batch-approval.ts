import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import {
  approveAttendanceAdjustment,
  approveLeaveRequest,
  getStandaloneOrganizationId,
} from "@/lib/api";
import { approveShiftSwap } from "@/lib/api/shift-swaps";
import { approveOt } from "@/lib/api/attendance-overtime";
import { logError } from "@/lib/logger";

import {
  BatchApprovalResult,
  UnifiedApprovalItem,
} from "../types/unified-approvals";

export interface UseBatchApprovalReturn {
  isProcessing: boolean;
  batchApprove: (items: UnifiedApprovalItem[]) => Promise<BatchApprovalResult>;
  batchReject: (
    items: UnifiedApprovalItem[],
    reason: string
  ) => Promise<BatchApprovalResult>;
}

export function useBatchApproval(): UseBatchApprovalReturn {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();
  const [isProcessing, setIsProcessing] = useState(false);

  const invalidateAll = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ["unified-inbox-leaves"] }),
      queryClient.invalidateQueries({
        queryKey: ["unified-inbox-adjustments"],
      }),
      queryClient.invalidateQueries({ queryKey: ["unified-inbox-ot"] }),
      queryClient.invalidateQueries({ queryKey: ["unified-inbox-swaps"] }),
      queryClient.invalidateQueries({ queryKey: ["leave-requests"] }),
      queryClient.invalidateQueries({ queryKey: ["attendance-adjustments"] }),
      queryClient.invalidateQueries({ queryKey: ["shift-swaps"] }),
      queryClient.invalidateQueries({
        queryKey: ["attendance-daily-summaries"],
      }),
    ]);
  };

  const executeItemAction = async (
    wsId: string,
    item: UnifiedApprovalItem,
    action: "approve" | "reject",
    reason?: string
  ): Promise<void> => {
    switch (item.type) {
      case "leave":
        await approveLeaveRequest(wsId, item.id, {
          status: action === "approve" ? "approved" : "rejected",
        });
        break;

      case "adjustment":
        await approveAttendanceAdjustment(wsId, item.id, {
          status: action === "approve" ? "approved" : "rejected",
        });
        break;

      case "shift_swap":
        await approveShiftSwap(wsId, item.id, {
          status: action === "approve" ? "approved" : "rejected",
        });
        break;

      case "overtime": {
        const minutes =
          typeof item.details.rawPayload?.claimed_minutes === "number"
            ? item.details.rawPayload.claimed_minutes
            : 60;
        await approveOt(wsId, item.requester.id, {
          date: item.targetDate,
          approved_minutes: action === "approve" ? minutes : 0,
          status: action === "approve" ? "approved" : "rejected",
          manager_note: reason,
        });
        break;
      }

      default:
        throw new Error(`Unsupported approval type: ${item.type}`);
    }
  };

  const processBatch = async (
    items: UnifiedApprovalItem[],
    action: "approve" | "reject",
    reason?: string
  ): Promise<BatchApprovalResult> => {
    const wsId = getStandaloneOrganizationId();
    if (!wsId) {
      toast.error(t("common.select_workspace", "Vui lòng chọn Workspace"));
      return { succeeded: [], failed: [] };
    }

    if (items.length === 0) {
      return { succeeded: [], failed: [] };
    }

    setIsProcessing(true);
    const succeeded: string[] = [];
    const failed: { id: string; error: string }[] = [];

    try {
      const results = await Promise.allSettled(
        items.map(async (item) => {
          await executeItemAction(wsId, item, action, reason);
          return item.id;
        })
      );

      results.forEach((res, index) => {
        const item = items[index];
        if (res.status === "fulfilled") {
          succeeded.push(item.id);
        } else {
          const err =
            res.reason instanceof Error
              ? res.reason.message
              : "Thao tác thất bại";
          logError(res.reason, {
            component: "useBatchApproval",
            extra: { itemId: item.id },
          });
          failed.push({ id: item.id, error: err });
        }
      });

      await invalidateAll();

      if (failed.length === 0) {
        toast.success(
          action === "approve"
            ? t(
                "approvals.batch.approve_success",
                "Đã duyệt thành công {{count}} yêu cầu",
                { count: succeeded.length }
              )
            : t(
                "approvals.batch.reject_success",
                "Đã từ chối {{count}} yêu cầu",
                { count: succeeded.length }
              )
        );
      } else {
        toast.error(
          t(
            "approvals.batch.partial_failure",
            "Thành công {{succeeded}}, thất bại {{failed}}",
            { failed: failed.length, succeeded: succeeded.length }
          )
        );
      }

      return { succeeded, failed };
    } finally {
      setIsProcessing(false);
    }
  };

  const batchApprove = async (
    items: UnifiedApprovalItem[]
  ): Promise<BatchApprovalResult> => {
    return processBatch(items, "approve");
  };

  const batchReject = async (
    items: UnifiedApprovalItem[],
    reason: string
  ): Promise<BatchApprovalResult> => {
    return processBatch(items, "reject", reason);
  };

  return {
    batchApprove,
    batchReject,
    isProcessing,
  };
}

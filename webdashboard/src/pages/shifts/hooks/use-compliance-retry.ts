import type { QueryClient } from "@tanstack/react-query";
import type { TFunction } from "i18next";

import { useCallback, useRef } from "react";

import type { useToast } from "@/components/ui/toast";

import { bulkUpsertRoster } from "@/lib/api";

import { runRosterBulk } from "./roster-bulk-utils";

type ToastApi = ReturnType<typeof useToast>;

interface ComplianceRetry {
  assignments: { employeeId: string; employmentId?: string; dates: string[] }[];
  onRowStatus?: (employeeId: string, status: "saving" | "saved" | "error") => void;
  shiftId: string | null;
}

interface UseComplianceRetryProps {
  queryClient: QueryClient;
  refetchAndPatchRow: (employmentId: string) => Promise<void>;
  setComplianceModalOpen: (v: boolean) => void;
  setIsComplianceResolving: (v: boolean) => void;
  t: TFunction;
  toast: ToastApi;
  wsId: string;
}

interface UseComplianceRetryReturn {
  /** Callback invoked by the compliance-warning modal after user confirmation. */
  onComplianceOverride: (reason: string) => Promise<void>;
  /** Ref that the bulk-assign handler writes to before opening the modal. */
  pendingRetryRef: React.RefObject<ComplianceRetry | null>;
}

/**
 * Compliance-conflict retry flow.
 *
 * When a bulk roster assign hits a `ComplianceConflictError`, the handler
 * stashes the pending payload into `pendingRetryRef` and opens the compliance
 * modal. The modal calls `onComplianceOverride(reason)` after the user
 * provides an override reason, which re-runs the bulk assign with
 * `allow_warnings: true` and `override_reason`.
 */
export function useComplianceRetry({
  wsId,
  queryClient,
  toast,
  t,
  refetchAndPatchRow,
  setComplianceModalOpen,
  setIsComplianceResolving,
}: UseComplianceRetryProps): UseComplianceRetryReturn {
  const pendingRetryRef = useRef<ComplianceRetry | null>(null);

  const onComplianceOverride = useCallback(
    async (reason: string) => {
      const pending = pendingRetryRef.current;
      if (!pending) return;
      setIsComplianceResolving(true);
      try {
        const { assignments, shiftId, onRowStatus } = pending;
        await runRosterBulk({
          items: assignments,
          worker: async ({ employeeId, employmentId, dates }) => {
            const items = dates.map((date) => ({ date, shift_id: shiftId }));
            await bulkUpsertRoster(wsId, employeeId, items, {
              allow_warnings: true,
              override_reason: reason,
            });
            if (employmentId) {
              try {
                await refetchAndPatchRow(employmentId);
              } catch {
                /* stale OK */
              }
            }
          },
          queryClient,
          invalidateKeys: [["assignment_history"]],
          toast,
          t,
          partialFailureKey: "shifts.messages.bulk_partial_failure",
          successKey: "shifts.messages.bulk_update_success",
          callbacks: {
            onItemStart: ({ employeeId }) => {
              onRowStatus?.(employeeId, "saving");
            },
            onItemSettle: ({ employeeId }, _index, error) => {
              onRowStatus?.(employeeId, error ? "error" : "saved");
            },
          },
        });
        setComplianceModalOpen(false);
        pendingRetryRef.current = null;
      } catch (e: unknown) {
        toast.error(e instanceof Error ? e.message : String(e));
      } finally {
        setIsComplianceResolving(false);
      }
    },
    [
      wsId,
      queryClient,
      toast,
      t,
      refetchAndPatchRow,
      setComplianceModalOpen,
      setIsComplianceResolving,
    ],
  );

  return { pendingRetryRef, onComplianceOverride };
}

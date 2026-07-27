import type { QueryClient } from "@tanstack/react-query";
import type { TFunction } from "i18next";

import { findComplianceFailure } from "@/pages/shifts/hooks/roster-utils";
import type { PoolCallbacks, PoolResult } from "@/lib/async-pool";
import { runPool } from "@/lib/async-pool";
import { ComplianceConflictError } from "@/lib/api/errors";

import type { useToast } from "@/components/ui/toast";

const ROSTER_REQUEST_CONCURRENCY = 4;

type ToastApi = ReturnType<typeof useToast>;

interface RunRosterBulkOptions<T> {
  /** Items to process concurrently. */
  items: T[];
  /** Async worker called for each item. */
  worker: (item: T) => Promise<void>;
  /** React Query client for cache invalidation. */
  queryClient: QueryClient;
  /** Query keys to invalidate after the bulk run completes. */
  invalidateKeys: string[][];
  /** Toast instance for user feedback. */
  toast: ToastApi;
  /** Translation function for i18n messages. */
  t: TFunction;
  /** i18n key for the partial-failure toast message. */
  partialFailureKey: string;
  /** i18n key for the success toast message. */
  successKey: string;
  /** Optional per-item lifecycle callbacks. */
  callbacks?: PoolCallbacks<T>;
  /** Called when a compliance conflict is detected (before toast). */
  onComplianceFailure?: (f: { error: ComplianceConflictError; item: T }) => void;
}

/**
 * Shared bulk-operation runner for roster mutations.
 *
 * Encapsulates the repeated pattern of:
 * 1. runPool(items, concurrency, worker, callbacks)
 * 2. invalidateQueries(invalidateKeys)
 * 3. compliance-failure check (optional early exit)
 * 4. partial-failure / success toast
 *
 * Callers that need special pre-/post-processing (e.g. stashing a retry ref
 * before the compliance modal opens) should use the `onComplianceFailure`
 * callback rather than trying to reimplement this pattern.
 */
export async function runRosterBulk<T>({
  items,
  worker,
  queryClient,
  invalidateKeys,
  toast,
  t,
  partialFailureKey,
  successKey,
  callbacks,
  onComplianceFailure,
}: RunRosterBulkOptions<T>): Promise<void> {
  const failures: PoolResult<T>[] = await runPool(
    items,
    ROSTER_REQUEST_CONCURRENCY,
    worker,
    callbacks,
  );

  invalidateKeys.forEach((key) => {
    void queryClient.invalidateQueries({ queryKey: key });
  });

  if (onComplianceFailure) {
    const cf = findComplianceFailure(failures);
    if (cf) {
      onComplianceFailure(cf);
      // Compliance conflicts still count as failed roster writes. Keep the
      // row-level/modal details, but always provide a visible toast so a bulk
      // operation can never fail silently.
      toast.error(t(partialFailureKey, { count: failures.length }));
      return;
    }
  }

  if (failures.length > 0) {
    toast.error(t(partialFailureKey, { count: failures.length }));
  } else {
    toast.success(t(successKey));
  }
}

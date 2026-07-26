import type { PoolResult } from "@/lib/async-pool";

import { ComplianceConflictError } from "@/lib/api/errors";

/**
 * Find the first compliance-conflict failure in a pool result set.
 * Returns the failed item and its error, or undefined if no compliance conflict occurred.
 */
export function findComplianceFailure<T>(
  failures: PoolResult<T>[],
): { error: ComplianceConflictError; item: T } | undefined {
  return failures.find(
    (f): f is { error: ComplianceConflictError; item: T } =>
      f.error instanceof ComplianceConflictError,
  );
}

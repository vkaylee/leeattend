import type { QueryClient } from "@tanstack/react-query";

import { attendanceKeys } from "@/lib/query-keys";

/**
 * Shared query invalidation for attendance-related cache keys.
 * Centralizes the 3-key invalidation pattern repeated across
 * OtApprovalsPage, AddPhotoDialog, and BulkUploadOtDialog.
 */
export function invalidateAttendanceQueries(queryClient: QueryClient): void {
  void queryClient.invalidateQueries({ queryKey: attendanceKeys.all() });
  void queryClient.invalidateQueries({ queryKey: attendanceKeys.report() });
  void queryClient.invalidateQueries({ queryKey: attendanceKeys.reportDelegated() });
}

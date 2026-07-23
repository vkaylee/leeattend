import type { QueryClient } from "@tanstack/react-query";

/**
 * Shared query invalidation for attendance-related cache keys.
 * Centralizes the 3-key invalidation pattern repeated across
 * OtApprovalsPage, AddPhotoDialog, and BulkUploadOtDialog.
 */
export function invalidateAttendanceQueries(queryClient: QueryClient): void {
  void queryClient.invalidateQueries({ queryKey: ["attendance"] });
  void queryClient.invalidateQueries({ queryKey: ["attendance-report"] });
  void queryClient.invalidateQueries({ queryKey: ["attendance-report-delegated"] });
}

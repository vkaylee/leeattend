import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import {
  closeAttendancePeriod,
  fetchAttendancePeriodStatus,
  reopenAttendancePeriod,
  type AttendanceClosureRequest,
} from "@/lib/api/attendance-closure";

export const attendanceClosureKeys = {
  all: ["attendance-closure"] as const,
  status: (year: number, month: number, employerId: string) =>
    [...attendanceClosureKeys.all, "status", year, month, employerId] as const,
};

export function useAttendanceClosure(
  year: number,
  month: number,
  employerId: string | null,
  enabled = true,
) {
  const queryClient = useQueryClient();
  const statusQuery = useQuery({
    enabled: enabled && Boolean(employerId),
    queryKey: attendanceClosureKeys.status(year, month, employerId ?? ""),
    queryFn: () => fetchAttendancePeriodStatus(year, month, employerId ?? ""),
  });

  const invalidateStatus = async () => {
    await queryClient.invalidateQueries({
      queryKey: attendanceClosureKeys.status(year, month, employerId ?? ""),
    });
  };

  const closeMutation = useMutation({
    mutationFn: (request: AttendanceClosureRequest) =>
      closeAttendancePeriod(year, month, employerId ?? "", request),
    onSuccess: invalidateStatus,
  });
  const reopenMutation = useMutation({
    mutationFn: (request: AttendanceClosureRequest) =>
      reopenAttendancePeriod(year, month, employerId ?? "", request),
    onSuccess: invalidateStatus,
  });

  return {
    ...statusQuery,
    close: closeMutation.mutateAsync,
    isClosing: closeMutation.isPending,
    closeError: closeMutation.error,
    reopen: reopenMutation.mutateAsync,
    isReopening: reopenMutation.isPending,
    reopenError: reopenMutation.error,
  };
}

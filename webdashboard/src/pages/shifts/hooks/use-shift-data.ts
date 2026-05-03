import { useQuery } from "@tanstack/react-query";

import { format } from "date-fns";

import {
  fetchAssignmentHistory,
  fetchShifts,
  fetchWorkingPatterns,
  fetchWorkspaceSchedules,
} from "@/lib/api";
import {
  AssignmentHistory,
  EmployeeSchedule,
  Shift,
  WorkingPattern,
} from "@/lib/api/types";

interface UseShiftDataProps {
  startOfWeek: Date;
  weekDays: Date[];
  wsId: string;
}

export function useShiftData({ wsId, startOfWeek, weekDays }: UseShiftDataProps) {
  const { data: shifts } = useQuery<Shift[]>({
    queryKey: ["shifts", wsId],
    queryFn: () => fetchShifts(wsId),
    enabled: !!wsId,
  });

  const { data: patterns } = useQuery<WorkingPattern[]>({
    queryKey: ["working-patterns", wsId],
    queryFn: () => fetchWorkingPatterns(wsId),
    enabled: !!wsId,
  });

  const { data: allSchedules } = useQuery<EmployeeSchedule[]>({
    queryKey: ["schedules", wsId, format(startOfWeek, 'yyyy-MM-dd')],
    queryFn: () => fetchWorkspaceSchedules(
      wsId,
      format(weekDays[0], 'yyyy-MM-dd'),
      format(weekDays[6], 'yyyy-MM-dd')
    ),
    enabled: !!wsId,
  });

  const { data: history } = useQuery<AssignmentHistory[]>({
    queryKey: ["assignment_history", wsId],
    queryFn: () => fetchAssignmentHistory(wsId),
    enabled: !!wsId,
  });

  return { shifts, patterns, allSchedules, history };
}

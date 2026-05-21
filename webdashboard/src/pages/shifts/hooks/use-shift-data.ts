import { useQuery } from "@tanstack/react-query";

import { format } from "date-fns";

import {
  fetchAssignmentHistory,
  fetchShifts,
  fetchWorkingPatterns,
  fetchWorkspaceSchedules,
  searchEmployees,
  fetchDepartments,
} from "@/lib/api";
import {
  AssignmentHistory,
  EmployeeSchedule,
  Shift,
  WorkingPattern,
  Employee,
  Department,
} from "@/lib/api/types";

import type { ViewMode } from "./use-shifts-types";

interface UseShiftDataProps {
  monthEnd?: Date;
  monthStart?: Date;
  startOfWeek: Date;
  viewMode: ViewMode;
  weekDays: Date[];
  wsId: string;
}

export function useShiftData({ wsId, startOfWeek, weekDays, viewMode, monthStart, monthEnd }: UseShiftDataProps) {
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

  const { data: monthSchedules, isLoading: isMonthLoading } = useQuery<EmployeeSchedule[]>({
    queryKey: ["schedules", wsId, monthStart ? format(monthStart, 'yyyy-MM-dd') : "", "monthly"],
    queryFn: () => {
      if (!monthStart || !monthEnd) {
        return Promise.resolve([]);
      }
      return fetchWorkspaceSchedules(
        wsId,
        format(monthStart, 'yyyy-MM-dd'),
        format(monthEnd, 'yyyy-MM-dd')
      );
    },
    enabled: !!wsId && viewMode === "month" && !!monthStart && !!monthEnd,
  });

  const { data: history } = useQuery<AssignmentHistory[]>({
    queryKey: ["assignment_history", wsId],
    queryFn: () => fetchAssignmentHistory(wsId),
    enabled: !!wsId,
  });

  const { data: employees } = useQuery<Employee[]>({
    queryKey: ["employees", wsId, "list-all"],
    queryFn: () => searchEmployees({ wsId, query: "", limit: 1000 }).then(res => res.data),
    enabled: !!wsId && viewMode === "month",
  });

  const { data: departments } = useQuery<Department[]>({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId && viewMode === "month",
  });

  return { shifts, patterns, allSchedules, history, monthSchedules, isMonthLoading, employees, departments };
}


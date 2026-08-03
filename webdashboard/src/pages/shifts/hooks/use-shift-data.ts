import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { format } from "date-fns";

import { useAllEmployees } from "@/hooks/use-employees";
import {
  fetchAssignmentHistory,
  fetchShifts,
  fetchWorkingPatterns,
  fetchWorkspaceSchedules,
  fetchDepartments,
} from "@/lib/api";
import {
  AssignmentHistory,
  EmployeeSchedule,
  Shift,
  WorkingPattern,
  Department,
  Employee,
} from "@/lib/api/types";

import type { ViewMode } from "./use-shifts-types";

interface UseShiftDataProps {
  monthEnd?: Date;
  monthStart?: Date;
  selectedEmployee?: Employee | null;
  selectedEmployeeIds?: string[];
  startOfWeek: Date;
  viewMode: ViewMode;
  weekDays: Date[];
  wsId: string;
}

   
export function useShiftData({
  wsId,
  startOfWeek,
  weekDays,
  viewMode,
  monthStart,
  monthEnd,
  selectedEmployee,
  selectedEmployeeIds,
}: UseShiftDataProps) {
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

  const selectedEmploymentId = selectedEmployee?.active_employment_id;

  const { data: allSchedules } = useQuery<EmployeeSchedule[]>({
    queryKey: ["schedules", wsId, format(startOfWeek, 'yyyy-MM-dd'), selectedEmploymentId],
    queryFn: () => fetchWorkspaceSchedules(
      wsId,
      format(weekDays[0], 'yyyy-MM-dd'),
      format(weekDays[6], 'yyyy-MM-dd'),
      selectedEmploymentId ? [selectedEmploymentId] : undefined
    ),
    enabled: !!wsId && (viewMode !== "week" || !!selectedEmploymentId),
  });

  const {
    employees,
    isLoading: isEmployeesLoading,
    isError: isEmployeesError,
    error: employeesError,
    refetch: refetchEmployees,
  } = useAllEmployees(wsId, { enabled: !!wsId && viewMode === "month" });

  const selectedEmploymentIds = useMemo(() => {
    // employees may be undefined before query resolves; selectedEmployeeIds can be empty
    const emps = employees as Employee[] | undefined;
    if (!emps || !selectedEmployeeIds || selectedEmployeeIds.length === 0) return [];
    return selectedEmployeeIds
      .map(id => employees.find(e => e.id === id)?.active_employment_id)
      .filter((id): id is string => !!id);
  }, [employees, selectedEmployeeIds]);

  const {
    data: monthSchedules,
    isLoading: isMonthLoading,
    isError: isMonthError,
    error: monthError,
    refetch: refetchMonthSchedules,
  } = useQuery<EmployeeSchedule[]>({
    queryKey: ["schedules", wsId, monthStart ? format(monthStart, 'yyyy-MM-dd') : "", "monthly", selectedEmploymentIds],
    queryFn: () => {
      if (!monthStart || !monthEnd) {
        return Promise.resolve([]);
      }
      return fetchWorkspaceSchedules(
        wsId,
        format(monthStart, 'yyyy-MM-dd'),
        format(monthEnd, 'yyyy-MM-dd'),
        selectedEmploymentIds.length > 0 ? selectedEmploymentIds : undefined
      );
    },
    enabled: !!wsId && viewMode === "month" && !!monthStart && !!monthEnd && selectedEmploymentIds.length > 0,
  });

  const { data: history } = useQuery<AssignmentHistory[]>({
    queryKey: ["assignment_history", wsId],
    queryFn: () => fetchAssignmentHistory(wsId),
    enabled: !!wsId,
  });

  const { data: departments } = useQuery<Department[]>({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
  });

  return {
    shifts,
    patterns,
    allSchedules,
    history,
    monthSchedules,
    isMonthLoading,
    isMonthError,
    monthError,
    refetchMonthSchedules,
    employees,
    isEmployeesLoading,
    isEmployeesError,
    employeesError,
    refetchEmployees,
    departments,
  };
}

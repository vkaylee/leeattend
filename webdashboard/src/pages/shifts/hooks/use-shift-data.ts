import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { format } from "date-fns";

import { useDepartments } from "@/hooks/use-departments";
import { useMyPermissions } from "@/hooks/use-permissions";
import { useAllEmployees } from "@/hooks/use-employees";
import {
  fetchAssignmentHistory,
  fetchShifts,
  fetchWorkingPatterns,
  fetchSchedules,
} from "@/lib/api";
import {
  AssignmentHistory,
  EmployeeSchedule,
  Shift,
  WorkingPattern,
  Employee,
} from "@/lib/api/types";
import { createQueryKey } from "@/lib/query-keys";

import type { ScheduleCoverageFilter } from "./use-shift-form-state";
import type { ViewMode } from "./use-shifts-types";


interface UseShiftDataProps { coverageFilter?: ScheduleCoverageFilter;
monthEnd?: Date;
monthStart?: Date;
selectedEmployee?: Employee | null;
selectedEmployeeIds?: string[];
startOfWeek: Date;
viewMode: ViewMode;
weekDays: Date[];  }


export function useShiftData({
  startOfWeek,
  weekDays,
  viewMode,
  monthStart,
  monthEnd,
  selectedEmployee,
  selectedEmployeeIds,
  coverageFilter = "all",
}: UseShiftDataProps) {
  const { hasPermission } = useMyPermissions();
  const canReadEmployees = hasPermission("employees:read");
  const { data: shifts } = useQuery<Shift[]>({
    queryKey: createQueryKey("shifts"),
    queryFn: () => fetchShifts(),
  });

  const { data: patterns } = useQuery<WorkingPattern[]>({
    queryKey: createQueryKey("working-patterns"),
    queryFn: () => fetchWorkingPatterns(),
  });

  const selectedEmploymentId = selectedEmployee?.active_employment_id;

  const {
    data: allSchedules,
    isLoading: isScheduleLoading,
    isError: isScheduleError,
    error: scheduleError,
  } = useQuery<EmployeeSchedule[]>({
    queryKey: createQueryKey("schedules", format(startOfWeek, 'yyyy-MM-dd'), selectedEmploymentId),
    queryFn: () => fetchSchedules(format(weekDays[0], 'yyyy-MM-dd'),
    format(weekDays[6], 'yyyy-MM-dd'),
    selectedEmploymentId ? [selectedEmploymentId] : undefined),
    enabled: viewMode !== "week" || !!selectedEmploymentId,
  });

  const {
    employees,
    isLoading: isEmployeesLoading,
    isError: isEmployeesError,
    error: employeesError,
    refetch: refetchEmployees,
  } = useAllEmployees({ enabled: viewMode === "month" && canReadEmployees });

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
    queryKey: createQueryKey("schedules", monthStart ? format(monthStart, 'yyyy-MM-dd') : "", "monthly", selectedEmploymentIds, coverageFilter),
    queryFn: () => {
      if (!monthStart || !monthEnd) {
        return Promise.resolve([]);
      }
      return fetchSchedules(format(monthStart, 'yyyy-MM-dd'),
      format(monthEnd, 'yyyy-MM-dd'),
      selectedEmploymentIds.length > 0 ? selectedEmploymentIds : undefined,
      coverageFilter);
    },
    enabled: viewMode === "month" && !!monthStart && !!monthEnd && selectedEmploymentIds.length > 0,
  });

  const { data: history } = useQuery<AssignmentHistory[]>({
    queryKey: createQueryKey("assignment_history"),
    queryFn: () => fetchAssignmentHistory(),
  });

  const { data: departments } = useDepartments();

  return {
    shifts,
    patterns,
    allSchedules,
    isScheduleLoading,
    isScheduleError,
    scheduleError,
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

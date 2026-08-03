import type { UseMutationResult } from "@tanstack/react-query";
import type { TFunction } from "i18next";

import { format, addDays } from "date-fns";

import type { useToast } from "@/components/ui/toast";
import type { AssignPatternRequest, EffectiveSchedule, Employee, EmployeeSchedule, EmployeeWorkingPattern } from "@/lib/api/types";

import { fetchWorkspaceSchedules } from "@/lib/api";
import { writeFrequentShift } from "@/lib/frequent-shifts-storage";

type ToastApi = ReturnType<typeof useToast>;

interface UseWeeklyRosterHandlersProps {
  allSchedules: EmployeeSchedule[] | undefined;
  endDate: string;
  isForever: boolean;
  mutations: {
    assignPatternMutation: UseMutationResult<EmployeeWorkingPattern, Error, AssignPatternRequest>;
    bulkRosterMutation: UseMutationResult<unknown, Error, { date: string; shift_id: string | null }[]>;
  };
  selectedDays: string[];
  selectedEmployee: Employee | null;
  selectedPatternId: string;
  startDate: string;
  startOfWeek: Date;
  t: TFunction;
  toast: ToastApi;
  weekDays: Date[];
  wsId: string;
}

interface UseWeeklyRosterHandlersReturn {
  handleApplyShift: (shiftId: string | null) => void;
  handleAssign: () => void;
  handleAssignAllWeek: (shiftId: string | null) => void;
  handleAssignBulk: (shiftId: string | null) => void;
  handleCopyPreviousWeek: () => Promise<void>;
}

/**
 * Weekly roster mutation handlers: single pattern assignment, bulk day
 * assignment, fill-entire-week, and copy-from-previous-week.
 */
export function useWeeklyRosterHandlers({
  wsId,
  selectedEmployee,
  selectedPatternId,
  startDate,
  endDate,
  isForever,
  selectedDays,
  weekDays,
  startOfWeek,
  allSchedules,
  mutations,
  t,
  toast,
}: UseWeeklyRosterHandlersProps): UseWeeklyRosterHandlersReturn {
  const handleAssign = () => {
    if (!selectedEmployee || !selectedPatternId) {
      toast.warning(t("shifts.messages.validation_missing_fields"));
      return;
    }
    mutations.assignPatternMutation.mutate({
      pattern_id: selectedPatternId,
      start_date: startDate,
      end_date: isForever ? null : endDate,
    });
  };

  const handleBulkAssign = (shiftId: string | null) => {
    if (!selectedEmployee || selectedDays.length === 0) return;
    const items = selectedDays.map((date) => ({ date, shift_id: shiftId }));
    mutations.bulkRosterMutation.mutate(items);
  };

  /** Quick-shift assignment: applies the shift to selected days and records it for the frequency ranking. */
  const handleApplyShift = (shiftId: string | null) => {
    if (shiftId) writeFrequentShift(shiftId);
    handleBulkAssign(shiftId);
  };

  const handleAssignAllWeek = (shiftId: string | null) => {
    if (!selectedEmployee) return;
    const items = weekDays.map((day) => ({
      date: format(day, "yyyy-MM-dd"),
      shift_id: shiftId,
    }));
    mutations.bulkRosterMutation.mutate(items);
  };

  const handleCopyPreviousWeek = async () => {
    if (!selectedEmployee || !allSchedules) return;
    const previousWeekStart = addDays(startOfWeek, -7);
    const previousWeekEnd = addDays(startOfWeek, -1);
    try {
      const previousSchedules = await fetchWorkspaceSchedules(
        wsId,
        format(previousWeekStart, "yyyy-MM-dd"),
        format(previousWeekEnd, "yyyy-MM-dd"),
      );
      const empPrevious = previousSchedules.find(
        (s: EmployeeSchedule) =>
          s.employee_id === selectedEmployee.id ||
          s.employment_id === selectedEmployee.active_employment_id,
      );
      if (!empPrevious) {
        toast.warning(t("shifts.messages.no_previous_data"));
        return;
      }
      const items = empPrevious.schedules.map(
        (s: EffectiveSchedule, index: number) => ({
          date: format(weekDays[index], "yyyy-MM-dd"),
          shift_id: s.shift_id ?? null,
        }),
      );
      mutations.bulkRosterMutation.mutate(items);
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  return {
    handleApplyShift,
    handleAssign,
    handleAssignAllWeek,
    handleAssignBulk: handleBulkAssign,
    handleCopyPreviousWeek,
  };
}

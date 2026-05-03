import type { UseMutationResult } from "@tanstack/react-query";

import { useTranslation } from "react-i18next";

import { format, addDays } from "date-fns";

import type { AssignPatternRequest, Employee, EmployeeSchedule, EmployeeWorkingPattern } from "@/lib/api/types";

import { useToast } from "@/components/ui/toast";
import { fetchWorkspaceSchedules } from "@/lib/api";

interface UseRosterHandlersProps {
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
  weekDays: Date[];
  wsId: string;
}

export function useRosterHandlers({
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
}: UseRosterHandlersProps) {
  const { t } = useTranslation();
  const toast = useToast();

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
    const items = selectedDays.map(date => ({ date, shift_id: shiftId }));
    mutations.bulkRosterMutation.mutate(items);
  };

  const handleAssignAllWeek = (shiftId: string | null) => {
    if (!selectedEmployee) return;
    const items = weekDays.map(day => ({ date: format(day, 'yyyy-MM-dd'), shift_id: shiftId }));
    mutations.bulkRosterMutation.mutate(items);
  };

  const handleCopyPreviousWeek = async () => {
    if (!selectedEmployee || !allSchedules) return;
    const previousWeekStart = addDays(startOfWeek, -7);
    const previousWeekEnd = addDays(startOfWeek, -1);
    try {
      const previousSchedules = await fetchWorkspaceSchedules(wsId, format(previousWeekStart, 'yyyy-MM-dd'), format(previousWeekEnd, 'yyyy-MM-dd'));
      const empPrevious = previousSchedules.find((s) => s.employee_id === selectedEmployee.id);
      if (!empPrevious) { toast.warning(t("shifts.messages.no_previous_data")); return; }
      const items = empPrevious.schedules.map((s, index: number) => ({ date: format(weekDays[index], 'yyyy-MM-dd'), shift_id: s.shift_id }));
      mutations.bulkRosterMutation.mutate(items);
    } catch (e: unknown) { toast.error(e instanceof Error ? e.message : String(e)); }
  };

  return {
    handleAssign,
    handleAssignAllWeek,
    handleAssignBulk: handleBulkAssign,
    handleCopyPreviousWeek,
  };
}

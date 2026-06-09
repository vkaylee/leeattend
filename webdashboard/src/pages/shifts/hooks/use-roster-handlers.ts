import type { UseMutationResult } from "@tanstack/react-query";

import { useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { format, addDays, subMonths, startOfMonth, endOfMonth } from "date-fns";

import type { AssignPatternRequest, Employee, EmployeeSchedule, EmployeeWorkingPattern, EffectiveSchedule } from "@/lib/api/types";

import { useToast } from "@/components/ui/toast";
import { fetchWorkspaceSchedules, bulkUpsertRoster, deleteRoster } from "@/lib/api";

interface UseRosterHandlersProps {
  allSchedules: EmployeeSchedule[] | undefined;
  currentMonth: Date;
  endDate: string;
  isForever: boolean;
  monthDays: Date[];
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
  currentMonth,
  monthDays,
}: UseRosterHandlersProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

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
      const empPrevious = previousSchedules.find((s: EmployeeSchedule) => s.employment_id === selectedEmployee.active_employment_id);
      if (!empPrevious) { toast.warning(t("shifts.messages.no_previous_data")); return; }
      const items = empPrevious.schedules.map((s: EffectiveSchedule, index: number) => ({ date: format(weekDays[index], 'yyyy-MM-dd'), shift_id: s.shift_id ?? null }));
      mutations.bulkRosterMutation.mutate(items);
    } catch (e: unknown) { toast.error(e instanceof Error ? e.message : String(e)); }
  };

  // Monthly handlers
  const handleMonthBulkAssign = async (
    assignments: { employeeId: string; dates: string[] }[],
    shiftId: string | null
  ) => {
    try {
      await Promise.all(
        assignments.map(async ({ employeeId, dates }) => {
          const items = dates.map(date => ({ date, shift_id: shiftId }));
          await bulkUpsertRoster(wsId, employeeId, items);
        })
      );
      toast.success(t("shifts.messages.bulk_update_success"));
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const handleMonthDeleteRoster = async (
    assignments: { employeeId: string; dates: string[] }[]
  ) => {
    try {
      await Promise.all(
        assignments.map(async ({ employeeId, dates }) => {
          await Promise.all(
            dates.map(date => deleteRoster(wsId, employeeId, date))
          );
        })
      );
      toast.success(t("shifts.messages.remove_override_success"));
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const handleCopyPreviousMonth = async () => {
    const prevMonth = subMonths(currentMonth, 1);
    const prevStartStr = format(startOfMonth(prevMonth), 'yyyy-MM-dd');
    const prevEndStr = format(endOfMonth(prevMonth), 'yyyy-MM-dd');
    
    try {
      const prevSchedules = await fetchWorkspaceSchedules(wsId, prevStartStr, prevEndStr);
      const currentYearMonth = format(currentMonth, 'yyyy-MM');
      const monthDayStrings = new Set(monthDays.map(d => format(d, 'yyyy-MM-dd')));
      
      let copyCount = 0;
      
      await Promise.all(
        prevSchedules.map(async (empSchedule) => {
          const rosterEntries = empSchedule.schedules
            .filter((s: EffectiveSchedule) => s.source === "roster" && !!s.shift_id)
            .map((s: EffectiveSchedule) => {
              const targetDateStr = `${currentYearMonth}-${s.date.substring(8)}`;
              if (monthDayStrings.has(targetDateStr)) {
                return { date: targetDateStr, shift_id: s.shift_id ?? null };
              }
              return null;
            })
            .filter((item): item is { date: string; shift_id: string | null } => item !== null);
            
          if (rosterEntries.length > 0) {
            copyCount += rosterEntries.length;
             await bulkUpsertRoster(wsId, empSchedule.employee_id ?? "", rosterEntries);
          }
        })
      );
      
      if (copyCount > 0) {
        toast.success(t("shifts.messages.bulk_update_success"));
        void queryClient.invalidateQueries({ queryKey: ["schedules"] });
        void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
      } else {
        toast.warning(t("shifts.messages.no_previous_data"));
      }
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  return {
    handleAssign,
    handleAssignAllWeek,
    handleAssignBulk: handleBulkAssign,
    handleCopyPreviousWeek,
    handleMonthBulkAssign,
    handleMonthDeleteRoster,
    handleCopyPreviousMonth,
  };
}

import { useNavigate } from "@tanstack/react-router";
import { getCurrentWorkspaceId } from "@/lib/api";

import { useMonthNavigation } from "./use-month-navigation";
import { useRosterHandlers } from "./use-roster-handlers";
import { useShiftData } from "./use-shift-data";
import { useShiftFormState } from "./use-shift-form-state";
import { useShiftMutations } from "./use-shift-mutations";
import { useShiftNavigation } from "./use-shift-navigation";
import { UseShiftsReturn } from "./use-shifts-types";
import type { Shift } from "@/lib/api/types";

export function useShifts(): UseShiftsReturn {
  const wsId = getCurrentWorkspaceId() ?? "";
  const navigate = useNavigate();
  const nav = useShiftNavigation();
  const monthNav = useMonthNavigation();
  const form = useShiftFormState();
  const data = useShiftData({
    wsId,
    startOfWeek: nav.startOfWeek,
    weekDays: nav.weekDays,
    viewMode: form.viewMode,
    monthStart: monthNav.monthStart,
    monthEnd: monthNav.monthEnd,
  });

  const mutations = useShiftMutations({
    wsId,
    selectedEmployeeId: form.selectedEmployee?.id,
    onSuccess: (type) => {
      if (type === "assignPattern") {
        form.setSelectedPatternId("");
        form.setEndDate("");
        form.setIsForever(true);
      } else if (type === "bulkRoster") {
        form.setSelectedDays([]);
      } else if (type === "createShift") {
        form.setNewShiftName("");
        form.setNewShiftSymbol("");
        form.setNewShiftStart("08:00");
        form.setNewShiftEnd("17:00");
        form.setNewShiftBreak(60);
        form.setNewShiftGrace(15);
        form.setNewBreakStart("");
        form.setNewBreakEnd("");
        form.setIsFixedBreak(false);
        form.setIsAutoDeduct(true);
        form.setIsBreakOvertime(false);
        form.setIsCreateModalOpen(false);
        form.setActiveTab("directory");
      } else if (type === "updateShift") {
        form.setEditingShift(null);
      }
    }
  });

  const rosterHandlers = useRosterHandlers({
    wsId,
    ...form,
    ...nav,
    allSchedules: data.allSchedules,
    mutations,
    currentMonth: monthNav.currentMonth,
    monthDays: monthNav.monthDays,
  });

  const openEditModal = (shift: Shift) => {
    void navigate({
      to: "/shifts/$shiftId/edit",
      params: { shiftId: shift.id }
    });
  };

  const handleCreateShift = () => {};
  const handleUpdateShift = () => {};

  const employeeSchedule = data.allSchedules?.find(s => s.employee_id === form.selectedEmployee?.id);

  return {
    ...form,
    ...nav,
    ...monthNav,
    ...data,
    ...mutations,
    ...rosterHandlers,
    openEditModal,
    handleCreateShift,
    handleUpdateShift,
    employeeSchedule,
    wsId,
    handleNavigateWeek: (dir: "prev" | "next") => { nav.handleNavigateWeek(dir, () => { form.setSelectedDays([]); }); }
  };
}

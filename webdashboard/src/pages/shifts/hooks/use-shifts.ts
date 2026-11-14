import { useNavigate } from "@tanstack/react-router";
import { useState, useCallback } from "react";

import type { ComplianceViolation } from "@/lib/api/errors";
import type { Shift, EmployeeSchedule } from "@/lib/api/types";

import { useMonthNavigation } from "./use-month-navigation";
import { useRosterHandlers } from "./use-roster-handlers";
import { useShiftData } from "./use-shift-data";
import { useShiftFormState } from "./use-shift-form-state";
import { useShiftMutations } from "./use-shift-mutations";
import { useShiftNavigation } from "./use-shift-navigation";
import { UseShiftsReturn } from "./use-shifts-types";

export function useShifts(): UseShiftsReturn {
  
  const navigate = useNavigate();
  const nav = useShiftNavigation();
  const monthNav = useMonthNavigation();
  const form = useShiftFormState();
  const data = useShiftData({
    startOfWeek: nav.startOfWeek,
    weekDays: nav.weekDays,
    viewMode: form.viewMode,
    monthStart: monthNav.monthStart,
    monthEnd: monthNav.monthEnd,
    selectedEmployee: form.selectedEmployee,
    selectedEmployeeIds: form.selectedEmployeeIds,
  });

  // ── Compliance state (shared between mutations and roster handlers) ───
  const [complianceModalOpen, setComplianceModalOpen] = useState(false);
  const [complianceViolations, setComplianceViolations] = useState<ComplianceViolation[]>([]);
  const [isComplianceResolving, setIsComplianceResolving] = useState(false);

  const onComplianceConflict = useCallback((error: { violations: ComplianceViolation[] }) => {
    setComplianceViolations(error.violations);
    setComplianceModalOpen(true);
  }, []);

  const mutations = useShiftMutations({
    selectedEmployeeId: form.selectedEmployee?.id,
    selectedEmploymentId: form.selectedEmployee?.active_employment_id ?? undefined,
    onComplianceConflict,
    onSuccess: (type) => {
      if (type === "assignPattern") {
        form.setSelectedPatternId("");
        form.setEndDate("");
        form.setIsForever(true);
        form.setIsPatternModalOpen(false);
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
    ...form,
    ...nav,
    allSchedules: data.allSchedules,
    mutations,
    currentMonth: monthNav.currentMonth,
    monthDays: monthNav.monthDays,
    // Compliance state — roster handlers own the retry logic, but share
    // these setters so both mutations and pool-based handlers can open the modal.
    setComplianceModalOpen,
    setComplianceViolations,
    setIsComplianceResolving,
  });

  const openEditModal = (shift: Shift) => {
    void navigate({
      to: "/shifts/$shiftId/edit",
      params: { shiftId: shift.id }
    });
  };

  const handleCreateShift = () => {
    // Handled by createShiftMutation
  };
  const handleUpdateShift = () => {
    // Handled by updateShiftMutation
  };

  const employeeSchedule = data.allSchedules?.find((s: EmployeeSchedule) =>
    s.employee_id === form.selectedEmployee?.id ||
    s.employment_id === form.selectedEmployee?.active_employment_id
  );

  return {
    ...form,
    ...nav,
    ...monthNav,
    ...data,
    ...mutations,
    ...rosterHandlers,
    complianceModalOpen,
    complianceViolations,
    isComplianceResolving,
    setComplianceModalOpen,
    openEditModal,
    handleCreateShift,
    handleUpdateShift,
    employeeSchedule,
    handleNavigateWeek: (dir: "prev" | "next") => { nav.handleNavigateWeek(dir, () => { form.setSelectedDays([]); }); }
  };
}

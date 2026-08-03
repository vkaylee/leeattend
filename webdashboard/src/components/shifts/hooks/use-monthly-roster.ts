import { useState, useCallback } from "react";

import { groupCellsByEmployee } from "@/components/shifts/roster/roster-utils";
import { writeFrequentShift } from "@/lib/frequent-shifts-storage";
import { useShifts } from "@/pages/shifts/hooks/use-shifts";
import { maybeNull } from "@/types/api-overrides";

import { useDragCellSelection } from "./use-drag-cell-selection";
import { useMonthlyRosterKeyboard } from "./use-monthly-roster-keyboard";
import { useMonthlySchedules } from "./use-monthly-schedules";

interface UseMonthlyRosterProps {
  hook: ReturnType<typeof useShifts>;
}

export function useMonthlyRoster({ hook }: UseMonthlyRosterProps) {
  const {
    monthDays,
    handleMonthBulkAssign,
    handleMonthDeleteRoster,
    handleCopyPreviousMonth,
    employees,
    monthSchedules,
    selectedEmployeeIds,
    setSelectedEmployeeIds,
    selectedEmployees,
    setSelectedEmployees,
  } = hook;

  const [selectedCells, setSelectedCells] = useState<Set<string>>(new Set());
  const [isShiftPickerOpen, setIsShiftPickerOpen] = useState(false);
  const [isCopying, setIsCopying] = useState(false);
  const [isMutating, setIsMutating] = useState(false);
  const [rowStatus, setRowStatus] = useState<
    Map<string, "saving" | "saved" | "error">
  >(new Map());

  const setRowStatusFor = useCallback(
    (employeeId: string, status: "saving" | "saved" | "error") => {
      setRowStatus((prev) => {
        const next = new Map(prev);
        next.set(employeeId, status);
        return next;
      });
      if (status === "saved") {
        window.setTimeout(() => {
          setRowStatus((prev) => {
            if (prev.get(employeeId) !== "saved") return prev;
            const next = new Map(prev);
            next.delete(employeeId);
            return next;
          });
        }, 1500);
      }
    },
    [],
  );

  const { selectedEmployeesMeta, combinedSchedules, filteredSchedules } =
    useMonthlySchedules({
      employees,
      monthSchedules,
      selectedEmployeeIds,
      selectedEmployees,
    });

  const { isDragging, handleCellPointerDown, handleCellPointerEnter } =
    useDragCellSelection({
      selectedCells,
      setSelectedCells,
      filteredSchedules,
      monthDays,
    });

  const handleApplyShift = async (shiftId: string | null) => {
    setIsMutating(true);
    try {
      if (shiftId) writeFrequentShift(shiftId);
      const groups = groupCellsByEmployee(selectedCells);
      const employmentByEmployee = new Map(
        filteredSchedules.map((s) => [s.employee_id, s.employment_id]),
      );
      const assignments = Object.entries(groups).map(([employeeId, dates]) => ({
        employeeId,
        employmentId: employmentByEmployee.get(employeeId),
        dates: maybeNull(dates) ?? [],
      }));
      await handleMonthBulkAssign(assignments, shiftId, setRowStatusFor);
      setSelectedCells(new Set());
    } finally {
      setIsMutating(false);
    }
  };

  const handleClearSelectedOverrides = async () => {
    setIsMutating(true);
    try {
      const groups = groupCellsByEmployee(selectedCells);
      const assignments = Object.entries(groups).map(([employeeId, dates]) => ({
        employeeId,
        dates: maybeNull(dates) ?? [],
      }));
      await handleMonthDeleteRoster(assignments);
      setSelectedCells(new Set());
    } finally {
      setIsMutating(false);
    }
  };

  const handleCopyMonth = async () => {
    setIsCopying(true);
    try {
      await handleCopyPreviousMonth();
    } finally {
      setIsCopying(false);
    }
  };

  useMonthlyRosterKeyboard({
    selectedCells,
    setIsShiftPickerOpen,
    setSelectedCells,
    onClearOverrides: handleClearSelectedOverrides,
  });

  return {
    selectedCells,
    setSelectedCells,
    isShiftPickerOpen,
    setIsShiftPickerOpen,
    isCopying,
    isMutating,
    selectedEmployeeIds,
    setSelectedEmployeeIds,
    selectedEmployees,
    setSelectedEmployees,
    selectedEmployeesMeta,
    combinedSchedules,
    filteredSchedules,
    isDragging,
    handleCellPointerDown,
    handleCellPointerEnter,
    handleApplyShift,
    handleClearSelectedOverrides,
    handleCopyMonth,
    rowStatus,
  };
}

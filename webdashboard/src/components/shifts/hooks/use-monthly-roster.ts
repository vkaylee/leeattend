import React, { useState, useEffect, useMemo } from "react";

import { format } from "date-fns";

import { useShifts } from "@/pages/shifts/hooks/use-shifts";

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

  const selectedEmployeesMeta = useMemo(() => {
    const list = [...selectedEmployees];
    if (employees) {
      const empMap = new Map(employees.map((e) => [e.id, e]));
      selectedEmployeeIds.forEach((id) => {
        if (!list.some((e) => e.id === id)) {
          const emp = empMap.get(id);
          if (emp) list.push(emp);
        }
      });
    }
    return list;
  }, [employees, selectedEmployeeIds, selectedEmployees]);

  const combinedSchedules = useMemo(() => {
    if (!employees) return [];
    const list = [...employees];
    selectedEmployees.forEach((selectedEmp) => {
      if (!list.some((e) => e.id === selectedEmp.id)) {
        list.push(selectedEmp);
      }
    });
    return list.map((emp) => {
      const existing = monthSchedules?.find(
        (s) => s.employee_id === emp.id || s.employment_id === emp.active_employment_id
      );
      if (existing) {
        return {
          ...existing,
          employee_id: emp.id,
        };
      }
      return {
        employee_id: emp.id,
        employment_id: emp.active_employment_id ?? "",
        full_name: emp.full_name,
        schedules: [],
      };
    });
  }, [employees, monthSchedules, selectedEmployees]);

  const filteredSchedules = useMemo(() => {
    if (selectedEmployeeIds.length === 0) return [];
    return combinedSchedules.filter((schedule) =>
      selectedEmployeeIds.includes(schedule.employee_id ?? "")
    );
  }, [combinedSchedules, selectedEmployeeIds]);

  const [dragStartCell, setDragStartCell] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragSnapshot, setDragSnapshot] = useState<Set<string>>(new Set());

  useEffect(() => {
    const handleGlobalMouseUp = () => {
      setIsDragging(false);
      setDragStartCell(null);
    };

    if (isDragging) {
      window.addEventListener("mouseup", handleGlobalMouseUp);
      return () => {
        window.removeEventListener("mouseup", handleGlobalMouseUp);
      };
    }
  }, [isDragging]);

  const handleCellMouseDown = (
    employeeId: string,
    dateStr: string,
    e: React.MouseEvent
  ) => {
    if (e.button !== 0) return; // Only left click
    const key = `${employeeId}:${dateStr}`;
    setIsDragging(true);
    setDragStartCell(key);

    setSelectedCells((prev) => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      setDragSnapshot(new Set(next));
      return next;
    });
  };

  const handleCellMouseEnter = (employeeId: string, dateStr: string) => {
    if (!isDragging || !dragStartCell) return;

    const [startEmpId, startDateStr] = dragStartCell.split(":");

    const startEmpIdx = filteredSchedules.findIndex(
      (s) => s.employee_id === startEmpId
    );
    const endEmpIdx = filteredSchedules.findIndex(
      (s) => s.employee_id === employeeId
    );

    if (startEmpIdx === -1 || endEmpIdx === -1) return;

    const startDayIdx = monthDays.findIndex(
      (d) => format(d, "yyyy-MM-dd") === startDateStr
    );
    const endDayIdx = monthDays.findIndex(
      (d) => format(d, "yyyy-MM-dd") === dateStr
    );

    if (startDayIdx === -1 || endDayIdx === -1) return;

    const minEmpIdx = Math.min(startEmpIdx, endEmpIdx);
    const maxEmpIdx = Math.max(startEmpIdx, endEmpIdx);
    const minDayIdx = Math.min(startDayIdx, endDayIdx);
    const maxDayIdx = Math.max(startDayIdx, endDayIdx);

    setSelectedCells(() => {
      const next = new Set(dragSnapshot);
      for (let i = minEmpIdx; i <= maxEmpIdx; i++) {
        const empId = filteredSchedules[i].employee_id ?? "";
        for (let j = minDayIdx; j <= maxDayIdx; j++) {
          const dStr = format(monthDays[j], "yyyy-MM-dd");
          next.add(`${empId}:${dStr}`);
        }
      }
      return next;
    });
  };

  const handleApplyShift = async (shiftId: string | null) => {
    setIsMutating(true);
    try {
      const groups = Array.from(selectedCells).reduce<
        Record<string, string[] | undefined>
      >((acc, key) => {
        const [empId, d] = key.split(":");
        const list = acc[empId];
        if (list) {
          list.push(d);
        } else {
          acc[empId] = [d];
        }
        return acc;
      }, {});
      const assignments = Object.entries(groups).map(([employeeId, dates]) => ({
        employeeId,
        dates: dates ?? [],
      }));
      await handleMonthBulkAssign(assignments, shiftId);
      setSelectedCells(new Set());
    } finally {
      setIsMutating(false);
    }
  };

  const handleClearSelectedOverrides = async () => {
    setIsMutating(true);
    try {
      const groups = Array.from(selectedCells).reduce<
        Record<string, string[] | undefined>
      >((acc, key) => {
        const [empId, d] = key.split(":");
        const list = acc[empId];
        if (list) {
          list.push(d);
        } else {
          acc[empId] = [d];
        }
        return acc;
      }, {});
      const assignments = Object.entries(groups).map(([employeeId, dates]) => ({
        employeeId,
        dates: dates ?? [],
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

  // Keyboard shortcuts when cells are selected
  useEffect(() => {
    if (selectedCells.size === 0) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if user is typing in inputs or editing content
      const activeEl = document.activeElement;
      if (
        activeEl &&
        (activeEl.tagName === "INPUT" ||
          activeEl.tagName === "TEXTAREA" ||
          activeEl.getAttribute("contenteditable") === "true")
      ) {
        return;
      }

      if (e.key === "a" || e.key === "A") {
        e.preventDefault();
        setIsShiftPickerOpen(true);
      } else if (e.key === "Delete" || e.key === "Backspace") {
        e.preventDefault();
        void handleClearSelectedOverrides();
      } else if (e.key === "Escape") {
        e.preventDefault();
        setSelectedCells(new Set());
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [selectedCells.size, setIsShiftPickerOpen, setSelectedCells]);

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
    handleCellMouseDown,
    handleCellMouseEnter,
    handleApplyShift,
    handleClearSelectedOverrides,
    handleCopyMonth,
  };
}

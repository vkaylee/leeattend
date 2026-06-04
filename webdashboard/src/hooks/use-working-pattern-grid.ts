import { useState, useCallback } from "react";

export interface UseWorkingPatternGridOptions {
  initialLength?: number;
  onDirty?: () => void;
}

export function useWorkingPatternGrid(options: UseWorkingPatternGridOptions = {}) {
  const { initialLength = 7, onDirty } = options;
  const [grid, setGrid] = useState<string[][]>(() =>
    Array.from({ length: initialLength }, () => [])
  );

  const markDirty = useCallback(() => {
    if (onDirty) {
      onDirty();
    }
  }, [onDirty]);

  const addShiftToDay = useCallback((dayIndex: number, shiftId: string) => {
    setGrid((prev) => {
      if (prev[dayIndex]?.includes(shiftId)) return prev;
      markDirty();
      const newGrid = prev.map((a) => [...a]);
      if (newGrid[dayIndex]) {
        newGrid[dayIndex] = [...newGrid[dayIndex], shiftId];
      }
      return newGrid;
    });
  }, [markDirty]);

  const removeShiftFromDay = useCallback((dayIndex: number, shiftId: string) => {
    setGrid((prev) => {
      if (prev[dayIndex] && !prev[dayIndex].includes(shiftId)) return prev;
      markDirty();
      const newGrid = prev.map((a) => [...a]);
      if (newGrid[dayIndex]) {
        newGrid[dayIndex] = newGrid[dayIndex].filter((id) => id !== shiftId);
      }
      return newGrid;
    });
  }, [markDirty]);

  const clearAllDays = useCallback(() => {
    markDirty();
    setGrid((prev) => Array.from({ length: prev.length }, () => []));
  }, [markDirty]);

  const prefillStandard = useCallback((defaultShiftId: string) => {
    markDirty();
    setGrid((prev) => {
      const newGrid = prev.map((a) => [...a]);
      for (let i = 1; i <= 5; i++) {
        if (newGrid[i]) newGrid[i] = [defaultShiftId];
      }
      return newGrid;
    });
  }, [markDirty]);

  return {
    grid,
    setGrid,
    addShiftToDay,
    removeShiftFromDay,
    clearAllDays,
    prefillStandard,
  };
}

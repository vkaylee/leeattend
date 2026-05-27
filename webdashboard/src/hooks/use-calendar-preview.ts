import { useState } from "react";

export function useCalendarPreview() {
  const [activeViewMode, setActiveViewMode] = useState<"design" | "preview">("design");
  const [previewAnchorDate, setPreviewAnchorDate] = useState<string>(() => new Date().toISOString().split("T")[0]);
  const [previewYear, setPreviewYear] = useState<number>(() => new Date().getFullYear());
  const [previewMonth, setPreviewMonth] = useState<number>(() => new Date().getMonth());

  const handlePreviousMonth = () => {
    if (previewMonth === 0) {
      setPreviewMonth(11);
      setPreviewYear((y) => y - 1);
    } else {
      setPreviewMonth((m) => m - 1);
    }
  };

  const handleNextMonth = () => {
    if (previewMonth === 11) {
      setPreviewMonth(0);
      setPreviewYear((y) => y + 1);
    } else {
      setPreviewMonth((m) => m + 1);
    }
  };

  const generateCalendarCells = (cycleLengthDays: number, grid: string[][]) => {
    const cells = [];
    const firstDay = new Date(previewYear, previewMonth, 1);
    const startOffset = firstDay.getDay();
    const totalDays = new Date(previewYear, previewMonth + 1, 0).getDate();
    const prevMonthTotalDays = new Date(previewYear, previewMonth, 0).getDate();

    for (let i = startOffset - 1; i >= 0; i--) {
      cells.push({
        date: new Date(previewYear, previewMonth - 1, prevMonthTotalDays - i),
        isCurrentMonth: false,
      });
    }

    for (let i = 1; i <= totalDays; i++) {
      cells.push({
        date: new Date(previewYear, previewMonth, i),
        isCurrentMonth: true,
      });
    }

    const remainingCells = 42 - cells.length;
    for (let i = 1; i <= remainingCells; i++) {
      cells.push({
        date: new Date(previewYear, previewMonth + 1, i),
        isCurrentMonth: false,
      });
    }

    const [ay, am, ad] = previewAnchorDate.split("-").map(Number);
    const anchorMidnight = ay && am && ad ? new Date(ay, am - 1, ad).getTime() : 0;
    const todayStr = new Date().toISOString().split("T")[0];

    return cells.map(({ date, isCurrentMonth }) => {
      const dateStr = date.toISOString().split("T")[0];
      const cellMidnight = new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
      const isToday = dateStr === todayStr;
      const isAnchor = anchorMidnight > 0 && cellMidnight === anchorMidnight;

      let cycleIndex = -1;
      let dayShifts: string[] = [];
      
      if (cycleLengthDays > 0 && anchorMidnight > 0) {
        const diffTime = cellMidnight - anchorMidnight;
        const daysDiff = Math.round(diffTime / (1000 * 60 * 60 * 24));
        cycleIndex = ((daysDiff % cycleLengthDays) + cycleLengthDays) % cycleLengthDays;
        dayShifts = grid[cycleIndex] || [];
      }

      return {
        date,
        isCurrentMonth,
        isToday,
        isAnchor,
        cycleIndex,
        dayShifts,
      };
    });
  };

  return {
    activeViewMode,
    setActiveViewMode,
    previewAnchorDate,
    setPreviewAnchorDate,
    previewYear,
    previewMonth,
    handlePreviousMonth,
    handleNextMonth,
    generateCalendarCells,
  };
}

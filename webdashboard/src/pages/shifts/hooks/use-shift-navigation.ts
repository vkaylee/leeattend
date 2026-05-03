import { useState, useMemo } from "react";

export function useShiftNavigation() {
  const [currentDate, setCurrentDate] = useState(new Date());

  const { startOfWeek, weekDays } = useMemo(() => {
    const start = new Date(currentDate);
    start.setDate(currentDate.getDate() - currentDate.getDay() + 1); // Monday
    const days = Array.from({ length: 7 }, (_, index) => {
      const d = new Date(start);
      d.setDate(start.getDate() + index);
      return d;
    });
    return { startOfWeek: start, weekDays: days };
  }, [currentDate]);

  const handleNavigateWeek = (direction: "prev" | "next", onNavigate?: () => void) => {
    const d = new Date(currentDate);
    d.setDate(d.getDate() + (direction === "prev" ? -7 : 7));
    setCurrentDate(d);
    onNavigate?.();
  };

  return {
    currentDate,
    setCurrentDate,
    startOfWeek,
    weekDays,
    handleNavigateWeek
  };
}

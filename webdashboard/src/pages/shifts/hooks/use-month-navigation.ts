import { useState, useMemo } from "react";

import { startOfMonth, endOfMonth, eachDayOfInterval, addMonths, subMonths } from "date-fns";

export function useMonthNavigation() {
  const [currentMonth, setCurrentMonth] = useState(new Date());

  const { monthStart, monthEnd, monthDays } = useMemo(() => {
    const start = startOfMonth(currentMonth);
    const end = endOfMonth(currentMonth);
    const days = eachDayOfInterval({ start, end });
    return { monthStart: start, monthEnd: end, monthDays: days };
  }, [currentMonth]);

  const handleNavigateMonth = (direction: "prev" | "next") => {
    setCurrentMonth(prev =>
      direction === "prev" ? subMonths(prev, 1) : addMonths(prev, 1)
    );
  };

  return { currentMonth, setCurrentMonth, monthStart, monthEnd, monthDays, handleNavigateMonth };
}

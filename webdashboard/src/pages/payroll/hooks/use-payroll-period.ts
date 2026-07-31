import { useState } from "react";

import { getMonthPair } from "@/hooks/use-month-pair";

export function usePayrollPeriod() {
  const [calcYear, setCalcYear] = useState(() => getMonthPair().prevYear);
  const [calcMonth, setCalcMonth] = useState(() => getMonthPair().prevMonth);

  const changeMonth = (delta: number) => {
    let nextMonth = calcMonth + delta;
    let nextYear = calcYear;
    if (nextMonth > 12) {
      nextMonth = 1;
      nextYear++;
    } else if (nextMonth < 1) {
      nextMonth = 12;
      nextYear--;
    }
    setCalcMonth(nextMonth);
    setCalcYear(nextYear);
  };

  return {
    calcMonth,
    calcYear,
    changeMonth,
    setCalcMonth,
    setCalcYear,
  };
}

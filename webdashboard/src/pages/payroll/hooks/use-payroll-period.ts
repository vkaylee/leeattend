import { useState } from "react";

export function usePayrollPeriod() {
  const [calcYear, setCalcYear] = useState(() => {
    const d = new Date();
    d.setMonth(d.getMonth() - 1);
    return d.getFullYear();
  });
  const [calcMonth, setCalcMonth] = useState(() => {
    const d = new Date();
    d.setMonth(d.getMonth() - 1);
    return d.getMonth() + 1;
  });

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

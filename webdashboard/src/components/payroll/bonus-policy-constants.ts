export const METRICS = [
  { value: "kpi_score", label: "KPI Score" },
  { value: "late_early_count", label: "Late/Early Count" },
  { value: "total_work_units", label: "Total Work Units" },
  { value: "total_leave_days", label: "Total Leave Days" },
  { value: "unpaid_leave_days", label: "Unpaid Leave Days" },
  { value: "sick_leave_days", label: "Sick Leave Days" }
] as const;

export const OPERATORS = [
  { value: ">=", label: "≥ Greater or Equal" },
  { value: "<=", label: "≤ Less or Equal" },
  { value: "==", label: "= Equals" },
  { value: ">", label: "> Greater Than" },
  { value: "<", label: "< Less Than" }
] as const;

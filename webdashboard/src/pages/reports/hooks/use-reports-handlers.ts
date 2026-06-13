import { format } from "date-fns";

import { AttendanceReportSort } from "@/lib/api";

import { FilterMode, getMonthRange } from "./use-reports-filters";

interface UseReportsHandlersProps {
  focusSearch: () => void;
  setDateRange: (range: { start: string; end: string }) => void;
  setFilterMode: (mode: FilterMode) => void;
  setSelectedDepartmentId: (id: string | null) => void;
  setSelectedEmployerId: (id: string | null) => void;
  setSelectedEmployee: (v: { id: string; name: string; employmentId?: string } | null) => void;
  setSortBy: (v: AttendanceReportSort | undefined) => void;
  setSpecificMonth: (val: { month: number; year: number }) => void;
  sortBy: AttendanceReportSort | undefined;
}

export function useReportsHandlers({
  setFilterMode,
  setDateRange,
  setSpecificMonth,
  setSelectedDepartmentId,
  setSelectedEmployerId,
  setSortBy,
  setSelectedEmployee,
  sortBy,
  focusSearch
}: UseReportsHandlersProps) {
  const handleFilterModeChange = (mode: FilterMode) => {
    setFilterMode(mode);
    let range;
    focusSearch();
    if (mode === "this_month") range = getMonthRange(0);
    else if (mode === "last_month") range = getMonthRange(-1);

    if (range) {
      setDateRange(range);
    }
  };

  const handleSpecificMonthChange = (m: number, y: number) => {
    setSpecificMonth({ month: m, year: y });
    setFilterMode("specific_month");
    const start = format(new Date(y, m - 1, 1), "yyyy-MM-dd");
    const end = format(new Date(y, m, 0), "yyyy-MM-dd");
    setDateRange({ start, end });
  };

  const handleDateRangeChange = (newRange: { start: string; end: string }) => {
    setDateRange(newRange);
  };

  const handleDepartmentChange = (departmentId: string | null) => {
    setSelectedDepartmentId(departmentId);
    focusSearch();
  };

  const handleEmployerChange = (employerId: string | null) => {
    setSelectedEmployerId(employerId);
    focusSearch();
  };

  const handleSort = (field: "leave_days" | "name" | "ot_hours" | "total_hours" | "work_hours") => {
    const defaultDesc = field !== "name";
    const descKey = `${field}_desc` as AttendanceReportSort;
    const ascKey = `${field}_asc` as AttendanceReportSort;
    if (defaultDesc) {
      setSortBy(sortBy === descKey ? ascKey : descKey);
    } else {
      setSortBy(sortBy === ascKey ? descKey : ascKey);
    }
  };

  const handleEmployeeClick = (employeeId: string, employeeName: string, employmentId?: string) => {
    setSelectedEmployee({ id: employeeId, name: employeeName, employmentId });
  };

  const handleDrawerClose = () => {
    setSelectedEmployee(null);
    focusSearch();
  };

  return {
    handleFilterModeChange,
    handleSpecificMonthChange,
    handleDateRangeChange,
    handleDepartmentChange,
    handleEmployerChange,
    handleSort,
    handleEmployeeClick,
    handleDrawerClose
  };
}

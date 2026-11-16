import { type KeyboardEvent, type RefObject, useCallback, useState } from "react";

import { useDebounce } from "@/hooks/use-debounce";
import { useAllEmployees } from "@/hooks/use-employees";
import {
  AttendanceReportSort,
  AttendanceSummary,
  Department,
  Employee,
  Employer,
} from "@/lib/api";

import { FilterMode, useReportsFilters } from "./use-reports-filters";
import { useReportsHandlers } from "./use-reports-handlers";
import { useReportsQueries } from "./use-reports-queries";

export type ReportViewMode = "summary" | "matrix";
export type { FilterMode };

export interface UseReportsPageReturn {
  allEmployees: Employee[];
  dateRange: { end: string; start: string };
  departments: Department[] | undefined;
  employeeStatus: string;
  employers: Employer[] | undefined;
  fetchNextPage: () => Promise<unknown>;
  filteredData: AttendanceSummary[];
  filterMode: FilterMode;
  handleDateRangeChange: (newRange: { end: string; start: string }) => void;
  handleDepartmentChange: (departmentId: string | null) => void;
  handleDrawerClose: () => void;
  handleEmployeeClick: (employeeId: string, employeeName: string, employerId?: string) => void;
  handleEmployerChange: (employerId: string | null) => void;
  handleFilterModeChange: (mode: FilterMode) => void;
  handleSort: (field: "leave_days" | "name" | "ot_hours" | "total_hours" | "work_hours") => void;
  handleSpecificMonthChange: (m: number, y: number) => void;
  hasNextPage: boolean | undefined;
  isExportDialogOpen: boolean;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  keyDownRef: RefObject<((e: KeyboardEvent<HTMLInputElement>) => void) | null>;
  quickStats: { otHours: string; stdHours: string; totalEmployees: number };
  searchInputRef: RefObject<HTMLInputElement | null>;
  searchQuery: string;
  selectedDepartmentId: string | null;
  selectedEmployee: { employerId?: string; id: string; name: string } | null;
  selectedEmployerId: string | null;
  setEmployeeStatus: (v: string) => void;
  setIsExportDialogOpen: (v: boolean) => void;
  setSearchQuery: (v: string) => void;
  setSelectedDepartmentId: (v: string | null) => void;
  setSelectedEmployee: (v: { employerId?: string; id: string; name: string } | null) => void;
  setSortBy: (v: AttendanceReportSort | undefined) => void;
  setViewMode: (mode: ReportViewMode) => void;
  sortBy: AttendanceReportSort | undefined;
  specificMonth: { month: number; year: number };
  viewMode: ReportViewMode;
}

export function useReportsPage(): UseReportsPageReturn {
  
  
  const {
    filterMode, setFilterMode,
    dateRange, setDateRange,
    specificMonth, setSpecificMonth,
    searchQuery, setSearchQuery,
    isExportDialogOpen, setIsExportDialogOpen,
    selectedDepartmentId, setSelectedDepartmentId,
    selectedEmployerId, setSelectedEmployerId,
    selectedEmployee, setSelectedEmployee,
    sortBy, setSortBy,
    employeeStatus, setEmployeeStatus,
    searchInputRef,
    keyDownRef,
    focusSearch
  } = useReportsFilters();
  const [viewMode, setViewModeState] = useState<ReportViewMode>(() => {
    if (typeof window === "undefined") return "summary";
    try {
      const stored = localStorage.getItem("leeattend_reports_view_mode");
      if (stored === "matrix" || stored === "summary") return stored;
    } catch {
      // ignore
    }
    return "summary";
  });

  const setViewMode = useCallback((mode: ReportViewMode) => {
    setViewModeState(mode);
    try {
      localStorage.setItem("leeattend_reports_view_mode", mode);
    } catch {
      // ignore
    }
  }, []);

  const { employees: allEmployees } = useAllEmployees();

  const debouncedSearch = useDebounce(searchQuery, 400);

  const {
    departments,
    employers,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    filteredData,
    quickStats
  } = useReportsQueries({
    debouncedSearch,
    dateRange,
    sortBy,
    selectedDepartmentId,
    employeeStatus,
    selectedEmployerId,
  });

  const {
    handleFilterModeChange,
    handleSpecificMonthChange,
    handleDateRangeChange,
    handleDepartmentChange,
    handleEmployerChange,
    handleSort,
    handleEmployeeClick,
    handleDrawerClose
  } = useReportsHandlers({
    setFilterMode,
    setDateRange,
    setSpecificMonth,
    setSelectedDepartmentId,
    setSelectedEmployerId,
    setSortBy,
    setSelectedEmployee,
    sortBy,
    focusSearch
  });

  return {
    allEmployees,
    dateRange,
    departments,
    employers,
    fetchNextPage,
    filteredData,
    filterMode,
    handleDateRangeChange,
    handleDepartmentChange,
    handleEmployerChange,
    handleDrawerClose,
    handleEmployeeClick,
    handleFilterModeChange,
    handleSort,
    handleSpecificMonthChange,
    hasNextPage,
    isExportDialogOpen,
    isFetchingNextPage,
    isLoading,
    keyDownRef,
    quickStats,
    searchInputRef,
    searchQuery,
    selectedDepartmentId,
    selectedEmployerId,
    selectedEmployee,
    employeeStatus,
    setEmployeeStatus,
    setIsExportDialogOpen,
    setSearchQuery,
    setSelectedDepartmentId,
    setSelectedEmployee,
    setSortBy,
    setViewMode,
    sortBy,
    specificMonth,
    viewMode,
  };
}

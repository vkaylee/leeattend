import React from "react";

import { useDebounce } from "@/hooks/use-debounce";
import {
  AttendanceReportSort,
  AttendanceSummary,
  Department,
  Employer,
  getCurrentWorkspaceId,
} from "@/lib/api";

import { FilterMode, useReportsFilters } from "./use-reports-filters";
import { useReportsHandlers } from "./use-reports-handlers";
import { useReportsQueries } from "./use-reports-queries";

export type { FilterMode };

export interface UseReportsPageReturn {
  dateRange: { end: string; start: string };
  departments: Department[] | undefined;
  employers: Employer[] | undefined;
  fetchNextPage: () => Promise<unknown>;
  filteredData: AttendanceSummary[];
  filterMode: FilterMode;
  handleDateRangeChange: (newRange: { end: string; start: string }) => void;
  handleDepartmentChange: (departmentId: string | null) => void;
  handleEmployerChange: (employerId: string | null) => void;
  handleDrawerClose: () => void;
  handleEmployeeClick: (employeeId: string, employeeName: string) => void;
  handleFilterModeChange: (mode: FilterMode) => void;
  handleSort: (field: "leave_days" | "name" | "ot_hours" | "total_hours" | "work_hours") => void;
  handleSpecificMonthChange: (m: number, y: number) => void;
  hasNextPage: boolean | undefined;
  isExportDialogOpen: boolean;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  keyDownRef: React.RefObject<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null>;
  quickStats: { otHours: string; stdHours: string; totalEmployees: number };
  searchInputRef: React.RefObject<HTMLInputElement | null>;
  searchQuery: string;
  selectedDepartmentId: string | null;
  selectedEmployerId: string | null;
  selectedEmployee: { id: string; name: string; employmentId?: string } | null;
  employeeStatus: string;
  setEmployeeStatus: (v: string) => void;
  setIsExportDialogOpen: (v: boolean) => void;
  setSearchQuery: (v: string) => void;
  setSelectedDepartmentId: (v: string | null) => void;
  setSelectedEmployee: (v: { id: string; name: string } | null) => void;
  setSortBy: (v: AttendanceReportSort | undefined) => void;
  sortBy: AttendanceReportSort | undefined;
  specificMonth: { month: number; year: number };
  wsId: string | null;
}

export function useReportsPage(): UseReportsPageReturn {
  const wsId = getCurrentWorkspaceId();
  
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
    wsId,
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
    sortBy,
    specificMonth,
    wsId,
  };
}

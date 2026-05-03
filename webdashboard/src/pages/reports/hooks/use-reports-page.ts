import React from "react";

import { useDebounce } from "@/hooks/use-debounce";
import {
  AttendanceReportSort,
  AttendanceSummary,
  Department,
  getCurrentWorkspaceId,
} from "@/lib/api";

import { FilterMode, useReportsFilters } from "./use-reports-filters";
import { useReportsHandlers } from "./use-reports-handlers";
import { useReportsQueries } from "./use-reports-queries";

export type { FilterMode };

export interface UseReportsPageReturn {
  dateRange: { end: string; start: string };
  departments: Department[] | undefined;
  fetchNextPage: () => Promise<unknown>;
  filteredData: AttendanceSummary[];
  filterMode: FilterMode;
  handleDateRangeChange: (newRange: { end: string; start: string }) => void;
  handleDepartmentChange: (departmentId: string | null) => void;
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
  selectedEmployee: { id: string; name: string } | null;
  setIsExportDialogOpen: (v: boolean) => void;
  setSearchQuery: (v: string) => void;
  setSelectedDepartmentId: (v: string | null) => void;
  setSelectedEmployee: (v: { id: string; name: string } | null) => void;
  setSortBy: (v: AttendanceReportSort | undefined) => void;
  sortBy: AttendanceReportSort | undefined;
  specificMonth: { month: number; year: number };
  workUnitMinutes: number;
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
    selectedEmployee, setSelectedEmployee,
    sortBy, setSortBy,
    searchInputRef,
    keyDownRef,
    focusSearch
  } = useReportsFilters();

  const debouncedSearch = useDebounce(searchQuery, 400);

  const {
    workspace,
    departments,
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
  });

  const {
    handleFilterModeChange,
    handleSpecificMonthChange,
    handleDateRangeChange,
    handleDepartmentChange,
    handleSort,
    handleEmployeeClick,
    handleDrawerClose
  } = useReportsHandlers({
    setFilterMode,
    setDateRange,
    setSpecificMonth,
    setSelectedDepartmentId,
    setSortBy,
    setSelectedEmployee,
    sortBy,
    focusSearch
  });

  const workUnitMinutes = workspace?.work_unit_minutes ?? 60;

  return {
    dateRange,
    departments,
    fetchNextPage,
    filteredData,
    filterMode,
    handleDateRangeChange,
    handleDepartmentChange,
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
    selectedEmployee,
    setIsExportDialogOpen,
    setSearchQuery,
    setSelectedDepartmentId,
    setSelectedEmployee,
    setSortBy,
    sortBy,
    specificMonth,
    workUnitMinutes,
    wsId,
  };
}

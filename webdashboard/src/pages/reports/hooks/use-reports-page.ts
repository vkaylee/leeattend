import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import React, { useEffect, useMemo, useRef, useState } from "react";

import { endOfMonth, format } from "date-fns";

import { useDebounce } from "@/hooks/use-debounce";
import {
  AttendanceReportSort,
  AttendanceSummary,
  Department,
  PaginatedResponse,
  Workspace,
  fetchAttendanceReport,
  fetchDepartments,
  fetchWorkspace,
  getCurrentWorkspaceId,
  searchEmployees,
} from "@/lib/api";
import { formatUnitsToHours } from "@/lib/utils/attendance";

const PAGE_SIZE = 50;

export type FilterMode = "this_month" | "last_month" | "custom" | "specific_month";

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

const getMonthRange = (offset = 0) => {
  const d = new Date();
  d.setMonth(d.getMonth() + offset);
  const firstDay = format(new Date(d.getFullYear(), d.getMonth(), 1), "yyyy-MM-dd");
  const lastDay = format(new Date(d.getFullYear(), d.getMonth() + 1, 0), "yyyy-MM-dd");
  return { start: firstDay, end: lastDay };
};

const getInitialFilterState = (urlFrom: string | null, urlTo: string | null) => {
  if (urlFrom && urlTo) {
    return {
      mode: "custom" as FilterMode,
      range: { start: urlFrom, end: urlTo },
    };
  }
  return {
    mode: "last_month" as FilterMode,
    range: getMonthRange(-1),
  };
};

const getInitialSpecificMonth = () => {
  const month = new Date().getMonth();
  const year = new Date().getFullYear();
  return {
    month: month === 0 ? 12 : month,
    year: month === 0 ? year - 1 : year,
  };
};

export function useReportsPage(): UseReportsPageReturn {
  const wsId = getCurrentWorkspaceId();
  const searchParams = new URLSearchParams(typeof window !== "undefined" ? window.location.search : "");
  
  const urlFrom = searchParams.get("from");
  const urlTo = searchParams.get("to");

  const { data: workspace } = useQuery<Workspace>({
    queryKey: ["workspace", wsId],
    queryFn: () => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return fetchWorkspace(wsId);
    },
    enabled: !!wsId,
  });

  const workUnitMinutes = workspace?.work_unit_minutes ?? 60;

  const initialFilter = getInitialFilterState(urlFrom, urlTo);
  const [filterMode, setFilterMode] = useState<FilterMode>(initialFilter.mode);
  const [dateRange, setDateRange] = useState(initialFilter.range);
  const [specificMonth, setSpecificMonth] = useState(getInitialSpecificMonth());

  useEffect(() => {
    if (urlFrom || urlTo) {
      const url = new URL(window.location.href);
      url.searchParams.delete("from");
      url.searchParams.delete("to");
      window.history.replaceState({}, "", url.toString());
    }
  }, [urlFrom, urlTo]);

  const [searchQuery, setSearchQuery] = useState("");
  const debouncedSearch = useDebounce(searchQuery, 400);
  const [isExportDialogOpen, setIsExportDialogOpen] = useState(false);
  const [selectedDepartmentId, setSelectedDepartmentId] = useState<string | null>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const keyDownRef = useRef<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null>(null);

  const focusSearch = () => setTimeout(() => searchInputRef.current?.focus(), 100);

  const [selectedEmployee, setSelectedEmployee] = useState<{ id: string; name: string } | null>(null);
  const [sortBy, setSortBy] = useState<AttendanceReportSort | undefined>(undefined);

  const { data: searchResults } = useQuery({
    queryKey: ["employees", wsId, "search", debouncedSearch],
    queryFn: () => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return searchEmployees(wsId, debouncedSearch);
    },
    enabled: !!wsId && debouncedSearch.trim().length > 0,
  });

  const { data: departments } = useQuery<Department[]>({
    queryKey: ["departments", wsId],
    queryFn: () => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return fetchDepartments(wsId);
    },
    enabled: !!wsId,
  });

  const employeeIdsFilter = useMemo(() => {
    if (!debouncedSearch.trim()) return undefined;
    if (!searchResults?.data.length) return ["__none__"];
    return searchResults.data.map((emp) => emp.id);
  }, [debouncedSearch, searchResults]);

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
    const end = format(endOfMonth(new Date(y, m - 1, 1)), "yyyy-MM-dd");
    setDateRange({ start, end });
  };

  const handleDateRangeChange = (newRange: { start: string; end: string }) => {
    setDateRange(newRange);
  };

  const handleDepartmentChange = (departmentId: string | null) => {
    setSelectedDepartmentId(departmentId);
    focusSearch();
  };

  const {
    data: reportPages,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery<PaginatedResponse<AttendanceSummary>>({
    queryKey: ["attendance-report", wsId, dateRange, "summary", PAGE_SIZE, sortBy, employeeIdsFilter, selectedDepartmentId],
    queryFn: ({ pageParam }) => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return fetchAttendanceReport(wsId, dateRange.start, dateRange.end, "summary", pageParam as number, PAGE_SIZE, sortBy, employeeIdsFilter, selectedDepartmentId) as Promise<PaginatedResponse<AttendanceSummary>>;
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
      const total = lastPage.total || 0;
      if (totalLoaded >= total) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId,
  });

  const filteredData = useMemo(() => {
    if (!reportPages) return [];
    return reportPages.pages.flatMap((p) => p.data);
  }, [reportPages]);

  const { data: allStatsResponse } = useQuery<PaginatedResponse<AttendanceSummary>>({
    queryKey: ["attendance-report-stats", wsId, dateRange, "summary", sortBy, employeeIdsFilter, selectedDepartmentId],
    queryFn: () => {
      if (!wsId) return Promise.reject(new Error("No workspace ID"));
      return fetchAttendanceReport(wsId, dateRange.start, dateRange.end, "summary", 1, 99999, sortBy, employeeIdsFilter, selectedDepartmentId) as Promise<PaginatedResponse<AttendanceSummary>>;
    },
    enabled: !!wsId,
  });

  const quickStats = useMemo(() => {
    const allData = allStatsResponse?.data ?? [];
    const stdHours = formatUnitsToHours(
      allData.reduce((acc, r) => acc + (r.total_std_units || 0), 0),
      workUnitMinutes
    );
    const otHours = formatUnitsToHours(
      allData.reduce((acc, r) => acc + (r.total_ot_units || 0), 0),
      workUnitMinutes
    );

    const totalEmployees = new Set(allData.map((r) => r.employee_id)).size;
    return { stdHours, otHours, totalEmployees };
  }, [allStatsResponse, workUnitMinutes]);

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

  const handleEmployeeClick = (employeeId: string, employeeName: string) => {
    setSelectedEmployee({ id: employeeId, name: employeeName });
  };

  const handleDrawerClose = () => {
    setSelectedEmployee(null);
    focusSearch();
  };

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

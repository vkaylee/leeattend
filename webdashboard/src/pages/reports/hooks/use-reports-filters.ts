import React, { useEffect, useRef, useState } from "react";

import { format } from "date-fns";

import { AttendanceReportSort } from "@/lib/api";

export type FilterMode = "this_month" | "last_month" | "custom" | "specific_month";

export const getMonthRange = (offset = 0) => {
  const d = new Date();
  d.setDate(1); // Prevent rollover when today is 31st
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

export function useReportsFilters() {
  const searchParams = new URLSearchParams(typeof window !== "undefined" ? window.location.search : "");
  const urlFrom = searchParams.get("from");
  const urlTo = searchParams.get("to");

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
  const [isExportDialogOpen, setIsExportDialogOpen] = useState(false);
  const [selectedDepartmentId, setSelectedDepartmentId] = useState<string | null>(null);
  const [selectedEmployee, setSelectedEmployee] = useState<{ id: string; name: string } | null>(null);
  const [sortBy, setSortBy] = useState<AttendanceReportSort | undefined>(undefined);
  const [employeeStatus, setEmployeeStatus] = useState<string>("active");

  const searchInputRef = useRef<HTMLInputElement>(null);
  const keyDownRef = useRef<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null>(null);

  const focusSearch = () => setTimeout(() => searchInputRef.current?.focus(), 100);

  return {
    filterMode, setFilterMode,
    dateRange, setDateRange,
    specificMonth, setSpecificMonth,
    searchQuery, setSearchQuery,
    isExportDialogOpen, setIsExportDialogOpen,
    selectedDepartmentId, setSelectedDepartmentId,
    selectedEmployee, setSelectedEmployee,
    sortBy, setSortBy,
    employeeStatus, setEmployeeStatus,
    searchInputRef,
    keyDownRef,
    focusSearch
  };
}

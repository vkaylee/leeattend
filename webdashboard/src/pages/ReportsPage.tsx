import React, { useState, useMemo, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { Download, CalendarCheck, FileBarChart, Clock, Users2 } from "lucide-react";
import { useSearchParams, Link } from "react-router-dom";
import { format, endOfMonth } from "date-fns";
import {
  fetchAttendanceReport,
  getCurrentWorkspaceId,
  fetchWorkspace,
  searchEmployees,
  fetchDepartments,
  AttendanceReportSort,
} from "../lib/api";
import { useDebounce } from "@/hooks/use-debounce";
import { formatUnitsToHours } from "@/lib/utils/attendance";
import ReportExportDialog from "@/components/features/reports/ReportExportDialog";
import ReportFilters from "@/components/features/reports/ReportFilters";
import SummaryReportTable from "@/components/features/reports/SummaryReportTable";
import EmployeeDailyDrawer from "@/components/features/reports/EmployeeDailyDrawer";

const PAGE_SIZE = 50;

export default function ReportsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [searchParams, setSearchParams] = useSearchParams();

  // Read URL params for cross-navigation
  const urlFrom = searchParams.get("from");
  const urlTo = searchParams.get("to");

  const { data: workspace } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => fetchWorkspace(wsId!),
    enabled: !!wsId,
  });

  const workUnitMinutes = workspace?.work_unit_minutes || 60;

  const getMonthRange = (offset: number = 0) => {
    const d = new Date(); d.setMonth(d.getMonth() + offset);
    const firstDay = format(new Date(d.getFullYear(), d.getMonth(), 1), 'yyyy-MM-dd');
    const lastDay = format(new Date(d.getFullYear(), d.getMonth() + 1, 0), 'yyyy-MM-dd');
    return { start: firstDay, end: lastDay };
  };

  const [filterMode, setFilterMode] = useState<"this_month" | "last_month" | "custom" | "specific_month">(
    urlFrom && urlTo ? "custom" : "last_month"
  );
  const [dateRange, setDateRange] = useState(
    urlFrom && urlTo ? { start: urlFrom, end: urlTo } : getMonthRange(-1)
  );
  const [specificMonth, setSpecificMonth] = useState({
    month: new Date().getMonth() === 0 ? 12 : new Date().getMonth(),
    year: new Date().getMonth() === 0 ? new Date().getFullYear() - 1 : new Date().getFullYear()
  });

  // Clear URL params after initialization
  useEffect(() => {
    if (urlFrom || urlTo) {
      setSearchParams({}, { replace: true });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const [searchQuery, setSearchQuery] = useState("");
  const debouncedSearch = useDebounce(searchQuery, 400);
  const [isExportDialogOpen, setIsExportDialogOpen] = useState(false);
  const [selectedDepartmentId, setSelectedDepartmentId] = useState<string | null>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  const focusSearch = () => setTimeout(() => searchInputRef.current?.focus(), 100);

  // Drill-down state
  const [selectedEmployee, setSelectedEmployee] = useState<{ id: string; name: string } | null>(null);

  const [sortBy, setSortBy] = useState<AttendanceReportSort | undefined>(undefined);
  const [focusedRowIndex, setFocusedRowIndex] = useState(0);

  // Reset focus when data changes
  useEffect(() => {
    setFocusedRowIndex(0);
  }, [searchQuery, filterMode, specificMonth, dateRange]);

  // Resolve search query → employee IDs (server-side)
  const { data: searchResults } = useQuery({
    queryKey: ["employees", wsId, "search", debouncedSearch],
    queryFn: () => searchEmployees(wsId!, debouncedSearch),
    enabled: !!wsId && debouncedSearch.trim().length > 0,
  });

  // Fetch departments for chip filter
  const { data: departments } = useQuery({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId!),
    enabled: !!wsId,
  });

  // Derive employee IDs from search results for server-side filtering
  const employeeIdsFilter = useMemo(() => {
    if (!debouncedSearch.trim()) return undefined; // No filter — show all
    if (!searchResults?.data?.length) return ["__none__"]; // Search returned nothing — show empty
    return searchResults.data.map((emp) => emp.id);
  }, [debouncedSearch, searchResults]);

  const handleFilterModeChange = (mode: "this_month" | "last_month" | "custom" | "specific_month") => {
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
    const start = format(new Date(y, m - 1, 1), 'yyyy-MM-dd');
    const end = format(endOfMonth(new Date(y, m - 1, 1)), 'yyyy-MM-dd');
    setDateRange({ start, end });
  };

  const handleDateRangeChange = (newRange: { start: string; end: string }) => {
    setDateRange(newRange);
  };

  const handleSearchQueryChange = (query: string) => {
    setSearchQuery(query);
  };

  const handleDepartmentChange = (departmentId: string | null) => {
    setSelectedDepartmentId(departmentId);
    focusSearch();
  };

  // Fetch summary with infinite scroll
  const {
    data: reportPages,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: ["attendance-report", wsId, dateRange, "summary", PAGE_SIZE, sortBy, employeeIdsFilter, selectedDepartmentId],
    queryFn: ({ pageParam }) =>
      fetchAttendanceReport(wsId!, dateRange.start, dateRange.end, "summary", pageParam as number, PAGE_SIZE, sortBy, employeeIdsFilter, selectedDepartmentId),
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data?.length || 0), 0);
      const total = lastPage.total || 0;
      if (totalLoaded >= total) return undefined;
      return allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId,
  });

  // Flatten all pages into a single array
  const filteredData = useMemo(() => {
    if (!reportPages) return [];
    return reportPages.pages.flatMap((p) => p.data || []);
  }, [reportPages]);

  // Separate query for ALL employees (unpaginated) — used only for stats computation
  const { data: allStatsResponse } = useQuery({
    queryKey: ["attendance-report-stats", wsId, dateRange, "summary", sortBy, employeeIdsFilter, selectedDepartmentId],
    queryFn: () => fetchAttendanceReport(wsId!, dateRange.start, dateRange.end, "summary", 1, 99999, sortBy, employeeIdsFilter, selectedDepartmentId),
    enabled: !!wsId,
  });

  // Compute quick stats from the FULL dataset (all employees, not just loaded pages)
  const quickStats = useMemo(() => {
    const allData = allStatsResponse?.data || [];
    const stdHours = formatUnitsToHours(
      allData.reduce((acc: number, r: any) => acc + (r.total_std_units || 0), 0),
      workUnitMinutes
    );
    const otHours = formatUnitsToHours(
      allData.reduce((acc: number, r: any) => acc + (r.total_ot_units || 0), 0),
      workUnitMinutes
    );
    const totalEmployees = new Set(allData.map((r: any) => r.employee_id)).size;
    return { stdHours, otHours, totalEmployees };
  }, [allStatsResponse, workUnitMinutes]);

  // Intersection Observer for infinite scroll trigger
  const loadMoreRef = useRef<HTMLTableRowElement>(null);

  useEffect(() => {
    if (!loadMoreRef.current) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasNextPage && !isFetchingNextPage) {
          fetchNextPage();
        }
      },
      { threshold: 0.1 }
    );
    observer.observe(loadMoreRef.current);
    return () => observer.disconnect();
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  const handleSort = (field: 'name' | 'work_hours' | 'leave_days') => {
    let nextSort: AttendanceReportSort;
    if (field === 'name') {
      nextSort = sortBy === 'name_asc' ? 'name_desc' : 'name_asc';
    } else if (field === 'work_hours') {
      nextSort = sortBy === 'work_hours_desc' ? 'work_hours_asc' : 'work_hours_desc';
    } else {
      nextSort = sortBy === 'leave_days_desc' ? 'leave_days_asc' : 'leave_days_desc';
    }
    setSortBy(nextSort);
  };

  const handleEmployeeClick = (employeeId: string, employeeName: string) => {
    setSelectedEmployee({ id: employeeId, name: employeeName });
  };

  const handleDrawerClose = () => {
    setSelectedEmployee(null);
    focusSearch();
  };

  if (!wsId) return <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 pb-20 animate-in fade-in duration-700">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="space-y-1">
          <h1 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">{t("reports.title")}</h1>
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("reports.subtitle")}</p>
        </div>

        <div className="flex items-center gap-3">
          <Link
            to={`/attendance-logs?from=${dateRange.start}&to=${dateRange.end}`}
            className="flex items-center gap-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 px-8 py-3 rounded-2xl hover:bg-primary-50 dark:hover:bg-primary-950 transition-all font-bold text-xs uppercase tracking-widest shadow-xl active:scale-95"
          >
            <CalendarCheck size={18} /> {t("reports.view_raw_logs").toUpperCase()}
          </Link>
          <button
            onClick={() => setIsExportDialogOpen(true)}
            className="flex items-center gap-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 px-8 py-3 rounded-2xl hover:bg-slate-50 dark:hover:bg-slate-800 transition-all font-bold text-xs uppercase tracking-widest shadow-xl active:scale-95"
          >
            <Download size={18} /> {t("reports.export_excel").toUpperCase()}
          </button>
        </div>
      </div>

      <ReportFilters
        filterMode={filterMode}
        dateRange={dateRange}
        specificMonth={specificMonth}
        searchQuery={searchQuery}
        searchInputRef={searchInputRef}
        t={t}
        onFilterModeChange={handleFilterModeChange}
        onDateRangeChange={handleDateRangeChange}
        onSpecificMonthChange={handleSpecificMonthChange}
        onSearchQueryChange={handleSearchQueryChange}
        departments={departments}
        selectedDepartmentId={selectedDepartmentId}
        onDepartmentChange={handleDepartmentChange}
        onSearchKeyDown={(e) => {
          if (e.key === 'ArrowDown') {
            e.preventDefault();
            setFocusedRowIndex(prev => Math.min(prev + 1, filteredData.length - 1));
          } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            setFocusedRowIndex(prev => Math.max(prev - 1, 0));
          } else if (e.key === 'Enter') {
            e.preventDefault();
            if (filteredData.length > 0 && filteredData[focusedRowIndex]) {
              e.currentTarget.blur();
              handleEmployeeClick(filteredData[focusedRowIndex].employee_id, filteredData[focusedRowIndex].employee_name);
            }
          }
        }}
        statsSlot={
          <>
            <div className="flex items-center gap-2 bg-slate-900 dark:bg-slate-800 px-4 py-2.5 rounded-2xl border border-slate-800 dark:border-slate-700 shadow-lg">
              <FileBarChart size={14} className="text-primary-400" />
              <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{t("reports.quick_stats.std_hours")}</span>
              <span className="font-mono font-bold text-white text-sm tracking-tighter">{quickStats.stdHours}h</span>
            </div>
            <div className="flex items-center gap-2 bg-gradient-to-r from-orange-900 to-orange-950 px-4 py-2.5 rounded-2xl border border-orange-800 shadow-lg">
              <Clock size={14} className="text-orange-400" />
              <span className="text-[10px] font-bold text-orange-400/70 uppercase tracking-widest">{t("reports.quick_stats.ot_hours")}</span>
              <span className="font-mono font-bold text-orange-100 text-sm tracking-tighter">{quickStats.otHours}h</span>
            </div>
            <div className="flex items-center gap-2 bg-white dark:bg-slate-900 px-4 py-2.5 rounded-2xl border border-slate-200 dark:border-slate-700 shadow-lg">
              <Users2 size={14} className="text-primary-600 dark:text-primary-400" />
              <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{t("common.employees")}</span>
              <span className="font-mono font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tighter">{quickStats.totalEmployees}</span>
            </div>
          </>
        }
      />

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
        <div className="overflow-auto max-h-[60vh]">
          <SummaryReportTable
            data={filteredData}
            isLoading={isLoading}
            workUnitMinutes={workUnitMinutes}
            sortBy={sortBy}
            focusedRowIndex={focusedRowIndex}
            t={t}
            onSort={handleSort}
            onEmployeeClick={handleEmployeeClick}
            loadMoreRef={loadMoreRef}
            isFetchingNextPage={isFetchingNextPage}
            hasNextPage={hasNextPage}
          />
        </div>
      </div>

      {/* Employee Daily Drill-down Drawer */}
      <EmployeeDailyDrawer
        key={selectedEmployee?.id || "closed"}
        isOpen={!!selectedEmployee}
        onClose={handleDrawerClose}
        workspaceId={wsId}
        employeeId={selectedEmployee?.id || ""}
        employeeName={selectedEmployee?.name || ""}
        dateRange={dateRange}
        workUnitMinutes={workUnitMinutes}
      />

      <ReportExportDialog
        isOpen={isExportDialogOpen}
        onClose={() => setIsExportDialogOpen(false)}
        wsId={wsId || ""}
        startDate={dateRange.start}
        endDate={dateRange.end}
        reportType="detail"
        employeeIds={employeeIdsFilter && employeeIdsFilter[0] !== "__none__" ? employeeIdsFilter : undefined}
      />

    </div>
  );
}

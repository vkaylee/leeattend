import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Download, CalendarCheck, FileBarChart, Clock, Users2, ArrowUp, ArrowDown, ArrowUpDown } from "lucide-react";
import React, { useState, useMemo, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { format, endOfMonth } from "date-fns";

import type { NavigableListColumn } from "@/components/common/navigable-list";

import NavigableList from "@/components/common/navigable-list";
import EmployeeDailyDrawer from "@/components/features/reports/employee-daily-drawer";
import ReportExportDialog from "@/components/features/reports/report-export-dialog";
import ReportFilters from "@/components/features/reports/report-filters";
import { useDebounce } from "@/hooks/use-debounce";
import { formatUnitsToHours } from "@/lib/utils/attendance";

import {
  fetchAttendanceReport,
  getCurrentWorkspaceId,
  fetchWorkspace,
  searchEmployees,
  fetchDepartments,
  AttendanceReportSort,
  AttendanceSummary,
} from "../lib/api";

const PAGE_SIZE = 50;

export default function ReportsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  // URL search params — read once on mount for cross-page navigation
  const searchParams = new URLSearchParams(window.location.search);
 
  const setSearchParams = (_params: Record<string, string>, _options?: unknown) => {
    // No-op: local state only
  };

  // Read URL params for cross-navigation
  const urlFrom = searchParams.get("from");
  const urlTo = searchParams.get("to");

  const { data: workspace } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => fetchWorkspace(wsId!),
    enabled: !!wsId,
  });

  const workUnitMinutes = workspace?.work_unit_minutes || 60;

  const getMonthRange = (offset = 0) => {
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
    // eslint-disable-next-line react-hooks/exhaustive-deps -- Intentional: effect should only fire when listed deps change
  }, []);

  const [searchQuery, setSearchQuery] = useState("");
  const debouncedSearch = useDebounce(searchQuery, 400);
  const [isExportDialogOpen, setIsExportDialogOpen] = useState(false);
  const [selectedDepartmentId, setSelectedDepartmentId] = useState<string | null>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const keyDownRef = useRef<((e: React.KeyboardEvent<HTMLInputElement>) => void) | null>(null);

  const focusSearch = () => setTimeout(() => searchInputRef.current?.focus(), 100);

  // Drill-down state
  const [selectedEmployee, setSelectedEmployee] = useState<{ id: string; name: string } | null>(null);

  const [sortBy, setSortBy] = useState<AttendanceReportSort | undefined>(undefined);



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
      fetchAttendanceReport(wsId!, dateRange.start, dateRange.end, "summary", pageParam, PAGE_SIZE, sortBy, employeeIdsFilter, selectedDepartmentId),
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
 
      (allData as AttendanceSummary[]).reduce((accumulator: number, r: AttendanceSummary) => accumulator + (r.total_std_units || 0), 0),
      workUnitMinutes
    );
    const otHours = formatUnitsToHours(
 
      (allData as AttendanceSummary[]).reduce((accumulator: number, r: AttendanceSummary) => accumulator + (r.total_ot_units || 0), 0),
      workUnitMinutes
    );
 
    const totalEmployees = new Set((allData as AttendanceSummary[]).map((r: AttendanceSummary) => r.employee_id)).size;
    return { stdHours, otHours, totalEmployees };
  }, [allStatsResponse, workUnitMinutes]);



  type SortField = 'name' | 'work_hours' | 'leave_days' | 'ot_hours' | 'total_hours';

  const getSortIcon = (field: SortField) => {
    const prefix = field;
    if (sortBy === `${prefix ?? ""}_asc`) return <ArrowUp className="ml-1 text-primary-600 dark:text-primary-400" size={12} />;
    if (sortBy === `${prefix ?? ""}_desc`) return <ArrowDown className="ml-1 text-primary-600 dark:text-primary-400" size={12} />;
    return <ArrowUpDown className="ml-1 opacity-20 group-hover:opacity-100 transition-opacity" size={12} />;
  };

  const handleSort = (field: SortField) => {
    const defaultDesc = field !== 'name'; // numeric fields default to desc first
    const descKey = `${field ?? ""}_desc` as AttendanceReportSort;
    const ascKey = `${field ?? ""}_asc` as AttendanceReportSort;
    if (defaultDesc) {
      setSortBy(sortBy === descKey ? ascKey : descKey);
    } else {
      setSortBy(sortBy === ascKey ? descKey : ascKey);
    }
  };

  // Column definitions for NavigableList
  const reportColumns: NavigableListColumn<AttendanceSummary>[] = [
    {
      key: "name",
      header: <div className="flex items-center">{t("common.employee")}{getSortIcon('name')}</div>,
      onHeaderClick: () => { handleSort('name'); },
      cell: (row) => (
        <div>
          <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">{row.employee_name}</div>
          <div className="text-[9px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest mt-0.5">{row.employee_id_number || "—"}</div>
        </div>
      ),
      headerClassName: "px-10",
      cellClassName: "px-10 py-6",
    },
    {
      key: "std_hours",
      header: <div className="flex items-center justify-end">{t("reports.table.std_hours")}{getSortIcon('work_hours')}</div>,
      onHeaderClick: () => { handleSort('work_hours'); },
      cell: (row) => (
        <div className="font-mono font-bold text-primary-600 dark:text-primary-400 text-base tracking-tighter cursor-default" title={`${row.total_std_units ?? ""} units`}>
          {formatUnitsToHours(row.total_std_units, workUnitMinutes)}h
        </div>
      ),
      headerClassName: "text-right",
      cellClassName: "py-6 text-right",
    },
    {
      key: "ot_hours",
      header: <div className="flex items-center justify-end">{t("reports.table.ot_hours")}{getSortIcon('ot_hours')}</div>,
      onHeaderClick: () => { handleSort('ot_hours'); },
      cell: (row) => row.total_ot_units > 0 ? (
        <div className="font-mono font-bold text-orange-600 dark:text-orange-400 text-base tracking-tighter cursor-default" title={`${row.total_ot_units ?? ""} OT units`}>
          {formatUnitsToHours(row.total_ot_units, workUnitMinutes)}h
        </div>
      ) : (
        <span className="text-slate-200 dark:text-slate-700 font-bold">—</span>
      ),
      headerClassName: "text-right text-orange-500 dark:text-orange-400",
      cellClassName: "py-6 text-right",
    },
    {
      key: "total_hours",
      header: <div className="flex items-center justify-end">{t("reports.table.total_hours")}{getSortIcon('total_hours')}</div>,
      onHeaderClick: () => { handleSort('total_hours'); },
      cell: (row) => (
        <div className="font-mono font-bold text-slate-700 dark:text-slate-200 text-base tracking-tighter cursor-default" title={`${row.total_std_units + row.total_ot_units} total units`}>
          {formatUnitsToHours(row.total_std_units + row.total_ot_units, workUnitMinutes)}h
        </div>
      ),
      headerClassName: "text-right",
      cellClassName: "py-6 text-right",
    },
    {
      key: "leave_days",
      header: <div className="flex items-center justify-end">{t("reports.table.leave_days")}{getSortIcon('leave_days')}</div>,
      onHeaderClick: () => { handleSort('leave_days'); },
      cell: (row) => <span className="font-mono font-bold text-orange-600 dark:text-orange-300 text-base tracking-tighter">{row.total_leave_days} {t("common.days_short")}</span>,
      headerClassName: "px-10 text-right",
      cellClassName: "px-10 py-6 text-right",
    },
    {
      key: "notes",
      header: t("common.notes"),
      cell: () => <span className="text-slate-200 dark:text-slate-700 font-bold tracking-widest">—</span>,
      headerClassName: "px-10 text-right",
      cellClassName: "px-10 py-6 text-right",
    },
  ];

  const handleEmployeeClick = (employeeId: string, employeeName: string) => {
    setSelectedEmployee({ id: employeeId, name: employeeName });
  };

  const handleDrawerClose = () => {
    setSelectedEmployee(null);
    focusSearch();
  };

  if (!wsId) return <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 pb-20 animate-in fade-in duration-700" data-testid="reports-page">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="space-y-1">
          <h1 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">{t("reports.title")}</h1>
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("reports.subtitle")}</p>
        </div>

        <div className="flex items-center gap-3">
          <Link
            className="flex items-center gap-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 px-8 py-3 rounded-2xl hover:bg-primary-50 dark:hover:bg-primary-950 transition-all font-bold text-xs uppercase tracking-widest shadow-xl active:scale-95"
 
            search={{ from: dateRange.start, to: dateRange.end } as Record<string, string>}
            to="/attendance-logs"
          >
            <CalendarCheck size={18} /> {t("reports.view_raw_logs").toUpperCase()}
          </Link>
          <button
            className="flex items-center gap-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 px-8 py-3 rounded-2xl hover:bg-slate-50 dark:hover:bg-slate-800 transition-all font-bold text-xs uppercase tracking-widest shadow-xl active:scale-95"
            data-testid="export-report-btn"
            onClick={() => { setIsExportDialogOpen(true); }}
          >
            <Download size={18} /> {t("reports.export_excel").toUpperCase()}
          </button>
        </div>
      </div>

      <NavigableList<AttendanceSummary>
        className="animate-in slide-in-from-bottom-8 duration-700"
        columns={reportColumns}
        emptyMessage={t("reports.messages.no_data")}
        fetchNextPage={fetchNextPage}
        hasNextPage={hasNextPage}
        headerSlot={
          <ReportFilters
            dateRange={dateRange}
            departments={departments}
            filterMode={filterMode}
            onDateRangeChange={handleDateRangeChange}
            onDepartmentChange={handleDepartmentChange}
            onFilterModeChange={handleFilterModeChange}
            onSearchKeyDown={(e) => keyDownRef.current?.(e)}
            onSearchQueryChange={handleSearchQueryChange}
            onSpecificMonthChange={handleSpecificMonthChange}
            searchInputRef={searchInputRef}
            searchQuery={searchQuery}
            selectedDepartmentId={selectedDepartmentId}
            specificMonth={specificMonth}
            statsSlot={
              <>
                <div className="flex items-center gap-2 bg-slate-900 dark:bg-slate-800 px-4 py-2.5 rounded-2xl border border-slate-800 dark:border-slate-700 shadow-lg">
                  <FileBarChart className="text-primary-400" size={14} />
                  <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{t("reports.quick_stats.std_hours")}</span>
                  <span className="font-mono font-bold text-white text-sm tracking-tighter">{quickStats.stdHours}h</span>
                </div>
                <div className="flex items-center gap-2 bg-gradient-to-r from-orange-900 to-orange-950 px-4 py-2.5 rounded-2xl border border-orange-800 shadow-lg">
                  <Clock className="text-orange-400" size={14} />
                  <span className="text-[10px] font-bold text-orange-400/70 uppercase tracking-widest">{t("reports.quick_stats.ot_hours")}</span>
                  <span className="font-mono font-bold text-orange-100 text-sm tracking-tighter">{quickStats.otHours}h</span>
                </div>
                <div className="flex items-center gap-2 bg-white dark:bg-slate-900 px-4 py-2.5 rounded-2xl border border-slate-200 dark:border-slate-700 shadow-lg">
                  <Users2 className="text-primary-600 dark:text-primary-400" size={14} />
                  <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{t("common.employees")}</span>
                  <span className="font-mono font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tighter">{quickStats.totalEmployees}</span>
                </div>
              </>
            }
            t={t}
          />
        }
        isFetchingNextPage={isFetchingNextPage}
        isLoading={isLoading}
        itemKey={(row) => row.employee_id}
        items={filteredData as AttendanceSummary[]}
        maxHeightClass=""
        onKeyDownRef={keyDownRef}
        onSearchChange={handleSearchQueryChange}
        onSelect={(row) => { handleEmployeeClick(row.employee_id, row.employee_name); }}
        resetDeps={[searchQuery, filterMode, specificMonth, dateRange]}
        rowIdPrefix="summary-row"
        searchInputRef={searchInputRef}
        searchPlaceholder={t("reports.search_placeholder")}
        searchTestId="report-search"
        searchValue={searchQuery}
        showChevron
      />

      {/* Employee Daily Drill-down Drawer */}
      <EmployeeDailyDrawer
        dateRange={dateRange}
        employeeId={selectedEmployee?.id || ""}
        employeeName={selectedEmployee?.name || ""}
        isOpen={!!selectedEmployee}
        key={selectedEmployee?.id || "closed"}
        onClose={handleDrawerClose}
        workspaceId={wsId}
        workUnitMinutes={workUnitMinutes}
      />

      <ReportExportDialog
        employeeIds={employeeIdsFilter && employeeIdsFilter[0] !== "__none__" ? employeeIdsFilter : undefined}
        endDate={dateRange.end}
        isOpen={isExportDialogOpen}
        onClose={() => { setIsExportDialogOpen(false); }}
        reportType="detail"
        startDate={dateRange.start}
        wsId={wsId || ""}
      />

    </div>
  );
}

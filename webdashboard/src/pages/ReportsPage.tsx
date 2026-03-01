import React, { useState, useMemo, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { Download, CalendarCheck } from "lucide-react";
import { useSearchParams, Link } from "react-router-dom";
import { format, endOfMonth, getDaysInMonth } from "date-fns";
import {
  fetchAttendanceReport,
  getCurrentWorkspaceId,
  fetchWorkspace,
  searchEmployees,
  AttendanceReportSort,
} from "../lib/api";
import { useDebounce } from "@/hooks/use-debounce";
import PaginationControls from "@/components/common/PaginationControls";
import ReportExportDialog from "@/components/features/reports/ReportExportDialog";
import ReportFilters from "@/components/features/reports/ReportFilters";
import SummaryReportTable from "@/components/features/reports/SummaryReportTable";
import ReportStats from "@/components/features/reports/ReportStats";
import EmployeeDailyDrawer from "@/components/features/reports/EmployeeDailyDrawer";

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
  const searchInputRef = useRef<HTMLInputElement>(null);

  const focusSearch = () => setTimeout(() => searchInputRef.current?.focus(), 100);

  // Drill-down state
  const [selectedEmployee, setSelectedEmployee] = useState<{ id: string; name: string } | null>(null);

  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(() => getDaysInMonth(new Date(getMonthRange(-1).start)));
  const [sortBy, setSortBy] = useState<AttendanceReportSort | undefined>(undefined);

  // Resolve search query → employee IDs (server-side)
  const { data: searchResults } = useQuery({
    queryKey: ["employees", wsId, "search", debouncedSearch],
    queryFn: () => searchEmployees(wsId!, debouncedSearch),
    enabled: !!wsId && debouncedSearch.trim().length > 0,
  });

  // Derive employee IDs from search results for server-side filtering
  const employeeIdsFilter = useMemo(() => {
    if (!debouncedSearch.trim()) return undefined; // No filter — show all
    if (!searchResults?.data?.length) return ["__none__"]; // Search returned nothing — show empty
    return searchResults.data.map((emp) => emp.id);
  }, [debouncedSearch, searchResults]);

  const updatePageSizeForDateRange = (start: string, end: string) => {
    const startDate = new Date(start);
    const endDate = new Date(end);
    if (startDate.getDate() === 1 && format(endDate, 'yyyy-MM-dd') === format(endOfMonth(startDate), 'yyyy-MM-dd')) {
      setPageSize(getDaysInMonth(startDate));
    }
    setPage(1);
  };

  const handleFilterModeChange = (mode: "this_month" | "last_month" | "custom" | "specific_month") => {
    setFilterMode(mode);
    let range;
    focusSearch();
    if (mode === "this_month") range = getMonthRange(0);
    else if (mode === "last_month") range = getMonthRange(-1);

    if (range) {
      setDateRange(range);
      updatePageSizeForDateRange(range.start, range.end);
    }
  };

  const handleSpecificMonthChange = (m: number, y: number) => {
    setSpecificMonth({ month: m, year: y });
    setFilterMode("specific_month");
    const start = format(new Date(y, m - 1, 1), 'yyyy-MM-dd');
    const end = format(endOfMonth(new Date(y, m - 1, 1)), 'yyyy-MM-dd');
    setDateRange({ start, end });
    updatePageSizeForDateRange(start, end);
  };

  const handleDateRangeChange = (newRange: { start: string; end: string }) => {
    setDateRange(newRange);
    updatePageSizeForDateRange(newRange.start, newRange.end);
  };

  const handleSearchQueryChange = (query: string) => {
    setSearchQuery(query);
    setPage(1); // Reset to first page when searching
  };

  // Fetch summary with server-side employee filter
  const { data: reportResponse, isLoading } = useQuery({
    queryKey: ["attendance-report", wsId, dateRange, "summary", page, pageSize, sortBy, employeeIdsFilter],
    queryFn: () => fetchAttendanceReport(wsId!, dateRange.start, dateRange.end, "summary", page, pageSize, sortBy, employeeIdsFilter),
    enabled: !!wsId,
  });

  const reportData = reportResponse?.data || (Array.isArray(reportResponse) ? reportResponse : []);
  const totalItems = reportResponse?.total || reportData.length;
  const totalPages = reportResponse?.total ? Math.ceil(reportResponse.total / pageSize) : 1;

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
    setPage(1);
  };

  const handleEmployeeClick = (employeeId: string, employeeName: string) => {
    setSelectedEmployee({ id: employeeId, name: employeeName });
  };

  const handleDrawerClose = () => {
    setSelectedEmployee(null);
    focusSearch();
  };

  const filteredData = Array.isArray(reportData) ? reportData : [];

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
        onSearchPressEnter={() => {
          if (filteredData.length > 0) {
            handleEmployeeClick(filteredData[0].employee_id, filteredData[0].employee_name);
          }
        }}
      />

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
        <PaginationControls
          currentPage={page}
          totalPages={totalPages}
          onPageChange={setPage}
          pageSize={pageSize}
          totalItems={totalItems}
          onPageSizeChange={setPageSize}
        />

        <SummaryReportTable
          data={filteredData}
          isLoading={isLoading}
          workUnitMinutes={workUnitMinutes}
          sortBy={sortBy}
          t={t}
          onSort={handleSort}
          onEmployeeClick={handleEmployeeClick}
        />

        <PaginationControls
          currentPage={page}
          totalPages={totalPages}
          onPageChange={setPage}
          pageSize={pageSize}
          totalItems={totalItems}
          onPageSizeChange={setPageSize}
        />
      </div>

      <ReportStats
        reportType="summary"
        filteredData={filteredData}
        workUnitMinutes={workUnitMinutes}
        t={t}
      />

      {/* Employee Daily Drill-down Drawer */}
      <EmployeeDailyDrawer
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

import { format } from "date-fns";
import { Building2, Download, UserX } from "lucide-react";
import React, { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useInfiniteQuery } from "@tanstack/react-query";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";
import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import {
  DailyAttendanceReport,
  PaginatedResponse,
  fetchAttendanceReport,
  getCurrentWorkspaceId,
  ScheduleTrace,
} from "@/lib/api";
import { exportNoShowToExcel, type NoShowRow } from "@/lib/excel-no-show";
import { useAllEmployees } from "@/hooks/use-employees";
import { ScheduleTracePopover } from "@/components/shifts/schedule-trace-popover";

// Number of employees fetched per backend page. For a single-day report this
// is roughly one row per employee, so this is also ~rows per page.
const EMPLOYEE_PAGE_SIZE = 30;

// ── Helpers ────────────────────────────────────────────────────────────────

function formatShiftTime(timeStr: string | null | undefined): string {
  if (!timeStr) return "—";
  // scheduled_start/end comes as HH:MM:SS from the backend
  try {
    const parts = timeStr.split(":");
    if (parts.length >= 2) {
      return `${parts[0].padStart(2, "0")}:${parts[1].padStart(2, "0")}`;
    }
  } catch {
    // fall through
  }
  return timeStr;
}

// ── Types ──────────────────────────────────────────────────────────────────

interface EnrichedRow extends DailyAttendanceReport {
  department_name: string;
  employee_id_number: string;
}

// ── Skeleton ───────────────────────────────────────────────────────────────

function TableSkeleton() {
  return (
    <div className="space-y-2 animate-pulse">
      {Array.from({ length: 6 }).map((_, i) => (
        <div
          className="h-14 rounded-xl bg-slate-100 dark:bg-slate-800"
          key={i}
        />
      ))}
    </div>
  );
}

// ── Empty State ────────────────────────────────────────────────────────────

function EmptyState({ empty, hint }: { empty: string; hint: string }) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-20 text-center">
      <div className="w-16 h-16 rounded-2xl bg-emerald-50 dark:bg-emerald-950 flex items-center justify-center shadow-inner">
        <UserX className="text-emerald-500 dark:text-emerald-400" size={28} />
      </div>
      <p className="font-bold text-sm text-slate-700 dark:text-slate-200">{empty}</p>
      <p className="text-xs text-slate-400 dark:text-slate-500 max-w-xs">{hint}</p>
    </div>
  );
}

// ── Main Component ─────────────────────────────────────────────────────────

export function UnrecordedShiftsTab() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [selectedDept, setSelectedDept] = useState<string>("__all__");

  const today = format(new Date(), "yyyy-MM-dd");

  // Infinite-scroll fetch of today's daily attendance report. The backend
  // paginates at the employee level (`total` = total matching employees), so
  // hasNextPage compares distinct employment_ids loaded against that total.
  const {
    data: reportData,
    isLoading: isReportLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery<PaginatedResponse<DailyAttendanceReport>>({
    queryKey: ["unrecorded-shifts", wsId, today],
    queryFn: async ({ pageParam }) =>
      (await fetchAttendanceReport({
        wsId: wsId!,
        start: today,
        end: today,
        type: "daily",
        page: pageParam as number,
        limit: EMPLOYEE_PAGE_SIZE,
      })) as PaginatedResponse<DailyAttendanceReport>,
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const loadedEmployees = new Set(
        allPages.flatMap((p) => p.data.map((r) => r.employment_id))
      ).size;
      return loadedEmployees >= lastPage.total ? undefined : allPages.length + 1;
    },
    enabled: !!wsId,
    staleTime: 2 * 60 * 1000, // 2 min — fresh enough for real-time HR usage
  });

  // Fetch all employees for department name resolution
  const { employees, isLoading: isEmpLoading } = useAllEmployees(wsId);

  const isLoading = isReportLoading || isEmpLoading;

  // IntersectionObserver sentinel fires fetchNextPage when scrolled into view.
  const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage: () => { void fetchNextPage(); },
  });

  // Build employee lookup map: employee_id -> { department_name, employee_id_number }
  const employeeMap = useMemo(() => {
    const map = new Map<string, { department_name: string; employee_id_number: string }>();
    for (const emp of employees) {
      map.set(emp.id, {
        department_name: emp.department_name ?? "—",
        employee_id_number: emp.employee_id_number ?? "—",
      });
    }
    return map;
  }, [employees]);

  // Flatten all loaded pages into a single row list.
  const allRows = useMemo(
    (): DailyAttendanceReport[] =>
      reportData?.pages?.flatMap((p) => p.data) ?? [],
    [reportData]
  );

  // Maps enriched rows to the Excel module's shape and triggers the download.
  const handleExport = useCallback(
    (rows: EnrichedRow[], reportDate: string) => {
      const noShowRows: NoShowRow[] = rows.map((r) => ({
        employee_id_number: r.employee_id_number,
        employee_name: r.employee_name,
        department_name: r.department_name,
        shift_name: r.shift_name ?? null,
        scheduled_start: r.scheduled_start ?? null,
        scheduled_end: r.scheduled_end ?? null,
      }));
      void exportNoShowToExcel(noShowRows, { t, date: reportDate });
    },
    [t],
  );

  // Filter daily rows to only absent ones and enrich with employee data
  const absentRows = useMemo((): EnrichedRow[] => {
    return allRows
      .filter((row) => row.status === "absent")
      .map((row) => {
        const empInfo = employeeMap.get(row.employee_id) ?? {
          department_name: "—",
          employee_id_number: "—",
        };
        return {
          ...row,
          department_name: empInfo.department_name,
          employee_id_number: empInfo.employee_id_number,
        };
      });
  }, [allRows, employeeMap]);

  // Collect unique department names for the filter dropdown
  const departments = useMemo(() => {
    const depts = new Set(absentRows.map((r) => r.department_name).filter((d) => d !== "—"));
    return Array.from(depts).sort();
  }, [absentRows]);

  // Apply department filter
  const filteredRows = useMemo(
    () =>
      selectedDept === "__all__"
        ? absentRows
        : absentRows.filter((r) => r.department_name === selectedDept),
    [absentRows, selectedDept]
  );

  const columns = {
    employee_code: t("unrecorded_shifts.columns.employee_code"),
    employee_name: t("unrecorded_shifts.columns.employee_name"),
    department: t("unrecorded_shifts.columns.department"),
    shift_type: t("unrecorded_shifts.columns.shift_type"),
    shift_in: t("unrecorded_shifts.columns.shift_in"),
    shift_out: t("unrecorded_shifts.columns.shift_out"),
  };

  // Build trace lookup: `${employment_id}-${date}` → ScheduleTrace[] — mirrors
  // the daily-report-table-row pattern so the Phân ca column can render the
  // ScheduleTracePopover when a row carries generation_metadata traces.
  const tracesByKey = useMemo(() => {
    const map = new Map<string, ScheduleTrace[]>();
    for (const row of allRows) {
      const meta = row.generation_metadata as { schedule_traces?: ScheduleTrace[] } | undefined;
      const traces = meta?.schedule_traces;
      if (traces && traces.length > 0) {
        map.set(`${row.employment_id}-${row.date}`, traces);
      }
    }
    return map;
  }, [allRows]);

  return (
    <div className="space-y-6 animate-in fade-in duration-500">
      {/* Header row */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h3 className="font-bold text-sm text-slate-800 dark:text-slate-100">
            {t("unrecorded_shifts.title")}
          </h3>
          <p className="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
            {t("unrecorded_shifts.subtitle")}
          </p>
        </div>

        <div className="flex items-center gap-3 shrink-0">
          {/* Department filter */}
          <div className="relative">
            <Building2
              className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 pointer-events-none"
              size={14}
            />
            <select
              className="pl-8 pr-8 py-2 rounded-xl text-[11px] font-bold uppercase tracking-widest bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 appearance-none cursor-pointer hover:border-primary-400 focus:outline-none focus:ring-2 focus:ring-primary-500/30 transition-all"
              id="unrecorded-dept-filter"
              onChange={(e) => { setSelectedDept(e.target.value); }}
              value={selectedDept}
            >
              <option value="__all__">{t("unrecorded_shifts.all_departments")}</option>
              {departments.map((dept) => (
                <option key={dept} value={dept}>
                  {dept}
                </option>
              ))}
            </select>
          </div>

          {/* Export button */}
          <Button
            className="flex items-center gap-2 px-4 py-2 rounded-xl text-[11px] font-bold uppercase tracking-widest"
            disabled={filteredRows.length === 0 || isLoading}
            id="unrecorded-export-btn"
            onClick={() => { handleExport(filteredRows, today); }}
            variant="outline"
          >
            <Download size={14} />
            {t("unrecorded_shifts.export_btn")}
            {filteredRows.length > 0 && (
              <span className="ml-1 px-2 py-0.5 bg-slate-100 dark:bg-slate-800 rounded-full text-[9px]">
                {filteredRows.length}
              </span>
            )}
          </Button>
        </div>
      </div>

      {/* Table */}
      {isLoading ? (
        <TableSkeleton />
      ) : filteredRows.length === 0 ? (
        <EmptyState
          empty={t("unrecorded_shifts.empty")}
          hint={t("unrecorded_shifts.empty_hint")}
        />
      ) : (
        <div className="rounded-2xl border border-slate-200 dark:border-slate-700 overflow-hidden shadow-sm">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="bg-slate-50 dark:bg-slate-800/60 border-b border-slate-200 dark:border-slate-700">
                  <th className="px-5 py-3 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                    {columns.employee_code}
                  </th>
                  <th className="px-5 py-3 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                    {columns.employee_name}
                  </th>
                  <th className="px-5 py-3 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                    {columns.department}
                  </th>
                  <th className="px-5 py-3 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                    {columns.shift_type}
                  </th>
                  <th className="px-5 py-3 text-center text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                    {columns.shift_in}
                  </th>
                  <th className="px-5 py-3 text-center text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                    {columns.shift_out}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
                {filteredRows.map((row, idx) => (
                  <tr
                    className={cn(
                      "transition-colors hover:bg-rose-50/40 dark:hover:bg-rose-950/20 group",
                      idx % 2 === 0
                        ? "bg-white dark:bg-slate-900"
                        : "bg-slate-50/50 dark:bg-slate-800/30"
                    )}
                    key={`${row.employment_id}-${row.date}`}
                  >
                    {/* Employee Code */}
                    <td className="px-5 py-3.5">
                      <span className="font-mono font-bold text-[10px] text-primary-600 dark:text-primary-400 uppercase tracking-tight bg-primary-50 dark:bg-primary-950 px-2 py-1 rounded-lg border border-primary-100 dark:border-primary-800">
                        {row.employee_id_number}
                      </span>
                    </td>

                    {/* Name */}
                    <td className="px-5 py-3.5">
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 rounded-xl bg-rose-100 dark:bg-rose-900/40 flex items-center justify-center shrink-0">
                          <UserX className="text-rose-500 dark:text-rose-400" size={14} />
                        </div>
                        <span className="font-bold text-sm text-slate-800 dark:text-slate-100 tracking-tight">
                          {row.employee_name}
                        </span>
                      </div>
                    </td>

                    {/* Department */}
                    <td className="px-5 py-3.5">
                      <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 rounded-xl text-[10px] font-bold uppercase tracking-widest border border-blue-100 dark:border-blue-800">
                        <Building2 size={10} />
                        {row.department_name}
                      </span>
                    </td>

                    {/* Phân ca (shift type + traceability popover) */}
                    <td className="px-5 py-3.5">
                      <div className="relative flex items-center gap-1 w-fit">
                        <span className="font-bold text-slate-700 dark:text-slate-200 text-xs tracking-tight">
                          {row.shift_name ?? "—"}
                        </span>
                        <ScheduleTracePopover
                          endTime={row.scheduled_end}
                          shiftName={row.shift_name}
                          shiftSymbol={row.shift_symbol}
                          source="system"
                          startTime={row.scheduled_start}
                          traces={tracesByKey.get(`${row.employment_id}-${row.date}`) ?? []}
                        />
                      </div>
                    </td>

                    {/* Shift In */}
                    <td className="px-5 py-3.5 text-center">
                      <span className="font-mono font-bold text-sm text-slate-700 dark:text-slate-200 tabular-nums bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-lg">
                        {formatShiftTime(row.scheduled_start)}
                      </span>
                    </td>

                    {/* Shift Out */}
                    <td className="px-5 py-3.5 text-center">
                      <span className="font-mono font-bold text-sm text-slate-700 dark:text-slate-200 tabular-nums bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-lg">
                        {formatShiftTime(row.scheduled_end)}
                      </span>
                    </td>
                  </tr>
                ))}
                {hasNextPage && (
                  <InfiniteScrollSentinel
                    colSpan={6}
                    isFetchingNextPage={!!isFetchingNextPage}
                    sentinelRef={loadMoreRef}
                  />
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}

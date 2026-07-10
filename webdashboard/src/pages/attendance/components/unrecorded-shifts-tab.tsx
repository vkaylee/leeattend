import { format } from "date-fns";
import {
  AlertTriangle,
  Building2,
  Check,
  Clock,
  Download,
  Loader2,
  UserX,
  X,
} from "lucide-react";
import React, { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useInfiniteQuery, useQueryClient } from "@tanstack/react-query";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";
import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import {
  DailyAttendanceReport,
  PaginatedResponse,
  ScheduleTrace,
  fetchAttendanceReport,
  getCurrentWorkspaceId,
} from "@/lib/api";
import { exportNoShowToExcel, type NoShowRow } from "@/lib/excel-no-show";
import { useAllEmployees } from "@/hooks/use-employees";
import { ScheduleTracePopover } from "@/components/shifts/schedule-trace-popover";
import {
  computeNoShowRows,
  useNoShowRows,
  type ZonedNoShowRow,
} from "../hooks/use-no-show-rows";
import { ShiftTypePicker } from "./shift-type-picker";

// Number of employees fetched per backend page. For a single-day report this
// is roughly one row per employee, so this is also ~rows per page.
const EMPLOYEE_PAGE_SIZE = 30;

const ZoneIcon = ({ zone }: { zone: ZonedNoShowRow["zone"] }) => {
  // Decorative — the accessible name lives on the parent cell, so hide from AT.
  if (zone === "b") return <AlertTriangle aria-hidden="true" className="size-3.5" />;
  if (zone === "a") return <Clock aria-hidden="true" className="size-3.5" />;
  return <Check aria-hidden="true" className="size-3.5" />;
};

// ── Helpers ────────────────────────────────────────────────────────────────

function formatShiftTime(timeStr: string | null | undefined): string {
  if (!timeStr) return "—";
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

/** i18n key + hint for a zone label. Kept next to the icon/text so the row is
 *  never distinguishable by color alone (WCAG 2.1 AA — multi-channel cues). */
function zoneLabel(
  row: ZonedNoShowRow,
  t: (k: string, o?: Record<string, unknown>) => string,
): { key: string; hint: string | null } {
  if (row.zone === "b") {
    return {
      key: t("unrecorded_shifts.zone_b"),
      hint: row.zoneMinutes == null
        ? null
        : t("unrecorded_shifts.zone_b_hint", { minutes: row.zoneMinutes }),
    };
  }
  if (row.zone === "a") {
    return {
      key: t("unrecorded_shifts.zone_a"),
      hint: row.zoneMinutes == null
        ? null
        : t("unrecorded_shifts.zone_a_hint", { minutes: row.zoneMinutes }),
    };
  }
  return { key: t("unrecorded_shifts.zone_c"), hint: null };
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
  const queryClient = useQueryClient();

  const today = format(new Date(), "yyyy-MM-dd");

  // ── Filter state ──────────────────────────────────────────────────────────
  const [dateRange, setDateRange] = useState<{ start: string; end: string }>({
    start: today,
    end: today,
  });
  const [selectedDept, setSelectedDept] = useState<string>("__all__");
  const [selectedShiftTypes, setSelectedShiftTypes] = useState<string[]>([]);
  const [employeeIds, setEmployeeIds] = useState<string[]>([]);
  const [onlyZoneB, setOnlyZoneB] = useState(false);
  const [isExporting, setIsExporting] = useState(false);

  // Live clock: re-renders once a second so Zone A countdown / Zone B elapsed
  // labels tick in real time, and rows flip A→B at `start + grace`.
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const id = setInterval(() => { setNow(new Date()); }, 1000);
    return () => { clearInterval(id); };
  }, []);

  // NOTE: `selectedDept` is intentionally excluded — department filtering is
  // client-side (instant, no refetch) matching the original behavior. Shift type
  // and employee filters are server-side, so they belong in the key.
  const queryKey = [
    "unrecorded-shifts",
    wsId,
    dateRange.start,
    dateRange.end,
    selectedShiftTypes,
    employeeIds,
  ];

  // Infinite-scroll fetch of the daily attendance report for the selected range.
  const {
    data: reportData,
    isLoading: isReportLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery<PaginatedResponse<DailyAttendanceReport>>({
    queryKey,
    queryFn: async ({ pageParam }) =>
      (await fetchAttendanceReport({
        wsId: wsId!,
        start: dateRange.start,
        end: dateRange.end,
        type: "daily",
        page: pageParam as number,
        limit: EMPLOYEE_PAGE_SIZE,
        employeeIds: employeeIds.length > 0 ? employeeIds : undefined,
        shiftType: selectedShiftTypes.length > 0 ? selectedShiftTypes : undefined,
      })) as PaginatedResponse<DailyAttendanceReport>,
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const loadedEmployees = new Set(
        allPages.flatMap((p) => p.data.map((r) => r.employment_id)),
      ).size;
      return loadedEmployees >= lastPage.total ? undefined : allPages.length + 1;
    },
    enabled: !!wsId,
    staleTime: 2 * 60 * 1000,
  });

  const { employees, isLoading: isEmpLoading } = useAllEmployees(wsId);
  const isLoading = isReportLoading || isEmpLoading;

  // Eagerly drain every page once the first page lands (export badge + export
  // handler read the full dataset from cache); table renders progressively.
  useEffect(() => {
    if (hasNextPage && !isFetchingNextPage) {
      void fetchNextPage();
    }
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage: () => { void fetchNextPage(); },
  });

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

  const allRows = useMemo(
    (): DailyAttendanceReport[] =>
      reportData?.pages?.flatMap((p) => p.data) ?? [],
    [reportData],
  );

  // ── Enrichment + zone computation (single source for table + export) ──────
  const absentRows = useNoShowRows({
    rows: allRows,
    employeeMap,
    now,
    viewingDate: dateRange.start,
  });

  // Department options from the loaded absent rows (client-side convenience).
  const departments = useMemo(() => {
    const depts = new Set(absentRows.map((r) => r.department_name).filter((d) => d !== "—"));
    return Array.from(depts).sort();
  }, [absentRows]);

  // Final filtered list: shift-type/employee are already filtered server-side;
  // department + Zone-B toggle are client-side.
  const filteredRows = useMemo(() => {
    let rows = absentRows;
    if (selectedDept !== "__all__") {
      rows = rows.filter((r) => r.department_name === selectedDept);
    }
    if (onlyZoneB) {
      rows = rows.filter((r) => r.zone === "b");
    }
    return rows;
  }, [absentRows, selectedDept, onlyZoneB]);

  // ── Export (drains all pages, then writes the .xlsx) ─────────────────────
  const handleExport = useCallback(async () => {
    if (!wsId) return;
    setIsExporting(true);
    try {
      let guard = 0;
      while (guard < 1000) {
        const cache = queryClient.getQueryData<{
          pages: PaginatedResponse<DailyAttendanceReport>[];
          pageParams: readonly number[];
        }>(queryKey);
        const pages = cache?.pages ?? [];
        if (pages.length === 0) break;
        const loadedEmployees = new Set(
          pages.flatMap((p) => p.data.map((r) => r.employment_id)),
        ).size;
        const total = pages[pages.length - 1]?.total ?? 0;
        if (loadedEmployees >= total) break;
        await fetchNextPage();
        guard++;
      }

      const fullCache = queryClient.getQueryData<{
        pages: PaginatedResponse<DailyAttendanceReport>[];
        pageParams: readonly number[];
      }>(queryKey);
      const allReportRows = fullCache?.pages?.flatMap((p) => p.data) ?? [];

      const rows = computeNoShowRows({
        rows: allReportRows,
        employeeMap,
        now,
        viewingDate: dateRange.start,
      }).filter((r) => (selectedDept === "__all__" ? true : r.department_name === selectedDept));

      const noShowRows: NoShowRow[] = rows.map((r) => ({
        employee_id_number: r.employee_id_number,
        employee_name: r.employee_name,
        department_name: r.department_name,
        shift_name: r.shift_name ?? null,
        scheduled_start: r.scheduled_start ?? null,
        scheduled_end: r.scheduled_end ?? null,
        zone_label: zoneLabel(r, t).key,
      }));
      void exportNoShowToExcel(noShowRows, { t, date: dateRange.start });
    } finally {
      setIsExporting(false);
    }
  }, [
    wsId,
    queryClient,
    queryKey,
    fetchNextPage,
    employeeMap,
    now,
    dateRange.start,
    selectedDept,
    t,
  ]);

  const isViewingPast = dateRange.start < today;

  const clearFilters = () => {
    setSelectedDept("__all__");
    setSelectedShiftTypes([]);
    setEmployeeIds([]);
    setOnlyZoneB(false);
  };

  const columns = {
    zone: t("unrecorded_shifts.zone_header"),
    employee_code: t("unrecorded_shifts.columns.employee_code"),
    employee_name: t("unrecorded_shifts.columns.employee_name"),
    department: t("unrecorded_shifts.columns.department"),
    shift_type: t("unrecorded_shifts.columns.shift_type"),
    shift_in: t("unrecorded_shifts.columns.shift_in"),
    shift_out: t("unrecorded_shifts.columns.shift_out"),
  };

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
      {/* Header */}
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
          <Button
            className="flex items-center gap-2 px-4 py-2 rounded-xl text-[11px] font-bold uppercase tracking-widest"
            disabled={absentRows.length === 0 || isLoading || isExporting}
            id="unrecorded-export-btn"
            onClick={() => { void handleExport(); }}
            variant="outline"
          >
            {isExporting ? <Loader2 className="animate-spin" size={14} /> : <Download size={14} />}
            {t("unrecorded_shifts.export_btn")}
            {absentRows.length > 0 && !isExporting && (
              <span className="ml-1 px-2 py-0.5 bg-slate-100 dark:bg-slate-800 rounded-full text-[9px]">
                {absentRows.length}
              </span>
            )}
          </Button>
        </div>
      </div>

      {/* Filters bar */}
      <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-700 shadow-sm p-3 flex flex-wrap items-center gap-3">
        {/* Date range */}
        <div className="flex items-center gap-2">
          <div className="relative">
            <Building2
              className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 pointer-events-none"
              size={14}
            />
            <input
              aria-label={t("common.from_date")}
              className="pl-8 pr-3 py-2 rounded-xl text-[11px] font-bold bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 text-slate-600 dark:text-slate-300 outline-none focus:ring-2 focus:ring-primary-500/30"
              onChange={(e) => { setDateRange((p) => ({ ...p, start: e.target.value })); }}
              type="date"
              value={dateRange.start}
            />
          </div>
          <span className="text-slate-300 dark:text-slate-600">–</span>
          <input
            aria-label={t("common.to_date")}
            className="px-3 py-2 rounded-xl text-[11px] font-bold bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 text-slate-600 dark:text-slate-300 outline-none focus:ring-2 focus:ring-primary-500/30"
            onChange={(e) => { setDateRange((p) => ({ ...p, end: e.target.value })); }}
            type="date"
            value={dateRange.end}
          />
        </div>

        {/* Department */}
        <div className="relative">
          <Building2
            className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 pointer-events-none"
            size={14}
          />
          <select
            className="pl-8 pr-8 py-2 rounded-xl text-[11px] font-bold uppercase tracking-widest bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 text-slate-600 dark:text-slate-300 appearance-none cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary-500/30"
            id="unrecorded-dept-filter"
            onChange={(e) => { setSelectedDept(e.target.value); }}
            value={selectedDept}
          >
            <option value="__all__">{t("unrecorded_shifts.all_departments")}</option>
            {departments.map((dept) => (
              <option key={dept} value={dept}>{dept}</option>
            ))}
          </select>
        </div>

        <ShiftTypePicker onChange={setSelectedShiftTypes} selected={selectedShiftTypes} />

        {/* Only Zone B toggle */}
        <Button
          aria-pressed={onlyZoneB}
          className={cn(
            "px-3 py-2 gap-2 normal-case",
            onlyZoneB
              ? "bg-amber-50 dark:bg-amber-950/40 border-amber-300 dark:border-amber-700 text-amber-700 dark:text-amber-300 hover:bg-amber-50 dark:hover:bg-amber-950/40"
              : "text-slate-500 dark:text-slate-400 hover:border-amber-300 dark:hover:border-amber-700",
          )}
          id="unrecorded-only-zone-b"
          onClick={() => { setOnlyZoneB((v) => !v); }}
          variant="outline"
        >
          <AlertTriangle className="size-3.5" />
          {t("unrecorded_shifts.only_zone_b")}
        </Button>

        <Button
          className="px-2 py-2 gap-1.5 normal-case font-bold text-slate-400 dark:text-slate-500"
          onClick={clearFilters}
          variant="ghost"
        >
          <X className="size-3.5" />
          {t("unrecorded_shifts.clear_filters")}
        </Button>

        {isViewingPast && (
          <span className="text-[10px] text-slate-400 italic">
            {t("unrecorded_shifts.viewing_past", {
              defaultValue: "Historical — fixed records",
            })}
          </span>
        )}
      </div>

      {/* Table */}
      {isLoading ? (
        <TableSkeleton />
      ) : filteredRows.length === 0 ? (
        <EmptyState
          empty={
            onlyZoneB
              ? t("unrecorded_shifts.empty_zone_b")
              : t("unrecorded_shifts.empty")
          }
          hint={
            onlyZoneB
              ? t("unrecorded_shifts.empty_zone_b_hint")
              : t("unrecorded_shifts.empty_hint")
          }
        />
      ) : (
        <div className="rounded-2xl border border-slate-200 dark:border-slate-700 overflow-hidden shadow-sm">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="bg-slate-50 dark:bg-slate-800/60 border-b border-slate-200 dark:border-slate-700">
                  <th className="px-5 py-3 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap">
                    {columns.zone}
                  </th>
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
                {filteredRows.map((row, idx) => {
                  const { key, hint } = zoneLabel(row, t);
                  const rowBg =
                    row.zone === "b"
                      ? "bg-amber-50/60 dark:bg-amber-950/25 border-l-2 border-l-amber-400 dark:border-l-amber-500"
                      : row.zone === "a"
                        ? "bg-slate-50/60 dark:bg-slate-800/40"
                        : "bg-rose-50/40 dark:bg-rose-950/15";
                  const chip =
                    row.zone === "b"
                      ? "bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-300"
                      : row.zone === "a"
                        ? "bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400"
                        : "bg-rose-100 dark:bg-rose-900/40 text-rose-700 dark:text-rose-300";
                  return (
                    <tr
                      className={cn(
                        "transition-colors group",
                        rowBg,
                        row.zone !== "b" && "hover:bg-rose-50/40 dark:hover:bg-rose-950/20",
                        idx % 2 === 0 ? "" : "bg-opacity-50",
                      )}
                      key={`${row.employment_id}-${row.date}`}
                    >
                      {/* Zone — never color-only: icon + text + aria-label */}
                      <td className="px-5 py-3.5">
                        <span
                          className={cn(
                            "inline-flex items-center gap-1.5 px-2 py-1 rounded-lg text-[10px] font-bold uppercase tracking-widest",
                            chip,
                          )}
                        >
                          <ZoneIcon zone={row.zone} />
                          <span>{key}</span>
                          {hint && <span className="opacity-70 normal-case tracking-normal">· {hint}</span>}
                          <span className="sr-only">
                            {key}{hint ? `, ${hint}` : ""}
                          </span>
                        </span>
                      </td>

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

                      {/* Phân ca */}
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
                  );
                })}
                {hasNextPage && (
                  <InfiniteScrollSentinel
                    colSpan={7}
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

import { useInfiniteQuery } from "@tanstack/react-query";
import { useEffect, useMemo } from "react";

import { useAllEmployees } from "@/hooks/use-employees";
import { useWorkspace } from "@/hooks/use-workspace";
import {
  NoShowReportRow,
  PaginatedResponse,
  ScheduleTrace,
  fetchNoShowReport,
  getStandaloneOrganizationId,
} from "@/lib/api";

import { useNoShowRows, type ZonedNoShowRow } from "../../hooks/use-no-show-rows";

const EMPLOYEE_PAGE_SIZE = 30;
const LIVE_REPORT_REFRESH_INTERVAL_MS = 30_000;

function getTodayIso(): string {
  const now = new Date();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${now.getFullYear()}-${month}-${day}`;
}

export interface UseUnrecordedShiftsDataParams {
  employeeIds: string[];
  onlyZoneB: boolean;
  selectedDate: string;
  selectedDept: string;
  selectedShiftTypes: string[];
}

export interface UseUnrecordedShiftsDataResult {
  absentRows: ZonedNoShowRow[];
  allRows: NoShowReportRow[];
  departments: string[];
  employeeMap: Map<string, { department_name: string; employee_id_number: string }>;
  fetchNextPage: () => Promise<unknown>;
  filteredRows: ZonedNoShowRow[];
  hasNextPage: boolean | undefined;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  queryKey: unknown[];
  tracesByKey: Map<string, ScheduleTrace[]>;
  workspaceTimezone: string;
  wsId: string | null;
}

export function getNextNoShowPageParam(
  lastPage: PaginatedResponse<NoShowReportRow>,
  loadedPageCount: number,
): number | undefined {
  if (lastPage.limit <= 0 || lastPage.total <= 0) return undefined;

  const totalCandidatePages = Math.ceil(lastPage.total / lastPage.limit);
  return loadedPageCount >= totalCandidatePages ? undefined : loadedPageCount + 1;
}

export function useUnrecordedShiftsData({
  employeeIds,
  selectedDate,
  selectedShiftTypes,
  onlyZoneB,
  selectedDept,
}: UseUnrecordedShiftsDataParams): UseUnrecordedShiftsDataResult {
  const wsId = getStandaloneOrganizationId();
  const { workspace } = useWorkspace(wsId ?? undefined);
  const workspaceTimezone = workspace?.timezone ?? "UTC";

  const queryKey = [
    "unrecorded-shifts",
    wsId,
    selectedDate,
    selectedShiftTypes,
    employeeIds,
  ];

  const {
    data: reportData,
    isLoading: isReportLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery<PaginatedResponse<NoShowReportRow>>({
    queryKey,
    queryFn: async ({ pageParam }) =>
      (await fetchNoShowReport({
        wsId: wsId!, // eslint-disable-line @typescript-eslint/no-non-null-assertion -- query is disabled when wsId is null
        date: selectedDate,
        page: pageParam as number,
        limit: EMPLOYEE_PAGE_SIZE,
        employeeIds: employeeIds.length > 0 ? employeeIds : undefined,
        shiftType: selectedShiftTypes.length > 0 ? selectedShiftTypes : undefined,
      })),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) =>
      getNextNoShowPageParam(lastPage, allPages.length),
    enabled: !!wsId,
    refetchInterval:
      selectedDate === getTodayIso() ? LIVE_REPORT_REFRESH_INTERVAL_MS : false,
    refetchIntervalInBackground: false,
    staleTime: LIVE_REPORT_REFRESH_INTERVAL_MS,
  });

  const { employees, isLoading: isEmpLoading } = useAllEmployees(wsId);
  const isLoading = isReportLoading || isEmpLoading;

  // Eagerly drain every page once the first page lands.
  useEffect(() => {
    if (hasNextPage && !isFetchingNextPage) {
      void fetchNextPage();
    }
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  const employeeMap = useMemo(() => {
    const map = new Map<string, { department_name: string; employee_id_number: string }>();
    for (const emp of employees) {
      map.set(emp.id, {
        department_name: emp.department_name ?? "—",
        employee_id_number: emp.employee_id_number || "—",
      });
    }
    return map;
  }, [employees]);

  const allRows = useMemo(
    (): NoShowReportRow[] =>
      reportData?.pages.flatMap((p) => p.data) ?? [],
    [reportData],
  );

  const absentRows = useNoShowRows({
    rows: allRows,
    employeeMap,
    viewingDate: selectedDate,
    workspaceTimezone,
  });

  const departments = useMemo(() => {
    const depts = new Set(absentRows.map((r) => r.department_name).filter((d) => d !== "—"));
    return Array.from(depts).sort();
  }, [absentRows]);

  const filteredRows = useMemo(() => {
    let rows = absentRows;
    if (selectedDept !== "__all__") {
      rows = rows.filter((r) => r.department_name === selectedDept);
    }
    if (onlyZoneB && selectedDate === getTodayIso()) {
      rows = rows.filter((r) => r.recording_issue !== "missing_punch_out" && r.zone === "b");
    }
    return rows;
  }, [absentRows, selectedDate, selectedDept, onlyZoneB]);

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

  return {
    absentRows,
    allRows,
    departments,
    employeeMap,
    filteredRows,
    hasNextPage,
    isFetchingNextPage,
    isLoading,
    queryKey,
    tracesByKey,
    workspaceTimezone,
    wsId,
    fetchNextPage,
  };
}

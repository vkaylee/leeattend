import { useQuery } from "@tanstack/react-query";

import { apiFetch } from "@/lib/api";

import { dashboardSchemaByKey, overtimeBreakdownSchema, type DashboardSchemaKey } from "./dashboard-schemas";
import {
  dashboardMetricsResponseSchema,
  type DashboardMetricsResponse,
} from "./dashboard-metric-contract";
import {
  dashboardMetricsQuery,
  type DashboardMetricsFilters,
} from "./dashboard-metrics-filters";

import {
  type AttendanceTrendPoint,
  type DepartmentAttendancePoint,
  type LatenessByDay,
  type LeaveTypeBreakdown,
  type OvertimeBreakdownResponse,
  type OvertimeTrendPoint,
  type PeakTimeSlot,
  type RateQuantity,
  type RecruitmentSourceBreakdown,
  type TurnoverAnalytics,
  type UnexcusedAbsencesPoint,
  type WorkspaceStats,
} from "./use-dashboard-types";
import { useWorkspace } from "./use-workspace";

export type {
  AttendanceTrendPoint,
  DepartmentAttendancePoint,
  LatenessByDay,
  LeaveTypeBreakdown,
  OvertimeBreakdownResponse,
  OvertimeTrendPoint,
  PeakTimeSlot,
  RateQuantity,
  RecruitmentSourceBreakdown,
  TurnoverAnalytics,
  TurnoverRate,
  UnexcusedAbsencesPoint,
  WorkspaceStats,
} from "./use-dashboard-types";

/** Shared query params for chart endpoints that accept an explicit custom range. */
export interface ChartRangeParams {
  startDate?: string;
  endDate?: string;
  timezone?: string;
}

/** Appends non-empty range params to a URLSearchParams instance. */
function setRangeParams(params: URLSearchParams, range: ChartRangeParams): void {
  if (range.startDate) params.set("start_date", range.startDate);
  if (range.endDate) params.set("end_date", range.endDate);
  if (range.timezone) params.set("timezone", range.timezone);
}

// ── Factory: reusable useQuery wrapper with workspace guard ────────────────

export type DashboardQueryErrorCode =
  | "NO_WORKSPACE"
  | "HTTP_ERROR"
  | "INVALID_PAYLOAD";

export class DashboardQueryError extends Error {
  readonly code: DashboardQueryErrorCode;
  readonly endpoint: string;
  readonly queryKey: string;
  readonly status?: number;

  constructor({
    code,
    endpoint,
    message,
    queryKey,
    status,
  }: {
    code: DashboardQueryErrorCode;
    endpoint: string;
    message: string;
    queryKey: string;
    status?: number;
  }) {
    super(message);
    this.name = "DashboardQueryError";
    this.code = code;
    this.endpoint = endpoint;
    this.queryKey = queryKey;
    this.status = status;
  }
}

interface DashboardQueryOptions<T> {
  /** Additional enabled condition ANDed with the workspace guard. */
  extraEnabled?: boolean;
  /** Extra values appended to the query key (e.g. range, dates). */
  extraQueryKeyParams?: readonly unknown[];
  /** Lazily builds the API path (captures hook args in closure). */
  getPath: () => string;
  /** Parses untrusted API JSON into T. */
  parse?: (payload: unknown) => T;
  /** Unique cache key segment (e.g. "workspace-stats"). */
  key: DashboardSchemaKey;
}

// eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters -- T determines return type through useQuery<T>
function useDashboardQuery<T>(options: DashboardQueryOptions<T>) {
  const { workspace } = useWorkspace();

  return useQuery<T>({
    queryKey: [options.key, workspace?.id, ...(options.extraQueryKeyParams ?? [])],
    queryFn: async () => {
      const endpoint = options.getPath();
      if (!workspace) {
        throw new DashboardQueryError({
          code: "NO_WORKSPACE",
          endpoint,
          message: "Select a workspace to load dashboard data.",
          queryKey: options.key,
        });
      }

      const response = await apiFetch(endpoint);
      if (!response.ok) {
        throw new DashboardQueryError({
          code: "HTTP_ERROR",
          endpoint,
          message: "Dashboard data could not be loaded.",
          queryKey: options.key,
          status: response.status,
        });
      }

      try {
        const payload: unknown = await response.json();
        return options.parse
        ? options.parse(payload)
        : dashboardSchemaByKey[options.key].parse(payload) as T;
      } catch {
        throw new DashboardQueryError({
          code: "INVALID_PAYLOAD",
          endpoint,
          message: "Dashboard returned an invalid response.",
          queryKey: options.key,
          status: response.status,
        });
      }
    },
    enabled: !!workspace && (options.extraEnabled ?? true),
  });
}

// ── Public hooks (one-liners) ─────────────────────────────────────────────

export const useDashboardStats = () =>
  useDashboardQuery<WorkspaceStats>({
    key: "workspace-stats",
    getPath: () => "/v1/dashboard/workspace-stats",
  });

export function useAttendanceTrend(range: ChartRangeParams = {}) {
  return useDashboardQuery<AttendanceTrendPoint[]>({
    key: "attendance-trend",
    getPath: () => {
      const params = new URLSearchParams();
      setRangeParams(params, range);
      const qs = params.toString();
      return `/v1/dashboard/attendance-trend${qs ? `?${qs}` : ""}`;
    },
    extraQueryKeyParams: [range.startDate, range.endDate, range.timezone],
  });
}

export function useDepartmentAttendance(options: {
  range?: "today" | "week";
  topN?: number;
  rangeParams?: ChartRangeParams;
} = {}) {
  const { range, topN, rangeParams } = options;
  return useDashboardQuery<DepartmentAttendancePoint[]>({
    key: "department-attendance",
    getPath: () => {
      const params = new URLSearchParams();
      if (range) params.set("range", range);
      if (topN) params.set("top_n", String(topN));
      if (rangeParams) setRangeParams(params, rangeParams);
      const qs = params.toString();
      return `/v1/dashboard/department-attendance${qs ? `?${qs}` : ""}`;
    },
    extraQueryKeyParams: [range, topN, rangeParams?.startDate, rangeParams?.endDate, rangeParams?.timezone],
  });
}

export function useUnexcusedAbsences(
  startDate?: string,
  endDate?: string,
  groupBy?: "department" | "employee",
  timezone?: string,
) {
  return useDashboardQuery<UnexcusedAbsencesPoint[]>({
    key: "unexcused-absences",
    getPath: () => {
      const params = new URLSearchParams();
      if (startDate) params.set("start_date", startDate);
      if (endDate) params.set("end_date", endDate);
      if (groupBy) params.set("group_by", groupBy);
      if (timezone) params.set("timezone", timezone);
      const qs = params.toString();
      return `/v1/dashboard/unexcused-absences${qs ? `?${qs}` : ""}`;
    },
    extraQueryKeyParams: [startDate, endDate, groupBy, timezone],
  });
}

export const useOvertimeTrend = () =>
  useDashboardQuery<OvertimeTrendPoint[]>({
    key: "overtime-trend",
    getPath: () => "/v1/dashboard/overtime-trend",
  });

export const usePeakTimes = () =>
  useDashboardQuery<PeakTimeSlot[]>({
    key: "peak-times",
    getPath: () => "/v1/dashboard/peak-times",
  });

export const useLeaveTypesBreakdown = () =>
  useDashboardQuery<LeaveTypeBreakdown[]>({
    key: "leave-types-breakdown",
    getPath: () => "/v1/dashboard/leave-types",
  });

export function useLatenessByDay(range: ChartRangeParams = {}) {
  return useDashboardQuery<LatenessByDay[]>({
    key: "lateness-by-day",
    getPath: () => {
      const params = new URLSearchParams();
      setRangeParams(params, range);
      const qs = params.toString();
      return `/v1/dashboard/lateness-by-day${qs ? `?${qs}` : ""}`;
    },
    extraQueryKeyParams: [range.startDate, range.endDate, range.timezone],
  });
}

export const useRecruitmentSourceBreakdown = () =>
  useDashboardQuery<RecruitmentSourceBreakdown[]>({
    key: "recruitment-sources-breakdown",
    getPath: () => "/v1/dashboard/recruitment-sources",
  });

export interface TurnoverAnalyticsParams extends ChartRangeParams {
  /** Previous-period / same-period-last-year comparison. Defaults to `previous_period`. */
  comparison?: "none" | "previous_period" | "same_period_last_year";
  /** Optional Decimal override for the turnover target (e.g. `"10"`). */
  target?: string;
}

export function useTurnoverAnalytics(params: TurnoverAnalyticsParams = {}) {
  return useDashboardQuery<TurnoverAnalytics>({
    key: "turnover-analytics",
    getPath: () => {
      const sp = new URLSearchParams({
        ...(params.startDate ? { start: params.startDate } : {}),
        ...(params.endDate ? { end: params.endDate } : {}),
        ...(params.comparison ? { comparison: params.comparison } : {}),
        ...(params.target ? { target: params.target } : {}),
      });
      const qs = sp.toString();
      return `/v1/dashboard/turnover-analytics${qs ? `?${qs}` : ""}`;
    },
    extraQueryKeyParams: [params.startDate, params.endDate, params.comparison, params.target],
    extraEnabled: !!params.startDate && !!params.endDate,
  });
}

export function useDashboardMetrics(filters: DashboardMetricsFilters) {
  const endpoint = `/v1/dashboard/executive-overview${dashboardMetricsQuery(filters)}`;

  return useDashboardQuery<DashboardMetricsResponse>({
    key: "executive-overview",
    getPath: () => endpoint,
    extraQueryKeyParams: [endpoint],
    parse: (payload) => dashboardMetricsResponseSchema.parse(payload),
  });
}

/**
 * Department-level overtime hours + cost drilldown for the executive area.
 * The endpoint resolves dates itself; pass the *resolved* reporting window from
 * the executive-overview response (`context.period`) so the breakdown matches
 * the KPI grid exactly. Non-custom periods like `this_month` carry no dates in
 * the raw filters — only the backend-resolved period has them.
 */
export function useOvertimeBreakdown(range: ChartRangeParams = {}, topN = 10) {
  const endpoint = `/v1/dashboard/overtime-breakdown?${new URLSearchParams({
    ...(range.startDate ? { start_date: range.startDate } : {}),
    ...(range.endDate ? { end_date: range.endDate } : {}),
    ...(range.timezone ? { timezone: range.timezone } : {}),
    top_n: String(topN),
  }).toString()}`;

  return useDashboardQuery<OvertimeBreakdownResponse>({
    key: "overtime-breakdown",
    getPath: () => endpoint,
    extraQueryKeyParams: [endpoint],
    parse: (payload) => overtimeBreakdownSchema.parse(payload),
  });
}

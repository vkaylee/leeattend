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
  type RecruitmentEffectivenessAnalytics,
  type RecruitmentSourceBreakdown,
  type TurnoverAnalytics,
  type UnexcusedAbsencesPoint,
  type OrganizationStats,
} from "./use-dashboard-types";

export type {
  AttendanceTrendPoint,
  DepartmentAttendancePoint,
  LatenessByDay,
  LeaveTypeBreakdown,
  OvertimeBreakdownResponse,
  OvertimeTrendPoint,
  PeakTimeSlot,
  RateQuantity,
  RecruitmentEffectivenessAnalytics,
  RecruitmentEffectivenessOverview,
  RecruitmentEffectivenessSource,
  RecruitmentFunnelStage,
  RecruitmentSourceBreakdown,
  RecruitmentTrendPoint,
  RecruitmentDataGap,
  TurnoverAnalytics,
  TurnoverRate,
  UnexcusedAbsencesPoint,
  OrganizationStats,
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

// ── Factory: reusable useQuery wrapper ────────────────────────────────────

export type DashboardQueryErrorCode =
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
  /** Additional enabled condition. */
  extraEnabled?: boolean;
  extraQueryKeyParams?: readonly unknown[];
  /** Lazily builds the API path (captures hook args in closure). */
  getPath: () => string;
  /** Parses untrusted API JSON into T. */
  parse?: (payload: unknown) => T;
  /** Unique cache key segment (e.g. "organization-stats"). */
  key: DashboardSchemaKey;
}

// eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters -- T determines return type through useQuery<T>
function useDashboardQuery<T>(options: DashboardQueryOptions<T>) {
  return useQuery<T>({
    queryKey: [options.key, ...(options.extraQueryKeyParams ?? [])],
    queryFn: async () => {
      const endpoint = options.getPath();

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
    enabled: options.extraEnabled ?? true,
  });
}

// ── Public hooks (one-liners) ─────────────────────────────────────────────

export const useDashboardStats = () =>
  useDashboardQuery<OrganizationStats>({
    key: "organization-stats",
    getPath: () => "/v1/dashboard/organization-stats",
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

export interface RecruitmentEffectivenessParams extends ChartRangeParams {
  /** Previous-period / same-period-last-year comparison. Defaults to `previous_period`. */
  comparison?: "none" | "previous_period" | "same_period_last_year";
  /** Comma-separated department UUIDs. Empty = no filter. */
  departmentIds?: string;
  /** Comma-separated location UUIDs. Empty = no filter. */
  locationIds?: string;
  /** Comma-separated job-role strings. Empty = no filter. */
  jobRoles?: string;
  /** Comma-separated source types (`direct`|`referral`|`agency`). Empty = no filter. */
  sourceTypes?: string;
  /** Comma-separated recruitment-source UUIDs. Empty = no filter. */
  sourceIds?: string;
}

export function useRecruitmentEffectiveness(params: RecruitmentEffectivenessParams = {}) {
  return useDashboardQuery<RecruitmentEffectivenessAnalytics>({
    key: "recruitment-effectiveness",
    getPath: () => {
      const sp = new URLSearchParams({
        ...(params.startDate ? { start: params.startDate } : {}),
        ...(params.endDate ? { end: params.endDate } : {}),
        ...(params.comparison ? { comparison: params.comparison } : {}),
        ...(params.departmentIds ? { department_ids: params.departmentIds } : {}),
        ...(params.locationIds ? { location_ids: params.locationIds } : {}),
        ...(params.jobRoles ? { job_roles: params.jobRoles } : {}),
        ...(params.sourceTypes ? { source_types: params.sourceTypes } : {}),
        ...(params.sourceIds ? { source_ids: params.sourceIds } : {}),
      });
      const qs = sp.toString();
      return `/v1/dashboard/recruitment-effectiveness${qs ? `?${qs}` : ""}`;
    },
    extraQueryKeyParams: [
      params.startDate,
      params.endDate,
      params.comparison,
      params.departmentIds,
      params.locationIds,
      params.jobRoles,
      params.sourceTypes,
      params.sourceIds,
    ],
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

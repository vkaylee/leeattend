import { useQuery } from "@tanstack/react-query";
import { endOfMonth, format, startOfMonth } from "date-fns";
import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { useDepartments } from "@/hooks/use-departments";
import { useAllEmployees } from "@/hooks/use-employees";
import {
  fetchWorkspaceAttendanceAdjustments,
  fetchWorkspaceLeaveRequests,
  getStandaloneOrganizationId,
} from "@/lib/api";
import { fetchAttendanceSummaries } from "@/lib/api/attendance-overtime";
import { fetchShiftSwaps } from "@/lib/api/shift-swaps";
import { removeDiacritics } from "@/lib/utils";

import {
  ApprovalItemStatus,
  ApprovalType,
  TeamImpactInfo,
  UnifiedApprovalItem,
  UnifiedApprovalRequester,
  UnifiedInboxFilter,
} from "../types/unified-approvals";

export interface UnifiedInboxCounts {
  all: number;
  leave: number;
  overtime: number;
  adjustment: number;
  shift_swap: number;
}

export interface UseUnifiedInboxReturn {
  items: UnifiedApprovalItem[];
  allItems: UnifiedApprovalItem[];
  pendingCounts: UnifiedInboxCounts;
  isLoading: boolean;
  filter: UnifiedInboxFilter;
  setFilter: (patch: Partial<UnifiedInboxFilter>) => void;
  selectedIds: Set<string>;
  toggleSelect: (id: string) => void;
  selectAll: () => void;
  clearSelection: () => void;
  isAllSelected: boolean;
  selectedItem: UnifiedApprovalItem | null;
  setSelectedItem: (item: UnifiedApprovalItem | null) => void;
  refetchAll: () => Promise<void>;
}

export function useUnifiedInbox(): UseUnifiedInboxReturn {
  const { t } = useTranslation();
  const wsId = getStandaloneOrganizationId();

  // ── Filters & Selection ──────────────────────────────────────────────
  const [filter, setFilterState] = useState<UnifiedInboxFilter>({
    type: "all",
    status: "pending",
    searchQuery: "",
  });

  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [selectedItem, setSelectedItem] = useState<UnifiedApprovalItem | null>(
    null
  );

  const setFilter = useCallback((patch: Partial<UnifiedInboxFilter>) => {
    setFilterState((prev) => ({ ...prev, ...patch }));
  }, []);

  // ── Organizational context ───────────────────────────────────────────
  const { employees } = useAllEmployees(wsId);
  const { data: departments } = useDepartments(wsId);

  const employeeMap = useMemo(() => {
    const map = new Map<string, UnifiedApprovalRequester>();
    for (const emp of employees ?? []) {
      const info: UnifiedApprovalRequester = {
        id: emp.id,
        name: emp.full_name,
        idNumber: emp.employee_id_number,
        departmentName: emp.department_name ?? undefined,
      };
      map.set(emp.id, info);
      if (emp.active_employment_id) {
        map.set(emp.active_employment_id, info);
      }
    }
    return map;
  }, [employees]);

  // ── Date range for monthly summaries ─────────────────────────────────
  const now = useMemo(() => new Date(), []);
  const startDate = useMemo(
    () => format(startOfMonth(now), "yyyy-MM-dd"),
    [now]
  );
  const endDate = useMemo(() => format(endOfMonth(now), "yyyy-MM-dd"), [now]);

  // ── Queries ──────────────────────────────────────────────────────────
  const leavesQuery = useQuery({
    queryKey: ["unified-inbox-leaves", wsId],
    queryFn: () =>
      wsId
        ? fetchWorkspaceLeaveRequests(wsId, { limit: 100 })
        : Promise.reject(new Error("No workspace")),
    enabled: !!wsId,
    staleTime: 30_000,
  });

  const adjustmentsQuery = useQuery({
    queryKey: ["unified-inbox-adjustments", wsId],
    queryFn: () =>
      wsId
        ? fetchWorkspaceAttendanceAdjustments(wsId, { limit: 100 })
        : Promise.reject(new Error("No workspace")),
    enabled: !!wsId,
    staleTime: 30_000,
  });

  const swapsQuery = useQuery({
    queryKey: ["unified-inbox-swaps", wsId],
    queryFn: () =>
      wsId ? fetchShiftSwaps(wsId) : Promise.reject(new Error("No workspace")),
    enabled: !!wsId,
    staleTime: 30_000,
  });

  const otQuery = useQuery({
    queryKey: ["unified-inbox-ot", wsId, startDate, endDate],
    queryFn: () =>
      wsId
        ? fetchAttendanceSummaries(wsId, startDate, endDate)
        : Promise.reject(new Error("No workspace")),
    enabled: !!wsId,
    staleTime: 30_000,
  });

  const isLoading =
    leavesQuery.isLoading ||
    adjustmentsQuery.isLoading ||
    swapsQuery.isLoading ||
    otQuery.isLoading;

  const refetchAll = useCallback(async () => {
    await Promise.all([
      leavesQuery.refetch(),
      adjustmentsQuery.refetch(),
      swapsQuery.refetch(),
      otQuery.refetch(),
    ]);
  }, [leavesQuery, adjustmentsQuery, swapsQuery, otQuery]);

  // ── Mapping to UnifiedApprovalItem[] ──────────────────────────────────
  const allItems = useMemo<UnifiedApprovalItem[]>(() => {
    const items: UnifiedApprovalItem[] = [];

    // 1. Leave Requests
    const leaves = leavesQuery.data?.data ?? [];
    for (const l of leaves) {
      const req: UnifiedApprovalRequester = employeeMap.get(l.employment_id) ?? {
        id: l.employment_id,
        name: l.employment_id,
      };

      // Calculate team impact on same date & department
      const colleaguesAway = leaves
        .filter(
          (other) =>
            other.id !== l.id &&
            other.status === "approved" &&
            other.start_date <= l.end_date &&
            other.end_date >= l.start_date &&
            req.departmentName &&
            employeeMap.get(other.employment_id)?.departmentName === req.departmentName
        )
        .map((o) => employeeMap.get(o.employment_id)?.name || o.employment_id);

      const impact: TeamImpactInfo = {
        departmentName: req.departmentName,
        colleaguesAwayCount: colleaguesAway.length,
        colleaguesAwayNames: colleaguesAway,
        isHighImpact: colleaguesAway.length >= 2,
      };

      items.push({
        id: l.id,
        type: "leave",
        status: (l.status as ApprovalItemStatus) || "pending",
        requester: req,
        submittedAt: l.created_at || new Date().toISOString(),
        targetDate: l.start_date,
        title: t("approvals.type.leave", "Đơn xin nghỉ phép"),
        subtitle: `${l.start_date} → ${l.end_date} (${l.duration_type})`,
        details: {
          dateStr:
            l.start_date === l.end_date
              ? l.start_date
              : `${l.start_date} → ${l.end_date}`,
          reason: l.reason ?? undefined,
          durationLabel: l.duration_type,
          rawPayload: l as unknown as Record<string, unknown>,
        },
        impact,
      });
    }

    // 2. Attendance Adjustments
    const adjustments = adjustmentsQuery.data?.data ?? [];
    for (const a of adjustments) {
      const req: UnifiedApprovalRequester = employeeMap.get(a.employment_id) ?? {
        id: a.employment_id,
        name: a.employment_id,
      };

      items.push({
        id: a.id,
        type: "adjustment",
        status: (a.status as ApprovalItemStatus) || "pending",
        requester: req,
        submittedAt: a.created_at || new Date().toISOString(),
        targetDate: a.requested_punch_time.split("T")[0] || a.created_at,
        title: t("approvals.type.adjustment", "Giải trình chấm công"),
        subtitle: `Thời gian yêu cầu: ${a.requested_punch_time.replace("T", " ").slice(0, 16)}`,
        details: {
          dateStr: a.requested_punch_time,
          reason: a.reason ?? undefined,
          rawPayload: a as unknown as Record<string, unknown>,
        },
      });
    }

    // 3. Shift Swaps
    const swaps = Array.isArray(swapsQuery.data) ? swapsQuery.data : [];
    for (const s of swaps) {
      const req = employeeMap.get(s.requester_id) ?? {
        id: s.requester_id,
        name: s.requester_id,
      };

      const normalizedStatus: ApprovalItemStatus =
        s.status === "pending_admin" || s.status === "pending_target"
          ? "pending"
          : (s.status as ApprovalItemStatus) || "pending";

      items.push({
        id: s.id,
        type: "shift_swap",
        status: normalizedStatus,
        requester: req,
        submittedAt: s.created_at || new Date().toISOString(),
        targetDate: s.swap_date,
        title: t("approvals.type.shift_swap", "Đổi ca làm việc"),
        subtitle: `Ngày đổi ca: ${s.swap_date}`,
        details: {
          dateStr: s.swap_date,
          reason: s.reason ?? undefined,
          rawPayload: s as unknown as Record<string, unknown>,
        },
      });
    }

    // 4. Overtime Summaries
    const otSummaries = Array.isArray(otQuery.data) ? otQuery.data : [];
    for (const ot of otSummaries) {
      if (!ot.claimed_ot_minutes && !ot.approved_ot_minutes) continue;

      const req = employeeMap.get(ot.employment_id) ?? {
        id: ot.employment_id,
        name: ot.employment_id,
      };

      const status: ApprovalItemStatus =
        ot.ot_status === "approved"
          ? "approved"
          : ot.ot_status === "rejected"
            ? "rejected"
            : "pending";

      const claimedMins = ot.claimed_ot_minutes || 60;
      items.push({
        id: ot.id || `${ot.employment_id}_${ot.date}`,
        type: "overtime",
        status,
        requester: req,
        submittedAt: ot.date,
        targetDate: ot.date,
        title: t("approvals.type.overtime", "Xác nhận tăng ca (OT)"),
        subtitle: `Thời lượng: ${claimedMins} phút (${(claimedMins / 60).toFixed(1)}h)`,
        details: {
          dateStr: ot.date,
          reason: ot.ot_reason ?? undefined,
          durationLabel: `${claimedMins} phút`,
          rawPayload: ot as unknown as Record<string, unknown>,
        },
      });
    }

    // Sort descending by targetDate or submittedAt
    return items.sort((a, b) => {
      const dateA = a.submittedAt || a.targetDate;
      const dateB = b.submittedAt || b.targetDate;
      return dateB.localeCompare(dateA);
    });
  }, [
    leavesQuery.data,
    adjustmentsQuery.data,
    swapsQuery.data,
    otQuery.data,
    employeeMap,
    t,
  ]);

  // ── Pending counts calculation ───────────────────────────────────────
  const pendingCounts = useMemo<UnifiedInboxCounts>(() => {
    let leave = 0;
    let overtime = 0;
    let adjustment = 0;
    let shift_swap = 0;

    for (const it of allItems) {
      if (it.status === "pending") {
        if (it.type === "leave") leave++;
        else if (it.type === "overtime") overtime++;
        else if (it.type === "adjustment") adjustment++;
        else if (it.type === "shift_swap") shift_swap++;
      }
    }

    return {
      all: leave + overtime + adjustment + shift_swap,
      leave,
      overtime,
      adjustment,
      shift_swap,
    };
  }, [allItems]);

  // ── Filtered items ───────────────────────────────────────────────────
  const items = useMemo(() => {
    const cleanSearch = removeDiacritics(
      (filter.searchQuery || "").trim().toLowerCase()
    );

    return allItems.filter((item) => {
      // Filter by type
      if (filter.type !== "all" && item.type !== filter.type) return false;

      // Filter by status
      if (filter.status !== "all" && item.status !== filter.status) return false;

      // Filter by department
      if (
        filter.departmentId &&
        item.requester.departmentId !== filter.departmentId
      ) {
        return false;
      }

      // Filter by search query
      if (cleanSearch) {
        const nameMatch = removeDiacritics(
          item.requester.name.toLowerCase()
        ).includes(cleanSearch);
        const idMatch = removeDiacritics(
          (item.requester.idNumber || "").toLowerCase()
        ).includes(cleanSearch);
        const titleMatch = removeDiacritics(item.title.toLowerCase()).includes(
          cleanSearch
        );
        const subtitleMatch = removeDiacritics(
          item.subtitle.toLowerCase()
        ).includes(cleanSearch);
        const reasonMatch = removeDiacritics(
          (item.details.reason || "").toLowerCase()
        ).includes(cleanSearch);

        return (
          nameMatch || idMatch || titleMatch || subtitleMatch || reasonMatch
        );
      }

      return true;
    });
  }, [allItems, filter]);

  // ── Selection helpers ────────────────────────────────────────────────
  const toggleSelect = useCallback((id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const selectAll = useCallback(() => {
    const pendingInView = items
      .filter((i) => i.status === "pending")
      .map((i) => i.id);
    setSelectedIds(new Set(pendingInView));
  }, [items]);

  const clearSelection = useCallback(() => {
    setSelectedIds(new Set());
  }, []);

  const isAllSelected = useMemo(() => {
    const pendingInView = items.filter((i) => i.status === "pending");
    return (
      pendingInView.length > 0 &&
      pendingInView.every((i) => selectedIds.has(i.id))
    );
  }, [items, selectedIds]);

  return {
    items,
    allItems,
    pendingCounts,
    isLoading,
    filter,
    setFilter,
    selectedIds,
    toggleSelect,
    selectAll,
    clearSelection,
    isAllSelected,
    selectedItem,
    setSelectedItem,
    refetchAll,
  };
}

import { useQuery } from "@tanstack/react-query";
import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { endOfMonth, format, startOfMonth } from "date-fns";

import { useDepartments } from "@/hooks/use-departments";
import { useAllEmployees } from "@/hooks/use-employees";
import { useMyPermissions } from "@/hooks/use-permissions";
import {
  fetchAttendanceAdjustments,
  fetchLeaveRequests,
} from "@/lib/api";
import { fetchAttendanceSummaries } from "@/lib/api/attendance-overtime";
import { fetchShiftSwaps } from "@/lib/api/shift-swaps";
import { createQueryKey } from "@/lib/query-keys";
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
  adjustment: number;
  all: number;
  leave: number;
  overtime: number;
  shift_swap: number;
}

export interface UseUnifiedInboxReturn {
  allItems: UnifiedApprovalItem[];
  clearSelection: () => void;
  filter: UnifiedInboxFilter;
  isAllSelected: boolean;
  isLoading: boolean;
  items: UnifiedApprovalItem[];
  pendingCounts: UnifiedInboxCounts;
  refetchAll: () => Promise<void>;
  selectAll: () => void;
  selectedIds: Set<string>;
  selectedItem: UnifiedApprovalItem | null;
  setFilter: (patch: Partial<UnifiedInboxFilter>) => void;
  setSelectedItem: (item: UnifiedApprovalItem | null) => void;
  toggleSelect: (id: string) => void;
}

export function useUnifiedInbox(): UseUnifiedInboxReturn {
  const { t } = useTranslation();
  const { hasAnyPermission } = useMyPermissions();

  const canReadLeaves = hasAnyPermission(
    "leave:requests:read",
    "leave:requests:approve",
    "leave:requests:submit_on_behalf",
  );
  const canApproveAdjustments = hasAnyPermission(
    "attendance:adjustments:approve",
  );
  const canManageSwaps = hasAnyPermission("shift_swaps:manage");
  const canApproveOvertime = hasAnyPermission("overtime:approve");

  // ── Filters & Selection ──────────────────────────────────────────────
  const [filter, setFilterState] = useState<UnifiedInboxFilter>(() => {
    let initialType: ApprovalType | "all" = "all";
    if (typeof window !== "undefined") {
      const params = new URLSearchParams(window.location.search);
      const urlType = params.get("type");
      if (urlType === "overtime" || urlType === "leave" || urlType === "adjustment" || urlType === "shift_swap") {
        initialType = urlType;
      }
    }
    return {
      type: initialType,
      status: "pending",
      searchQuery: "",
    };
  });

  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [selectedItem, setSelectedItem] = useState<UnifiedApprovalItem | null>(
    null
  );

  const setFilter = useCallback((patch: Partial<UnifiedInboxFilter>) => {
    setFilterState((prev) => ({ ...prev, ...patch }));
  }, []);

  // ── Organizational context ───────────────────────────────────────────
  const { employees } = useAllEmployees();
  const { data: departments } = useDepartments();

  const employeeMap = useMemo(() => {
    const map = new Map<string, UnifiedApprovalRequester>();
    for (const emp of employees) {
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
    queryKey: createQueryKey("unified-inbox-leaves"),
    queryFn: () => fetchLeaveRequests({ limit: 100 }),
    enabled: canReadLeaves,
    staleTime: 30_000,
  });

  const adjustmentsQuery = useQuery({
    queryKey: createQueryKey("unified-inbox-adjustments"),
    queryFn: () => fetchAttendanceAdjustments({ limit: 100 }),
    enabled: canApproveAdjustments,
    staleTime: 30_000,
  });

  const swapsQuery = useQuery({
    queryKey: createQueryKey("unified-inbox-swaps"),
    queryFn: () => fetchShiftSwaps(),
    enabled: canManageSwaps,
    staleTime: 30_000,
  });

  const otQuery = useQuery({
    queryKey: createQueryKey("unified-inbox-ot", startDate, endDate),
    queryFn: () => fetchAttendanceSummaries(startDate, endDate),
    enabled: canApproveOvertime,
    staleTime: 30_000,
  });

  const isLoading =
    (canReadLeaves && leavesQuery.isLoading) ||
    (canApproveAdjustments && adjustmentsQuery.isLoading) ||
    (canManageSwaps && swapsQuery.isLoading) ||
    (canApproveOvertime && otQuery.isLoading);

  const refetchAll = useCallback(async () => {
    await Promise.all([
      ...(canReadLeaves ? [leavesQuery.refetch()] : []),
      ...(canApproveAdjustments ? [adjustmentsQuery.refetch()] : []),
      ...(canManageSwaps ? [swapsQuery.refetch()] : []),
      ...(canApproveOvertime ? [otQuery.refetch()] : []),
    ]);
  }, [
    canApproveAdjustments,
    canApproveOvertime,
    canManageSwaps,
    canReadLeaves,
    leavesQuery,
    adjustmentsQuery,
    swapsQuery,
    otQuery,
  ]);

  // ── Helper: Compute Team Impact across any approval item (LSP & DRY) ──
  const approvedLeaves = useMemo(() => {
    const rawLeaves = leavesQuery.data?.data ?? [];
    return rawLeaves.filter((l) => l.status === "approved");
  }, [leavesQuery.data?.data]);

  const getTeamImpact = useCallback(
    (targetDate: string, deptName?: string, itemId?: string): TeamImpactInfo | undefined => {
      if (!deptName || !targetDate) return undefined;
      const dateStr = targetDate.split("T")[0];

      const colleaguesAway = approvedLeaves
        .filter(
          (l) =>
            l.id !== itemId &&
            l.start_date <= dateStr &&
            l.end_date >= dateStr &&
            employeeMap.get(l.employment_id)?.departmentName === deptName
        )
        .map((l) => employeeMap.get(l.employment_id)?.name || l.employment_id);

      return {
        departmentName: deptName,
        colleaguesAwayCount: colleaguesAway.length,
        colleaguesAwayNames: colleaguesAway,
        isHighImpact: colleaguesAway.length >= 2,
      };
    },
    [approvedLeaves, employeeMap]
  );

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

      const impact = getTeamImpact(l.start_date, req.departmentName, l.id);
      items.push({
        id: l.id,
        type: "leave",
        status: l.status as ApprovalItemStatus,
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
          rawPayload: l,
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

      const targetDate = a.requested_punch_time.split("T")[0] || a.created_at;
      const impact = getTeamImpact(targetDate, req.departmentName, a.id);

      items.push({
        id: a.id,
        type: "adjustment",
        status: a.status as ApprovalItemStatus,
        requester: req,
        submittedAt: a.created_at || new Date().toISOString(),
        targetDate,
        title: t("approvals.type.adjustment", "Giải trình chấm công"),
        subtitle: `Thời gian yêu cầu: ${a.requested_punch_time.replace("T", " ").slice(0, 16)}`,
        details: {
          dateStr: a.requested_punch_time,
          reason: a.reason ?? undefined,
          rawPayload: a as unknown as Record<string, unknown>,
        },
        impact,
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
          : s.status;

      const impact = getTeamImpact(s.swap_date, req.departmentName, s.id);

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
        impact,
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
      const impact = getTeamImpact(ot.date, req.departmentName, ot.id);

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
        impact,
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
    getTeamImpact,
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
        else shift_swap++;
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

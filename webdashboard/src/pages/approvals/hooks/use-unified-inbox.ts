import { useQuery } from "@tanstack/react-query";
import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { endOfMonth, format, startOfMonth } from "date-fns";

import { useAllEmployees } from "@/hooks/use-employees";
import { useMyPermissions } from "@/hooks/use-permissions";
import { fetchAttendanceAdjustments, fetchLeaveRequests } from "@/lib/api";
import { fetchAttendanceSummaries } from "@/lib/api/attendance-overtime";
import { fetchShiftSwaps } from "@/lib/api/shift-swaps";
import { createQueryKey } from "@/lib/query-keys";

import type {
  UnifiedApprovalItem,
  UnifiedInboxFilter,
} from "../types/unified-approvals";

import {
  countPendingItems,
  filterApprovalItems,
  isApprovalType,
  type UnifiedInboxCounts,
} from "./unified-inbox-filters";
import { buildApprovalItems } from "./unified-inbox-item-builders";
import {
  createEmployeeMap,
  createTeamImpactResolver,
} from "./unified-inbox-items";

export type { UnifiedInboxCounts } from "./unified-inbox-filters";

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

function getInitialFilter(): UnifiedInboxFilter {
  const urlType = typeof window === "undefined"
    ? null
    : new URLSearchParams(window.location.search).get("type");

  return {
    type: isApprovalType(urlType) ? urlType : "all",
    status: "pending",
    searchQuery: "",
  };
}

interface ApprovalPermissions {
  canApproveAdjustments: boolean;
  canApproveOvertime: boolean;
  canManageSwaps: boolean;
  canReadLeaves: boolean;
}

function isAnyApprovalQueryLoading(
  permissions: ApprovalPermissions,
  loadingStates: readonly (readonly [boolean, boolean])[],
): boolean {
  return loadingStates.some(([permissionsEnabled, isLoading]) => permissionsEnabled && isLoading);
}

async function refetchEnabledQueries(
  entries: readonly (readonly [boolean, () => Promise<unknown>])[],
): Promise<void> {
  await Promise.all(
    entries.filter(([isEnabled]) => isEnabled).map(([, refetch]) => refetch()),
  );
}

function useApprovalQueries(
  startDate: string,
  endDate: string,
  permissions: ApprovalPermissions,
) {
  const leavesQuery = useQuery({
    queryKey: createQueryKey("unified-inbox-leaves"),
    queryFn: () => fetchLeaveRequests({ limit: 100 }),
    enabled: permissions.canReadLeaves,
    staleTime: 30_000,
  });
  const adjustmentsQuery = useQuery({
    queryKey: createQueryKey("unified-inbox-adjustments"),
    queryFn: () => fetchAttendanceAdjustments({ limit: 100 }),
    enabled: permissions.canApproveAdjustments,
    staleTime: 30_000,
  });
  const swapsQuery = useQuery({
    queryKey: createQueryKey("unified-inbox-swaps"),
    queryFn: () => {
      return fetchShiftSwaps();
    },
    enabled: permissions.canManageSwaps,
    staleTime: 30_000,
  });
  const overtimeQuery = useQuery({
    queryKey: createQueryKey("unified-inbox-ot", startDate, endDate),
    queryFn: () => fetchAttendanceSummaries(startDate, endDate),
    enabled: permissions.canApproveOvertime,
    staleTime: 30_000,
  });

  return { adjustmentsQuery, leavesQuery, overtimeQuery, swapsQuery };
}

function useSelection(items: UnifiedApprovalItem[]) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  const toggleSelect = useCallback((id: string) => {
    setSelectedIds((previous) => {
      const next = new Set(previous);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);
  const selectAll = useCallback(() => {
    setSelectedIds(
      new Set(items.filter((item) => item.status === "pending").map((item) => item.id)),
    );
  }, [items]);
  const clearSelection = useCallback(() => { setSelectedIds(new Set()); }, []);
  const isAllSelected = useMemo(() => {
    const pendingItems = items.filter((item) => item.status === "pending");
    return pendingItems.length > 0 && pendingItems.every((item) => selectedIds.has(item.id));
  }, [items, selectedIds]);

  return { clearSelection, isAllSelected, selectAll, selectedIds, toggleSelect };
}

export function useUnifiedInbox(): UseUnifiedInboxReturn {
  const { t } = useTranslation();
  const { hasAnyPermission } = useMyPermissions();
  const permissions = useMemo<ApprovalPermissions>(() => ({
    canReadLeaves: hasAnyPermission(
      "leave:requests:read",
      "leave:requests:approve",
      "leave:requests:submit_on_behalf",
    ),
    canApproveAdjustments: hasAnyPermission("attendance:adjustments:approve"),
    canManageSwaps: hasAnyPermission("shift_swaps:manage"),
    canApproveOvertime: hasAnyPermission("overtime:approve"),
  }), [hasAnyPermission]);
  const [filter, setFilterState] = useState<UnifiedInboxFilter>(getInitialFilter);
  const [selectedItem, setSelectedItem] = useState<UnifiedApprovalItem | null>(null);
  const setFilter = useCallback((patch: Partial<UnifiedInboxFilter>) => {
    setFilterState((previous) => ({ ...previous, ...patch }));
  }, []);

  const { employees } = useAllEmployees();
  const employeeMap = useMemo(() => createEmployeeMap(employees), [employees]);
  const now = useMemo(() => new Date(), []);
  const startDate = format(startOfMonth(now), "yyyy-MM-dd");
  const endDate = format(endOfMonth(now), "yyyy-MM-dd");
  const { adjustmentsQuery, leavesQuery, overtimeQuery, swapsQuery } = useApprovalQueries(
    startDate,
    endDate,
    permissions,
  );
  const isLoading = isAnyApprovalQueryLoading(permissions, [
    [permissions.canReadLeaves, leavesQuery.isLoading],
    [permissions.canApproveAdjustments, adjustmentsQuery.isLoading],
    [permissions.canManageSwaps, swapsQuery.isLoading],
    [permissions.canApproveOvertime, overtimeQuery.isLoading],
  ]);
  const refetchAll = useCallback(
    () => refetchEnabledQueries([
      [permissions.canReadLeaves, () => leavesQuery.refetch()],
      [permissions.canApproveAdjustments, () => adjustmentsQuery.refetch()],
      [permissions.canManageSwaps, () => swapsQuery.refetch()],
      [permissions.canApproveOvertime, () => overtimeQuery.refetch()],
    ]),
    [adjustmentsQuery, leavesQuery, overtimeQuery, permissions, swapsQuery],
  );

  const approvedLeaves = useMemo(
    () => (leavesQuery.data?.data ?? []).filter((leave) => leave.status === "approved"),
    [leavesQuery.data?.data],
  );
  const getTeamImpact = useMemo(
    () => createTeamImpactResolver(approvedLeaves, employeeMap),
    [approvedLeaves, employeeMap],
  );
  const allItems = useMemo(
    () => buildApprovalItems({
      adjustments: adjustmentsQuery.data?.data ?? [],
      employeeMap,
      getTeamImpact,
      leaves: leavesQuery.data?.data ?? [],
      overtime: Array.isArray(overtimeQuery.data) ? overtimeQuery.data : [],
      swaps: Array.isArray(swapsQuery.data) ? swapsQuery.data : [],
      translate: t,
    }),
    [
      adjustmentsQuery.data?.data,
      employeeMap,
      getTeamImpact,
      leavesQuery.data?.data,
      overtimeQuery.data,
      swapsQuery.data,
      t,
    ],
  );
  const pendingCounts = useMemo(() => countPendingItems(allItems), [allItems]);
  const items = useMemo(() => filterApprovalItems(allItems, filter), [allItems, filter]);
  const selection = useSelection(items);

  return {
    allItems,
    clearSelection: selection.clearSelection,
    filter,
    isAllSelected: selection.isAllSelected,
    isLoading,
    items,
    pendingCounts,
    refetchAll,
    selectAll: selection.selectAll,
    selectedIds: selection.selectedIds,
    selectedItem,
    setFilter,
    setSelectedItem,
    toggleSelect: selection.toggleSelect,
  };
}

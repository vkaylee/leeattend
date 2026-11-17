import { removeDiacritics } from "@/lib/utils";

import type {
  ApprovalType,
  UnifiedApprovalItem,
  UnifiedInboxFilter,
} from "../types/unified-approvals";

export interface UnifiedInboxCounts {
  adjustment: number;
  all: number;
  leave: number;
  overtime: number;
  shift_swap: number;
}

export function countPendingItems(items: UnifiedApprovalItem[]): UnifiedInboxCounts {
  const counts = {
    adjustment: 0,
    leave: 0,
    overtime: 0,
    shift_swap: 0,
  } satisfies Omit<UnifiedInboxCounts, "all">;

  for (const item of items) {
    if (item.status === "pending") counts[item.type]++;
  }

  return {
    ...counts,
    all: Object.values(counts).reduce((total, count) => total + count, 0),
  };
}

function matchesSearch(item: UnifiedApprovalItem, searchQuery: string): boolean {
  if (!searchQuery) return true;

  const fields = [
    item.requester.name,
    item.requester.idNumber ?? "",
    item.title,
    item.subtitle,
    item.details.reason ?? "",
  ];

  return fields.some((field) =>
    removeDiacritics(field.toLowerCase()).includes(searchQuery),
  );
}

export function filterApprovalItems(
  allItems: UnifiedApprovalItem[],
  filter: UnifiedInboxFilter,
): UnifiedApprovalItem[] {
  const searchQuery = removeDiacritics((filter.searchQuery ?? "").trim().toLowerCase());

  return allItems.filter((item) => {
    if (filter.type !== "all" && item.type !== filter.type) return false;
    if (filter.status !== "all" && item.status !== filter.status) return false;
    if (filter.departmentId && item.requester.departmentId !== filter.departmentId) return false;
    return matchesSearch(item, searchQuery);
  });
}

export function isApprovalType(value: string | null): value is ApprovalType {
  return value === "overtime" ||
    value === "leave" ||
    value === "adjustment" ||
    value === "shift_swap";
}

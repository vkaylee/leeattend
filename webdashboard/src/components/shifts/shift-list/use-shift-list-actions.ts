import { useCallback } from "react";

import type { Shift } from "@/lib/api";

import type { UseShiftListActionsOptions, UseShiftListActionsReturn } from "./types";

/**
 * useShiftListActions — wraps CRUD action callbacks in memoized handlers.
 * Provides stable function references for edit, delete, and view-history
 * actions forwarded to the shift list.
 */
export function useShiftListActions({
  onEdit,
  onDelete,
  onViewHistory,
}: UseShiftListActionsOptions): UseShiftListActionsReturn {
  const handleEdit = useCallback(
    (shift: Shift) => { onEdit(shift); },
    [onEdit],
  );

  const handleDelete = useCallback(
    (shift: Shift) => { onDelete?.(shift); },
    [onDelete],
  );

  const handleViewHistory = useCallback(
    (shift: Shift) => { onViewHistory?.(shift); },
    [onViewHistory],
  );

  return { handleEdit, handleDelete, handleViewHistory };
}

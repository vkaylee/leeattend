import type { Shift } from "@/lib/api";

// ── Filter ─────────────────────────────────────────────────────────────

export type ListFilter = "all" | "active" | "inactive" | "night";

export interface ChipDef {
  key: ListFilter;
  labelKey: string;
}

// ── ShiftListTable ─────────────────────────────────────────────────────

export interface ShiftListTableProps {
  shifts: Shift[];
  onEdit: (shift: Shift) => void;
  onDelete?: (shift: Shift) => void;
  onViewHistory?: (shift: Shift) => void;
  isFiltered: boolean;
  formatTime: (time: string) => string;
}

// ── useShiftListActions ────────────────────────────────────────────────

export interface UseShiftListActionsOptions {
  onEdit: (shift: Shift) => void;
  onDelete?: (shift: Shift) => void;
  onViewHistory?: (shift: Shift) => void;
}

export interface UseShiftListActionsReturn {
  handleEdit: (shift: Shift) => void;
  handleDelete: (shift: Shift) => void;
  handleViewHistory: (shift: Shift) => void;
}

// Re-export ListFilter and ChipDef for consumers
export type { Shift };

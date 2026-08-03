import type { Shift } from "@/lib/api";

// ── Filter ─────────────────────────────────────────────────────────────

export type ListFilter = "all" | "active" | "inactive" | "night";

export interface ChipDef {
  key: ListFilter;
  labelKey: string;
}

// ── ShiftListTable ─────────────────────────────────────────────────────

export interface ShiftListTableProps {
  formatTime: (time: string) => string;
  isFiltered: boolean;
  onDelete?: (shift: Shift) => void;
  onEdit: (shift: Shift) => void;
  onViewHistory?: (shift: Shift) => void;
  shifts: Shift[];
}

// ── useShiftListActions ────────────────────────────────────────────────

export interface UseShiftListActionsOptions {
  onDelete?: (shift: Shift) => void;
  onEdit: (shift: Shift) => void;
  onViewHistory?: (shift: Shift) => void;
}

export interface UseShiftListActionsReturn {
  handleDelete: (shift: Shift) => void;
  handleEdit: (shift: Shift) => void;
  handleViewHistory: (shift: Shift) => void;
}

// Re-export ListFilter and ChipDef for consumers
export type { Shift };

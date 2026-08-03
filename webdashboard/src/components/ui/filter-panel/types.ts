// ── Types ────────────────────────────────────────────────────────────
import React from "react";

export interface FilterOption {
  disabled?: boolean;
  icon?: React.ReactNode;
  label: string;
  value: string;
}

export type FilterMode = "multi" | "single";
export type FilterContainer = "popover" | "dropdown";

export interface FilterPanelProps {
  // ── Layout ──
  /** Popover alignment */
  align?: "start" | "center" | "end";
  // ── Custom content ──
  /** Render custom content inside the panel instead of the default option list */
  children?: React.ReactNode;
  /** Additional CSS for the panel content */
  className?: string;
  /** Container type */
  container: FilterContainer;
  // ── Test ──
  "data-testid"?: string;
  /** Message when options list is empty */
  emptyMessage?: string;

  // ── Display ──
  /** Header label shown at top of panel */
  headerLabel?: string;

  // ── Keyboard ──
  /** Keyboard shortcut to open the panel (e.g. "Ctrl+Shift+F"). Opt-in — no default. */
  keyboardShortcut?: string;
  /** Selection mode */
  mode: FilterMode;
  /** Called when Apply is clicked (receives the final selected values) */
  onApply?: (selected: string[]) => void;

  /** Called to clear all selections */
  onClear: () => void;
  /** Called when a value is toggled (multi) or selected (single) */
  onToggle: (value: string) => void;

  /** Options to display */
  options: FilterOption[];
  // ── Search ──
  /** Enable cmdk-powered search */
  searchable?: boolean;

  /** Placeholder for search input */
  searchPlaceholder?: string;

  /** Currently selected values */
  selected: string[];
  // ── Confirmation ──
  /** Use Apply/Cancel buttons instead of live toggle. Draft state is internal. */
  showApplyButton?: boolean;

  /** Show "Clear filter" button in panel */
  showClearButton?: boolean;

  // ── Footer ──
  /** Show "Selected X/Y" count in footer */
  showSelectionCount?: boolean;

  // ── Trigger ──
  /** Trigger element (e.g., <FilterTrigger />). FilterPanel wraps it in the appropriate Radix trigger. */
  trigger: React.ReactNode;
}

export interface OptionListProps {
  emptyMessage?: string;
  mode: FilterMode;
  onClear: () => void;
  onToggle: (value: string) => void;
  options: FilterOption[];
  searchable?: boolean;
  searchPlaceholder?: string;
  selected: string[];
  showClearButton?: boolean;
  testId?: string;
}

export interface FilterActionsProps {
  onApply?: () => void;
  onCancel: () => void;
  onClear: () => void;
  selectedCount: number;
  showApplyButton?: boolean;
  showSelectionCount?: boolean;
  totalCount: number;
}

export interface FilterGroupProps {
  className?: string;
  children?: React.ReactNode;
  headerLabel?: string;
  // Option list props (used when no children)
  emptyMessage?: string;
  mode: FilterMode;
  onClear: () => void;
  onToggle: (value: string) => void;
  options: FilterOption[];
  searchable?: boolean;
  searchPlaceholder?: string;
  selected: string[];
  showClearButton?: boolean;
  testId?: string;
  // Actions props
  onApply?: () => void;
  onCancel: () => void;
  showApplyButton?: boolean;
  showSelectionCount?: boolean;
}

export interface UseFilterPanelStateOptions {
  selected: string[];
  onToggle: (value: string) => void;
  onClear: () => void;
  onApply?: (selected: string[]) => void;
  mode: FilterMode;
  showApplyButton: boolean;
  keyboardShortcut?: string;
}

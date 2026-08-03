import * as Popover from "@radix-ui/react-popover";

import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

import type { FilterPanelProps } from "./filter-panel/types";

import { FilterDropdownContent } from "./filter-panel/filter-dropdown-content";
import { FilterGroup } from "./filter-panel/filter-group";
import { useFilterPanelState } from "./filter-panel/use-filter-panel-state";

export type {
  FilterOption,
  FilterMode,
  FilterContainer,
  FilterPanelProps,
} from "./filter-panel/types";

/**
 * FilterPanel — reusable filter dropdown/popover with consistent UX.
 *
 * Supports:
 * - `container="popover"`: Radix Popover with rounded-[2rem] animation
 * - `container="dropdown"`: DropdownMenu with backdrop-blur
 * - `mode="multi"`: Checkbox toggles, panel stays open
 * - `mode="single"`: Radio-style, panel closes on select
 * - `searchable`: cmdk-powered search input
 * - `showApplyButton`: Draft state, Apply/Cancel buttons
 *
 * Sub-components exported from `@/components/ui/filter-panel/index`.
 */
export function FilterPanel({
  options,
  selected,
  onToggle,
  onClear,
  mode,
  container,
  trigger,
  headerLabel,
  emptyMessage,
  showClearButton = true,
  searchable = false,
  searchPlaceholder,
  showApplyButton = false,
  onApply,
  showSelectionCount = false,
  align = "start",
  className,
  children,
  keyboardShortcut,
  "data-testid": testId,
}: FilterPanelProps) {
  const {
    open,
    setOpen,
    effectiveSelected,
    handleToggle,
    handleDropdownToggle,
    handleApply,
    handleCancel,
    handleClear,
  } = useFilterPanelState({
    selected,
    onToggle,
    onClear,
    onApply,
    mode,
    showApplyButton,
    keyboardShortcut,
  });

  // ── Dropdown container ───────────────────────────────────────────────
  if (container === "dropdown") {
    return (
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          {trigger}
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align={align}
          className="max-h-80 w-64 overflow-y-auto custom-scrollbar"
          data-testid={testId ? `${testId}-content` : undefined}
          sideOffset={8}
        >
          <FilterDropdownContent
            align={align}
            effectiveSelected={effectiveSelected}
            mode={mode}
            onApply={handleApply}
            onCancel={handleCancel}
            onClear={onClear}
            onDropdownToggle={handleDropdownToggle}
            options={options}
            showApplyButton={showApplyButton}
            testId={testId}
          />
        </DropdownMenuContent>
      </DropdownMenu>
    );
  }

  // ── Popover container ────────────────────────────────────────────────
  return (
    <Popover.Root onOpenChange={setOpen} open={open}>
      <Popover.Trigger asChild>
        {trigger}
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          align={align}
          className={cn(
            "z-[10000] overflow-hidden rounded-[2rem] border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 p-2.5 shadow-2xl dark:shadow-black/30 animate-in fade-in zoom-in-95 duration-200",
            searchable ? "w-80" : "w-56",
          )}
          data-testid={testId ? `${testId}-content` : undefined}
          sideOffset={8}
        >
          <FilterGroup
            className={className}
            emptyMessage={emptyMessage}
            headerLabel={headerLabel}
            mode={mode}
            onApply={handleApply}
            onCancel={handleCancel}
            onClear={handleClear}
            onToggle={handleToggle}
            options={options}
            searchable={searchable}
            searchPlaceholder={searchPlaceholder}
            selected={effectiveSelected}
            showApplyButton={showApplyButton}
            showClearButton={!!showClearButton && !showApplyButton}
            showSelectionCount={showSelectionCount}
            testId={testId}
          >
            {children}
          </FilterGroup>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

export default FilterPanel;

import * as Popover from "@radix-ui/react-popover";
import { Search, Check, X } from "lucide-react";
import React, { useState, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Command } from "cmdk";

import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";
import { useDraftSelection } from "@/components/ui/hooks/use-draft-selection";
import { useKeyboardShortcut } from "@/hooks/use-keyboard-shortcut";
import { cn } from "@/lib/utils";

// ── Types ────────────────────────────────────────────────────────────

export interface FilterOption {
  value: string;
  label: string;
  icon?: React.ReactNode;
  disabled?: boolean;
}

export type FilterMode = "multi" | "single";
export type FilterContainer = "popover" | "dropdown";

export interface FilterPanelProps {
  /** Options to display */
  options: FilterOption[];
  /** Currently selected values */
  selected: string[];
  /** Called when a value is toggled (multi) or selected (single) */
  onToggle: (value: string) => void;
  /** Called to clear all selections */
  onClear: () => void;
  /** Selection mode */
  mode: FilterMode;
  /** Container type */
  container: FilterContainer;

  // ── Trigger ──
  /** Trigger element (e.g., <FilterTrigger />). FilterPanel wraps it in the appropriate Radix trigger. */
  trigger: React.ReactNode;

  // ── Display ──
  /** Header label shown at top of panel */
  headerLabel?: string;
  /** Message when options list is empty */
  emptyMessage?: string;
  /** Show "Clear filter" button in panel */
  showClearButton?: boolean;

  // ── Search ──
  /** Enable cmdk-powered search */
  searchable?: boolean;
  /** Placeholder for search input */
  searchPlaceholder?: string;

  // ── Confirmation ──
  /** Use Apply/Cancel buttons instead of live toggle. Draft state is internal. */
  showApplyButton?: boolean;
  /** Called when Apply is clicked (receives the final selected values) */
  onApply?: (selected: string[]) => void;

  // ── Footer ──
  /** Show "Selected X/Y" count in footer */
  showSelectionCount?: boolean;

  // ── Layout ──
  /** Popover alignment */
  align?: "start" | "center" | "end";
  /** Additional CSS for the panel content */
  className?: string;

  // ── Custom content ──
  /** Render custom content inside the panel instead of the default option list */
  children?: React.ReactNode;

  // ── Keyboard ──
  /** Keyboard shortcut to open the panel (e.g. "Ctrl+Shift+F"). Opt-in — no default. */
  keyboardShortcut?: string;

  // ── Test ──
  "data-testid"?: string;
}

// ── Option Row ───────────────────────────────────────────────────────

function MultiOptionRow({
  option,
  checked,
  onToggle,
  stopPropagation,
  testId,
}: {
  checked: boolean;
  onToggle: () => void;
  option: FilterOption;
  stopPropagation?: boolean;
  testId?: string;
}) {
  return (
    <Button
      aria-selected={checked}
      className={cn(
        "h-auto w-full px-3 py-2.5 flex items-center gap-3 rounded-xl text-[11px] font-bold uppercase tracking-wide transition-colors border-none shadow-none justify-start",
        checked
          ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 hover:bg-primary-100 dark:hover:bg-primary-900"
          : "text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800",
      )}
      data-testid={testId ? `${testId}-option-${option.value}` : undefined}
      disabled={option.disabled}
      key={option.value}
      onClick={(e) => { e.preventDefault(); if (stopPropagation) e.stopPropagation(); onToggle(); }}
      role="option"
      type="button"
      variant="ghost"
    >
      <span
        className={cn(
          "w-4 h-4 rounded border flex items-center justify-center shrink-0 transition-colors",
          checked
            ? "bg-primary-600 border-primary-600 text-white"
            : "border-slate-300 dark:border-slate-600",
        )}
      >
        {checked ? <Check className="size-3" strokeWidth={3} /> : null}
      </span>
      {option.icon ? (
        <span className="opacity-50 shrink-0">{option.icon}</span>
      ) : null}
      <span className="truncate">{option.label}</span>
    </Button>
  );
}

function SingleOptionRow({
  option,
  checked,
  onSelect,
  stopPropagation,
  testId,
}: {
  checked: boolean;
  onSelect: () => void;
  option: FilterOption;
  stopPropagation?: boolean;
  testId?: string;
}) {
  return (
    <Button
      className={cn(
        "h-auto w-full px-3 py-2.5 flex items-center gap-3 rounded-xl text-[11px] font-bold uppercase tracking-wide transition-colors border-none shadow-none justify-start",
        checked
          ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 hover:bg-primary-100 dark:hover:bg-primary-900"
          : "text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800",
      )}
      data-testid={testId ? `${testId}-option-${option.value}` : undefined}
      disabled={option.disabled}
      key={option.value}
      onClick={(e) => { e.preventDefault(); if (stopPropagation) e.stopPropagation(); onSelect(); }}
      role="option"
      type="button"
      variant="ghost"
    >
      <span
        className={cn(
          "w-4 h-4 rounded-full border flex items-center justify-center shrink-0 transition-colors",
          checked
            ? "bg-primary-600 border-primary-600 text-white"
            : "border-slate-300 dark:border-slate-600",
        )}
      >
        {checked ? <Check size={10} strokeWidth={3} /> : null}
      </span>
      {option.icon ? (
        <span className="opacity-50 shrink-0">{option.icon}</span>
      ) : null}
      <span className="truncate">{option.label}</span>
    </Button>
  );
}

// ── Option List ──────────────────────────────────────────────────────

interface OptionListProps {
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

function OptionList({
  options,
  selected,
  mode,
  searchable,
  searchPlaceholder,
  emptyMessage,
  showClearButton,
  onToggle,
  onClear,
  testId,
}: OptionListProps) {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");

  const filteredOptions = useMemo(() => {
    if (!searchable || !search.trim()) return options;
    const q = search.toLowerCase();
    return options.filter((opt) => opt.label.toLowerCase().includes(q));
  }, [options, search, searchable]);

  const handleToggle = useCallback(
    (value: string) => {
      onToggle(value);
      if (mode === "single") {
        // For single mode, we let the parent handle close via onToggle
      }
    },
    [onToggle, mode],
  );

  return (
    <>
      {searchable ? (
        <Command>
          <div className="flex items-center border-b border-slate-50 dark:border-slate-800 px-3 pb-3 mb-1">
            <Search className="mr-3 h-4 w-4 shrink-0 text-slate-300 dark:text-slate-500" />
            <Command.Input
              autoFocus
              className="flex h-10 w-full rounded-md bg-transparent text-sm text-primary-700 dark:text-primary-300 outline-none placeholder:text-slate-300 dark:placeholder:text-slate-600 font-bold"
              onValueChange={setSearch}
              placeholder={searchPlaceholder ?? t("common.combobox.search")}
              value={search}
            />
          </div>
        </Command>
      ) : null}

      {showClearButton && selected.length > 0 ? (
        <Button
          className="h-auto w-full px-3 py-2 flex items-center gap-3 rounded-xl text-[11px] font-bold uppercase tracking-wide text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors mb-1 border-none shadow-none justify-start"
          data-testid={testId ? `${testId}-clear` : undefined}
          onClick={onClear}
          type="button"
          variant="ghost"
        >
          <X className="shrink-0" size={12} />
          {t("common.clear_filter", "Xóa bộ lọc")}
        </Button>
      ) : null}

      <div className="max-h-60 overflow-y-auto custom-scrollbar">
        {filteredOptions.length === 0 ? (
          <div className="px-3 py-6 text-center text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest opacity-60">
            {emptyMessage ?? t("common.no_options", "Không có tùy chọn")}
          </div>
        ) : (
          filteredOptions.map((opt) => {
            const checked = selected.includes(opt.value);
            if (mode === "multi") {
              return (
                <MultiOptionRow
                  checked={checked}
                  key={opt.value}
                  onToggle={() => { handleToggle(opt.value); }}
                  option={opt}
                  testId={testId}
                />
              );
            }
            return (
              <SingleOptionRow
                checked={checked}
                key={opt.value}
                onSelect={() => { handleToggle(opt.value); }}
                option={opt}
                testId={testId}
              />
            );
          })
        )}
      </div>
    </>
  );
}

// ── Footer ───────────────────────────────────────────────────────────

interface FilterFooterProps {
  onApply?: () => void;
  onCancel: () => void;
  onClear: () => void;
  selectedCount: number;
  showApplyButton?: boolean;
  showSelectionCount?: boolean;
  totalCount: number;
}

function FilterFooter({
  onApply,
  onCancel,
  onClear,
  selectedCount,
  showApplyButton,
  showSelectionCount,
  totalCount,
}: FilterFooterProps) {
  const { t } = useTranslation();
  const showFooter = showApplyButton || showSelectionCount;
  if (!showFooter) return null;

  return (
    <div className="pt-3 mt-1 border-t border-slate-50 dark:border-slate-800 flex items-center justify-between">
      {showSelectionCount ? (
        <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider">
          {selectedCount > 0
            ? t("common.selected_count", { selected: selectedCount, total: totalCount })
            : t("common.all_selected", "Tất cả")}
        </span>
      ) : (
        <span />
      )}

      {showApplyButton ? (
        <div className="flex gap-2">
          <Button onClick={onClear} size="sm" variant="ghost">
            {t("common.clear")}
          </Button>
          <Button onClick={onCancel} size="sm" variant="ghost">
            {t("common.cancel")}
          </Button>
          <Button
            icon={<Check size={12} />}
            onClick={onApply}
            size="sm"
            variant="primary"
          >
            {t("common.apply")}
          </Button>
        </div>
      ) : selectedCount > 0 ? (
        <Button
          className="text-[10px] font-bold text-red-500 dark:text-red-400 hover:underline uppercase tracking-wide cursor-pointer h-auto p-0"
          onClick={onClear}
          variant="ghost"
        >
          {t("common.clear")}
        </Button>
      ) : null}
    </div>
  );
}

// ── Main Component ──────────────────────────────────────────────────

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
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);

  // Draft state for apply-button mode
  const { draftSelected, setDraftSelected } = useDraftSelection({
    selected,
    isOpen: open,
    enabled: showApplyButton,
  });

  // Keyboard shortcut (opt-in, no default)
  useKeyboardShortcut({
    shortcut: keyboardShortcut,
    onTrigger: () => { setOpen(true); },
    disabled: open,
  });

  const effectiveSelected = showApplyButton ? draftSelected : selected;

  const handleToggle = useCallback(
    (value: string) => {
      if (showApplyButton) {
        setDraftSelected((prev) =>
          prev.includes(value) ? prev.filter((v) => v !== value) : [...prev, value],
        );
        return;
      }

      onToggle(value);
      if (mode === "single") {
        setOpen(false);
      }
    },
    [showApplyButton, onToggle, mode],
  );

  const handleApply = useCallback(() => {
    if (onApply) {
      onApply(draftSelected);
    }
    setOpen(false);
  }, [onApply, draftSelected]);

  const handleCancel = useCallback(() => {
    setDraftSelected([...selected]);
    setOpen(false);
  }, [selected]);

  const handleClear = useCallback(() => {
    if (showApplyButton) {
      setDraftSelected([]);
    } else {
      onClear();
    }
  }, [showApplyButton, onClear]);

  const handleDropdownToggle = useCallback(
    (value: string) => {
      if (showApplyButton) {
        setDraftSelected((prev) =>
          prev.includes(value) ? prev.filter((v) => v !== value) : [...prev, value],
        );
        // prevent close on multi mode
        if (mode === "single") {
          setOpen(false);
          setTimeout(() => { onToggle(value); }, 0);
        }
        return;
      }

      onToggle(value);
      if (mode === "single") {
        // DropdownMenu auto-closes on item select
      }
    },
    [showApplyButton, onToggle, mode],
  );

  const panelContent = (
    <div className={cn("flex flex-col", className)}>
      {headerLabel ? (
        <div className="px-3 pt-2 pb-1.5">
          <div className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em]">
            {headerLabel}
          </div>
        </div>
      ) : null}

      {children ?? (
        <OptionList
          emptyMessage={emptyMessage}
          mode={mode}
          onClear={handleClear}
          onToggle={handleToggle}
          options={options}
          searchable={searchable}
          searchPlaceholder={searchPlaceholder}
          selected={effectiveSelected}
          showClearButton={showClearButton && !showApplyButton}
          testId={testId}
        />
      )}

      <FilterFooter
        onApply={handleApply}
        onCancel={handleCancel}
        onClear={handleClear}
        selectedCount={effectiveSelected.length}
        showApplyButton={showApplyButton}
        showSelectionCount={showSelectionCount}
        totalCount={options.length}
      />
    </div>
  );

  // ── Dropdown container ───────────────────────────────────────────
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
          {/* All / reset item (multi mode) */}
          {mode === "multi" && !showApplyButton ? (
            <>
              <Button
                className={cn(
                  "h-auto w-full px-3 py-2.5 flex items-center gap-3 rounded-xl text-[11px] font-bold uppercase tracking-wide transition-colors border-none shadow-none justify-start",
                  effectiveSelected.length === 0
                    ? "text-primary-600 dark:text-primary-400"
                    : "text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800",
                )}
                data-testid={testId ? `${testId}-option-all` : undefined}
                onClick={(e) => { e.preventDefault(); e.stopPropagation(); onClear(); }}
                type="button"
                variant="ghost"
              >
                {t("common.all_departments", "All")}
              </Button>
              <DropdownMenuSeparator />
            </>
          ) : null}

          {options.map((opt) => {
            const checked = effectiveSelected.includes(opt.value);
            const isMultiLive = mode === "multi" && !showApplyButton;
            if (mode === "multi") {
              return (
                <MultiOptionRow
                  checked={checked}
                  key={opt.value}
                  onToggle={() => { handleDropdownToggle(opt.value); }}
                  option={opt}
                  stopPropagation={isMultiLive}
                  testId={testId}
                />
              );
            }
            return (
              <SingleOptionRow
                checked={checked}
                key={opt.value}
                onSelect={() => { handleDropdownToggle(opt.value); }}
                option={opt}
                testId={testId}
              />
            );
          })}

          {showApplyButton ? (
            <div className="pt-2 mt-1 border-t border-slate-100 dark:border-slate-800 flex justify-end gap-2">
              <Button onClick={handleCancel} size="sm" variant="ghost">
                {t("common.cancel")}
              </Button>
              <Button
                icon={<Check size={12} />}
                onClick={handleApply}
                size="sm"
                variant="primary"
              >
                {t("common.apply")}
              </Button>
            </div>
          ) : effectiveSelected.length > 0 ? (
            <>
              <DropdownMenuSeparator />
              <Button
                className="h-auto w-full px-3 py-2.5 flex items-center gap-3 rounded-xl text-[11px] font-bold uppercase tracking-wide text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors border-none shadow-none justify-start"
                data-testid={testId ? `${testId}-clear` : undefined}
                onClick={(e) => { e.preventDefault(); e.stopPropagation(); onClear(); }}
                type="button"
                variant="ghost"
              >
                <X className="shrink-0" size={12} />
                {t("common.clear_filter", "Xóa bộ lọc")}
              </Button>
            </>
          ) : null}
        </DropdownMenuContent>
      </DropdownMenu>
    );
  }

  // ── Popover container ────────────────────────────────────────────
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
          {panelContent}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

export default FilterPanel;

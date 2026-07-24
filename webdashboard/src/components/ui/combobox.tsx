import * as Popover from "@radix-ui/react-popover";
import { Search, Check, ChevronDown, Loader2 } from "lucide-react";
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { Command } from "cmdk";

import { cn } from "@/lib/utils";

interface ComboboxOption {
  description?: string | React.ReactNode;
  displayDescription?: React.ReactNode;
  displayLabel?: React.ReactNode;
  icon?: React.ReactNode;
  label: string;
  value: string;
}

interface ComboboxProps {
  autoFocus?: boolean;
  className?: string;
  clearOnEscape?: boolean;
  "data-testid"?: string;
  disabled?: boolean;
  emptyText?: string;
  hasMore?: boolean;
  id?: string;
  isLoadingMore?: boolean;
  keyboardShortcut?: string | string[];
  onChange: (value: string) => void;
  onLoadMore?: () => void;
  onSearchChange?: (value: string) => void;
  options: ComboboxOption[];
  placeholder?: string;
  searchPlaceholder?: string;
  shouldFilter?: boolean;
  value?: string;
}

interface KeyboardShortcutOptions {
  clearOnEscape?: boolean;
  keyboardShortcut?: string | string[];
  onChange: (value: string) => void;
  onSearchChange?: (value: string) => void;
  open: boolean;
  setOpen: (open: boolean) => void;
  value?: string;
}

function useComboboxKeyboardShortcut({
  clearOnEscape,
  keyboardShortcut,
  open,
  value,
  onChange,
  onSearchChange,
  setOpen
}: KeyboardShortcutOptions) {
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (!clearOnEscape || e.key !== 'Escape' || open || value === "") return false;
      e.preventDefault();
      onChange("");
      if (onSearchChange) onSearchChange("");
      return true;
    };

    const isInputTarget = (target: HTMLElement) => {
      return ['INPUT', 'TEXTAREA'].includes(target.tagName) || target.isContentEditable;
    };

    const handleShortcut = (e: KeyboardEvent) => {
      if (!keyboardShortcut || open) return false;
      if (isInputTarget(e.target as HTMLElement)) return false;

      const shortcuts = Array.isArray(keyboardShortcut) ? keyboardShortcut : [keyboardShortcut];
      if (shortcuts.includes(e.key)) {
        e.preventDefault();
        setOpen(true);
        if (onSearchChange) onSearchChange('');
        return true;
      }
      return false;
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (handleEscape(e)) return;
      handleShortcut(e);
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => { window.removeEventListener('keydown', handleKeyDown); };
  }, [keyboardShortcut, onSearchChange, open, clearOnEscape, value, onChange, setOpen]);
}

function removeDiacritics(str: string): string {
  return str
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/đ/g, "d")
    .replace(/Đ/g, "D");
}

function getFirstItemValue(options: ComboboxOption[]): string {
  if (options.length === 0) return "";
  const first = options[0];
  const firstLabelWithDiacritics = first.label;
  const firstLabelWithoutDiacritics = removeDiacritics(first.label);
  const desc = typeof first.description === "string" ? first.description : "";
  return `${firstLabelWithDiacritics} ${firstLabelWithoutDiacritics} ${desc} ${first.value}`.toLowerCase();
}

function ComboboxOptionItem({
  option,
  isSelected,
  onSelect,
  testId
}: {
  isSelected: boolean;
  onSelect: () => void;
  option: ComboboxOption;
  testId?: string;
}) {
  const labelWithDiacritics = option.label;
  const labelWithoutDiacritics = removeDiacritics(option.label);
  const desc = typeof option.description === 'string' ? option.description : "";
  const cmdkValue = `${labelWithDiacritics} ${labelWithoutDiacritics} ${desc} ${option.value}`.toLowerCase();

  return (
    <Command.Item
      className={cn(
        "relative flex cursor-pointer select-none items-center rounded-2xl px-4 py-3.5 text-sm font-bold outline-none transition-all hover:bg-primary-50 dark:hover:bg-primary-950 hover:text-primary-600 dark:hover:text-primary-300 data-[selected=true]:bg-primary-50 dark:data-[selected=true]:bg-primary-950 data-[selected=true]:text-primary-600 dark:data-[selected=true]:text-primary-300 uppercase tracking-tight",
        isSelected && "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300 shadow-inner"
      )}
      data-testid={testId ? `${testId}-option-${option.value}` : `combo-option-${option.value}`}
      key={option.value}
      onSelect={onSelect}
      value={cmdkValue}
    >
      <div className="flex flex-1 items-center gap-4">
        {option.icon}
        <div className="flex flex-col">
          <span className="leading-tight">{option.displayLabel ?? option.label}</span>
          {Boolean(option.description) && <div className="mt-1 opacity-60">{option.displayDescription ?? option.description}</div>}
        </div>
      </div>
      {isSelected ? <Check className="ml-auto h-4 w-4 text-primary-600 dark:text-primary-300" /> : null}
    </Command.Item>
  );
}

function ComboboxTrigger({
  autoFocus,
  className,
  disabled,
  id,
  keyboardShortcut,
  open,
  placeholder,
  selectedOption,
  testId,
}: {
  autoFocus?: boolean;
  className?: string;
  disabled?: boolean;
  id?: string;
  keyboardShortcut?: string | string[];
  open: boolean;
  placeholder: string;
  selectedOption?: ComboboxOption;
  testId?: string;
}) {
  return (
    <Popover.Trigger asChild>
      <button
        // eslint-disable-next-line jsx-a11y/no-autofocus -- combobox trigger needs autoFocus for keyboard navigation UX
        autoFocus={autoFocus}
        className={cn("flex h-[var(--ui-height-md)] w-full items-center justify-between rounded-2xl border border-slate-100 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 px-6 text-sm font-bold text-primary-600 dark:text-primary-400 transition-all hover:border-primary-200 dark:hover:border-primary-800 hover:bg-white dark:hover:bg-slate-700 focus:outline-none focus:ring-4 focus:ring-primary-500/10 shadow-inner uppercase tracking-tight disabled:opacity-50 disabled:cursor-not-allowed", className)}
        data-testid={testId}
        disabled={disabled}
        id={id}
        type="button"
      >
        <span className="flex items-center gap-3 truncate">
          {selectedOption?.icon}
          {selectedOption ? (
            selectedOption.label
          ) : (
            <span className="text-primary-400 font-bold opacity-60 flex items-center gap-2">
              {placeholder}
              {Boolean(keyboardShortcut) && (
                <kbd className="hidden sm:inline-flex items-center h-5 px-1.5 text-[10px] uppercase font-mono bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded text-slate-400 dark:text-slate-500 shadow-sm ml-2">
                  {Array.isArray(keyboardShortcut) ? keyboardShortcut[0] : keyboardShortcut}
                </kbd>
              )}
            </span>
          )}
        </span>
        <ChevronDown className={cn("ml-2 h-4 w-4 shrink-0 text-slate-300 transition-transform duration-300", open && "rotate-180 text-primary-500")} />
      </button>
    </Popover.Trigger>
  );
}
interface ComboboxContentProps {
  emptyText?: string;
  hasMore?: boolean;
  highlightedValue: string;
  isLoadingMore?: boolean;
  onChange: (value: string) => void;
  onLoadMore?: () => void;
  onSearchChange?: (value: string) => void;
  options: ComboboxOption[];
  searchPlaceholder?: string;
  setHighlightedValue: (value: string) => void;
  setOpen: (open: boolean) => void;
  shouldFilter?: boolean;
  testId?: string;
  value?: string;
}

function ComboboxContent({
  emptyText,
  hasMore,
  highlightedValue,
  isLoadingMore,
  onChange,
  onLoadMore,
  onSearchChange,
  options,
  searchPlaceholder,
  setHighlightedValue,
  setOpen,
  shouldFilter,
  testId,
  value,
}: ComboboxContentProps) {
  const { t } = useTranslation();

  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const canLoad = Boolean(onLoadMore) && Boolean(hasMore) && !isLoadingMore;
    if (!canLoad) return;
    const target = e.currentTarget;
    if (target.scrollHeight - target.scrollTop <= target.clientHeight + 20) {
      onLoadMore?.();
    }
  };

  return (
    <Popover.Portal>
      <Popover.Content className="z-[10000] w-[var(--radix-popover-trigger-width)] min-w-[240px] overflow-hidden rounded-[2rem] border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 p-2.5 shadow-2xl dark:shadow-black/30 animate-in fade-in zoom-in-95 duration-300" sideOffset={8}>
        <Command className="flex flex-col gap-3" onValueChange={setHighlightedValue} shouldFilter={shouldFilter ?? !onSearchChange} value={highlightedValue}>
          <div className="flex items-center border-b border-slate-50 dark:border-slate-800 px-4 pb-3 pt-1">
            <Search className="mr-3 h-4 w-4 shrink-0 text-slate-300 dark:text-slate-500" />
            {/*
 eslint-disable-next-line jsx-a11y/no-autofocus -- search input must autoFocus when combobox opens for immediate typing */}
            <Command.Input autoFocus className="flex h-10 w-full rounded-md bg-transparent text-sm text-primary-700 dark:text-primary-300 outline-none placeholder:text-slate-300 dark:placeholder:text-slate-600 disabled:cursor-not-allowed disabled:opacity-50 font-bold" onValueChange={onSearchChange} placeholder={searchPlaceholder ?? t("common.combobox.search")} />
          </div>

          <Command.List
            className="max-h-[320px] overflow-y-auto overflow-x-hidden p-1 custom-scrollbar"
            onScroll={handleScroll}
          >
            <Command.Empty className="py-10 text-center text-[10px] text-primary-400 font-bold uppercase tracking-widest opacity-50">{emptyText ?? t("common.combobox.no_results")}</Command.Empty>
            <Command.Group>
              {options.map((option) => (
                <ComboboxOptionItem
                  isSelected={value === option.value}
                  key={option.value}
                  onSelect={() => { onChange(option.value); setOpen(false); }}
                  option={option}
                  testId={testId}
                />
              ))}
            </Command.Group>
            {!!isLoadingMore && (
              <div className="py-3 flex justify-center items-center gap-2 text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider">
                <Loader2 className="w-4 h-4 animate-spin text-primary-500" />
                <span>{t("common.loading")}</span>
              </div>
            )}
          </Command.List>
        </Command>
      </Popover.Content>
    </Popover.Portal>
  );
}

export function Combobox({
  options,
  value,
  onChange,
  onSearchChange,
  placeholder,
  searchPlaceholder,
  emptyText,
  className,
  id,
  "data-testid": testId,
  shouldFilter,
  autoFocus,
  keyboardShortcut,
  clearOnEscape,
  disabled,
  onLoadMore,
  isLoadingMore,
  hasMore,
}: ComboboxProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(autoFocus ?? false);

  useComboboxKeyboardShortcut({ clearOnEscape, keyboardShortcut, open, value, onChange, onSearchChange, setOpen });

  const firstItemValue = getFirstItemValue(options);
  const [highlightedValue, setHighlightedValue] = useState(firstItemValue);
  const [prevFirst, setPrevFirst] = useState(firstItemValue);

  if (firstItemValue !== prevFirst) {
    setPrevFirst(firstItemValue);
    setHighlightedValue(firstItemValue);
  }

  const handleOpenChange = (isOpen: boolean) => {
    setOpen(isOpen);
    if (!isOpen && onSearchChange) {
      onSearchChange("");
    }
  };

  const triggerPlaceholder = placeholder ?? t("common.combobox.placeholder");
  const selectedOption = options.find((opt) => opt.value === value);

  return (
    <Popover.Root
      onOpenChange={handleOpenChange}
      open={open}
    >
      <select aria-hidden="true" className="sr-only" data-testid={testId ? `${testId}-select` : undefined} id={id ? `${id}-select` : undefined} onChange={(e) => { onChange(e.target.value); }} tabIndex={-1} value={value}>
        <option value="">{triggerPlaceholder}</option>
        {options.map(opt => (<option key={opt.value} value={opt.value}>{opt.label}</option>))}
      </select>

      <ComboboxTrigger
        // eslint-disable-next-line jsx-a11y/no-autofocus -- native select fallback needs autoFocus for keyboard nav
        autoFocus={autoFocus}
        className={className}
        disabled={disabled}
        id={id}
        keyboardShortcut={keyboardShortcut}
        open={open}
        placeholder={triggerPlaceholder}
        selectedOption={selectedOption}
        testId={testId}
      />

      <ComboboxContent
        emptyText={emptyText}
        hasMore={hasMore}
        highlightedValue={highlightedValue}
        isLoadingMore={isLoadingMore}
        onChange={onChange}
        onLoadMore={onLoadMore}
        onSearchChange={onSearchChange}
        options={options}
        searchPlaceholder={searchPlaceholder}
        setHighlightedValue={setHighlightedValue}
        setOpen={setOpen}
        shouldFilter={shouldFilter}
        testId={testId}
        value={value}
      />
    </Popover.Root>
  );
}



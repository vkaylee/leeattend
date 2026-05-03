import * as Popover from "@radix-ui/react-popover";
import { Search, Check, ChevronDown } from "lucide-react";
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
  id?: string;
  keyboardShortcut?: string | string[];
  onChange: (value: string) => void;
  onSearchChange?: (value: string) => void;
  options: ComboboxOption[];
  placeholder?: string;
  searchPlaceholder?: string;
  shouldFilter?: boolean;
  value?: string;
}

function useComboboxKeyboardShortcut(
  clearOnEscape: boolean | undefined,
  keyboardShortcut: string | string[] | undefined,
  open: boolean,
  value: string | undefined,
  onChange: (value: string) => void,
  onSearchChange: ((value: string) => void) | undefined,
  setOpen: (open: boolean) => void
) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (clearOnEscape && e.key === 'Escape' && !open && value !== "") {
        e.preventDefault();
        onChange("");
        if (onSearchChange) onSearchChange("");
        return;
      }

      if (!keyboardShortcut) return;
      const shortcuts = Array.isArray(keyboardShortcut) ? keyboardShortcut : [keyboardShortcut];

      if (open) return;
      const target = e.target as HTMLElement;
      const tagName = target.tagName;
      if (tagName === 'INPUT' || tagName === 'TEXTAREA' || target.isContentEditable) return;
      if (shortcuts.includes(e.key)) {
        e.preventDefault();
        setOpen(true);
        if (onSearchChange) onSearchChange('');
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => { window.removeEventListener('keydown', handleKeyDown); };
  }, [keyboardShortcut, onSearchChange, open, clearOnEscape, value, onChange, setOpen]);
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
        autoFocus={autoFocus}
        className={cn("flex w-full items-center justify-between rounded-[1.25rem] border border-slate-100 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 px-5 py-4 text-sm font-bold text-primary-600 dark:text-primary-400 transition-all hover:border-primary-200 dark:hover:border-primary-800 hover:bg-white dark:hover:bg-slate-700 focus:outline-none focus:ring-4 focus:ring-primary-500/10 shadow-inner uppercase tracking-tight disabled:opacity-50 disabled:cursor-not-allowed", className)}
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
              {!!keyboardShortcut && (
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
}: ComboboxProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(autoFocus ?? false);

  useComboboxKeyboardShortcut(clearOnEscape, keyboardShortcut, open, value, onChange, onSearchChange, setOpen);

  const finalPlaceholder = placeholder ?? t("common.combobox.placeholder");
  const finalSearchPlaceholder = searchPlaceholder ?? t("common.combobox.search");
  const finalEmptyText = emptyText ?? t("common.combobox.no_results");

  const selectedOption = options.find((opt) => opt.value === value);

  const firstItemCmdkValue = options.length > 0
    ? `${options[0].label} ${typeof options[0].description === 'string' ? options[0].description : ""} ${options[0].value}`
    : "";
  const [highlightedValue, setHighlightedValue] = useState(firstItemCmdkValue);

  const [prevFirstItem, setPrevFirstItem] = useState(firstItemCmdkValue);

  if (firstItemCmdkValue !== prevFirstItem) {
    setPrevFirstItem(firstItemCmdkValue);
    setHighlightedValue(firstItemCmdkValue);
  }

  return (
    <Popover.Root onOpenChange={setOpen} open={open}>
      <select aria-hidden="true" className="sr-only" data-testid={testId ? `${testId}-select` : undefined} id={id ? `${id}-select` : undefined} onChange={(e) => { onChange(e.target.value); }} tabIndex={-1} value={value}>
        <option value="">{finalPlaceholder}</option>
        {options.map(opt => (<option key={opt.value} value={opt.value}>{opt.label}</option>))}
      </select>

      <ComboboxTrigger
        autoFocus={autoFocus}
        className={className}
        disabled={disabled}
        id={id}
        keyboardShortcut={keyboardShortcut}
        open={open}
        placeholder={finalPlaceholder}
        selectedOption={selectedOption}
        testId={testId}
      />

      <Popover.Portal>
        <Popover.Content className="z-[10000] w-[var(--radix-popover-trigger-width)] min-w-[240px] overflow-hidden rounded-[2rem] border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 p-2.5 shadow-2xl dark:shadow-black/30 animate-in fade-in zoom-in-95 duration-300" sideOffset={8}>
          <Command className="flex flex-col gap-3" onValueChange={setHighlightedValue} shouldFilter={shouldFilter ?? !onSearchChange} value={highlightedValue}>
            <div className="flex items-center border-b border-slate-50 dark:border-slate-800 px-4 pb-3 pt-1">
              <Search className="mr-3 h-4 w-4 shrink-0 text-slate-300 dark:text-slate-500" />
              <Command.Input autoFocus className="flex h-10 w-full rounded-md bg-transparent text-sm text-primary-700 dark:text-primary-300 outline-none placeholder:text-slate-300 dark:placeholder:text-slate-600 disabled:cursor-not-allowed disabled:opacity-50 font-bold" onValueChange={onSearchChange} placeholder={finalSearchPlaceholder} />
            </div>

            <Command.List className="max-h-[320px] overflow-y-auto overflow-x-hidden p-1 custom-scrollbar">
              <Command.Empty className="py-10 text-center text-[10px] text-primary-400 font-bold uppercase tracking-widest opacity-50">{finalEmptyText}</Command.Empty>
              <Command.Group>
                {options.map((option) => (
                  <Command.Item
                    className={cn(
                      "relative flex cursor-pointer select-none items-center rounded-2xl px-4 py-3.5 text-sm font-bold outline-none transition-all hover:bg-primary-50 dark:hover:bg-primary-950 hover:text-primary-600 dark:hover:text-primary-300 data-[selected=true]:bg-primary-50 dark:data-[selected=true]:bg-primary-950 data-[selected=true]:text-primary-600 dark:data-[selected=true]:text-primary-300 uppercase tracking-tight",
                      value === option.value && "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300 shadow-inner"
                    )}
                    data-testid={testId ? `${testId}-option-${option.value}` : `combo-option-${option.value}`}
                    key={option.value}
                    onSelect={() => { onChange(option.value); setOpen(false); }}
                    value={`${option.label} ${typeof option.description === 'string' ? option.description : ""} ${option.value}`}
                  >
                    <div className="flex flex-1 items-center gap-4">
                      {option.icon}
                      <div className="flex flex-col">
                        <span className="leading-tight">{option.displayLabel ?? option.label}</span>
                        {!!option.description && <div className="mt-1 opacity-60">{option.displayDescription ?? option.description}</div>}
                      </div>
                    </div>
                    {value === option.value && <Check className="ml-auto h-4 w-4 text-primary-600 dark:text-primary-300" />}
                  </Command.Item>
                ))}
              </Command.Group>
            </Command.List>
          </Command>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

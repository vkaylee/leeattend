import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Command } from "cmdk";
import { Search, Check, ChevronDown } from "lucide-react";
import * as Popover from "@radix-ui/react-popover";
import { cn } from "@/lib/utils";

interface ComboboxOption {
  value: string;
  label: string;
  description?: string;
  icon?: React.ReactNode;
}

interface ComboboxProps {
  options: ComboboxOption[];
  value?: string;
  onChange: (value: string) => void;
  onSearchChange?: (value: string) => void;
  placeholder?: string;
  searchPlaceholder?: string;
  emptyText?: string;
  className?: string;
  id?: string;
  "data-testid"?: string;
  shouldFilter?: boolean;
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
}: ComboboxProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  
  placeholder = placeholder || t("common.combobox.placeholder");
  searchPlaceholder = searchPlaceholder || t("common.combobox.search");
  emptyText = emptyText || t("common.combobox.no_results");

  const selectedOption = options.find((opt) => opt.value === value);

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      <select id={id ? `${id}-select` : undefined} value={value} onChange={(e) => onChange(e.target.value)} className="sr-only" aria-hidden="true" tabIndex={-1} data-testid={testId ? `${testId}-select` : undefined}>
        <option value="">{placeholder}</option>
        {options.map(opt => (<option key={opt.value} value={opt.value}>{opt.label}</option>))}
      </select>

      <Popover.Trigger asChild>
        <button id={id} data-testid={testId} type="button" className={cn("flex w-full items-center justify-between rounded-[1.25rem] border border-slate-100 bg-slate-50 px-5 py-4 text-sm font-black text-slate-700 transition-all hover:border-primary-200 hover:bg-white focus:outline-none focus:ring-4 focus:ring-primary-500/10 shadow-inner uppercase tracking-tight", className)}>
          <span className="flex items-center gap-3 truncate">
            {selectedOption?.icon}
            {selectedOption ? selectedOption.label : <span className="text-slate-400 font-bold opacity-60">{placeholder}</span>}
          </span>
          <ChevronDown className={cn("ml-2 h-4 w-4 shrink-0 text-slate-300 transition-transform duration-300", open && "rotate-180 text-primary-500")} />
        </button>
      </Popover.Trigger>

      <Popover.Portal>
        <Popover.Content className="z-[10000] w-[var(--radix-popover-trigger-width)] min-w-[240px] overflow-hidden rounded-[2rem] border border-slate-100 bg-white p-2.5 shadow-2xl animate-in fade-in zoom-in-95 duration-300" sideOffset={8}>
          <Command shouldFilter={shouldFilter ?? !onSearchChange} className="flex flex-col gap-3">
            <div className="flex items-center border-b border-slate-50 px-4 pb-3 pt-1">
              <Search className="mr-3 h-4 w-4 shrink-0 text-slate-300" />
              <Command.Input placeholder={searchPlaceholder} onValueChange={onSearchChange} className="flex h-10 w-full rounded-md bg-transparent text-sm outline-none placeholder:text-slate-300 disabled:cursor-not-allowed disabled:opacity-50 font-bold" />
            </div>
            
            <Command.List className="max-h-[320px] overflow-y-auto overflow-x-hidden p-1 custom-scrollbar">
              <Command.Empty className="py-10 text-center text-[10px] text-slate-400 font-black uppercase tracking-widest opacity-50">{emptyText}</Command.Empty>
              <Command.Group>
                {options.map((option) => (
                  <Command.Item
                    key={option.value}
                    value={`${option.label} ${option.description || ""} ${option.value}`}
                    data-testid={testId ? `${testId}-option-${option.value}` : `combo-option-${option.value}`}
                    onSelect={() => { onChange(option.value); setOpen(false); }}
                    className={cn(
                      "relative flex cursor-pointer select-none items-center rounded-2xl px-4 py-3.5 text-sm font-black outline-none transition-all hover:bg-primary-50 hover:text-primary-600 data-[selected=true]:bg-primary-50 data-[selected=true]:text-primary-600 uppercase tracking-tight",
                      value === option.value && "bg-primary-50 text-primary-600 shadow-inner"
                    )}
                  >
                    <div className="flex flex-1 items-center gap-4">
                      {option.icon}
                      <div className="flex flex-col">
                        <span className="leading-tight">{option.label}</span>
                        {option.description && <div className="mt-1 opacity-60">{option.description}</div>}
                      </div>
                    </div>
                    {value === option.value && <Check className="ml-auto h-4 w-4 text-primary-600" />}
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

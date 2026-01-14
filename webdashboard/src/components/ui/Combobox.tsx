import React, { useState } from "react";
import { Command } from "cmdk";
import { Search, Check, ChevronDown } from "lucide-react";
import * as Popover from "@radix-ui/react-popover";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

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
  placeholder?: string;
  searchPlaceholder?: string;
  emptyText?: string;
  className?: string;
  id?: string;
  "data-testid"?: string;
}

export function Combobox({
  options,
  value,
  onChange,
  placeholder = "Chọn một mục...",
  searchPlaceholder = "Tìm kiếm...",
  emptyText = "Không tìm thấy kết quả.",
  className,
  id,
  "data-testid": testId,
}: ComboboxProps) {
  const [open, setOpen] = useState(false);
  const selectedOption = options.find((opt) => opt.value === value);

  return (
    <Popover.Root open={open} onOpenChange={setOpen}>
      {/* Hidden select for accessibility and testing */}
      <select 
        id={id} 
        value={value} 
        onChange={(e) => onChange(e.target.value)}
        className="sr-only"
        aria-hidden="true"
        tabIndex={-1}
        data-testid={testId ? `${testId}-select` : undefined}
      >
        <option value="">{placeholder}</option>
        {options.map(opt => (
          <option key={opt.value} value={opt.value}>{opt.label}</option>
        ))}
      </select>

      <Popover.Trigger asChild>
        <button
          data-testid={testId}
          type="button"
          className={cn(
            "flex w-full items-center justify-between rounded-2xl border border-slate-100 bg-slate-50 px-4 py-3 text-sm font-bold text-slate-700 transition-all hover:border-violet-200 hover:bg-white focus:outline-none focus:ring-4 focus:ring-violet-500/10",
            className
          )}
        >
          <span className="flex items-center gap-2 truncate">
            {selectedOption?.icon}
            {selectedOption ? selectedOption.label : <span className="text-slate-400 font-medium">{placeholder}</span>}
          </span>
          <ChevronDown className={cn("ml-2 h-4 w-4 shrink-0 text-slate-400 transition-transform duration-200", open && "rotate-180")} />
        </button>
      </Popover.Trigger>

      <Popover.Portal>
        <Popover.Content
          className="z-50 w-[var(--radix-popover-trigger-width)] min-w-[200px] overflow-hidden rounded-[1.5rem] border border-slate-50 bg-white p-2 shadow-2xl animate-in fade-in zoom-in-95 duration-200"
          sideOffset={8}
        >
          <Command className="flex flex-col gap-2">
            <div className="flex items-center border-b border-slate-50 px-3 pb-2">
              <Search className="mr-2 h-4 w-4 shrink-0 text-slate-400" />
              <Command.Input
                placeholder={searchPlaceholder}
                className="flex h-10 w-full rounded-md bg-transparent py-3 text-sm outline-none placeholder:text-slate-400 disabled:cursor-not-allowed disabled:opacity-50 font-medium"
              />
            </div>
            
            <Command.List className="max-h-[300px] overflow-y-auto overflow-x-hidden p-1 custom-scrollbar">
              <Command.Empty className="py-6 text-center text-sm text-slate-500 font-medium">
                {emptyText}
              </Command.Empty>
              
              <Command.Group>
                {options.map((option) => (
                  <Command.Item
                    key={option.value}
                    value={option.label} // search by label
                    onSelect={() => {
                      onChange(option.value);
                      setOpen(false);
                    }}
                    className={cn(
                      "relative flex cursor-pointer select-none items-center rounded-xl px-3 py-3 text-sm font-bold outline-none transition-all hover:bg-violet-50 hover:text-violet-600 data-[selected=true]:bg-violet-50 data-[selected=true]:text-violet-600",
                      value === option.value && "bg-violet-50 text-violet-600"
                    )}
                  >
                    <div className="flex flex-1 items-center gap-3">
                      {option.icon}
                      <div className="flex flex-col">
                        <span>{option.label}</span>
                        {option.description && (
                          <span className="text-xs font-medium text-slate-400 leading-tight">
                            {option.description}
                          </span>
                        )}
                      </div>
                    </div>
                    {value === option.value && (
                      <Check className="ml-auto h-4 w-4 text-violet-600" />
                    )}
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

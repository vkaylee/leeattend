import React from "react";

import { cn } from "@/lib/utils";

export interface SegmentedOption<T extends string = string> {
  disabled?: boolean;
  icon?: React.ReactNode;
  label: React.ReactNode;
  testId?: string;
  value: T;
}

export interface SettingsSegmentedControlProps<T extends string = string> {
  className?: string;
  onChange: (value: T) => void;
  options: SegmentedOption<T>[];
  testId?: string;
  value: T;
}

export function SettingsSegmentedControl<T extends string = string>({
  className,
  onChange,
  options,
  testId,
  value,
}: SettingsSegmentedControlProps<T>) {
  return (
    <div
      className={cn(
        "inline-flex items-center gap-1 rounded-xl bg-slate-100/90 p-1 border border-slate-200/60 dark:bg-slate-800/80 dark:border-slate-700/60",
        className
      )}
      data-testid={testId}
      role="tablist"
    >
      {options.map((opt) => {
        const isSelected = opt.value === value;
        return (
          <button
            aria-selected={isSelected}
            className={cn(
              "flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 select-none whitespace-nowrap shrink-0",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500",
              isSelected
                ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200",
              opt.disabled && "opacity-40 pointer-events-none"
            )}
            data-testid={opt.testId}
            disabled={opt.disabled}
            key={opt.value}
            onClick={() => onChange(opt.value)}
            role="tab"
            type="button"
          >
            {opt.icon && <span className="shrink-0">{opt.icon}</span>}
            <span>{opt.label}</span>
          </button>
        );
      })}
    </div>
  );
}

export default SettingsSegmentedControl;

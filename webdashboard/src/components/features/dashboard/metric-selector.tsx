import { useId, useRef } from "react";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

export interface MetricOption<T extends string> {
  label: string;
  value: T;
}

interface MetricSelectorProps<T extends string> {
  "aria-label"?: string;
  className?: string;
  onChange: (value: T) => void;
  options: MetricOption<T>[];
  value: T;
}

/**
 * Keyboard-accessible single-metric selector (radio group). Arrow keys move
 * between options; Enter/Space activate. Keeps a chart to a single series so no
 * two quantities ever share an axis.
 */
export function MetricSelector<T extends string>({
  value,
  onChange,
  options,
  className,
  ...rest
}: MetricSelectorProps<T>) {
  const baseId = useId();
  const refs = useRef<Record<string, HTMLButtonElement | null>>({});

  const onKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>, index: number) => {
    const count = options.length;
    let next = -1;
    if (event.key === "ArrowRight" || event.key === "ArrowDown") next = (index + 1) % count;
    else if (event.key === "ArrowLeft" || event.key === "ArrowUp") next = (index - 1 + count) % count;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = count - 1;
    if (next >= 0) {
      event.preventDefault();
      onChange(options[next].value);
      refs.current[options[next].value]?.focus();
    }
  };

  return (
    <div
      aria-label={rest["aria-label"]}
      className={cn(
        "flex shrink-0 max-w-full overflow-x-auto scrollbar-hide bg-slate-100 dark:bg-slate-700/50 p-1 rounded-[14px] border border-slate-200 dark:border-slate-700/50 shadow-inner",
        className,
      )}
      role="radiogroup"
    >
      {options.map((option, index) => {
        const selected = option.value === value;
        return (
          <Button
            aria-checked={selected}
            className={cn(
              "px-3 py-1.5 rounded-[10px] text-[9px] font-bold uppercase tracking-widest transition-all flex items-center gap-1.5 h-auto shadow-none border border-transparent whitespace-nowrap",
              selected
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-transparent",
            )}
            id={`${baseId}-${option.value}`}
            key={option.value}
            onClick={() => { onChange(option.value); }}
            onKeyDown={(e) => { onKeyDown(e, index); }}
            ref={(node) => { refs.current[option.value] = node; }}
            role="radio"
            type="button"
            variant="ghost"
          >
            {option.label}
          </Button>
        );
      })}
    </div>
  );
}

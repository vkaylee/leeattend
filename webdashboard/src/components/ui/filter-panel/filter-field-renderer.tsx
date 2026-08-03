import { Check } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

import type { FilterOption } from "./types";

// ── Multi-Select Option Row ────────────────────────────────────────────

interface MultiOptionRowProps {
  checked: boolean;
  onToggle: () => void;
  option: FilterOption;
  stopPropagation?: boolean;
  testId?: string;
}

export function MultiOptionRow({
  option,
  checked,
  onToggle,
  stopPropagation,
  testId,
}: MultiOptionRowProps) {
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

// ── Single-Select Option Row ───────────────────────────────────────────

interface SingleOptionRowProps {
  checked: boolean;
  onSelect: () => void;
  option: FilterOption;
  stopPropagation?: boolean;
  testId?: string;
}

export function SingleOptionRow({
  option,
  checked,
  onSelect,
  stopPropagation,
  testId,
}: SingleOptionRowProps) {
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

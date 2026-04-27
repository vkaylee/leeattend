import React from "react";
import { cn } from "@/lib/utils";

// ── Variant styles ─────────────────────────────────────────────────

const VARIANT_STYLES = {
  success:
    "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800",
  warning:
    "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800",
  danger:
    "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800",
  info:
    "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800",
  primary:
    "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800",
  neutral:
    "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60",
  indigo:
    "bg-indigo-50 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-300 border-indigo-100 dark:border-indigo-800",
  emerald:
    "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400 border-emerald-100 dark:border-emerald-800",
  orange:
    "bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300 border-orange-100 dark:border-orange-800",
} as const;

// ── Component ──────────────────────────────────────────────────────

export type StatusBadgeVariant = keyof typeof VARIANT_STYLES;

export interface StatusBadgeProps {
  variant?: StatusBadgeVariant;
  children: React.ReactNode;
  icon?: React.ReactNode;
  /** Pulse animation for "pending" states */
  pulse?: boolean;
  className?: string;
}

const StatusBadge = React.memo(function StatusBadge({
  variant = "neutral",
  children,
  icon,
  pulse = false,
  className,
}: StatusBadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm",
        VARIANT_STYLES[variant],
        pulse && "animate-pulse",
        className,
      )}
    >
      {icon}
      {children}
    </span>
  );
});

export default StatusBadge;

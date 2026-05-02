import type { LucideIcon } from "lucide-react";

import React, { memo } from "react";

import { cn } from "@/lib/utils";

// ── Component ──────────────────────────────────────────────────────

export interface StatCardProps {
  className?: string;
  /** Tailwind color classes for the icon container (bg + text + border) */
  colorClasses: string;
  /** Lucide icon component */
  icon: LucideIcon;
  /** Whether the card is in loading state */
  isLoading?: boolean;
  /** i18n-translated label displayed above the value */
  label: string;
  /** Display value with monospace font */
  mono?: boolean;
  /** Size variant — 'sm' for KPI grids (6-col), 'lg' for summary rows */
  size?: "sm" | "lg";
  /** Numeric or string value to display */
  value: string | number;
}

const StatCard = memo(function StatCard({
  label,
  value,
  icon: Icon,
  colorClasses,
  mono,
  isLoading,
  size = "sm",
  className,
}: StatCardProps) {
  if (size === "lg") {
    return (
      <div
        className={cn(
          "bg-white dark:bg-slate-900 p-6 sm:p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm flex items-center gap-4 sm:gap-6 group hover:shadow-2xl hover:shadow-primary-100/10 transition-all",
          className,
        )}
      >
        <div
          className={cn(
            colorClasses,
            "w-12 sm:w-14 h-12 sm:h-14 rounded-[1.2rem] flex items-center justify-center shadow-inner border group-hover:scale-110 transition-transform",
          )}
        >
          <Icon size={24} />
        </div>
        <div>
          <p className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mb-1">
            {label}
          </p>
          <p
            className={cn(
              "text-2xl sm:text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter",
              mono && "font-mono",
            )}
          >
            {isLoading ? "..." : value}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div
      className={cn(
        "bg-white dark:bg-slate-900 p-4 sm:p-6 rounded-[2rem] border border-slate-100 dark:border-slate-800 shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden",
        className,
      )}
    >
      <div
        className={cn(
          colorClasses,
          "w-10 sm:w-12 h-10 sm:h-12 rounded-[1rem] flex items-center justify-center mb-3 sm:mb-5 group-hover:scale-110 group-hover:rotate-3 transition-all shadow-inner border",
        )}
      >
        <Icon size={20} />
      </div>
      <p className="text-[8px] sm:text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mb-1">
        {label}
      </p>
      <p
        className={cn(
          "text-2xl sm:text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter",
          mono && "font-mono",
        )}
      >
        {isLoading ? "..." : value}
      </p>
      <Icon className="absolute -right-4 -bottom-4 w-20 sm:w-24 h-20 sm:h-24 text-slate-50 dark:text-slate-800 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
    </div>
  );
});

export default StatCard;

import React, { memo } from "react";

import { cn } from "@/lib/utils";

// ── Component ──────────────────────────────────────────────────────

export interface PageHeaderProps {
  /** Action buttons rendered on the right side */
  actions?: React.ReactNode;
  className?: string;
  /** data-testid for the h1 element */
  "data-testid"?: string;
  /** Optional icon rendered in a styled container before the title */
  icon?: React.ReactNode;
  subtitle?: string;
  title: string;
}

 
const PageHeader = memo(function PageHeader({
  title,
  subtitle,
  icon,
  actions,
  "data-testid": testId,
  className,
}: PageHeaderProps) {
  return (
    <div
      className={cn(
        "flex flex-col md:flex-row md:items-center justify-between gap-6 px-2",
        className,
      )}
    >
      <div className="flex items-center gap-6">
        {!!icon && (
          <div className="h-14 w-14 rounded-2xl bg-white dark:bg-slate-900 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-xl shadow-primary-100/20 dark:shadow-black/20 border border-slate-100 dark:border-slate-800 animate-in zoom-in-95 duration-500">
            {icon}
          </div>
        )}
        <div>
          <h1 className="text-3xl font-black text-slate-800 dark:text-slate-100 tracking-tight leading-snug mb-2 uppercase" data-testid={testId}>
            {title}
          </h1>
          {!!subtitle && (
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">
              {subtitle}
            </p>
          )}
        </div>
      </div>
      {!!actions && (
        <div className="flex items-center gap-3 animate-in fade-in slide-in-from-right-4 duration-700">
          {actions}
        </div>
      )}
    </div>
  );
});

export default PageHeader;

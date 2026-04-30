import React, { memo } from "react";

import { cn } from "@/lib/utils";

// ── Component ──────────────────────────────────────────────────────

export interface PageHeaderProps {
  title: string;
  subtitle?: string;
  /** Action buttons rendered on the right side */
  actions?: React.ReactNode;
  /** data-testid for the h1 element */
  "data-testid"?: string;
  className?: string;
}

 
const PageHeader = memo(function PageHeader({
  title,
  subtitle,
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
      <div className="space-y-1">
        <h1
          data-testid={testId}
          className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 uppercase leading-none"
        >
          {title}
        </h1>
        {subtitle && (
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">
            {subtitle}
          </p>
        )}
      </div>
      {actions && (
        <div className="flex items-center gap-3">{actions}</div>
      )}
    </div>
  );
});

export default PageHeader;

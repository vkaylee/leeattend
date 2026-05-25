import React, { memo } from "react";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

// ── Component ──────────────────────────────────────────────────────

export interface EmptyStateProps {
  /** Optional call-to-action button */
  action?: React.ReactNode;
  className?: string;
  /** Supporting description text */
  description?: string;
  /** Lucide icon or custom ReactNode rendered in the circular container */
  icon: React.ReactNode;
  /** Primary heading text */
  title: string;
}

const EmptyState = memo(function EmptyState({
  icon,
  title,
  description,
  action,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        "col-span-full py-32 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center animate-in zoom-in-95 duration-500",
        className,
      )}
    >
      <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-6 border border-slate-100 dark:border-slate-800 text-slate-200 dark:text-slate-700">
        {icon}
      </div>
      <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h3">
        {title}
      </Typography>
      {!!description && (
        <Typography className="text-slate-500 dark:text-slate-400 text-[11px] font-bold uppercase tracking-widest leading-relaxed max-w-sm mx-auto" variant="p">
          {description}
        </Typography>
      )}
      {!!action && (
        <div className="pt-4 scale-110">
          {action}
        </div>
      )}
    </div>
  );
});

export default EmptyState;

import React from "react";

import { cn } from "@/lib/utils";

export interface SettingsGroupProps {
  children: React.ReactNode;
  className?: string;
  description?: React.ReactNode;
  testId?: string;
  title?: React.ReactNode;
}

export function SettingsGroup({
  children,
  className,
  description,
  testId,
  title,
}: SettingsGroupProps) {
  return (
    <div className={cn("space-y-1.5", className)} data-testid={testId}>
      {title && (
        <div className="px-3 text-[11px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400 select-none">
          {title}
        </div>
      )}
      <div className="overflow-hidden rounded-2xl border border-slate-200/80 bg-white shadow-2xs divide-y divide-slate-100 dark:border-slate-800/80 dark:bg-slate-900/90 dark:divide-slate-800/70">
        {children}
      </div>
      {description && (
        <div className="px-3 text-[11px] leading-relaxed text-slate-400 dark:text-slate-500">
          {description}
        </div>
      )}
    </div>
  );
}

export default SettingsGroup;

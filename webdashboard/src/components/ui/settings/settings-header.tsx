import React from "react";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

export interface SettingsHeaderProps {
  actions?: React.ReactNode;
  badge?: React.ReactNode;
  className?: string;
  description?: React.ReactNode;
  icon?: React.ReactNode;
  iconBg?: string;
  testId?: string;
  title: React.ReactNode;
}

export function SettingsHeader({
  actions,
  badge,
  className,
  description,
  icon,
  iconBg,
  testId,
  title,
}: SettingsHeaderProps) {
  return (
    <div
      className={cn(
        "flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4 border-b border-slate-200/60 dark:border-slate-800/60",
        className
      )}
      data-testid={testId}
    >
      <div className="flex items-center gap-3.5 min-w-0">
        {!!icon && (
          <div
            className={cn(
              "w-10 h-10 rounded-xl flex items-center justify-center shrink-0 shadow-2xs border",
              iconBg ||
                "bg-primary-50 dark:bg-primary-950/70 text-primary-600 dark:text-primary-400 border-primary-100/50 dark:border-primary-900/50"
            )}
          >
            {icon}
          </div>
        )}
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2 flex-wrap">
            {typeof title === "string" ? (
              <Typography
                className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100 truncate"
                variant="h1"
              >
                {title}
              </Typography>
            ) : (
              title
            )}
            {badge}
          </div>
          {!!description && (
            <div className="text-xs text-slate-500 dark:text-slate-400 font-medium truncate mt-0.5">
              {description}
            </div>
          )}
        </div>
      </div>
      {!!actions && <div className="flex items-center gap-2 shrink-0">{actions}</div>}
    </div>
  );
}

export default SettingsHeader;

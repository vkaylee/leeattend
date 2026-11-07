import React, { memo } from "react";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

// ── Component ──────────────────────────────────────────────────────

export interface PageHeaderProps {
  /** Action buttons rendered on the right side */
  actions?: React.ReactNode;
  className?: string;
  /** data-testid for the h1 element */
  "data-testid"?: string;
  /** Hides the title block (title, subtitle, icon) to avoid duplicate headers */
  hideTitle?: boolean;
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
  hideTitle,
}: PageHeaderProps) {
  return (
    <div
      className={cn(
        "flex flex-col items-stretch md:flex-row md:items-center justify-between gap-6",
        className,
      )}
    >
      {hideTitle ? (
        <div />
      ) : (
        <div className="flex items-center gap-4">
          {!!icon && (
            <div className="h-12 w-12 rounded-xl bg-white dark:bg-slate-900 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-md shadow-primary-100/10 dark:shadow-black/20 border border-slate-100 dark:border-slate-800 animate-in zoom-in-95 duration-500">
              {icon}
            </div>
          )}
          <div>
            <Typography as="h1" className="font-bold mb-0.5 normal-case" data-testid={testId} variant="h3">
              {title}
            </Typography>
            {!!subtitle && (
              <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium opacity-80" variant="small">
                {subtitle}
              </Typography>
            )}
          </div>
        </div>
      )}
      {!!actions && (
        <div className="flex w-full flex-wrap items-center justify-start gap-3 animate-in fade-in slide-in-from-right-4 duration-700 md:w-auto md:justify-end">
          {actions}
        </div>
      )}
    </div>
  );
});

export { PageHeader };
export default PageHeader;

import React, { memo } from "react";

import EmptyState from "./empty-state";
import { Typography } from "./typography";

export interface TableEmptyRowProps {
  /** Call-to-action button */
  action?: React.ReactNode;
  className?: string;
  /** Number of table columns to span (MANDATORY to prevent grid collapse) */
  colSpan: number;
  /** Supporting description text */
  description?: string;
  /** Lucide icon or custom ReactNode rendered in the circular container */
  icon?: React.ReactNode;
  /** Primary heading text */
  title: string;
}

/**
 * Standardized Table Empty Row Component (DRY & LSP compliant).
 * Guarantees proper HTML `tr > td` table layout spanning the entire column width.
 * Reuses the design-system `EmptyState` component.
 */
export const TableEmptyRow = memo(function TableEmptyRow({
  colSpan,
  icon,
  title,
  description,
  action,
  className,
}: TableEmptyRowProps) {
  return (
    <tr>
      <td
        className="p-6 md:p-12 text-center border-b border-border/40"
        colSpan={colSpan}
      >
        <div className="flex justify-center w-full">
          {icon ? (
            <EmptyState
              action={action}
              className={className}
              description={description}
              icon={icon}
              title={title}
            />
          ) : (
            <div className="py-12 flex flex-col items-center justify-center max-w-md mx-auto text-center space-y-2">
              <Typography className="text-sm font-semibold text-foreground/90" variant="p">{title}</Typography>
              {Boolean(description) && (
                <Typography className="text-xs text-muted-foreground leading-relaxed" variant="p">{description}</Typography>
              )}
              {Boolean(action) && <div className="pt-3">{action}</div>}
            </div>
          )}
        </div>
      </td>
    </tr>
  );
});

export default TableEmptyRow;

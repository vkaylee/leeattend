import React from "react";

import { cn } from "@/lib/utils";

export interface ToolbarRowProps {
  children: React.ReactNode;
  className?: string;
}

/** Shared single-line control row. Parent owns layout; controls own geometry. */
export function ToolbarRow({ children, className }: ToolbarRowProps) {
  return (
    <div
      className={cn(
        "flex min-h-[var(--ui-height-sm)] w-full flex-wrap items-center gap-3",
        className,
      )}
      data-component="ToolbarRow"
    >
      {children}
    </div>
  );
}

export default ToolbarRow;

import type { HTMLAttributes, ReactNode } from "react";

import { cn } from "@/lib/utils";

export interface PageContainerProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

/** Shared authenticated-page shell; horizontal gutters belong to AppShell. */
export function PageContainer({ children, className, ...props }: PageContainerProps) {
  return (
    <div
      className={cn(
        "w-full space-y-6 pb-20 animate-in fade-in duration-700",
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}

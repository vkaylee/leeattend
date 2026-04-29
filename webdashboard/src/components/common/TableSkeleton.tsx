import React from 'react';
import { cn } from "@/lib/utils";

interface TableSkeletonProps {
  rows?: number;
  variant?: "primary" | "warning";
}

export function TableSkeleton({ rows = 5, variant = "primary" }: TableSkeletonProps) {
  const isWarning = variant === "warning";
  
  return (
    <>
      {Array.from({ length: rows }).map((_, i) => (
        <tr key={`skeleton-${i}`} className="animate-pulse border-b border-slate-50 dark:border-slate-800/50">
          <td className="px-8 py-5">
            <div className="flex items-center gap-4">
              <div className={cn("w-10 h-10 rounded-xl", isWarning ? "bg-amber-50 dark:bg-amber-950/50" : "bg-slate-200 dark:bg-slate-800")}></div>
              <div className="space-y-2">
                <div className="h-4 w-24 bg-slate-200 dark:bg-slate-800 rounded"></div>
                <div className="h-3 w-16 bg-slate-200 dark:bg-slate-800 rounded"></div>
              </div>
            </div>
          </td>
          <td className="px-8 py-5">
            <div className="space-y-2">
              <div className={cn("h-4 bg-slate-200 dark:bg-slate-800 rounded", isWarning ? "w-24" : "w-32")}></div>
              <div className={cn("h-3 bg-slate-200 dark:bg-slate-800 rounded", isWarning ? "w-16" : "w-20")}></div>
            </div>
          </td>
          <td className="px-8 py-5">
            <div className={cn("h-5 bg-slate-200 dark:bg-slate-800 rounded", isWarning ? "w-20" : "w-24")}></div>
          </td>
          <td className="px-8 py-5 text-right">
            {isWarning ? (
              <div className="flex justify-end gap-2">
                <div className="h-6 w-16 bg-slate-200 dark:bg-slate-800 rounded-md inline-block"></div>
                <div className="h-6 w-16 bg-slate-200 dark:bg-slate-800 rounded-md inline-block"></div>
              </div>
            ) : (
              <div className="h-6 w-20 bg-slate-200 dark:bg-slate-800 rounded-md inline-block"></div>
            )}
          </td>
        </tr>
      ))}
    </>
  );
}

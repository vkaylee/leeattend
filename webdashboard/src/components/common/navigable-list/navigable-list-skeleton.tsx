import { Loader2 } from "lucide-react";
import React, { memo } from "react";

import { Skeleton } from "@/components/ui/skeleton";

export const NavigableListSkeleton = memo(function NavigableListSkeleton({
  skeletonRows,
  colSpan,
}: {
  colSpan: number;
  skeletonRows: number;
}) {
  return (
    <>
      {Array.from({ length: skeletonRows }).map((_, i) => (
        <tr className="animate-pulse border-b border-slate-50 dark:border-slate-800" key={i}>
          <td className="px-8 py-6" colSpan={colSpan}>
            <div className="h-4 bg-slate-100 dark:bg-slate-800 rounded-lg w-2/3" />
          </td>
        </tr>
      ))}
    </>
  );
});

export const InfiniteScrollSentinel = memo(function InfiniteScrollSentinel({
  isFetchingNextPage,
  sentinelRef,
  colSpan,
}: {
  colSpan: number;
  isFetchingNextPage: boolean;
  sentinelRef: React.Ref<HTMLTableRowElement>;
}) {
  return (
    <tr ref={sentinelRef}>
      <td className="px-8 py-4 text-center" colSpan={colSpan}>
        {isFetchingNextPage ? (
          <div className="flex items-center justify-center gap-3">
            <Loader2 className="animate-spin text-slate-400" size={16} />
            <Skeleton className="h-2.5 w-16 bg-slate-200 dark:bg-slate-700" />
          </div>
        ) : null}
      </td>
    </tr>
  );
});

export function NavigableListMobileSkeleton({ rows }: { rows: number }) {
  return (
    <div className="space-y-3">
      {Array.from({ length: rows }).map((_, index) => (
        <div className="animate-pulse rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900" key={index}>
          <div className="flex items-center gap-3">
            <Skeleton className="h-11 w-11 rounded-xl bg-slate-200 dark:bg-slate-800" />
            <div className="flex-1 space-y-2">
              <Skeleton className="h-4 w-2/3 bg-slate-200 dark:bg-slate-800" />
              <Skeleton className="h-3 w-1/3 bg-slate-200 dark:bg-slate-800" />
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

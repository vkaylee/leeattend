import { Loader2 } from "lucide-react";
import React, { memo } from "react";

import { Skeleton } from "@/components/ui/skeleton";

interface InfiniteScrollSentinelProps {
    /** Number of columns to span */
    colSpan?: number;
    /** Whether next page is currently being fetched */
    isFetchingNextPage: boolean;
    /** Ref from useInfiniteScroll hook */
    sentinelRef: React.Ref<HTMLTableRowElement>;
}

/**
 * Sentinel table row that triggers loading the next page
 * when it becomes visible via IntersectionObserver.
 */
 
const InfiniteScrollSentinel = memo(function InfiniteScrollSentinel({
    sentinelRef,
    isFetchingNextPage,
    colSpan = 5,
}: InfiniteScrollSentinelProps) {
    return (
        <tr ref={sentinelRef}>
            <td className="py-4 text-center" colSpan={colSpan}>
                {isFetchingNextPage ? (
                    <div className="flex items-center justify-center gap-3">
                        <Loader2 className="animate-spin text-slate-400 dark:text-slate-500" size={16} />
                        <Skeleton className="h-2.5 w-16 bg-slate-200 dark:bg-slate-700" />
                    </div>
                  ) : null}
            </td>
        </tr>
    );
});

export default InfiniteScrollSentinel;

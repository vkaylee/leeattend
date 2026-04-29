import React from "react";

import { Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";

interface InfiniteScrollSentinelProps {
    /** Ref from useInfiniteScroll hook */
    sentinelRef: React.RefObject<HTMLTableRowElement | null>;
    /** Whether next page is currently being fetched */
    isFetchingNextPage: boolean;
    /** Number of columns to span */
    colSpan?: number;
}

/**
 * Sentinel table row that triggers loading the next page
 * when it becomes visible via IntersectionObserver.
 */
const InfiniteScrollSentinel = React.memo(function InfiniteScrollSentinel({
    sentinelRef,
    isFetchingNextPage,
    colSpan = 5,
}: InfiniteScrollSentinelProps) {
    const { t } = useTranslation();

    return (
        <tr ref={sentinelRef}>
            <td colSpan={colSpan} className="py-4 text-center">
                {isFetchingNextPage && (
                    <div className="flex items-center justify-center gap-2 text-slate-400 dark:text-slate-500">
                        <Loader2 size={16} className="animate-spin" />
                        <span className="text-[10px] font-bold uppercase tracking-widest">
                            {t("common.loading", "Loading...")}
                        </span>
                    </div>
                )}
            </td>
        </tr>
    );
});

export default InfiniteScrollSentinel;

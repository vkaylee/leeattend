import { Loader2 } from "lucide-react";
import React, { memo } from "react";
import { useTranslation } from "react-i18next";

interface InfiniteScrollSentinelProps {
    /** Number of columns to span */
    colSpan?: number;
    /** Whether next page is currently being fetched */
    isFetchingNextPage: boolean;
    /** Ref from useInfiniteScroll hook */
    sentinelRef: React.Ref<any>;
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
    const { t } = useTranslation();

    return (
        <tr ref={sentinelRef}>
            <td className="py-4 text-center" colSpan={colSpan}>
                {isFetchingNextPage && (
                    <div className="flex items-center justify-center gap-2 text-slate-400 dark:text-slate-500">
                        <Loader2 className="animate-spin" size={16} />
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
